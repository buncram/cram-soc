#!/usr/bin/env python3
#
# Copyright (c) 2022 Cramium Labs, Inc.
# Derived from litex_soc_gen.py:
# Copyright (c) 2022 Florent Kermarrec <florent@enjoy-digital.fr>
# SPDX-License-Identifier: BSD-2-Clause

# sim dependencies:
# verilator (from source), libevent-dev, libjson-c-dev
import argparse
from pathlib import Path

from migen import *
from migen.genlib.cdc import MultiReg
from litex.soc.interconnect import stream

from litex.build.generic_platform import *
from litex.build.sim import SimPlatform
from litex.build.sim.config import SimConfig

from litex.soc.integration.soc_core import *
from litex.soc.integration.soc import SoCRegion, SoCIORegion
from litex.soc.integration.builder import *
from litex.soc.interconnect import wishbone
from litex.soc.interconnect import axi

from litex.build.xilinx import XilinxPlatform, VivadoProgrammer
from litex.soc.cores.clock import S7MMCM, S7IDELAYCTRL
from migen.genlib.resetsync import AsyncResetSynchronizer
from litex.soc.interconnect.csr import *

from litex.soc.interconnect.axi import AXIInterface, AXILiteInterface
from litex.soc.integration.soc import SoCBusHandler
from litex.soc.cores import uart
from litex.soc.integration.doc import AutoDoc, ModuleDoc

from deps.gateware.gateware import memlcd

from axi_crossbar import AXICrossbar
from axi_adapter import AXIAdapter
from axi_ram import AXIRAM
from axil_crossbar import AXILiteCrossbar
from axil_cdc import AXILiteCDC
from axi_common import *

from axil_ahb_adapter import AXILite2AHBAdapter
from litex.soc.interconnect import ahb

from cram_common import CramSoC

import subprocess
import shutil

VEX_VERILOG_PATH = "VexRiscv/VexRiscv_CramSoC.v"
PRODUCTION_MODELS = False

# IOs ----------------------------------------------------------------------------------------------

_io = [
    # Clk / Rst.
    ("sys_clk", 0, Pins(1)),
    ("p_clk", 0, Pins(1)),
    ("pio_clk", 0, Pins(1)),
    # ("sys_reset", 0, Pins(1)),

    ("jtag", 0,
         Subsignal("tck", Pins("U11"), IOStandard("LVCMOS18")),
         Subsignal("tms", Pins("P6"), IOStandard("LVCMOS18")),
         Subsignal("tdi", Pins("P7"), IOStandard("LVCMOS18")),
         Subsignal("tdo", Pins("R6"), IOStandard("LVCMOS18")),
    ),

    # mapped to GPIOs 0-4
    ("jtag_cpu", 0,
         Subsignal("tck", Pins("F14"), IOStandard("LVCMOS33")),
         Subsignal("tms", Pins("F15"), IOStandard("LVCMOS33")),
         Subsignal("tdi", Pins("E16"), IOStandard("LVCMOS33")),
         Subsignal("tdo", Pins("G15"), IOStandard("LVCMOS33")),
         Subsignal("trst", Pins("H15"), IOStandard("LVCMOS33")),
         Misc("SLEW=SLOW"),
    ),

    ("serial", 0, # wired to the RPi
        Subsignal("tx", Pins("V6")),
        Subsignal("rx", Pins("V7"), Misc("PULLUP True")),
        IOStandard("LVCMOS18"),
        Misc("SLEW=SLOW"),
    ),

    # Simulation UART log
    ("sim_uart", 0,
        Subsignal("kernel", Pins(8)),
        Subsignal("kernel_valid", Pins(1)),
        Subsignal("log", Pins(8)),
        Subsignal("log_valid", Pins(1)),
        Subsignal("app", Pins(8)),
        Subsignal("app_valid", Pins(1)),
    ),

    ("duart", 0,
        Subsignal("tx", Pins(1)),
    ),
    ("pio", 0,
        Subsignal("gpio", Pins(32)),
    ),

    # LCD interface
    ("lcd", 0,
        Subsignal("sclk", Pins("H17")), # DVT
        Subsignal("scs",  Pins("G17")), # DVT
        Subsignal("si",   Pins("H18")), # DVT
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
        Misc("DRIVE=4"),
     ),

     # Simulation "I/O"
     ("simio", 0,
        Subsignal("success", Pins(1)),
        Subsignal("done", Pins(1)),
        Subsignal("report", Pins(32)),
        Subsignal("coreuser", Pins(1)),
        Subsignal("sysclk", Pins(1)),
     ),

    # Trimming bits
     ("trimming", 0,
        Subsignal("reset", Pins(32)),
        Subsignal("reset_ena", Pins(1)),
     )
]

# BtGpio -------------------------------------------------------------------------------------------

class BtGpio(Module, AutoDoc, AutoCSR):
    def __init__(self):
        self.intro = ModuleDoc("""BtGpio - GPIO interface for betrusted""")

        self.uartsel = CSRStorage(2, name="uartsel", description="Used to select which UART is routed to physical pins, 00 = kernel debug, 01 = console, others reserved based on build")

# Dummy module that just copies the UART data to a register and immediately indicates we're good to go.
class SimPhyTx(Module):
    def __init__(self, sim_data_out, sim_data_out_valid):
        self.sink = sink = stream.Endpoint([("data", 8)])
        valid_r = Signal()

        self.sync += [
            valid_r.eq(sink.valid),
            If(sink.valid,
                sim_data_out.eq(sink.data),
                sink.ready.eq(1)
            ).Else(
                sim_data_out.eq(sim_data_out),
                sink.ready.eq(0),
            ),
            sim_data_out_valid.eq(~valid_r & sink.valid),
        ]

# Dummy module that injects nothing. This is written so that we can extend it to have the test bench inject data eventually if we wanted to.
class SimPhyRx(Module):
    def __init__(self, data_in, data_valid, data_ready):
        self.source = source = stream.Endpoint([("data", 8)])

        # # #
        self.comb += [
            source.valid.eq(data_valid),
            source.data.eq(data_in),
            data_ready.eq(source.ready),
        ]

# Simulation UART ----------------------------------------------------------------------------------
class SimUartPhy(Module, AutoCSR):
    def __init__(self, data_in, data_in_valid, data_in_ready, data_out, data_out_valid, clk_freq, baudrate=115200, with_dynamic_baudrate=False):
        tuning_word = int((baudrate/clk_freq)*2**32)
        if with_dynamic_baudrate:
            self._tuning_word  = CSRStorage(32, reset=tuning_word)
            tuning_word = self._tuning_word.storage
        self.submodules.tx = SimPhyTx(data_out, data_out_valid)
        self.submodules.rx = SimPhyRx(data_in, data_in_valid, data_in_ready)
        self.sink, self.source = self.tx.sink, self.rx.source

# Simulation CRG -----------------------------------------------------------------------------------
class SimCRG(Module):
    def __init__(self, clk, p_clk, pio_clk, rst, sleep_req):
        self.clock_domains.cd_sys = ClockDomain()
        self.clock_domains.cd_por = ClockDomain(reset_less=True)
        self.clock_domains.cd_p = ClockDomain()
        self.clock_domains.cd_pio = ClockDomain()
        self.clock_domains.cd_sys_always_on = ClockDomain()

        # Power on Reset (vendor agnostic)
        int_rst = Signal(reset=1)
        self.sync.por += int_rst.eq(rst)
        self.comb += [
            self.cd_sys.clk.eq(clk & ~sleep_req),
            self.cd_por.clk.eq(clk),
            self.cd_sys.rst.eq(int_rst),
            self.cd_p.clk.eq(p_clk),
            self.cd_p.rst.eq(int_rst),
            self.cd_pio.clk.eq(pio_clk),
            self.cd_pio.rst.eq(int_rst),
            self.cd_sys_always_on.clk.eq(clk),
        ]

# Tune the common platform for simulation ----------------------------------------------------------
def sim_extensions(self, nosave=False):
    if self.sim_debug:
        self.platform.add_debug(self, reset=1 if self.trace_reset_on else 0)
    else:
        # Enable waveform dumping (slows down simulation a bit)
        # Can set to zero for a much faster sim, but no waveforms are saved.
        if nosave:
            self.comb += self.platform.trace.eq(0)
        else:
            self.comb += self.platform.trace.eq(1)

    # Clockgen cluster -------------------------------------------------------------------------
    reset_cycles = 32
    reset_counter = Signal(log2_int(reset_cycles), reset=reset_cycles - 1)
    ic_reset      = Signal(reset=1)
    self.sync.por += \
        If(reset_counter != 0,
            reset_counter.eq(reset_counter - 1)
        ).Else(
            ic_reset.eq(0)
        )
    self.crg = SimCRG(
        self.platform.request("sys_clk"),
        self.platform.request("p_clk"), self.platform.request("pio_clk"),
        ic_reset, self.sleep_req)

    # Add SoC memory regions
    for (name, region) in self.axi_mem_map.items():
        self.add_memory_region(name=name, origin=region[0], length=region[1])

    # Add crossbar ports for memory
    reram_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
    self.submodules.axi_reram = AXIRAM(self.platform, reram_axi, size=self.axi_mem_map["reram"][1], name="reram", init=self.bios_data)
    sram_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
    self.submodules.axi_sram = AXIRAM(self.platform, sram_axi, size=self.axi_mem_map["sram"][1], name="sram")
    xip_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
    self.submodules.xip_sram = AXIRAM(self.platform, xip_axi, size=65536, name="xip") # just a small amount of RAM for testing
    # vex debug is internal to the core, no interface to build

    self.mbus.add_master(name = "reram", m_axi=reram_axi, origin=self.axi_mem_map["reram"][0], size=self.axi_mem_map["reram"][1])
    self.mbus.add_master(name = "sram",  m_axi=sram_axi,  origin=self.axi_mem_map["sram"][0],  size=self.axi_mem_map["sram"][1])
    self.mbus.add_master(name = "xip",  m_axi=xip_axi,  origin=self.axi_mem_map["xip"][0],  size=self.axi_mem_map["sram"][1])

    # Muxed UARTS ---------------------------------------------------------------------------
    # this is necessary for Xous to recognize the UARTs out of the box. We can simplify this later on.

    self.submodules.gpio = BtGpio()
    self.add_csr("gpio")

    uart_pins = self.platform.request("serial")
    serial_layout = [("tx", 1), ("rx", 1)]
    kernel_pads = Record(serial_layout)
    console_pads = Record(serial_layout)
    app_uart_pads = Record(serial_layout)
    self.comb += [
        If(self.gpio.uartsel.storage == 0,
            uart_pins.tx.eq(kernel_pads.tx),
            kernel_pads.rx.eq(uart_pins.rx),
        ).Elif(self.gpio.uartsel.storage == 1,
            uart_pins.tx.eq(console_pads.tx),
            console_pads.rx.eq(uart_pins.rx),
        ).Else(
            uart_pins.tx.eq(app_uart_pads.tx),
            app_uart_pads.rx.eq(uart_pins.rx),
        )
    ]
    sim_uart_pins = self.platform.request("sim_uart")
    kernel_input = Signal(8)
    kernel_input_valid = Signal()
    kernel_input_ready = Signal()
    self.submodules.uart_phy = SimUartPhy(
        kernel_input,
        kernel_input_valid,
        kernel_input_ready,
        sim_uart_pins.kernel,
        sim_uart_pins.kernel_valid,
        clk_freq=self.sys_clk_freq,
        baudrate=115200)
    self.submodules.uart = ResetInserter()(
        uart.UART(self.uart_phy,
            tx_fifo_depth=16, rx_fifo_depth=16)
        )

    self.add_csr("uart_phy")
    self.add_csr("uart")
    self.irq.add("uart")

    console_data_in = Signal(8, reset=13) # 13=0xd
    console_data_valid = Signal(reset = 0)
    console_data_ready = Signal()
    self.submodules.console_phy = SimUartPhy(
        console_data_in,
        console_data_valid,
        console_data_ready,
        sim_uart_pins.log,
        sim_uart_pins.log_valid,
        clk_freq=self.sys_clk_freq,
        baudrate=115200)
    self.submodules.console = ResetInserter()(
        uart.UART(self.console_phy,
            tx_fifo_depth=16, rx_fifo_depth=16)
        )

    self.add_csr("console_phy")
    self.add_csr("console")
    self.irq.add("console")

    # extra PHY for "application" uses -- mainly things like the FCC testing agent
    app_data_in = Signal(8, reset=13) # 13=0xd
    app_data_valid = Signal(reset = 0)
    app_data_ready = Signal()
    self.submodules.app_uart_phy = SimUartPhy(
        app_data_in,
        app_data_valid,
        app_data_ready,
        sim_uart_pins.app,
        sim_uart_pins.app_valid,
        clk_freq=self.sys_clk_freq,
        baudrate=115200)
    self.submodules.app_uart = ResetInserter()(
        uart.UART(self.app_uart_phy,
            tx_fifo_depth=16, rx_fifo_depth=16)
        )

    self.add_csr("app_uart_phy")
    self.add_csr("app_uart")
    self.irq.add("app_uart")

    self.specials += Instance("uart_print",
        p_TYPE="log",
        i_uart_data=sim_uart_pins.log,
        i_uart_data_valid=sim_uart_pins.log_valid,
        i_resetn=~ResetSignal(),
        i_clk=ClockSignal(),
    )
    self.specials += Instance("uart_print",
        p_TYPE="kernel",
        i_uart_data=sim_uart_pins.kernel,
        i_uart_data_valid=sim_uart_pins.kernel_valid,
        i_resetn=~ResetSignal(),
        i_clk=ClockSignal(),
    )
    self.platform.add_source("sim_support/uart_print.v")

    # Simulation framework I/O ----------------------------------------------------------------------
    self.sim = sim = self.platform.request("simio")
    self.comb += [
        sim.report.eq(self.sim_report.storage),
        sim.success.eq(self.sim_success.storage),
        sim.done.eq(self.sim_done.storage),
        sim.coreuser.eq(self.coreuser),
    ]

    self.specials += Instance("finisher",
        i_kuart_from_cpu=sim_uart_pins.kernel,
        i_kuart_from_cpu_valid=sim_uart_pins.kernel_valid,
        o_kuart_to_cpu=kernel_input,
        o_kuart_to_cpu_valid=kernel_input_valid,
        i_kuart_to_cpu_ready=kernel_input_ready,
        i_report=self.sim.report,
        i_success=self.sim.success,
        i_done=self.sim.done,
        i_clk = ClockSignal(),
    )
    self.platform.add_source("sim_support/finisher.v")

    # pass the UART IRQ back to the common framework
    self.comb += self.uart_irq.eq(self.uart.ev.irq)


# Platform -----------------------------------------------------------------------------------------

class Platform(SimPlatform):
    def __init__(self):
        SimPlatform.__init__(self, "SIM", _io)

def keep_only(axi_group, names=[]):
    sig_list = axi_group.layout
    ret = []
    for signal in sig_list:
        if signal[0] in names:
            ret += [getattr(axi_group, signal[0])]
        if signal[0] == "payload":
            for subsignal in signal[1]:
                if subsignal[0] in names:
                    ret += [getattr(axi_group.payload, subsignal[0])]
    return ret

# Build --------------------------------------------------------------------------------------------
def generate_gtkw_savefile(builder, vns, trace_fst=False):
    from litex.build.sim import gtkwave as gtkw
    dumpfile = os.path.join(builder.gateware_dir, "sim.{}".format("fst" if trace_fst else "vcd"))
    savefile = os.path.join(builder.gateware_dir, "sim.gtkw")
    soc = builder.soc

    with gtkw.GTKWSave(vns, savefile=savefile, dumpfile=dumpfile) as save:
        save.clocks()
        save.fsm_states(soc)

        ar_rec = keep_only(soc.mbus.s_axis["dbus"].axi.ar, ["addr", "burst", "id", "len", "valid", "ready"])
        for s in ar_rec:
            save.add(s, mappers=[gtkw.axi_ar_sorter(), gtkw.axi_ar_colorer()])

        #save.add(soc.mbus.s_axis["dbus"].axi.aw, mappers=[gtkw.axi_sorter(), gtkw.axi_colorer()])
        #save.add(soc.mbus.s_axis["dbus"].axi.b, mappers=[gtkw.axi_sorter(), gtkw.axi_colorer()])
        #save.add(soc.mbus.s_axis["dbus"].axi.r, mappers=[gtkw.axi_sorter(), gtkw.axi_colorer()])
        #save.add(soc.mbus.s_axis["dbus"].axi.w, mappers=[gtkw.axi_sorter(), gtkw.axi_colorer()])


def sim_args(parser):
    # Speed. In reality, just selects whether we save a waveform, or not.
    parser.add_argument("--speed",                type=str, default="normal", help="Run at `normal` or `fast` speed. Fast runs do not save waveform data.")

    # Analyzer.
    parser.add_argument("--with-analyzer",        action="store_true",     help="Enable Analyzer support.")

    # Debug/Waveform.
    parser.add_argument("--sim-debug",            action="store_true",     help="Add simulation debugging modules.")
    parser.add_argument("--gtkwave-savefile",     action="store_true",     help="Generate GTKWave savefile.")
    parser.add_argument("--non-interactive",      action="store_true",     help="Run simulation without user input.")

    # Build just the SVDs
    parser.add_argument("--svd-only",             action="store_true",     help="Just build the SVDs for the OS build")

    # specify BIOS path
    parser.add_argument("--bios", type=str, default='..{}xous-cramium{}simspi.init'.format(os.path.sep, os.path.sep), help="Override default BIOS location")

    # compatibility with demo scripts
    parser.add_argument("--build",                action="store_true",     help="compatibility flag, ignored by this script")

def main():
    from litex.build.parser import LiteXArgumentParser
    parser = LiteXArgumentParser(description="LiteX SoC Simulation utility")
    parser.set_platform(SimPlatform)
    sim_args(parser)
    args = parser.parse_args()

    soc_kwargs = soc_core_argdict(args)

    sys_clk_freq = int(800e6)
    sim_config   = SimConfig()
    sim_config.add_clocker("sys_clk", freq_hz=sys_clk_freq)
    sim_config.add_clocker("p_clk", freq_hz=100e6) # simulated down to 50MHz, but left at 100MHz to speed up simulations
    sim_config.add_clocker("pio_clk", freq_hz=200e6)

    bios_path = args.bios
    soc = CramSoC(
        Platform(),
        variant="sim",
        bios_path=bios_path,
        sys_clk_freq=800e6,
        sim_debug          = args.sim_debug,
        trace_reset_on     = False,
        **soc_kwargs
    )
    if args.speed == "fast":
        nosave = True
    else:
        nosave = False
    CramSoC.sim_extensions = sim_extensions
    soc.sim_extensions(nosave)

    def pre_run_callback(vns):
        generate_gtkw_savefile(builder, vns)

    ##### setup the builder and run it
    builder = Builder(soc,
        csr_csv="build/csr.csv",
        csr_svd="build/software/soc.svd",
    )
    builder.software_packages=[] # necessary to bypass Meson dependency checks required by Litex libc

    # turn off regular_comb for simulation
    rc=False

    if args.svd_only:
        builder.build(run=False)
    else:
        shutil.copy('./build/gateware/reram_mem.init', './build/sim/gateware/')
        shutil.copy('./VexRiscv/VexRiscv_CramSoC.v_toplevel_memory_AesPlugin_rom_storage.bin', './build/sim/gateware/')
        shutil.copy('do_not_checkin/rtl/amba/template.sv', './build/sim/gateware/')

        # this runs the sim
        builder.build(
            sim_config       = sim_config,
            interactive      = not args.non_interactive,
            pre_run_callback = pre_run_callback,
            regular_comb     = rc,
            **parser.toolchain_argdict,
        )

if __name__ == "__main__":
    from datetime import datetime
    start = datetime.now()
    ret = main()
    print("Run completed in {}".format(datetime.now()-start))

    sys.exit(ret)
