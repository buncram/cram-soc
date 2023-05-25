#!/usr/bin/env python3
#
# Copyright (c) 2022 Cramium Labs, Inc.
# Derived from litex_soc_gen.py:
# Copyright (c) 2022 Florent Kermarrec <florent@enjoy-digital.fr>
# SPDX-License-Identifier: BSD-2-Clause

import argparse
from pathlib import Path

from migen import *
from migen.genlib.cdc import MultiReg
from litex.soc.interconnect import stream
from litex.build.generic_platform import *

from litex.soc.integration.soc_core import *
from litex.soc.integration.soc import SoCRegion, SoCIORegion
from litex.soc.integration.builder import *
from litex.soc.interconnect import wishbone
from litex.soc.interconnect import axi

from litex.build.xilinx import XilinxPlatform, VivadoProgrammer
from litex.soc.cores.clock import S7MMCM, S7IDELAYCTRL
from litex.soc.cores.spi_opi import S7SPIOPI
from migen.genlib.resetsync import AsyncResetSynchronizer
from litex.soc.interconnect.csr import *

from litex.soc.interconnect.axi import AXIInterface, AXILiteInterface
from litex.soc.interconnect.axi import AXILite2Wishbone
from axi_axil_adapter import AXI2AXILiteAdapter
from litex.soc.integration.soc import SoCBusHandler
from litex.soc.integration.doc import AutoDoc, ModuleDoc
from litex.soc.cores import uart

from deps.gateware.gateware import memlcd
from deps.gateware.gateware import sram_32_cached
from deps.gateware.gateware import sram_32

from axi_crossbar import AXICrossbar
from axil_crossbar import AXILiteCrossbar
from axi_adapter import AXIAdapter
from axi_ram import AXIRAM
from axi_common import *

from axil_ahb_adapter import AXILite2AHBAdapter
from litex.soc.interconnect import ahb

import subprocess


VEX_VERILOG_PATH = "VexRiscv/VexRiscv_CramSoC.v"

# Equivalent to the powershell Get-Command, and kinda like `which`
def get_command(cmd):
    if os.name == 'nt':
        path_ext = os.environ["PATHEXT"].split(os.pathsep)
    else:
        path_ext = [""]
    for ext in path_ext:
        for path in os.environ["PATH"].split(os.pathsep):
            if os.path.exists(path + os.path.sep + cmd + ext):
                return path + os.path.sep + cmd + ext
    return None

def check_vivado():
    vivado_path = get_command("vivado")
    if vivado_path == None:
        # Look for the default Vivado install directory
        if os.name == 'nt':
            base_dir = r"C:\Xilinx\Vivado"
        else:
            base_dir = "/opt/Xilinx/Vivado"
        if os.path.exists(base_dir):
            for file in os.listdir(base_dir):
                bin_dir = base_dir + os.path.sep + file + os.path.sep + "bin"
                if os.path.exists(bin_dir + os.path.sep + "vivado"):
                    os.environ["PATH"] += os.pathsep + bin_dir
                    vivado_path = bin_dir
                    break
    if vivado_path == None:
        return (False, "toolchain not found in your PATH", "download it from https://www.xilinx.com/support/download.html")
    return (True, "found at {}".format(vivado_path))

# IOs ----------------------------------------------------------------------------------------------

_io = [
    # Clk / Rst.
    ("clk12", 0, Pins("R3"), IOStandard("LVCMOS33")),
    ("lpclk", 0, Pins("N15"), IOStandard("LVCMOS33")),
    ("reset", 0, Pins(1)),

    ("jtag", 0,
         Subsignal("tck", Pins("U11"), IOStandard("LVCMOS33")),
         Subsignal("tms", Pins("P6"), IOStandard("LVCMOS33")),
         Subsignal("tdi", Pins("P7"), IOStandard("LVCMOS33")),
         Subsignal("tdo", Pins("R6"), IOStandard("LVCMOS33")),
    ),

    # mapped to GPIOs 0-4
    ("jtag_cpu", 0,
         Subsignal("tck", Pins("F15"), IOStandard("LVCMOS33")),
         Subsignal("tms", Pins("F14"), IOStandard("LVCMOS33")),
         Subsignal("tdi", Pins("E6"), IOStandard("LVCMOS33")),
         Subsignal("tdo", Pins("G15"), IOStandard("LVCMOS33")),
         Subsignal("trst", Pins("H15"), IOStandard("LVCMOS33")),
         Misc("SLEW=SLOW"),
    ),

    ("serial", 0, # wired to the RPi
        Subsignal("tx", Pins("V6")),
        Subsignal("rx", Pins("V7"), Misc("PULLUP True")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
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

    # SPI Flash
    ("spiflash_1x", 0, # clock needs to be accessed through STARTUPE2
        Subsignal("cs_n", Pins("M13")),
        Subsignal("copi", Pins("K17")),
        Subsignal("cipo", Pins("K18")),
        Subsignal("wp",   Pins("L14")), # provisional
        Subsignal("hold", Pins("M15")), # provisional
        IOStandard("LVCMOS33")
    ),
    ("spiflash_8x", 0, # clock needs a separate override to meet timing
        Subsignal("cs_n", Pins("M13")),
        Subsignal("dq",   Pins("K17 K18 L14 M15 L17 L18 M14 N14")),
        Subsignal("dqs",  Pins("R14")),
        Subsignal("ecs_n", Pins("L16")),
        Subsignal("sclk", Pins("C12")),  # DVT
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
     ),

    # SRAM
    ("sram", 0,
        Subsignal("adr", Pins(
            "V12 M5 P5 N4  V14 M3 R17 U15",
            "M4  L6 K3 R18 U16 K1 R5  T2",
            "U1  N1 L5 K2  M18 T6"),
            IOStandard("LVCMOS33"),
            Misc("SLEW=SLOW"),
        ),
        Subsignal("ce_n", Pins("V5"),  IOStandard("LVCMOS33"), Misc("PULLUP True")),
        Subsignal("oe_n", Pins("U12"), IOStandard("LVCMOS33"), Misc("PULLUP True")),
        Subsignal("we_n", Pins("K4"),  IOStandard("LVCMOS33"), Misc("PULLUP True")),
        Subsignal("zz_n", Pins("V17"), IOStandard("LVCMOS33"), Misc("PULLUP True"), Misc("SLEW=SLOW")),
        Subsignal("d", Pins(
            "M2  R4  P2  L4  L1  M1  R1  P1",
            "U3  V2  V4  U2  N2  T1  K6  J6",
            "V16 V15 U17 U18 P17 T18 P18 M17",
            "N3  T4  V13 P15 T14 R15 T3  R7"),
            IOStandard("LVCMOS33"),
            Misc("SLEW=SLOW"),
        ),
        Subsignal("dm_n", Pins("V3 R2 T5 T13"), IOStandard("LVCMOS33")),
    ),

    ("analog", 0,
     Subsignal("usbdet_p", Pins("C3"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("usbdet_n", Pins("A3"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("vbus_div", Pins("C4"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("noise0", Pins("C5"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("noise1", Pins("A8"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("gpio2", Pins("E16"), IOStandard("LVCMOS33")), # PVT2
     Subsignal("gpio5", Pins("D7"), IOStandard("LVCMOS33")),  # PVT2
     # diff grounds
     Subsignal("usbdet_p_n", Pins("B3"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("usbdet_n_n", Pins("A2"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("vbus_div_n", Pins("B4"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("noise0_n", Pins("B5"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("noise1_n", Pins("A7"), IOStandard("LVCMOS33")),  # DVT
     Subsignal("gpio2_n", Pins("E17"), IOStandard("LVCMOS33")), # PVT2
     Subsignal("gpio5_n", Pins("C7"), IOStandard("LVCMOS33")),  # PVT2
     # dedicated pins (no I/O standard applicable)
     Subsignal("ana_vn", Pins("K9")),
     Subsignal("ana_vp", Pins("J10")),
     ),

    ("noise", 0,
     Subsignal("noisebias_on", Pins("H14"), IOStandard("LVCMOS33")),  # PVT2
     # Noise generator
     Subsignal("noise_on", Pins("P14 R13"), IOStandard("LVCMOS33")),
     ),

    ("pio", 0,
        # 32 bogus signals to satisfy vivado
        Subsignal("gpio", Pins(
            "B13 A13 A14 A15 B16 A16 C14 B17",
            "A17 C17 B18 D16 D17 C18 E14 E15",
            "F13 K16 J16 H13 K14 J15 J13 K13",
            "A6 J5 D6 E5 D5 F4 E4 E1"
        ), IOStandard("LVCMOS33")
        ),
    ),

     # Simulation "I/O"
     ("sim", 0,
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
     ),

    # Simulation UART log
    ("sim_uart", 0,
        Subsignal("kernel", Pins(8)),
        Subsignal("kernel_valid", Pins(1)),
        Subsignal("log", Pins(8)),
        Subsignal("log_valid", Pins(1)),
        Subsignal("app", Pins(8)),
        Subsignal("app_valid", Pins(1)),
    )
]

# CRG ----------------------------------------------------------------------------------------------

class CRG(Module):
    def __init__(self, platform, sys_clk_freq, spinor_edge_delay_ns=2.5, sim=False):
        self.warm_reset = Signal()
        self.power_down = Signal()
        self.crypto_on = Signal()

        self.clock_domains.cd_sys   = ClockDomain()
        self.clock_domains.cd_spi   = ClockDomain()
        self.clock_domains.cd_lpclk = ClockDomain()
        self.clock_domains.cd_spinor = ClockDomain()
        self.clock_domains.cd_clk200 = ClockDomain()
        self.clock_domains.cd_clk50 = ClockDomain()
        self.clock_domains.cd_usb_48 = ClockDomain()
        self.clock_domains.cd_usb_12 = ClockDomain()
        self.clock_domains.cd_raw_12 = ClockDomain()

        self.clock_domains.cd_clk200_crypto = ClockDomain()
        self.clock_domains.cd_sys_crypto = ClockDomain()
        self.clock_domains.cd_sys_always_on = ClockDomain()
        self.clock_domains.cd_clk50_always_on = ClockDomain()

        # # #

        sysclk_ns = 1e9 / sys_clk_freq
        # convert delay request in ns to degrees, where 360 degrees is one whole clock period
        phase_f = (spinor_edge_delay_ns / sysclk_ns) * 360
        # round phase to the nearest multiple of 7.5 (needs to be a multiple of 45 / CLKOUT2_DIVIDE = 45 / 6 = 7.5
        # note that CLKOUT2_DIVIDE is automatically calculated by mmcm.create_clkout() below
        phase = round(phase_f / 7.5) * 7.5

        clk32khz = platform.request("lpclk")
        self.specials += Instance("BUFG", i_I=clk32khz, o_O=self.cd_lpclk.clk)
        platform.add_platform_command("create_clock -name lpclk -period {:0.3f} [get_nets lpclk]".format(1e9 / 32.768e3))

        clk12 = platform.request("clk12")
        # Note: below feature cannot be used because Litex appends this *after* platform commands! This causes the generated
        # clock derived constraints immediately below to fail, because .xdc file is parsed in-order, and the main clock needs
        # to be created before the derived clocks. Instead, we use the line afterwards.
        # platform.add_period_constraint(clk12, 1e9 / 12e6)
        platform.add_platform_command("create_clock -name clk12 -period {:0.3f} [get_nets clk12]".format(1e9 / 12e6))
        # The above constraint must strictly proceed the below create_generated_clock constraints in the .XDC file

        # This allows PLLs/MMCMEs to be placed anywhere and reference the input clock
        self.clk12_bufg = Signal()
        self.specials += Instance("BUFG", i_I=clk12, o_O=self.clk12_bufg)
        self.comb += self.cd_raw_12.clk.eq(self.clk12_bufg)

        self.submodules.mmcm = mmcm = S7MMCM(speedgrade=-1)
        mmcm.register_clkin(self.clk12_bufg, 12e6)
        # we count on clocks being assigned to the MMCME2_ADV in order. If we make more MMCME2 or shift ordering, these constraints must change.
        mmcm.create_clkout(self.cd_usb_48, 48e6, with_reset=False, buf="bufgce", ce=mmcm.locked) # 48 MHz for USB; always-on
        platform.add_platform_command("create_generated_clock -name usb_48 [get_pins MMCME2_ADV/CLKOUT0]")

        mmcm.create_clkout(self.cd_spi, 20e6, with_reset=False, buf="bufgce", ce=mmcm.locked & ~self.power_down)
        platform.add_platform_command("create_generated_clock -name spi_clk [get_pins MMCME2_ADV/CLKOUT1]")

        mmcm.create_clkout(self.cd_spinor, sys_clk_freq, phase=phase, with_reset=False, buf="bufgce", ce=mmcm.locked & ~self.power_down)  # delayed version for SPINOR cclk (different from COM SPI above)
        platform.add_platform_command("create_generated_clock -name spinor [get_pins MMCME2_ADV/CLKOUT2]")

        # clk200 does not gate off because we want to keep the IDELAYCTRL block "warm"
        mmcm.create_clkout(self.cd_clk200, 200e6, with_reset=False, buf="bufg",
            gated_replicas={self.cd_clk200_crypto : (mmcm.locked & (~self.power_down | self.crypto_on))}) # 200MHz always-on required for IDELAYCTL
        platform.add_platform_command("create_generated_clock -name clk200 [get_pins MMCME2_ADV/CLKOUT3]")

        # clk50 is explicitly for the crypto unit, so it doesn't have the _crypto suffix, consfusingly...
        mmcm.create_clkout(self.cd_clk50, 50e6, with_reset=False, buf="bufgce", ce=(mmcm.locked & (~self.power_down | self.crypto_on)),
            gated_replicas={self.cd_clk50_always_on: mmcm.locked}) # 50MHz for ChaCha conditioner, attached to the always-on TRNG
        platform.add_platform_command("create_generated_clock -name clk50 [get_pins MMCME2_ADV/CLKOUT4]")

        mmcm.create_clkout(self.cd_usb_12, 12e6, with_reset=False, buf="bufgce", ce=mmcm.locked) # 12 MHz for USB; always-on
        platform.add_platform_command("create_generated_clock -name usb_12 [get_pins MMCME2_ADV/CLKOUT5]")

        # needs to be exactly 100MHz hence margin=0
        mmcm.create_clkout(self.cd_sys, sys_clk_freq, margin=0, with_reset=False, buf="bufgce", ce=(~self.power_down & mmcm.locked),
            gated_replicas={self.cd_sys_crypto : (mmcm.locked & (~self.power_down | self.crypto_on)), self.cd_sys_always_on : mmcm.locked})
        platform.add_platform_command("create_generated_clock -name sys_clk [get_pins MMCME2_ADV/CLKOUT6]")

        # timing to the "S" pins is not sensitive because we don't care if there is an extra clock pulse relative
        # to the gating. Glitch-free operation is guaranteed regardless!
        platform.add_platform_command('set_false_path -through [get_pins BUFGCTRL*/S*]')
        # platform.add_platform_command('set_false_path -through [get_nets vns_rst_meta*]') # fixes for a later version of vivado

        self.ignore_locked = Signal()
        reset_combo = Signal()
        if sim:
            self.comb += reset_combo.eq(self.warm_reset | (~mmcm.locked & ~self.ignore_locked) | platform.request("reset"))
        else:
            self.comb += reset_combo.eq(self.warm_reset | (~mmcm.locked & ~self.ignore_locked))
        # See https://forums.xilinx.com/t5/Other-FPGA-Architecture/MMCM-Behavior-After-Its-PWRDWN-Port-Is-Asserted-and-Then/td-p/792324
        # "The DRP functional logic itself does not behave differently for PWRDWN or RST.
        # The "registers" programmed previously through the DRP (or any other once) are not affected either
        # way because they are configuration cells and are only overwritten if you re-program the part or
        # by another DRP operation. Typically, from an application perspective, PWRDWN and RST are identical.
        # The difference is obviously that in the PWRDWN case the MMCM completely shuts down for an extended period
        # of time even if asserted only briefly. Takes a while to bring back the regulators vs simply reset.
        # In addition, since power is turned of, it takes longer to reacquire LOCK vs RST because the VCO starts from scratch."
        #self.comb += mmcm.reset.eq(self.power_down)
        #self.comb += mmcm.power_down.eq(self.power_down)
        self.specials += [
            AsyncResetSynchronizer(self.cd_usb_48, reset_combo),
            AsyncResetSynchronizer(self.cd_spi, reset_combo),
            AsyncResetSynchronizer(self.cd_spinor, reset_combo),
            AsyncResetSynchronizer(self.cd_clk200, reset_combo),
            AsyncResetSynchronizer(self.cd_clk50, reset_combo),
            AsyncResetSynchronizer(self.cd_usb_12, reset_combo),
            AsyncResetSynchronizer(self.cd_sys, reset_combo),

            AsyncResetSynchronizer(self.cd_clk200_crypto, reset_combo),
            AsyncResetSynchronizer(self.cd_sys_crypto, reset_combo),

            AsyncResetSynchronizer(self.cd_sys_always_on, reset_combo),
            AsyncResetSynchronizer(self.cd_clk50_always_on, reset_combo),
        ]

        # Add an IDELAYCTRL primitive for the SpiOpi block
        self.submodules += S7IDELAYCTRL(self.cd_clk200, reset_cycles=32) # 155ns @ 200MHz, min 59.28ns


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
    def __init__(self, data_in, data_valid):
        self.source = source = stream.Endpoint([("data", 8)])

        # # #
        self.comb += [
            source.valid.eq(data_valid),
            source.data.eq(data_in),
        ]

# Simulation UART ----------------------------------------------------------------------------------
class SimUartPhy(Module, AutoCSR):
    def __init__(self, data_in, data_in_valid, data_out, data_out_valid, clk_freq, baudrate=115200, with_dynamic_baudrate=False):
        tuning_word = int((baudrate/clk_freq)*2**32)
        if with_dynamic_baudrate:
            self._tuning_word  = CSRStorage(32, reset=tuning_word)
            tuning_word = self._tuning_word.storage
        self.submodules.tx = SimPhyTx(data_out, data_out_valid)
        self.submodules.rx = SimPhyRx(data_in, data_in_valid)
        self.sink, self.source = self.tx.sink, self.rx.source


# System constants ---------------------------------------------------------------------------------

boot_offset    = 0x500000 # enough space to hold 2x FPGA bitstreams before the firmware start
bios_size      = 0x10000
SPI_FLASH_SIZE = 128 * 1024 * 1024
SRAM_EXT_SIZE  = 0x1000000

# CramSoC ------------------------------------------------------------------------------------------

class CramSoC(SoCMini):
    mem_map = {**SoCCore.mem_map, **{
        "csr": 0x4010_0000,  # reserve first 0x10_0000 for interop with Cramium SoC
    }}
    #csr_map = {  # leave holes in the CSR map at offsets indexed by page
    #    "interop"  : 0,
    # 1 is allocateable
    #    "interop1" : 2,
    #}
    def __init__(self, platform, bios_path=None, sys_clk_freq=75e6, sim=False, litex_axi=False, real_ram=True, cached=True):
        axi_map = {
            "spiflash"  : 0x20000000,
            "reram"     : 0x6000_0000, # +3M
            "sram"      : 0x6100_0000, # +2M
            "p_bus"     : 0x4000_0000, # +256M
            "memlcd"    : 0x4200_0000,
            "vexriscv_debug": 0xefff_0000,
        }
        self.platform = platform

        # Clockgen cluster -------------------------------------------------------------------------
        warm_reset = Signal()
        self.submodules.crg = CRG(platform, sys_clk_freq, spinor_edge_delay_ns=2.5, sim=sim)
        self.comb += self.crg.warm_reset.eq(warm_reset) # mirror signal here to hit the Async reset injectors
        # lpclk/sys paths are async
        self.platform.add_platform_command('set_clock_groups -asynchronous -group [get_clocks sys_clk] -group [get_clocks lpclk]')
        # 12 always-on/sys paths are async
        self.platform.add_platform_command('set_clock_groups -asynchronous -group [get_clocks sys_clk] -group [get_clocks clk12]')

        # Add standalone SoC sources.
        platform.add_source("build/gateware/cram_axi.v")
        platform.add_source(VEX_VERILOG_PATH)
        platform.add_source("sim_support/ram_1w_1ra.v")
        platform.add_source("sim_support/ram_1w_1rs.v")
        platform.add_source("sim_support/prims.v")

        # this must be pulled in manually because it's instantiated in the core design, but not in the SoC design
        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "verilog-axi", "rtl")
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter.v"))
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter_wr.v"))
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter_rd.v"))

        #platform.add_source("build/femtorv_soc/gateware/femtorv_soc.v")
        #platform.add_source("build/femtorv_soc/gateware/femtorv_soc_rom.init", copy=True)

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
        self.add_memory_region(name="sram", origin=axi_map["sram"], length=16*1024*1024)

        # Wire up peripheral SoC busses
        p_axi = axi.AXILiteInterface(name="pbus")
        jtag_cpu = platform.request("jtag_cpu")

        # Add simulation "output pins" -----------------------------------------------------
        self.sim_report = CSRStorage(32, name = "report", description="A 32-bit value to report sim state")
        self.sim_success = CSRStorage(1, name = "success", description="Determines the result code for the simulation. 0 means fail, 1 means pass")
        self.sim_done = CSRStorage(1, name ="done", description="Set to `1` if the simulation should auto-terminate")
        # test that caching is OFF for the I/O regions
        self.sim_coherence_w = CSRStorage(32, name= "wdata", description="Write values here to check cache coherence issues")
        self.sim_coherence_r = CSRStatus(32, name="rdata", description="Data readback derived from coherence_w")
        self.sim_coherence_inc = CSRStatus(32, name="rinc", description="Every time this is read, the base value is incremented by 3", reset=0)

        # work around AXIL->CSR bugs in Litex. The spec says that "we" should be a single pulse. But,
        # it seems that the AXIL->CSR adapter will happily generate a longer pulse. Seems to have to do with
        # some "clever hack" that was done to adapt AXIL to simple csrs, where axi_lite_to_simple() inside axi_lite.py
        # is not your usual Module but some function that returns a tuple of FSMs and combs to glom into the parent
        # object. But because of this everything in it has to be computed in just one cycle, but actually it seems
        # that this causes the "do_read" to trigger a cycle earlier than the FSM's state, which later on gets
        # OR'd together to create a 2-long cycle for WE, violating the CSR spec. Moving "do_read" back a cycle doesn't
        # quite fix it because you also need to gate off the "adr" signal, and I can't seem to find that code.
        # Anyways, this is a Litex-specific bug, so I'm not going to worry about it for SoC integration simulations.
        sim_coherence_axil_bug = Signal()
        self.sync += [
            sim_coherence_axil_bug.eq(self.sim_coherence_inc.we),
            If(self.sim_coherence_inc.we & ~sim_coherence_axil_bug,
                self.sim_coherence_inc.status.eq(self.sim_coherence_inc.status + 3)
            ).Else(
                self.sim_coherence_inc.status.eq(self.sim_coherence_inc.status)
            )
        ]
        self.comb += [
            self.sim_coherence_r.status.eq(self.sim_coherence_w.storage + 5)
        ]
        if sim:
            sim_pads = platform.request("sim")
            self.comb += [
                sim_pads.report.eq(self.sim_report.storage),
                sim_pads.success.eq(self.sim_success.storage),
                sim_pads.done.eq(self.sim_done.storage),
            ]


        # Add AXI RAM to SoC (Through AXI Crossbar).
        # ------------------------------------------

        # 1) Create AXI interface and connect it to SoC.
        dbus_axi = AXIInterface(data_width=32, address_width=32, id_width=1, bursting=True)
        ibus32_axi = AXIInterface(data_width=32, address_width=32, id_width=1, bursting=True)
        ibus64_axi = AXIInterface(data_width=64, address_width=32, id_width=1, bursting=True)
        self.submodules += AXIAdapter(platform, s_axi = ibus64_axi, m_axi = ibus32_axi, convert_burst=True, convert_narrow_burst=True)

        # 2) Add 2 X AXILiteSRAM to emulate ReRAM and SRAM; much smaller now just for testing
        if bios_path is not None:
            with open(bios_path, 'rb') as bios:
                bios_data = bios.read()
        else:
            bios_data = []

        # SPI flash controller ---------------------------------------------------------------------
        reram_axi = AXIInterface(data_width=32, address_width=32, id_width=2, bursting=True)
        reram_axil = AXILiteInterface(data_width=32, address_width=32, bursting=False)
        self.submodules += AXI2AXILiteAdapter(platform, reram_axi, reram_axil)
        sclk_instance_name="SCLK_ODDR"
        iddr_instance_name="SPI_IDDR"
        cipo_instance_name="CIPO_FDRE"
        spiread=False
        spipads = platform.request("spiflash_8x")
        self.submodules.spinor = S7SPIOPI(spipads,
                sclk_name=sclk_instance_name, iddr_name=iddr_instance_name, cipo_name=cipo_instance_name, spiread=spiread, sim=sim)
        self.spinor.add_timing_constraints(platform, "spiflash_8x")
        self.specials += MultiReg(warm_reset, self.spinor.gsr)

        self.submodules.reram_axi_to_wb = AXILite2Wishbone(reram_axil, self.spinor.bus, base_address=axi_map["reram"])
        self.add_csr("spinor")

        # External SRAM ----------------------------------------------------------------------------
        # Cache fill time is ~320ns for 8 words.
        # smaller cache to reduce resource utilization
        sram_axi = AXIInterface(data_width=32, address_width=32, id_width=2, bursting=True)
        if real_ram:
            sram_axil = AXILiteInterface(data_width=32, address_width=32, bursting=False)
            self.submodules += AXI2AXILiteAdapter(platform, sram_axi, sram_axil)
            if cached:
                self.submodules.sram_ext = sram_32_cached.SRAM32(platform.request("sram"), rd_timing=7, wr_timing=7, page_rd_timing=3, l2_cache_size=0x1_0000)
            else:
                # note: something is broken in this implementation, it doesn't work in simulation. I think it may have bit-rotted and/or doesn't work with the narrow verilog model of the RAM that I made.
                self.submodules.sram_ext = sram_32.SRAM32(platform.request("sram"), rd_timing=7, wr_timing=7, page_rd_timing=3)

            self.add_csr("sram_ext")
            # self.bus.add_slave(name="sram_ext", slave=self.sram_ext.bus, region=SoCRegion(self.mem_map["sram_ext"], size=SRAM_EXT_SIZE))
            # A bit of a bodge -- the path is actually async, so what we are doing is trying to constrain intra-channel skew by pushing them up against clock limits (PS I'm not even sure this works...)
            self.platform.add_platform_command("set_input_delay -clock [get_clocks sys_clk] -min -add_delay 4.0 [get_ports {{sram_d[*]}}]")
            self.platform.add_platform_command("set_input_delay -clock [get_clocks sys_clk] -max -add_delay 9.0 [get_ports {{sram_d[*]}}]")
            self.platform.add_platform_command("set_output_delay -clock [get_clocks sys_clk] -min -add_delay 0.0 [get_ports {{sram_adr[*] sram_d[*] sram_ce_n sram_oe_n sram_we_n sram_zz_n sram_dm_n[*]}}]")
            self.platform.add_platform_command("set_output_delay -clock [get_clocks sys_clk] -max -add_delay 3.0 [get_ports {{sram_adr[*] sram_d[*] sram_ce_n sram_oe_n sram_we_n sram_zz_n sram_dm_n[*]}}]")
            # ODDR falling edge ignore
            self.platform.add_platform_command("set_false_path -fall_from [get_clocks sys_clk] -through [get_ports {{sram_d[*] sram_adr[*] sram_ce_n sram_oe_n sram_we_n sram_zz_n sram_dm_n[*]}}]")
            self.platform.add_platform_command("set_false_path -fall_to [get_clocks sys_clk] -through [get_ports {{sram_d[*]}}]")
            #self.platform.add_platform_command("set_false_path -fall_from [get_clocks sys_clk] -through [get_nets {net}]", net=self.sram_ext.load)
            #self.platform.add_platform_command("set_false_path -fall_to [get_clocks sys_clk] -through [get_nets {net}]", net=self.sram_ext.load)
            self.platform.add_platform_command("set_false_path -rise_from [get_clocks sys_clk] -fall_to [get_clocks sys_clk]")  # sort of a big hammer but should be OK

            # relax OE driver constraint (setup time of data to write enable edge is 23ns only, 70ns total cycle time given)
            self.platform.add_platform_command("set_multicycle_path 2 -setup -through [get_pins {net}_reg/Q]", net=self.sram_ext.sync_oe_n)
            self.platform.add_platform_command("set_multicycle_path 1 -hold -through [get_pins {net}_reg/Q]", net=self.sram_ext.sync_oe_n)

            self.submodules.sram_axi_to_wb = AXILite2Wishbone(sram_axil, self.sram_ext.bus, base_address=axi_map["sram"])
        else:
            self.submodules.axi_sram = AXIRAM(platform, sram_axi, size=2*1024*1024, name="sram")

        # 3) Add AXICrossbar  (2 Slave / 2 Master).
        mbus = AXICrossbar(platform=platform)
        self.submodules += mbus
        mbus.add_slave(name = "dbus", s_axi=dbus_axi,
            aw_reg = AXIRegister.BYPASS,
            w_reg  = AXIRegister.BYPASS,
            b_reg  = AXIRegister.BYPASS,
            ar_reg = AXIRegister.BYPASS,
            r_reg  = AXIRegister.BYPASS,
        )
        mbus.add_slave(name = "ibus", s_axi=ibus32_axi,
            aw_reg = AXIRegister.BYPASS,
            w_reg  = AXIRegister.BYPASS,
            b_reg  = AXIRegister.BYPASS,
            ar_reg = AXIRegister.BYPASS,
            r_reg  = AXIRegister.BYPASS,
        )
        mbus.add_master(name = "reram", m_axi=reram_axi, origin=axi_map["reram"], size=0x0100_0000)
        mbus.add_master(name = "sram",  m_axi=sram_axi,  origin=axi_map["sram"],  size=0x0100_0000)

        # 4) Add peripherals
        # This region is used for testbench elements (e.g., does not appear in the final SoC):
        # these are peripherals that are inferred by LiteX in this module such as the UART to facilitate debug
        testbench_region = SoCIORegion(0x4000_0000, 0x20_0000, mode="rw", cached=False)
        testbench_axil = AXILiteInterface()
        # This region is used for SoC elements (e.g. items in the final SoC that we want to verify,
        # but are not necessarily in their final address offset or topological configuration)
        soc_region = SoCIORegion(0x4020_0000, 0x20_0000, mode="rw", cached=False)
        soc_axil = AXILiteInterface()

        self.submodules.pxbar = pxbar = AXILiteCrossbar(platform)
        pxbar.add_slave(
            name = "p_axil", s_axil = p_axi
        )
        pxbar.add_master(
            name = "testbench",
            m_axil = testbench_axil,
            origin = testbench_region.origin,
            size = testbench_region.size
        )
        pxbar.add_master(
            name = "soc",
            m_axil = soc_axil,
            origin = soc_region.origin,
            size = soc_region.size
        )
        #    masters=[p_axil],
        #    slaves =[(testbench_region.decoder(p_axil), testbench_axil), (soc_region.decoder(p_axil), soc_axil)],
        #    register = False,
        self.bus.add_master(name="pbus", master=testbench_axil)
        # setup p_axi as the local bus master
        #self.bus.add_master(name="pbus", master=p_axi)
        local_ahb = ahb.Interface()
        self.submodules += AXILite2AHBAdapter(platform, soc_axil, local_ahb)
        # from duart_adapter import DuartAdapter
        # self.submodules += DuartAdapter(platform, local_ahb, pads=platform.request("duart"), sel_addr=0x201000)
        from pio_adapter import PioAdapter
        pio_irq0 = Signal()
        pio_irq1 = Signal()
        self.submodules += ClockDomainsRenamer({"pio":"sys"})(
            PioAdapter(platform, local_ahb, platform.request("pio"), pio_irq0, pio_irq1, sel_addr=0x202000, sim=sim))
        # 100->50MHz domain false paths. Should really find a way to exclude the async pulse generators, but for now this will do.
        # platform.add_platform_command("set_multicycle_path 2 -setup -start -from [get_clocks sys_clk] -to [get_clocks clk50] -through [get_cells pio_ahb/rp_pio/*]")
        # platform.add_platform_command("set_multicycle_path 1 -hold -end -from [get_clocks sys_clk] -to [get_clocks clk50] -through [get_cells pio_ahb/rp_pio/*]")
        # 50->100MHz domain false paths
        # platform.add_platform_command("set_multicycle_path 3 -setup -start -from [get_clocks clk50] -to [get_clocks sys_clk] -through [get_cells pio_ahb/rp_pio/*]")
        # platform.add_platform_command("set_multicycle_path 2 -hold -end -from [get_clocks clk50] -to [get_clocks sys_clk] -through [get_cells pio_ahb/rp_pio/*]")

        # add interrupt handler
        interrupt = Signal(32)
        self.cpu.interrupt = interrupt
        self.irq.enable()
        # this interrupt is added now due to the ordering of modules
        self.irq.add("spinor")

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
        if sim:
            sim_uart_pins = platform.request("sim_uart")
            uart_data_in = Signal(8, reset=13) # 13=0xd
            uart_data_valid = Signal(reset = 0)
            self.submodules.uart_phy = SimUartPhy(
                uart_data_in,
                uart_data_valid,
                sim_uart_pins.kernel,
                sim_uart_pins.kernel_valid,
                clk_freq=sys_clk_freq,
                baudrate=115200)
        else:
            self.submodules.uart_phy = uart.UARTPHY(
                pads=kernel_pads,
                clk_freq=sys_clk_freq,
                baudrate=115200)
        self.submodules.uart = ResetInserter()(
            uart.UART(self.uart_phy,
                tx_fifo_depth=16, rx_fifo_depth=16)
            )

        self.add_csr("uart_phy")
        self.add_csr("uart")
        self.irq.add("uart")

        if sim:
            console_data_in = Signal(8, reset=13) # 13=0xd
            console_data_valid = Signal(reset = 0)
            self.submodules.console_phy = SimUartPhy(
                console_data_in,
                console_data_valid,
                sim_uart_pins.log,
                sim_uart_pins.log_valid,
                clk_freq=sys_clk_freq,
                baudrate=115200)
        else:
            self.submodules.console_phy = uart.UARTPHY(
                pads=console_pads,
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
        if sim:
            app_data_in = Signal(8, reset=13) # 13=0xd
            app_data_valid = Signal(reset = 0)
            self.submodules.app_uart_phy = SimUartPhy(
                app_data_in,
                app_data_valid,
                sim_uart_pins.app,
                sim_uart_pins.app_valid,
                clk_freq=sys_clk_freq,
                baudrate=115200)
        else:
            self.submodules.app_uart_phy = uart.UARTPHY(
                pads=app_uart_pads,
                clk_freq=sys_clk_freq,
                baudrate=115200)
        self.submodules.app_uart = ResetInserter()(
            uart.UART(self.app_uart_phy,
                tx_fifo_depth=16, rx_fifo_depth=16)
            )

        self.add_csr("app_uart_phy")
        self.add_csr("app_uart")
        self.irq.add("app_uart")

        # LCD interface ----------------------------------------------------------------------------
        if False:
            self.submodules.memlcd = ClockDomainsRenamer({"sys":"sys_always_on"})(memlcd.MemLCD(platform.request("lcd"), interface="axi-lite"))
            self.add_csr("memlcd")
            self.bus.add_slave("memlcd", self.memlcd.bus, SoCRegion(origin=axi_map["memlcd"], size=self.memlcd.fb_depth*4, mode="rw", cached=False))

        # XADC analog interface---------------------------------------------------------------------
        if ~sim:
            from litex.soc.cores.xadc import analog_layout
            analog_pads = Record(analog_layout)
            analog = platform.request("analog")
            self.comb += [
                analog_pads.vp.eq(analog.ana_vp),
                analog_pads.vn.eq(analog.ana_vn),
            ]
            # use explicit dummies to tie the analog inputs, otherwise the name space during finalization changes
            # (e.g. FHDL adds 'betrustedsoc_' to the beginning of every netlist name to give a prefix to unnamed signals)
            # notet that the added prefix messes up the .XDC constraints
            dummy4 = Signal(4, reset=0)
            dummy1 = Signal(1, reset=0)
            self.comb += analog_pads.vauxp.eq(Cat(dummy4,          # 0,1,2,3
                                                analog.noise1,        # 4
                                                analog.gpio5,         # 5
                                                analog.vbus_div,      # 6
                                                dummy4,               # 7,8,9,10
                                                analog.gpio2,         # 11
                                                analog.noise0,        # 12
                                                dummy1,               # 13
                                                analog.usbdet_p,      # 14
                                                analog.usbdet_n,      # 15
                                        )),
            self.comb += analog_pads.vauxn.eq(Cat(dummy4,          # 0,1,2,3
                                                analog.noise1_n,      # 4
                                                analog.gpio5_n,       # 5
                                                analog.vbus_div_n,    # 6
                                                dummy4,               # 7,8,9,10
                                                analog.gpio2_n,       # 11
                                                analog.noise0_n,      # 12
                                                dummy1,               # 13
                                                analog.usbdet_p_n,    # 14
                                                analog.usbdet_n_n,    # 15
                                        )),

        # Managed TRNG Interface -------------------------------------------------------------------
        if False: # ~sim  commented out for now just for synthesis/timing closure studies
            from deps.gateware.gateware.trng.trng_managed import TrngManaged, TrngManagedKernel, TrngManagedServer
            self.submodules.trng_kernel = ClockDomainsRenamer({"sys":"sys_always_on"})(TrngManagedKernel())
            self.add_csr("trng_kernel")
            self.irq.add("trng_kernel")
            self.submodules.trng_server = ClockDomainsRenamer({"sys":"sys_always_on"})(TrngManagedServer(ro_cores=4))
            self.add_csr("trng_server")
            self.irq.add("trng_server")
            # put the TRNG proper into an always on domain. It has its own power manager and health tests.
            # The TRNG adds about an 8.5mW power burden when it is in standby mode but clocks on
            self.submodules.trng = ClockDomainsRenamer({"sys":"sys_always_on", "clk50":"clk50_always_on"})(
                TrngManaged(platform, analog_pads, platform.request("noise"),
                    server=self.trng_server, kernel=self.trng_kernel, revision='pvt2', ro_cores=4, sim=sim))
            self.add_csr("trng")

        # Internal reset ---------------------------------------------------------------------------
        if sim:
            gsr = Signal()
            self.specials += MultiReg(warm_reset, gsr)
            self.specials += [
                # De-activate the CCLK interface, parallel it with a GPIO
                Instance("STARTUPE2",
                    i_CLK       = 0,
                    i_GSR       = gsr,
                    i_GTS       = 0,
                    i_KEYCLEARB = 1,
                    i_PACK      = 0,
                    i_USRDONEO  = 1,
                    i_USRDONETS = 1,
                    i_USRCCLKO  = 0,
                    i_USRCCLKTS = 1,  # Force to tristate
                    # o_CFGMCLK   = self.cfgmclk,
                ),
            ]

        # Cramium platform -------------------------------------------------------------------------
        trimming_reset = Signal(32, reset=axi_map["reram"])
        zero_irq = Signal(20)
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
        coreuser = Signal()
        if sim:
            self.comb += sim_pads.coreuser.eq(coreuser)
            self.comb += sim_pads.sysclk.eq(ClockSignal())

        # Pull in DUT IP ---------------------------------------------------------------------------
        self.specials += Instance("cram_axi",
            i_aclk                = ClockSignal("sys"),
            i_rst                 = ResetSignal("sys"),
            i_always_on           = ClockSignal("sys"),
            i_trimming_reset      = trimming_reset,
            i_trimming_reset_ena  = 1, # load code from SPI flash directly
            o_p_axi_awvalid       = p_axi.aw.valid,
            i_p_axi_awready       = p_axi.aw.ready,
            o_p_axi_awaddr        = p_axi.aw.addr ,
            o_p_axi_awprot        = p_axi.aw.prot ,
            o_p_axi_wvalid        = p_axi.w.valid ,
            i_p_axi_wready        = p_axi.w.ready ,
            o_p_axi_wdata         = p_axi.w.data  ,
            o_p_axi_wstrb         = p_axi.w.strb  ,
            i_p_axi_bvalid        = p_axi.b.valid ,
            o_p_axi_bready        = p_axi.b.ready ,
            i_p_axi_bresp         = p_axi.b.resp  ,
            o_p_axi_arvalid       = p_axi.ar.valid,
            i_p_axi_arready       = p_axi.ar.ready,
            o_p_axi_araddr        = p_axi.ar.addr ,
            o_p_axi_arprot        = p_axi.ar.prot ,
            i_p_axi_rvalid        = p_axi.r.valid ,
            o_p_axi_rready        = p_axi.r.ready ,
            i_p_axi_rresp         = p_axi.r.resp  ,
            i_p_axi_rdata         = p_axi.r.data  ,
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

            o_coreuser            = coreuser          ,
            i_irqarray_bank0      = self.irqtest0.fields.trigger,
            i_irqarray_bank1      = self.irqtest1.fields.trigger,
            i_irqarray_bank2      = Cat(pio_irq0, pio_irq1, zero_irq[2:]),
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

            o_sleep_req            = wfi_active,
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

class Platform(XilinxPlatform):
    def __init__(self, io, toolchain="vivado", programmer="vivado", encrypt=False, make_mod=False, bbram=False, strategy='default'):
        part = "xc7a100t-csg324-1"
        XilinxPlatform.__init__(self, part, io, toolchain=toolchain)

        if strategy != 'default':
            self.toolchain.vivado_route_directive = strategy
            self.toolchain.vivado_post_route_phys_opt_directive = "Explore"  # always explore if we're in a non-default strategy

        # NOTE: to do quad-SPI mode, the QE bit has to be set in the SPINOR status register. OpenOCD
        # won't do this natively, have to find a work-around (like using iMPACT to set it once)
        self.add_platform_command(
            "set_property CONFIG_VOLTAGE 3.3 [current_design]")
        self.add_platform_command(
            "set_property CFGBVS GND [current_design]")
        self.add_platform_command(
            "set_property BITSTREAM.CONFIG.CONFIGRATE 66 [current_design]")
        self.add_platform_command(
            "set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 1 [current_design]")
        self.toolchain.bitstream_commands = [
            "set_property CONFIG_VOLTAGE 3.3 [current_design]",
            "set_property CFGBVS GND [current_design]",
            "set_property BITSTREAM.CONFIG.CONFIGRATE 66 [current_design]",
            "set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 1 [current_design]",
        ]
        if encrypt:
            type = 'eFUSE'
            if bbram:
                type = 'BBRAM'
            self.toolchain.bitstream_commands += [
                "set_property BITSTREAM.ENCRYPTION.ENCRYPT YES [current_design]",
                "set_property BITSTREAM.ENCRYPTION.ENCRYPTKEYSELECT {} [current_design]".format(type),
                "set_property BITSTREAM.ENCRYPTION.KEYFILE ../../dummy.nky [current_design]"
            ]

        self.toolchain.additional_commands += \
            ["write_cfgmem -verbose -force -format bin -interface spix1 -size 64 "
             "-loadbit \"up 0x0 {build_name}.bit\" -file {build_name}.bin"]
        self.programmer = programmer

        self.toolchain.additional_commands += [
            "report_timing -delay_type min_max -max_paths 100 -slack_less_than 0 -sort_by group -input_pins -routable_nets -name failures -file timing-failures.txt"
        ]

    def create_programmer(self):
        if self.programmer == "vivado":
            return VivadoProgrammer(flash_part="n25q128-1.8v-spi-x1_x2_x4")
        else:
            raise ValueError("{} programmer is not supported".format(self.programmer))

    def do_finalize(self, fragment):
        XilinxPlatform.do_finalize(self, fragment)


# Build --------------------------------------------------------------------------------------------

def main():
    global _io
    # build environment setup
    os.environ["PYTHONHASHSEED"] = "1" # do it manually here
    check_vivado()

    if os.environ['PYTHONHASHSEED'] != "1":
        print( "PYTHONHASHEED must be set to 1 for consistent validation results. Failing to set this results in non-deterministic compilation results")
        return 1
    # used to be -e blank.nky -u debug -x -s NoTimingRelaxation -r pvt2 -p
    # now is -e blank.nky
    parser = argparse.ArgumentParser(description="Build the Cramium OSS SoC")
    parser.add_argument(
        "-D", "--document-only", default=False, action="store_true", help="Build docs only"
    )
    parser.add_argument(
        "-S", "--sim", default=False, action="store_true", help="Run simulation only"
    )
    # debug iteration limit loops with `ptrace on` in GUI before running a step.
    parser.add_argument(
        "-c", "--ci", default=False, action="store_true", help="Run simulation in non-interactive mode. Only valid with -S"
    )
    parser.add_argument(
        "-e", "--encrypt", help="Format output for encryption using the specified dummy key. Image is re-encrypted at sealing time with a secure key.", type=str
    )
    parser.add_argument(
        "-b", "--bbram", help="encrypt to bbram, not efuse. Defaults to efuse. Only meaningful in -e is also specified.", default=False, action="store_true"
    )
    parser.add_argument(
        "-s", "--strategy", choices=['Explore', 'default', 'NoTimingRelaxation'], help="Pick the routing strategy. Defaults to NoTimingRelaxation.", default='NoTimingRelaxation', type=str
    )
    parser.add_argument(
        "--simple-boot", help="Fall back to the simple, unsigned bootloader", default=False, action="store_true",
    )

    ##### extract user arguments
    args = parser.parse_args()
    compile_gateware = True
    compile_software = True

    if args.document_only or args.sim:
        compile_gateware = False
        compile_software = True

    bbram = False
    if args.encrypt == None:
        encrypt = False
    else:
        encrypt = True
        if args.bbram:
            bbram = True

    io = _io

    ##### build the "bios"
    if compile_software:
        if args.simple_boot:
            os.system("riscv64-unknown-elf-as -fpic loader{}loader.S -o loader{}loader.elf".format(os.path.sep, os.path.sep))
            os.system("riscv64-unknown-elf-objcopy -O binary loader{}loader.elf loader{}bios.bin".format(os.path.sep, os.path.sep))
            print("**** WARNING: using 'simple boot' method -- no signature verification checks are done on boot! ****")
            bios_path = 'loader{}bios.bin'.format(os.path.sep)
        else:
            # do a first-pass to create the soc.svd file
            platform = Platform(io, encrypt=encrypt, bbram=bbram, strategy=args.strategy)
            soc = CramSoC(platform, bios_path=None, sim=args.sim)
            builder = Builder(soc, output_dir="build", csr_csv="build/csr.csv", csr_svd="build/software/soc.svd",
                compile_software=False, compile_gateware=False)
            builder.software_packages=[] # necessary to bypass Meson dependency checks required by Litex libc
            vns = builder.build(run=False)

            subprocess.run(["cargo", "xtask", "boot-image", "--feature", "sim"], check=True, cwd="boot")
            bios_path = 'boot{}boot.bin'.format(os.path.sep)
    else:
        bios_path=None

    ##### second pass to build the actual chip. Note any changes below need to be reflected into the first pass...might be a good idea to modularize that
    ##### setup platform
    platform = Platform(io, encrypt=encrypt, bbram=bbram, strategy=args.strategy)

    ##### define the soc
    if args.sim:
        clk_freq = 100e6
    else:
        clk_freq = 50e6 # slow it down for the actual FPGA, can't close timing at 100MHz

    soc = CramSoC(
        platform,
        bios_path=bios_path,
        sys_clk_freq=clk_freq,
        sim=args.sim,
    )

    ##### setup the builder and run it
    builder = Builder(soc, output_dir="build",
        csr_csv="build/csr.csv", csr_svd="build/software/soc.svd",
        compile_software=False, compile_gateware=compile_gateware)
    builder.software_packages=[] # necessary to bypass Meson dependency checks required by Litex libc

    # turn off regular_comb for simulation. Can't just use ~ because Python.
    if args.sim:
        rc=False
    else:
        rc=True
    vns = builder.build(regular_comb=rc)

    # now re-encrypt the binary if needed
    if encrypt and not (args.document_only or args.sim):
        # check if we need to re-encrypt to a set key
        # my.nky -- indicates the fuses have been burned on the target device, and needs re-encryption
        # keystore.bin -- indicates we want to initialize the on-chip key ROM with a set of known values
        if Path(args.encrypt).is_file():
            print('Found {}, re-encrypting binary to the specified fuse settings.'.format(args.encrypt))
            #if not Path('keystore.bin').is_file():  # i think we always want to regenerate this file from source...
            subprocess.call([sys.executable, './gen_keyrom.py', '--efuse-key', args.encrypt, '--dev-pubkey', './devkey/dev-x509.crt', '--output', 'keystore.bin'])

            print('Found keystore.bin, patching bitstream to contain specified keystore values.')
            with open('keystore.patch', 'w') as patchfile:
                subprocess.call([sys.executable, './key2bits.py', '-kkeystore.bin', '-rrom.db'], stdout=patchfile)
                keystore_args = '-pkeystore.patch'
                if bbram:
                    enc = [sys.executable, 'deps/encrypt-bitstream-python/encrypt-bitstream.py', '--bbram','-fbuild/gateware/cran_soc.bin', '-idummy.nky', '-k' + args.encrypt, '-obuild/gateware/encrypted'] + [keystore_args]
                else:
                    enc = [sys.executable, 'deps/encrypt-bitstream-python/encrypt-bitstream.py', '-fbuild/gateware/cran_soc.bin', '-idummy.nky', '-k' + args.encrypt, '-obuild/gateware/encrypted'] + [keystore_args]

            subprocess.call(enc)

            pad = [sys.executable, './append_csr.py', '-bbuild/gateware/encrypted.bin', '-cbuild/csr.csv', '-obuild/gateware/soc_csr.bin']
            subprocess.call(pad)
        else:
            print('Specified key file {} does not exist'.format(args.encrypt))
            return 1

    if args.sim and not args.document_only:
        from sim_support.sim_bench import SimRunner
        SimRunner(args.ci, [], vex_verilog_path=VEX_VERILOG_PATH, tb='top_tb_fpga')

    return 0

if __name__ == "__main__":
    from datetime import datetime
    start = datetime.now()
    ret = main()
    print("Run completed in {}".format(datetime.now()-start))

    sys.exit(ret)
