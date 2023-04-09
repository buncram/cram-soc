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

from litex.soc.interconnect.axi import AXIInterface
from litex.soc.integration.soc import SoCBusHandler
from litex.soc.cores import uart
from litex.soc.integration.doc import AutoDoc, ModuleDoc

from deps.gateware.gateware import memlcd

from axi_crossbar import AXICrossbar
from axi_adapter import AXIAdapter
from axi_ram import AXIRAM
from axi_common import *

import subprocess
import shutil

VEX_VERILOG_PATH = "VexRiscv/src/VexRiscv_CramSoC.v"

# IOs ----------------------------------------------------------------------------------------------

_io = [
    # Clk / Rst.
    ("sys_clk", 0, Pins(1)),
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

# CramSoC ------------------------------------------------------------------------------------------

class CramSoC(SoCMini):
    mem_map = {**SoCCore.mem_map, **{
        "csr": 0x4010_0000, # save bottom 0x10_0000 for compatibility with Cramium native registers
    }}
    def __init__(self,
        bios_path=None,
        sys_clk_freq=800e6,
        sim_debug=False,
        trace_reset_on=False,
        # bogus arg handlers - we are doing SoCMini, but the simulator passes args for a full SoC
        bus_standard=None,
        bus_data_width=None,
        bus_address_width=None,
        bus_timeout=None,
        bus_bursting=None,
        bus_interconnect=None,
        cpu_type                 = None,
        cpu_reset_address        = None,
        cpu_variant              = None,
        cpu_cfu                  = None,
        cfu_filename             = None,
        csr_data_width           = None,
        csr_address_width        = None,
        csr_paging               = None,
        csr_ordering             = None,
        integrated_rom_size      = None,
        integrated_rom_mode      = None,
        integrated_rom_init      = None,
        integrated_sram_size     = None,
        integrated_sram_init     = None,
        integrated_main_ram_size = None,
        integrated_main_ram_init = None,
        irq_n_irqs               = None,
        ident                    = None,
        ident_version            = None,
        with_uart                = None,
        uart_name                = None,
        uart_baudrate            = None,
        uart_fifo_depth          = None,
        with_timer               = None,
        timer_uptime             = None,
        with_ctrl                = None,
        l2_size                  = None,
    ):
        AHB_TEST = False
        platform = Platform()
        axi_map = {
            "reram"     : 0x6000_0000, # +3M
            "sram"      : 0x6100_0000, # +2M
            "p_bus"     : 0x4000_0000, # +256M
            "memlcd"    : 0x42000000,
            "vexriscv_debug": 0xefff_0000,
        }
        SRAM_SIZE = 2*1024*1024
        self.platform = platform

        # Clockgen cluster -------------------------------------------------------------------------
        self.crg = CRG(platform.request("sys_clk"))
        self.clock_domains.cd_sys_always_on = ClockDomain()
        self.comb += self.cd_sys_always_on.clk.eq(ClockSignal())

        # Simulation debugging ----------------------------------------------------------------------
        if sim_debug:
            platform.add_debug(self, reset=1 if trace_reset_on else 0)
        else:
            self.comb += platform.trace.eq(1)

        # Add standalone SoC sources.
        platform.add_source("build/gateware/cram_axi.v")
        platform.add_source(VEX_VERILOG_PATH)
        platform.add_source("sim_support/ram_1w_1ra.v")
        platform.add_source("sim_support/ram_1w_1rs.v")
        platform.add_source("sim_support/fdre_cosim.v")

        # this must be pulled in manually because it's instantiated in the core design, but not in the SoC design
        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "verilog-axi", "rtl")
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter.v"))
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter_wr.v"))
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter_rd.v"))

        # SoCMini ----------------------------------------------------------------------------------
        SoCMini.__init__(self, platform, clk_freq=int(sys_clk_freq),
            csr_paging           = 4096,  # increase paging to 1 page size
            csr_address_width    = 16,    # increase to accommodate larger page size
            bus_standard         = "axi-lite",
            # bus_timeout          = None,         # use this if regular_comb=True on the builder
            with_ctrl            = False,
            io_regions           = {
                # Origin, Length.
                0x4000_0000 : 0x2000_0000,
                0xa000_0000 : 0x6000_0000,
            },
        )
        self.add_memory_region(name="sram", origin=axi_map["sram"], length=SRAM_SIZE)

        # Wire up peripheral SoC busses
        p_axil = axi.AXILiteInterface(name="pbus")
        jtag_cpu = platform.request("jtag_cpu")

        # Add simulation "output pins" -----------------------------------------------------
        self.sim_report = CSRStorage(32, name = "report", description="A 32-bit value to report sim state")
        self.sim_success = CSRStorage(1, name = "success", description="Determines the result code for the simulation. 0 means fail, 1 means pass")
        self.sim_done = CSRStorage(1, name ="done", description="Set to `1` if the simulation should auto-terminate")

        sim = platform.request("simio")
        self.comb += [
            sim.report.eq(self.sim_report.storage),
            sim.success.eq(self.sim_success.storage),
            sim.done.eq(self.sim_done.storage),
        ]

        # test that caching is OFF for the I/O regions
        self.sim_coherence_w = CSRStorage(32, name= "wdata", description="Write values here to check cache coherence issues")
        self.sim_coherence_r = CSRStatus(32, name="rdata", description="Data readback derived from coherence_w")
        self.sim_coherence_inc = CSRStatus(32, name="rinc", description="Every time this is read, the base value is incremented by 3", reset=0)

        self.sync += [
            If(self.sim_coherence_inc.we,
                self.sim_coherence_inc.status.eq(self.sim_coherence_inc.status + 3)
            ).Else(
                self.sim_coherence_inc.status.eq(self.sim_coherence_inc.status)
            )
        ]
        self.comb += [
            self.sim_coherence_r.status.eq(self.sim_coherence_w.storage + 5)
        ]

        # Add AXI RAM to SoC (Through AXI Crossbar).
        # ------------------------------------------

        # 1) Create AXI interface and connect it to SoC.
        dbus_axi = AXIInterface(data_width=32, address_width=32, id_width=1, bursting=True)
        dbus64_axi = AXIInterface(data_width=64, address_width=32, id_width=1, bursting=True)
        self.submodules += AXIAdapter(platform, s_axi = dbus_axi, m_axi = dbus64_axi, convert_burst=True, convert_narrow_burst=True)
        ibus64_axi = AXIInterface(data_width=64, address_width=32, id_width=1, bursting=True)

        # 2) Add 2 X AXILiteSRAM to emulate ReRAM and SRAM; much smaller now just for testing
        if bios_path is not None:
            with open(bios_path, 'rb') as bios:
                bios_data = bios.read()
        else:
            bios_data = []

        reram_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
        self.submodules.axi_reram = AXIRAM(platform, reram_axi, size=0x30_0000, name="reram", init=bios_data)

        sram_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
        self.submodules.axi_sram = AXIRAM(platform, sram_axi, size=SRAM_SIZE, name="sram")

        # 3) Add AXICrossbar  (2 Slave / 2 Master).
        self.submodules.mbus = mbus = AXICrossbar(platform=platform)
        mbus.add_slave(name = "dbus", s_axi=dbus64_axi,
            aw_reg = AXIRegister.BYPASS,
            w_reg  = AXIRegister.BYPASS,
            b_reg  = AXIRegister.BYPASS,
            ar_reg = AXIRegister.BYPASS,
            r_reg  = AXIRegister.BYPASS,
        )
        mbus.add_slave(name = "ibus", s_axi=ibus64_axi,
            aw_reg = AXIRegister.BYPASS,
            w_reg  = AXIRegister.BYPASS,
            b_reg  = AXIRegister.BYPASS,
            ar_reg = AXIRegister.BYPASS,
            r_reg  = AXIRegister.BYPASS,
        )
        mbus.add_master(name = "reram", m_axi=reram_axi, origin=axi_map["reram"], size=0x0100_0000)
        mbus.add_master(name = "sram",  m_axi=sram_axi,  origin=axi_map["sram"],  size=0x0100_0000)

        # 4) Add peripherals
        # setup p_axi as the local bus master
        if AHB_TEST is False:
            self.bus.add_master(name="pbus", master=p_axil)
        else:
            from axil_ahb_adapter import AXILite2AHBAdapter
            from litex.soc.interconnect import ahb
            from duart_adapter import DuartAdapter
            local_ahb = ahb.Interface()
            self.submodules += AXILite2AHBAdapter(platform, p_axil, local_ahb)
            self.submodules += DuartAdapter(platform, local_ahb, pads=platform.request("duart"), sel_addr=0x1000)

        # add interrupt handler
        interrupt = Signal(32)
        self.cpu.interrupt = interrupt
        self.irq.enable()

        # GPIO module ------------------------------------------------------------------------------
        self.submodules.gpio = BtGpio()
        self.add_csr("gpio")

        # Muxed UARTS ---------------------------------------------------------------------------
        uart_pins = platform.request("serial")
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
        sim_uart_pins = platform.request("sim_uart")
        kernel_input = Signal(8)
        kernel_input_valid = Signal()
        kernel_input_ready = Signal()
        self.submodules.uart_phy = SimUartPhy(
            kernel_input,
            kernel_input_valid,
            kernel_input_ready,
            sim_uart_pins.kernel,
            sim_uart_pins.kernel_valid,
            clk_freq=sys_clk_freq,
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
            clk_freq=sys_clk_freq,
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
            clk_freq=sys_clk_freq,
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
        platform.add_source("sim_support/uart_print.v")

        self.specials += Instance("finisher",
            i_kuart_from_cpu=sim_uart_pins.kernel,
            i_kuart_from_cpu_valid=sim_uart_pins.kernel_valid,
            o_kuart_to_cpu=kernel_input,
            o_kuart_to_cpu_valid=kernel_input_valid,
            i_kuart_to_cpu_ready=kernel_input_ready,
            i_report=sim.report,
            i_success=sim.success,
            i_done=sim.done,
            i_clk = ClockSignal(),
        )
        platform.add_source("sim_support/finisher.v")

        # LCD interface ----------------------------------------------------------------------------
        self.submodules.memlcd = ClockDomainsRenamer({"sys":"sys_always_on"})(memlcd.MemLCD(platform.request("lcd"), interface="axi-lite"))
        self.add_csr("memlcd")
        self.bus.add_slave("memlcd", self.memlcd.bus, SoCRegion(origin=axi_map["memlcd"], size=self.memlcd.fb_depth*4, mode="rw", cached=False))

        # Cramium platform -------------------------------------------------------------------------
        zero_irq = Signal(20)
        irq0_wire_or = Signal(20)
        self.comb += [
            irq0_wire_or[0].eq(self.uart.ev.irq)
        ]
        self.irqtest0 = CSRStorage(fields=[
            CSRField(
                name = "trigger", size=20, description="Triggers for interrupt testing bank 0", pulse=False
            )
        ])
        self.irqtest1 = CSRStorage(fields=[
            CSRField(
                name = "trigger", size=20, description="Triggers for interrupt testing bank 0", pulse=True
            )
        ])
        # wfi breakout
        wfi_active = Signal()
        wfi_loopback = Signal(20)
        wfi_delay = Signal(7, reset=64) # coded as a one-shot
        self.sync += [
            If(wfi_active & (wfi_delay > 0),
                wfi_delay.eq(wfi_delay - 1),
            ),
            If(wfi_delay == 1,
                wfi_loopback.eq(1), # creates an exactly one-cycle wide wfi wakeup trigger
            ).Else(
                wfi_loopback.eq(0),
            )
        ]

        # Pull in DUT IP ---------------------------------------------------------------------------
        self.specials += Instance("cram_axi",
            i_aclk                = ClockSignal("sys"),
            i_rst                 = ResetSignal("sys"),
            i_always_on           = ClockSignal("sys"),
            i_trimming_reset      = 0x6000_0000,
            i_trimming_reset_ena  = 1,
            o_p_axi_awvalid       = p_axil.aw.valid,
            i_p_axi_awready       = p_axil.aw.ready,
            o_p_axi_awaddr        = p_axil.aw.addr ,
            o_p_axi_awprot        = p_axil.aw.prot ,
            o_p_axi_wvalid        = p_axil.w.valid ,
            i_p_axi_wready        = p_axil.w.ready ,
            o_p_axi_wdata         = p_axil.w.data  ,
            o_p_axi_wstrb         = p_axil.w.strb  ,
            i_p_axi_bvalid        = p_axil.b.valid ,
            o_p_axi_bready        = p_axil.b.ready ,
            i_p_axi_bresp         = p_axil.b.resp  ,
            o_p_axi_arvalid       = p_axil.ar.valid,
            i_p_axi_arready       = p_axil.ar.ready,
            o_p_axi_araddr        = p_axil.ar.addr ,
            o_p_axi_arprot        = p_axil.ar.prot ,
            i_p_axi_rvalid        = p_axil.r.valid ,
            o_p_axi_rready        = p_axil.r.ready ,
            i_p_axi_rresp         = p_axil.r.resp  ,
            i_p_axi_rdata         = p_axil.r.data  ,
            o_ibus_axi_awvalid    = ibus64_axi.aw.valid ,
            i_ibus_axi_awready    = ibus64_axi.aw.ready ,
            o_ibus_axi_awaddr     = ibus64_axi.aw.addr  ,
            o_ibus_axi_awburst    = ibus64_axi.aw.burst ,
            o_ibus_axi_awlen      = ibus64_axi.aw.len   ,
            o_ibus_axi_awsize     = ibus64_axi.aw.size  ,
            o_ibus_axi_awlock     = ibus64_axi.aw.lock  ,
            o_ibus_axi_awprot     = ibus64_axi.aw.prot  ,
            o_ibus_axi_awcache    = ibus64_axi.aw.cache ,
            o_ibus_axi_awqos      = ibus64_axi.aw.qos   ,
            o_ibus_axi_awregion   = ibus64_axi.aw.region,
            o_ibus_axi_awid       = ibus64_axi.aw.id    ,
            #o_ibus_axi_awdest     = ibus64_axi.aw.dest  ,
            o_ibus_axi_awuser     = ibus64_axi.aw.user  ,
            o_ibus_axi_wvalid     = ibus64_axi.w.valid  ,
            i_ibus_axi_wready     = ibus64_axi.w.ready  ,
            o_ibus_axi_wlast      = ibus64_axi.w.last   ,
            o_ibus_axi_wdata      = ibus64_axi.w.data   ,
            o_ibus_axi_wstrb      = ibus64_axi.w.strb   ,
            #o_ibus_axi_wid        = ibus64_axi.w.id     ,
            #o_ibus_axi_wdest      = ibus64_axi.w.dest   ,
            o_ibus_axi_wuser      = ibus64_axi.w.user   ,
            i_ibus_axi_bvalid     = ibus64_axi.b.valid  ,
            o_ibus_axi_bready     = ibus64_axi.b.ready  ,
            i_ibus_axi_bresp      = ibus64_axi.b.resp   ,
            i_ibus_axi_bid        = ibus64_axi.b.id     ,
            #i_ibus_axi_bdest      = ibus64_axi.b.dest   ,
            i_ibus_axi_buser      = ibus64_axi.b.user   ,
            o_ibus_axi_arvalid    = ibus64_axi.ar.valid ,
            i_ibus_axi_arready    = ibus64_axi.ar.ready ,
            o_ibus_axi_araddr     = ibus64_axi.ar.addr  ,
            o_ibus_axi_arburst    = ibus64_axi.ar.burst ,
            o_ibus_axi_arlen      = ibus64_axi.ar.len   ,
            o_ibus_axi_arsize     = ibus64_axi.ar.size  ,
            o_ibus_axi_arlock     = ibus64_axi.ar.lock  ,
            o_ibus_axi_arprot     = ibus64_axi.ar.prot  ,
            o_ibus_axi_arcache    = ibus64_axi.ar.cache ,
            o_ibus_axi_arqos      = ibus64_axi.ar.qos   ,
            o_ibus_axi_arregion   = ibus64_axi.ar.region,
            o_ibus_axi_arid       = ibus64_axi.ar.id    ,
            #o_ibus_axi_ardest     = ibus64_axi.ar.dest  ,
            o_ibus_axi_aruser     = ibus64_axi.ar.user  ,
            i_ibus_axi_rvalid     = ibus64_axi.r.valid  ,
            o_ibus_axi_rready     = ibus64_axi.r.ready  ,
            i_ibus_axi_rlast      = ibus64_axi.r.last   ,
            i_ibus_axi_rresp      = ibus64_axi.r.resp   ,
            i_ibus_axi_rdata      = ibus64_axi.r.data   ,
            i_ibus_axi_rid        = ibus64_axi.r.id     ,
            #i_ibus_axi_rdest      = ibus64_axi.r.dest   ,
            i_ibus_axi_ruser      = ibus64_axi.r.user   ,
            o_dbus_axi_awvalid    = dbus_axi.aw.valid ,
            i_dbus_axi_awready    = dbus_axi.aw.ready ,
            o_dbus_axi_awaddr     = dbus_axi.aw.addr  ,
            o_dbus_axi_awburst    = dbus_axi.aw.burst ,
            o_dbus_axi_awlen      = dbus_axi.aw.len   ,
            o_dbus_axi_awsize     = dbus_axi.aw.size  ,
            o_dbus_axi_awlock     = dbus_axi.aw.lock  ,
            o_dbus_axi_awprot     = dbus_axi.aw.prot  ,
            o_dbus_axi_awcache    = dbus_axi.aw.cache ,
            o_dbus_axi_awqos      = dbus_axi.aw.qos   ,
            o_dbus_axi_awregion   = dbus_axi.aw.region,
            o_dbus_axi_awid       = dbus_axi.aw.id    ,
            #o_dbus_axi_awdest     = dbus_axi.aw.dest  ,
            o_dbus_axi_awuser     = dbus_axi.aw.user  ,
            o_dbus_axi_wvalid     = dbus_axi.w.valid  ,
            i_dbus_axi_wready     = dbus_axi.w.ready  ,
            o_dbus_axi_wlast      = dbus_axi.w.last   ,
            o_dbus_axi_wdata      = dbus_axi.w.data   ,
            o_dbus_axi_wstrb      = dbus_axi.w.strb   ,
            #o_dbus_axi_wid        = dbus_axi.w.id     ,
            #o_dbus_axi_wdest      = dbus_axi.w.dest  ,
            o_dbus_axi_wuser      = dbus_axi.w.user  ,
            i_dbus_axi_bvalid     = dbus_axi.b.valid  ,
            o_dbus_axi_bready     = dbus_axi.b.ready  ,
            i_dbus_axi_bresp      = dbus_axi.b.resp   ,
            i_dbus_axi_bid        = dbus_axi.b.id     ,
            #i_dbus_axi_bdest      = dbus_axi.b.dest  ,
            i_dbus_axi_buser      = dbus_axi.b.user  ,
            o_dbus_axi_arvalid    = dbus_axi.ar.valid ,
            i_dbus_axi_arready    = dbus_axi.ar.ready ,
            o_dbus_axi_araddr     = dbus_axi.ar.addr  ,
            o_dbus_axi_arburst    = dbus_axi.ar.burst ,
            o_dbus_axi_arlen      = dbus_axi.ar.len   ,
            o_dbus_axi_arsize     = dbus_axi.ar.size  ,
            o_dbus_axi_arlock     = dbus_axi.ar.lock  ,
            o_dbus_axi_arprot     = dbus_axi.ar.prot  ,
            o_dbus_axi_arcache    = dbus_axi.ar.cache ,
            o_dbus_axi_arqos      = dbus_axi.ar.qos   ,
            o_dbus_axi_arregion   = dbus_axi.ar.region,
            o_dbus_axi_arid       = dbus_axi.ar.id    ,
            #o_dbus_axi_ardest     = dbus_axi.ar.dest  ,
            o_dbus_axi_aruser     = dbus_axi.ar.user  ,
            i_dbus_axi_rvalid     = dbus_axi.r.valid  ,
            o_dbus_axi_rready     = dbus_axi.r.ready  ,
            i_dbus_axi_rlast      = dbus_axi.r.last   ,
            i_dbus_axi_rresp      = dbus_axi.r.resp   ,
            i_dbus_axi_rdata      = dbus_axi.r.data   ,
            i_dbus_axi_rid        = dbus_axi.r.id     ,
            #i_dbus_axi_rdest      = dbus_axi.r.dest  ,
            i_dbus_axi_ruser      = dbus_axi.r.user  ,
            i_jtag_tdi            = jtag_cpu.tdi      ,
            o_jtag_tdo            = jtag_cpu.tdo      ,
            i_jtag_tms            = jtag_cpu.tms      ,
            i_jtag_tck            = jtag_cpu.tck      ,
            i_jtag_trst           = jtag_cpu.trst     ,

            o_coreuser            = sim.coreuser      ,
            i_irqarray_bank0      = self.irqtest0.fields.trigger | irq0_wire_or,
            i_irqarray_bank1      = self.irqtest1.fields.trigger,
            i_irqarray_bank2      = zero_irq,
            i_irqarray_bank3      = zero_irq,
            i_irqarray_bank4      = zero_irq,
            i_irqarray_bank5      = zero_irq,
            i_irqarray_bank6      = zero_irq,
            i_irqarray_bank7      = zero_irq,
            i_irqarray_bank8      = zero_irq,
            i_irqarray_bank9      = zero_irq,
            i_irqarray_bank10      = zero_irq,
            i_irqarray_bank11      = zero_irq,
            i_irqarray_bank12      = zero_irq,
            i_irqarray_bank13      = zero_irq,
            i_irqarray_bank14      = zero_irq,
            i_irqarray_bank15      = zero_irq,
            i_irqarray_bank16      = zero_irq,
            i_irqarray_bank17      = zero_irq,
            i_irqarray_bank18      = zero_irq,
            i_irqarray_bank19      = wfi_loopback,

            o_wfi_active           = wfi_active,
        )

    def add_custom_ram(self, custom_bus, name, origin, size, contents=[], mode="rwx"):
        ram_cls = {
            "wishbone": wishbone.SRAM,
            "axi-lite": axi.AXILiteSRAM,
            "axi"     : axi.AXILiteSRAM, # FIXME: Use AXI-Lite for now, create AXISRAM.
        }[custom_bus.standard]
        interface_cls = {
            "wishbone": wishbone.Interface,
            "axi-lite": axi.AXILiteInterface,
            "axi"     : axi.AXILiteInterface, # FIXME: Use AXI-Lite for now, create AXISRAM.
        }[custom_bus.standard]
        ram_bus = interface_cls(
            data_width    = custom_bus.data_width,
            address_width = custom_bus.address_width,
            bursting      = custom_bus.bursting
        )
        ram     = ram_cls(size, bus=ram_bus, init=contents, read_only=("w" not in mode), name=name)
        custom_bus.add_slave(name, ram.bus, SoCRegion(origin=origin, size=size, mode=mode))
        self.check_if_exists(name)
        self.logger.info("RAM {} {} {}.".format(
            colorer(name),
            colorer("added", color="green"),
            custom_bus.regions[name]))
        setattr(self, name, ram)
        if contents != []:
            self.add_config(f"{name}_INIT", 1)

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
    # Analyzer.
    parser.add_argument("--with-analyzer",        action="store_true",     help="Enable Analyzer support.")

    # Debug/Waveform.
    parser.add_argument("--sim-debug",            action="store_true",     help="Add simulation debugging modules.")
    parser.add_argument("--gtkwave-savefile",     action="store_true",     help="Generate GTKWave savefile.")
    parser.add_argument("--non-interactive",      action="store_true",     help="Run simulation without user input.")

    # Build just the SVDs
    parser.add_argument("--svd-only",             action="store_true",     help="Just build the SVDs for the OS build")

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

    ##### second pass to build the actual chip. Note any changes below need to be reflected into the first pass...might be a good idea to modularize that
    ##### define the soc
    bios_path = '..{}xous-cramium{}simspi.init'.format(os.path.sep, os.path.sep)
    soc = CramSoC(
        bios_path=bios_path,
        sys_clk_freq=800e6,
        sim_debug          = args.sim_debug,
        trace_reset_on     = False,
        **soc_kwargs
    )

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
        shutil.copy('./VexRiscv/src/VexRiscv_CramSoC.v_toplevel_memory_AesPlugin_rom_storage.bin', './build/sim/gateware/')
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
