#!/usr/bin/env python3

# This variable defines all the external programs that this module
# relies on.  lxbuildenv reads this variable in order to ensure
# the build will finish without exiting due to missing third-party
# programs.

LX_DEPENDENCIES = ["riscv", "vivado"]

# Import lxbuildenv to integrate the deps/ directory
import lxbuildenv
import litex.soc.doc as lxsocdoc
from pathlib import Path
import subprocess
import sys

from random import SystemRandom
import argparse

from migen import *
from migen.genlib.resetsync import AsyncResetSynchronizer
from migen.genlib.cdc import MultiReg

from litex.build.generic_platform import *
from litex.build.xilinx import XilinxPlatform, VivadoProgrammer

from litex.soc.interconnect.csr import *
from litex.soc.interconnect.csr_eventmanager import *
from litex.soc.integration.soc_core import *
from litex.soc.integration.builder import *
from litex.soc.integration.doc import AutoDoc, ModuleDoc
from litex.soc.cores.clock import S7MMCM, S7IDELAYCTRL
from litex.soc.cores.i2s import S7I2SSlave
from litex.soc.cores.spi_opi import S7SPIOPI

from gateware import info
from gateware import sram_32
from gateware import memlcd
from gateware import spi_7series as spi
from gateware import messible
from gateware import i2c
from gateware import ticktimer

from gateware import spinor
from gateware import keyboard

from gateware import trng

from gateware import jtag_phy

# IOs ----------------------------------------------------------------------------------------------


_io_dvt = [   # DVT-generation I/Os
    ("clk12", 0, Pins("R3"), IOStandard("LVCMOS18")),

    ("analog", 0,
        Subsignal("usbc_cc1",    Pins("C5"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("usbc_cc2",    Pins("A8"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("vbus_div",    Pins("C4"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("noise0",      Pins("A3"), IOStandard("LVCMOS33")), # DVT
        Subsignal("noise1",      Pins("A5"), IOStandard("LVCMOS33")),  # DVT
        # diff grounds
        Subsignal("usbc_cc1_n",  Pins("B5"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("usbc_cc2_n",  Pins("A7"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("vbus_div_n",  Pins("B4"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("noise0_n",    Pins("A2"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("noise1_n",    Pins("A4"), IOStandard("LVCMOS33")),  # DVT
        # dedicated pins (no I/O standard applicable)
        Subsignal("ana_vn", Pins("K9")),
        Subsignal("ana_vp", Pins("J10")),
     ),

    ("jtag", 0,
         Subsignal("tck", Pins("U11"), IOStandard("LVCMOS18")),  # DVT
         Subsignal("tms", Pins("P6"), IOStandard("LVCMOS18")),   # DVT
         Subsignal("tdi", Pins("P7"), IOStandard("LVCMOS18")),   # DVT
         Subsignal("tdo", Pins("R6"), IOStandard("LVCMOS18")),   # DVT
    ),

    ("lpclk", 0, Pins("N15"), IOStandard("LVCMOS18")),  # wifi_lpclk

    # Power control signals
    ("power", 0,
        Subsignal("audio_on",     Pins("G13"), IOStandard("LVCMOS33")),
        Subsignal("fpga_sys_on",  Pins("N13"), IOStandard("LVCMOS18")),
        Subsignal("noisebias_on", Pins("A13"), IOStandard("LVCMOS33")),  # DVT
        Subsignal("allow_up5k_n", Pins("U7"), IOStandard("LVCMOS18")),
        Subsignal("pwr_s0",       Pins("U6"), IOStandard("LVCMOS18")),
        Subsignal("pwr_s1",       Pins("L13"), IOStandard("LVCMOS18")),  # DVT
        # Noise generator
        Subsignal("noise_on",     Pins("P14 R13"), IOStandard("LVCMOS18")),
        # vibe motor
        Subsignal("vibe_on",      Pins("B13"), IOStandard("LVCMOS33")),  # DVT
        Misc("SLEW=SLOW"),
    ),

    # Audio interface
    ("i2s", 0,
       Subsignal("clk", Pins("D14")),
       Subsignal("tx", Pins("D12")), # au_sdi1
       Subsignal("rx", Pins("C13")), # au_sdo1
       Subsignal("sync", Pins("B15")),
       IOStandard("LVCMOS33"),
       Misc("SLEW=SLOW"), Misc("DRIVE=4"),
     ),
    ("au_mclk", 0, Pins("D18"), IOStandard("LVCMOS33"), Misc("SLEW=SLOW"), Misc("DRIVE=8")),

    # I2C1 bus -- to RTC and audio CODEC
    ("i2c", 0,
        Subsignal("scl", Pins("C14"), IOStandard("LVCMOS33")),
        Subsignal("sda", Pins("A14"), IOStandard("LVCMOS33")),
        Misc("SLEW=SLOW"),
    ),

    # RTC interrupt
    ("rtc_irq", 0, Pins("N5"), IOStandard("LVCMOS18")),

    # COM interface to UP5K
    ("com", 0,
        Subsignal("csn",  Pins("T15"), IOStandard("LVCMOS18")),
        Subsignal("miso", Pins("P16"), IOStandard("LVCMOS18")),
        Subsignal("mosi", Pins("N18"), IOStandard("LVCMOS18")),
        Subsignal("sclk", Pins("R16"), IOStandard("LVCMOS18")),
     ),
    ("com_irq", 0, Pins("M16"), IOStandard("LVCMOS18")),

    # Top-side internal FPC header (B18 and D15 are used by the serial bridge)
    ("gpio", 0, Pins("A16 B16 D16"), IOStandard("LVCMOS33"), Misc("SLEW=SLOW")),

    # Keyboard scan matrix
    ("kbd", 0,
        # "key" 0-8 are rows, 9-18 are columns
        # column scan with 1's, so PD to default 0
        Subsignal("row", Pins("F15 E17 G17 E14 E15 H15 G15 H14 H16"), Misc("PULLDOWN True")),
        Subsignal("col", Pins("H17 E18 F18 G18 E13 H18 F13 H13 J13 K13")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
        Misc("DRIVE=4"),
     ),

    # LCD interface
    ("lcd", 0,
        Subsignal("sclk", Pins("A17")),
        Subsignal("scs",  Pins("C18")),
        Subsignal("si",   Pins("D17")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
        Misc("DRIVE=4"),
     ),

    # SD card (TF) interface
    ("sdcard", 0,
        Subsignal("data", Pins("J15 J14 K16 K14"), Misc("PULLUP True")),
        Subsignal("cmd",  Pins("J16"), Misc("PULLUP True")),
        Subsignal("clk",  Pins("G16")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW")
     ),

    # SPI Flash
    ("spiflash_1x", 0, # clock needs to be accessed through STARTUPE2
        Subsignal("cs_n", Pins("M13")),
        Subsignal("mosi", Pins("K17")),
        Subsignal("miso", Pins("K18")),
        Subsignal("wp",   Pins("L14")), # provisional
        Subsignal("hold", Pins("M15")), # provisional
        IOStandard("LVCMOS18")
    ),
    ("spiflash_8x", 0, # clock needs a separate override to meet timing
        Subsignal("cs_n", Pins("M13")),
        Subsignal("dq",   Pins("K17 K18 L14 M15 L17 L18 M14 N14")),
        Subsignal("dqs",  Pins("R14")),
        Subsignal("ecs_n", Pins("L16")),
        Subsignal("sclk", Pins("C12")),  # DVT
        IOStandard("LVCMOS18"),
        Misc("SLEW=SLOW"),
     ),

    # SRAM
    ("sram", 0,
        Subsignal("adr", Pins(
            "V12 M5 P5 N4  V14 M3 R17 U15",
            "M4  L6 K3 R18 U16 K1 R5  T2",
            "U1  N1 L5 K2  M18 T6"),
            IOStandard("LVCMOS18")),
        Subsignal("ce_n", Pins("V5"),  IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("oe_n", Pins("U12"), IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("we_n", Pins("K4"),  IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("zz_n", Pins("V17"), IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("d", Pins(
            "M2  R4  P2  L4  L1  M1  R1  P1",
            "U3  V2  V4  U2  N2  T1  K6  J6",
            "V16 V15 U17 U18 P17 T18 P18 M17",
            "N3  T4  V13 P15 T14 R15 T3  R7"),
            IOStandard("LVCMOS18")),
        Subsignal("dm_n", Pins("V3 R2 T5 T13"), IOStandard("LVCMOS18")),
    ),
]

_io_evt = [
    ("clk12", 0, Pins("R3"), IOStandard("LVCMOS18")),

    ("analog", 0,
        Subsignal("usbc_cc1",    Pins("C17"), IOStandard("LVCMOS33")),
        Subsignal("usbc_cc2",    Pins("E16"), IOStandard("LVCMOS33")),
        Subsignal("vbus_div",    Pins("E12"), IOStandard("LVCMOS33")),
        Subsignal("noise0",      Pins("B13"), IOStandard("LVCMOS33")),
        Subsignal("noise1",      Pins("B14"), IOStandard("LVCMOS33")),
        Subsignal("ana_vn",      Pins("K9")),  # no I/O standard as this is a dedicated pin
        Subsignal("ana_vp",      Pins("J10")), # no I/O standard as this is a dedicated pin
        Subsignal("noise0_n",    Pins("A13"), IOStandard("LVCMOS33")),  # PATCH
     ),

    ("lpclk", 0, Pins("N15"), IOStandard("LVCMOS18")),  # wifi_lpclk

    # Power control signals
    ("power", 0,
        Subsignal("audio_on",     Pins("G13"), IOStandard("LVCMOS33")),
        Subsignal("fpga_sys_on",  Pins("N13"), IOStandard("LVCMOS18")),
        # Subsignal("noisebias_on", Pins("A13"), IOStandard("LVCMOS33")),  # PATCH
        Subsignal("allow_up5k_n", Pins("U7"), IOStandard("LVCMOS18")),
        Subsignal("pwr_s0",       Pins("U6"), IOStandard("LVCMOS18")),
        # Subsignal("pwr_s1",       Pins("L13"), IOStandard("LVCMOS18")),  # PATCH
        # Noise generator
        Subsignal("noise_on", Pins("P14 R13"), IOStandard("LVCMOS18")),
        Misc("SLEW=SLOW"),
    ),

    # Audio interface
    ("i2s", 0,
       Subsignal("clk", Pins("D14")),
       Subsignal("tx", Pins("D12")), # au_sdi1
       Subsignal("rx", Pins("C13")), # au_sdo1
       Subsignal("sync", Pins("B15")),
       IOStandard("LVCMOS33"),
       Misc("SLEW=SLOW"), Misc("DRIVE=4"),
     ),
    # ("i2s", 1,  # speaker
    #    Subsignal("clk", Pins("F14")),
    #    Subsignal("tx", Pins("A15")), # au_sdi2
    #    Subsignal("sync", Pins("B17")),
    #    IOStandard("LVCMOS33"),
    #    Misc("SLEW=SLOW"), Misc("DRIVE=4"),
    # ),
    ("au_mclk", 0, Pins("D18"), IOStandard("LVCMOS33"), Misc("SLEW=SLOW"), Misc("DRIVE=8")),

    # I2C1 bus -- to RTC and audio CODEC
    ("i2c", 0,
        Subsignal("scl", Pins("C14"), IOStandard("LVCMOS33")),
        Subsignal("sda", Pins("A14"), IOStandard("LVCMOS33")),
        Misc("SLEW=SLOW"),
    ),

    # RTC interrupt
    ("rtc_irq", 0, Pins("N5"), IOStandard("LVCMOS18")),

    # COM interface to UP5K
    ("com", 0,
        Subsignal("csn",  Pins("T15"), IOStandard("LVCMOS18")),
        Subsignal("miso", Pins("P16"), IOStandard("LVCMOS18")),
        Subsignal("mosi", Pins("N18"), IOStandard("LVCMOS18")),
        Subsignal("sclk", Pins("R16"), IOStandard("LVCMOS18")),
     ),
    ("com_irq", 0, Pins("M16"), IOStandard("LVCMOS18")),

    # Top-side internal FPC header (B18 and D15 are used by the serial bridge)
    ("gpio", 0, Pins("A16 B16 D16"), IOStandard("LVCMOS33"), Misc("SLEW=SLOW")),

    # Keyboard scan matrix
    ("kbd", 0,
        # "key" 0-8 are rows, 9-18 are columns
        # column scan with 1's, so PD to default 0
        Subsignal("row", Pins("F15 E17 G17 E14 E15 H15 G15 H14 H16"), Misc("PULLDOWN True")),
        Subsignal("col", Pins("H17 E18 F18 G18 E13 H18 F13 H13 J13 K13")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
        Misc("DRIVE=4"),
     ),

    # LCD interface
    ("lcd", 0,
        Subsignal("sclk", Pins("A17")),
        Subsignal("scs",  Pins("C18")),
        Subsignal("si",   Pins("D17")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
        Misc("DRIVE=4"),
     ),

    # SD card (TF) interface
    ("sdcard", 0,
        Subsignal("data", Pins("J15 J14 K16 K14"), Misc("PULLUP True")),
        Subsignal("cmd",  Pins("J16"), Misc("PULLUP True")),
        Subsignal("clk",  Pins("G16")),
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW")
     ),

    # SPI Flash
    ("spiflash_4x", 0, # clock needs to be accessed through STARTUPE2
        Subsignal("cs_n", Pins("M13")),
        Subsignal("dq", Pins("K17 K18 L14 M15")),
        IOStandard("LVCMOS18")
    ),
    ("spiflash_1x", 0, # clock needs to be accessed through STARTUPE2
        Subsignal("cs_n", Pins("M13")),
        Subsignal("mosi", Pins("K17")),
        Subsignal("miso", Pins("K18")),
        Subsignal("wp",   Pins("L14")), # provisional
        Subsignal("hold", Pins("M15")), # provisional
        IOStandard("LVCMOS18")
    ),
    ("spiflash_8x", 0, # clock needs a separate override to meet timing
        Subsignal("cs_n", Pins("M13")),
        Subsignal("dq",   Pins("K17 K18 L14 M15 L17 L18 M14 N14")),
        Subsignal("dqs",  Pins("R14")),
        Subsignal("ecs_n", Pins("L16")),
        Subsignal("sclk", Pins("L13")),
        IOStandard("LVCMOS18"),
        Misc("SLEW=SLOW"),
     ),

    # SRAM
    ("sram", 0,
        Subsignal("adr", Pins(
            "V12 M5 P5 N4  V14 M3 R17 U15",
            "M4  L6 K3 R18 U16 K1 R5  T2",
            "U1  N1 L5 K2  M18 T6"),
            IOStandard("LVCMOS18")),
        Subsignal("ce_n", Pins("V5"),  IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("oe_n", Pins("U12"), IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("we_n", Pins("K4"),  IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("zz_n", Pins("V17"), IOStandard("LVCMOS18"), Misc("PULLUP True")),
        Subsignal("d", Pins(
            "M2  R4  P2  L4  L1  M1  R1  P1",
            "U3  V2  V4  U2  N2  T1  K6  J6",
            "V16 V15 U17 U18 P17 T18 P18 M17",
            "N3  T4  V13 P15 T14 R15 T3  R7"),
            IOStandard("LVCMOS18")),
        Subsignal("dm_n", Pins("V3 R2 T5 T13"), IOStandard("LVCMOS18")),
    ),
]

_io_uart_debug = [
    ("debug", 0,  # wired to the Rpi
        Subsignal("tx", Pins("V6")),
        Subsignal("rx", Pins("V7")),
        IOStandard("LVCMOS18"),
        Misc("SLEW=SLOW"),
    ),

    ("serial", 0, # wired to the internal flex
        Subsignal("tx", Pins("B18")), # debug0 breakout
        Subsignal("rx", Pins("D15")), # debug1
        IOStandard("LVCMOS33"),
        Misc("SLEW=SLOW"),
     ),
]

_io_uart_debug_swapped = [
    ("serial", 0, # wired to the RPi
     Subsignal("tx", Pins("V6")),
     Subsignal("rx", Pins("V7")),
     IOStandard("LVCMOS18"),
     ),

    ("debug", 0, # wired to the internal flex
     Subsignal("tx", Pins("B18")), # debug0 breakout
     Subsignal("rx", Pins("D15")), # debug1
     IOStandard("LVCMOS33"),
     ),
]

# Platform -----------------------------------------------------------------------------------------

class Platform(XilinxPlatform):
    def __init__(self, io, toolchain="vivado", programmer="vivado", part="50", encrypt=False, make_mod=False):
        part = "xc7s" + part + "-csga324-1il"
        XilinxPlatform.__init__(self, part, io, toolchain=toolchain)

        # NOTE: to do quad-SPI mode, the QE bit has to be set in the SPINOR status register. OpenOCD
        # won't do this natively, have to find a work-around (like using iMPACT to set it once)
        self.add_platform_command(
            "set_property CONFIG_VOLTAGE 1.8 [current_design]")
        self.add_platform_command(
            "set_property CFGBVS VCCO [current_design]")
        self.add_platform_command(
            "set_property BITSTREAM.CONFIG.CONFIGRATE 66 [current_design]")
        self.add_platform_command(
            "set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 1 [current_design]")
        self.toolchain.bitstream_commands = [
            "set_property CONFIG_VOLTAGE 1.8 [current_design]",
            "set_property CFGBVS GND [current_design]",
            "set_property BITSTREAM.CONFIG.CONFIGRATE 66 [current_design]",
            "set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 1 [current_design]",
        ]
        if encrypt:
            self.toolchain.bitstream_commands += [
                "set_property BITSTREAM.ENCRYPTION.ENCRYPT YES [current_design]",
                "set_property BITSTREAM.ENCRYPTION.ENCRYPTKEYSELECT eFUSE [current_design]",
                "set_property BITSTREAM.ENCRYPTION.KEYFILE ../../dummy.nky [current_design]"
            ]

        self.toolchain.additional_commands += \
            ["write_cfgmem -verbose -force -format bin -interface spix1 -size 64 "
             "-loadbit \"up 0x0 {build_name}.bit\" -file {build_name}.bin"]
        self.programmer = programmer

        self.toolchain.additional_commands += [
            "report_timing -delay_type min_max -max_paths 10 -slack_less_than 0 -sort_by group -input_pins -routable_nets -name failures -file timing-failures.txt"
        ]
        # this routine retained in case we have to re-explore the bitstream to find the location of the ROM LUTs
        if make_mod:
            # build a version of the bitstream with a different INIT value for the ROM lut, so the offset frame can
            # be discovered by diffing
            for bit in range(0, 32):
                for lut in range(4):
                    if lut == 0:
                        lutname = 'A'
                    elif lut == 1:
                        lutname = 'B'
                    elif lut == 2:
                        lutname = 'C'
                    else:
                        lutname = 'D'

                    self.toolchain.additional_commands += ["set_property INIT 64'hA6C355555555A6C3 [get_cells KEYROM" + str(bit) + lutname + "]"]

            self.toolchain.additional_commands += ["write_bitstream -bin_file -force top-mod.bit"]

    def create_programmer(self):
        if self.programmer == "vivado":
            return VivadoProgrammer(flash_part="n25q128-1.8v-spi-x1_x2_x4")
        else:
            raise ValueError("{} programmer is not supported".format(self.programmer))

    def do_finalize(self, fragment):
        XilinxPlatform.do_finalize(self, fragment)

# CRG ----------------------------------------------------------------------------------------------

class CRG(Module, AutoCSR):
    def __init__(self, platform, sys_clk_freq, spinor_edge_delay_ns=2.5):
        self.warm_reset = Signal()

        self.clock_domains.cd_sys   = ClockDomain()
        self.clock_domains.cd_spi   = ClockDomain()
        self.clock_domains.cd_lpclk = ClockDomain()
        self.clock_domains.cd_spinor = ClockDomain()
        self.clock_domains.cd_clk200 = ClockDomain()
        self.clock_domains.cd_clk50 = ClockDomain()

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

        self.submodules.mmcm = mmcm = S7MMCM(speedgrade=-1)
        self.comb += mmcm.reset.eq(self.warm_reset)
        mmcm.register_clkin(self.clk12_bufg, 12e6)
        # we count on clocks being assigned to the MMCME2_ADV in order. If we make more MMCME2 or shift ordering, these constraints must change.
        mmcm.create_clkout(self.cd_sys, sys_clk_freq, margin=0) # there should be a precise solution by design
        platform.add_platform_command("create_generated_clock -name sys_clk [get_pins MMCME2_ADV/CLKOUT0]")
        mmcm.create_clkout(self.cd_spi, 20e6)
        platform.add_platform_command("create_generated_clock -name spi_clk [get_pins MMCME2_ADV/CLKOUT1]")
        mmcm.create_clkout(self.cd_spinor, sys_clk_freq, phase=phase)  # delayed version for SPINOR cclk (different from COM SPI above)
        platform.add_platform_command("create_generated_clock -name spinor [get_pins MMCME2_ADV/CLKOUT2]")
        mmcm.create_clkout(self.cd_clk200, 200e6) # 200MHz required for IDELAYCTL
        platform.add_platform_command("create_generated_clock -name clk200 [get_pins MMCME2_ADV/CLKOUT3]")
        mmcm.create_clkout(self.cd_clk50, 50e6) # 50MHz for SHA-block
        platform.add_platform_command("create_generated_clock -name clk50 [get_pins MMCME2_ADV/CLKOUT4]")
        mmcm.expose_drp()

        # Add an IDELAYCTRL primitive for the SpiOpi block
        self.submodules += S7IDELAYCTRL(self.cd_clk200, reset_cycles=32) # 155ns @ 200MHz, min 59.28ns

# WarmBoot -----------------------------------------------------------------------------------------

class WarmBoot(Module, AutoCSR):
    def __init__(self, parent, reset_vector=0):
        self.ctrl = CSRStorage(size=8)
        self.addr = CSRStorage(size=32, reset=reset_vector)
        self.do_reset = Signal()
        # "Reset Key" is 0xac (0b101011xx)
        self.comb += self.do_reset.eq((self.ctrl.storage & 0xfc) == 0xac)

# BtEvents -----------------------------------------------------------------------------------------

class BtEvents(Module, AutoCSR, AutoDoc):
    def __init__(self, com, rtc):
        self.submodules.ev = EventManager()
        self.ev.com_int    = EventSourcePulse()   # rising edge triggered
        self.ev.rtc_int    = EventSourceProcess() # falling edge triggered
        self.ev.finalize()

        com_int = Signal()
        rtc_int = Signal()
        self.specials += MultiReg(com, com_int)
        self.specials += MultiReg(rtc, rtc_int)
        self.comb += self.ev.com_int.trigger.eq(com_int)
        self.comb += self.ev.rtc_int.trigger.eq(rtc_int)

# BtPower ------------------------------------------------------------------------------------------

class BtPower(Module, AutoCSR, AutoDoc):
    def __init__(self, pads, revision='evt'):
        self.intro = ModuleDoc("""BtPower - power control pins
        """)

        self.power = CSRStorage(8, fields=[
            CSRField("audio",     size=1, description="Write `1` to power on the audio subsystem"),
            CSRField("self",      size=1, description="Writing `1` forces self power-on (overrides the EC's ability to power me down)", reset=1),
            CSRField("ec_snoop",  size=1, description="Writing `1` allows the insecure EC to snoop a couple keyboard pads for wakeup key sequence recognition"),
            CSRField("state",     size=2, description="Current SoC power state. 0x=off or not ready, 10=on and safe to shutdown, 11=on and not safe to shut down, resets to 01 to allow extSRAM access immediately during init", reset=1),
            CSRField("noisebias", size=1, description="Writing `1` enables the primary bias supply for the noise generator"),
            CSRField("noise",     size=2, description="Controls which of two noise channels are active; all combos valid. noisebias must be on first.")
        ])
        # future-proofing this: we might want to add e.g. PWM levels and so forth, so give it its own register
        self.vibe = CSRStatus(1, description="Vibration motor configuration register", fields=[
            CSRField("vibe", size=1, description="Turn on vibration motor"),
        ])

        self.comb += [
            pads.audio_on.eq(self.power.fields.audio),
            pads.fpga_sys_on.eq(self.power.fields.self),
            # This signal automatically enables snoop when SoC is powered down
            pads.allow_up5k_n.eq(~self.power.fields.ec_snoop),
            # Ensure SRAM isolation during reset (CE & ZZ = 1 by pull-ups)
            pads.pwr_s0.eq(self.power.fields.state[0] & ~ResetSignal()),
            pads.noise_on.eq(self.power.fields.noise),
        ]
        if revision == 'dvt':
            self.comb += [
                pads.pwr_s1.eq(self.power.fields.state[1]),
                pads.noisebias_on.eq(self.power.fields.noisebias),
                pads.vibe_on.eq(self.vibe.fields.vibe)
            ]


# BtGpio -------------------------------------------------------------------------------------------

class BtGpio(Module, AutoDoc, AutoCSR):
    def __init__(self, pads):
        self.intro = ModuleDoc("""BtGpio - GPIO interface for betrusted""")

        gpio_in  = Signal(pads.nbits)
        gpio_out = Signal(pads.nbits)
        gpio_oe  = Signal(pads.nbits)

        for g in range(0, pads.nbits):
            gpio_ts = TSTriple(1)
            self.specials += gpio_ts.get_tristate(pads[g])
            self.comb += [
                gpio_ts.oe.eq(gpio_oe[g]),
                gpio_ts.o.eq(gpio_out[g]),
                gpio_in[g].eq(gpio_ts.i),
            ]

        self.output = CSRStorage(pads.nbits, name="output", description="Values to appear on GPIO when respective `drive` bit is asserted")
        self.input  = CSRStatus(pads.nbits,  name="input",  description="Value measured on the respective GPIO pin")
        self.drive  = CSRStorage(pads.nbits, name="drive",  description="When a bit is set to `1`, the respective pad drives its value out")
        self.intena = CSRStatus(pads.nbits,  name="intena", description="Enable interrupts when a respective bit is set")
        self.intpol = CSRStatus(pads.nbits,  name="intpol", description="When a bit is `1`, falling-edges cause interrupts. Otherwise, rising edges cause interrupts.")

        self.specials += MultiReg(gpio_in, self.input.status)
        self.comb += [
            gpio_out.eq(self.output.storage),
            gpio_oe.eq(self.drive.storage),
        ]

        self.submodules.ev = EventManager()

        for i in range(0, pads.nbits):
            setattr(self.ev, "gpioint" + str(i), EventSourcePulse() ) # pulse => rising edge

        self.ev.finalize()

        for i in range(0, pads.nbits):
            # pull from input.status because it's after the MultiReg synchronizer
            self.comb += getattr(self.ev, "gpioint" + str(i)).trigger.eq(self.input.status[i] ^ self.intpol.status[i])
            # note that if you change the polarity on the interrupt it could trigger an interrupt

# BtSeed -------------------------------------------------------------------------------------------

class BtSeed(Module, AutoDoc, AutoCSR):
    def __init__(self, reproduceable=False):
        self.intro = ModuleDoc("""Place and route seed. Set to a fixed number for reproduceable builds.
        Use a random number or your own number if you are paranoid about hardware implants that target
        fixed locations within the FPGA.""")

        if reproduceable:
          seed_reset = int(4) # chosen by fair dice roll. guaranteed to be random.
        else:
          rng        = SystemRandom()
          seed_reset = rng.getrandbits(64)
        self.seed = CSRStatus(64, name="seed", description="Seed used for the build", reset=seed_reset)

# RomTest -----------------------------------------------------------------------------------------

class RomTest(Module, AutoDoc, AutoCSR):
    def __init__(self, platform):
        self.intro = ModuleDoc("""Test for bitstream insertion of BRAM initialization contents""")
        platform.toolchain.attr_translate["KEEP"] = ("KEEP", "TRUE")
        platform.toolchain.attr_translate["DONT_TOUCH"] = ("DONT_TOUCH", "TRUE")

        import binascii
        self.address = CSRStorage(8, name="address", description="address for ROM")
        self.data = CSRStatus(32, name="data", description="data from ROM")

        rng = SystemRandom()
        with open("rom.db", "w") as f:
            for bit in range(0,32):
                lutsel = Signal(4)
                for lut in range(4):
                    if lut == 0:
                        lutname = 'A'
                    elif lut == 1:
                        lutname = 'B'
                    elif lut == 2:
                        lutname = 'C'
                    else:
                        lutname = 'D'
                    romval = rng.getrandbits(64)
                    # print("rom bit ", str(bit), lutname, ": ", binascii.hexlify(romval.to_bytes(8, byteorder='big')))
                    rom_name = "KEYROM" + str(bit) + lutname
                    # X36Y99 and counting down
                    if bit % 2 == 0:
                        platform.toolchain.attr_translate[rom_name] = ("LOC", "SLICE_X36Y" + str(50 + bit // 2))
                    else:
                        platform.toolchain.attr_translate[rom_name] = ("LOC", "SLICE_X37Y" + str(50 + bit // 2))
                    platform.toolchain.attr_translate[rom_name + 'BEL'] = ("BEL", lutname + '6LUT')
                    platform.toolchain.attr_translate[rom_name + 'LOCK'] = ( "LOCK_PINS", "I5:A6, I4:A5, I3:A4, I2:A3, I1:A2, I0:A1" )
                    self.specials += [
                        Instance( "LUT6",
                                  name=rom_name,
                                  # p_INIT=0x0000000000000000000000000000000000000000000000000000000000000000,
                                  p_INIT=romval,
                                  i_I0= self.address.storage[0],
                                  i_I1= self.address.storage[1],
                                  i_I2= self.address.storage[2],
                                  i_I3= self.address.storage[3],
                                  i_I4= self.address.storage[4],
                                  i_I5= self.address.storage[5],
                                  o_O= lutsel[lut],
                                  attr=("KEEP", "DONT_TOUCH", rom_name, rom_name + 'BEL', rom_name + 'LOCK')
                                  )
                    ]
                    # record the ROM LUT locations in a DB and annotate the initial random value given
                    f.write("KEYROM " + str(bit) + ' ' + lutname + ' ' + platform.toolchain.attr_translate[rom_name][1] +
                            ' ' + str(binascii.hexlify(romval.to_bytes(8, byteorder='big'))) + '\n')
                self.comb += [
                    If( self.address.storage[6:] == 0,
                        self.data.status[bit].eq(lutsel[2]))
                    .Elif(self.address.storage[6:] == 1,
                          self.data.status[bit].eq(lutsel[3]))
                    .Elif(self.address.storage[6:] == 2,
                          self.data.status[bit].eq(lutsel[0]))
                    .Else(self.data.status[bit].eq(lutsel[1]))
                ]

        platform.add_platform_command("create_pblock keyrom")
        platform.add_platform_command('resize_pblock [get_pblocks keyrom] -add ' + '{{SLICE_X36Y50:SLICE_X37Y65}}')
        #platform.add_platform_command("set_property CONTAIN_ROUTING true [get_pblocks keyrom]")  # should be fine to mingle the routing for this pblock
        platform.add_platform_command("add_cells_to_pblock [get_pblocks keyrom] [get_cells KEYROM*]")


class Aes(Module, AutoDoc, AutoCSR):
    def __init__(self, platform):
        self.key_0_q = CSRStorage(fields=[
            CSRField("key_0", size=32, description="least significant key word")
        ])
        self.key_1_q = CSRStorage(fields=[
            CSRField("key_1", size=32, description="key word 1")
        ])
        self.key_2_q = CSRStorage(fields=[
            CSRField("key_2", size=32, description="key word 2")
        ])
        self.key_3_q = CSRStorage(fields=[
            CSRField("key_3", size=32, description="key word 3")
        ])
        self.key_4_q = CSRStorage(fields=[
            CSRField("key_4", size=32, description="key word 4")
        ])
        self.key_5_q = CSRStorage(fields=[
            CSRField("key_5", size=32, description="key word 5")
        ])
        self.key_6_q = CSRStorage(fields=[
            CSRField("key_6", size=32, description="key word 6")
        ])
        self.key_7_q = CSRStorage(fields=[
            CSRField("key_7", size=32, description="most significant key word")
        ])

        self.dataout_0 = CSRStatus(fields=[
            CSRField("data", size=32, description="data output from cipher")
        ])
        self.dataout_1 = CSRStatus(fields=[
            CSRField("data", size=32, description="data output from cipher")
        ])
        self.dataout_2 = CSRStatus(fields=[
            CSRField("data", size=32, description="data output from cipher")
        ])
        self.dataout_3 = CSRStatus(fields=[
            CSRField("data", size=32, description="data output from cipher")
        ])

        self.datain_0 = CSRStorage(fields=[
            CSRField("data", size=32, description="data input")
        ], write_from_dev=True)
        self.datain_1 = CSRStorage(fields=[
            CSRField("data", size=32, description="data input")
        ], write_from_dev=True)
        self.datain_2 = CSRStorage(fields=[
            CSRField("data", size=32, description="data input")
        ], write_from_dev=True)
        self.datain_3 = CSRStorage(fields=[
            CSRField("data", size=32, description="data input")
        ], write_from_dev=True)
        datain_clear = Signal(4)
        self.comb += [
            self.datain_0.dat_w.eq(0),
            self.datain_1.dat_w.eq(0),
            self.datain_2.dat_w.eq(0),
            self.datain_3.dat_w.eq(0),
            self.datain_0.we.eq(datain_clear[0]),
            self.datain_1.we.eq(datain_clear[1]),
            self.datain_2.we.eq(datain_clear[2]),
            self.datain_3.we.eq(datain_clear[3]),
        ]

        self.ctrl = CSRStorage(fields=[
            CSRField("mode", size=1, description="set to `0' for AES_ENC, `1` for AES_DEC"),
            CSRField("key_len", size=3, description="length of the aes block", values=[
                    ("001", "AES128"),
                    ("010", "AES192"),
                    ("100", "AES256"),
            ]),
            CSRField("manual_start", size=1, description="If `0`, operation starts as soon as all data words are written"),
            CSRField("force_data_overwrite", size=1, description="If `0`, output is not updated until it is read"),
        ])
        self.status = CSRStatus(fields=[
            CSRField("idle", size=1, description="Core idle"),
            CSRField("stall", size=1, description="Core stall"),
            CSRField("output_valid", size=1, description="Data output valid"),
            CSRField("input_ready", size=1, description="Input value has been latched and it is OK to update to a new value"),
            CSRField("key_len_rbk", size=3, description="Actual key length selected by the hardware")
        ])

        self.trigger = CSRStorage(fields=[
            CSRField("start", size=1, description="Triggers an AES computation if manual_start is selected"),
            CSRField("key_clear", size=1, description="Clears the key"),
            CSRField("data_in_clear", size=1, description="Clears data input"),
            CSRField("data_out_clear", size=1, description="Clears the data output"),
        ])

        self.specials += Instance("aes_reg_top",
            i_clk_i = ClockSignal(),
            i_rst_ni = ~ResetSignal(),

            # TODO implement key clearing?
            i_key_0_q=self.key_0_q.fields.key_0,
            i_key_0_qe=self.key_0_q.re,
            i_key_1_q=self.key_1_q.fields.key_1,
            i_key_1_qe=self.key_1_q.re,
            i_key_2_q=self.key_2_q.fields.key_2,
            i_key_2_qe=self.key_2_q.re,
            i_key_3_q=self.key_3_q.fields.key_3,
            i_key_3_qe=self.key_3_q.re,
            i_key_4_q=self.key_4_q.fields.key_4,
            i_key_4_qe=self.key_4_q.re,
            i_key_5_q=self.key_5_q.fields.key_5,
            i_key_5_qe=self.key_5_q.re,
            i_key_6_q=self.key_6_q.fields.key_6,
            i_key_6_qe=self.key_6_q.re,
            i_key_7_q=self.key_7_q.fields.key_7,
            i_key_7_qe=self.key_7_q.re,

            o_data_out_0=self.dataout_0.fields.data,
            i_data_out_0_re=self.dataout_0.we,
            o_data_out_1=self.dataout_1.fields.data,
            i_data_out_1_re=self.dataout_1.we,
            o_data_out_2=self.dataout_2.fields.data,
            i_data_out_2_re=self.dataout_2.we,
            o_data_out_3=self.dataout_3.fields.data,
            i_data_out_3_re=self.dataout_3.we,

            i_data_in_0_to_core=self.datain_0.fields.data,
            i_data_in_1_to_core=self.datain_1.fields.data,
            i_data_in_2_to_core=self.datain_2.fields.data,
            i_data_in_3_to_core=self.datain_3.fields.data,
            o_data_in_0_de_from_core=datain_clear[0],
            o_data_in_1_de_from_core=datain_clear[1],
            o_data_in_2_de_from_core=datain_clear[2],
            o_data_in_3_de_from_core=datain_clear[3],

            i_ctrl_mode=self.ctrl.fields.mode,
            i_ctrl_key_len=self.ctrl.fields.key_len,
            o_ctrl_key_len_rbk=self.status.fields.key_len_rbk,
            i_ctrl_manual_start_trigger=self.ctrl.fields.manual_start,
            i_ctrl_force_data_overwrite=self.ctrl.fields.force_data_overwrite,
            i_ctrl_update=self.ctrl.re,

            o_idle=self.status.fields.idle,
            o_stall=self.status.fields.stall,
            o_output_valid=self.status.fields.output_valid,
            o_input_ready=self.status.fields.input_ready,

            i_start=self.trigger.fields.start,
            i_key_clear=self.trigger.fields.key_clear,
            i_data_in_clear=self.trigger.fields.data_in_clear,
            i_data_out_clear=self.trigger.fields.data_out_clear,
        )
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_reg_pkg.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_pkg.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_control.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_key_expand.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_mix_columns.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_mix_single_column.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_sbox_canright.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_sbox_lut.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_sbox.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_shift_rows.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_sub_bytes.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "aes", "rtl", "aes_core.sv"))
        platform.add_source(os.path.join("deps", "gateware", "gateware", "aes_reg_litex.sv"))


from litex.soc.interconnect import wishbone
from migen.genlib.cdc import BlindTransfer
class Hmac(Module, AutoDoc, AutoCSR):
    def __init__(self, platform):
        self.bus = bus = wishbone.Interface()
        wdata=Signal(32)
        wmask=Signal(4)
        wdata_we=Signal()
        wdata_avail=Signal()
        wdata_ready=Signal()
        self.sync.clk50 += [
            wdata_avail.eq(bus.cyc & bus.stb & bus.we),
            If(bus.cyc & bus.stb & bus.we & ~bus.ack,
                If(wdata_ready,
                    wdata.eq(bus.dat_w),
                    wmask.eq(bus.sel),
                    wdata_we.eq(1),
                    bus.ack.eq(1),  #### TODO check that this works with the clk50->clk100 domain crossing
                ).Else(
                    wdata_we.eq(0),
                    bus.ack.eq(0),
                )
               ).Else(
                wdata_we.eq(0),
                bus.ack.eq(0),
            )
        ]

        self.key_re = Signal(8)
        for k in range(0, 8):
            setattr(self, "key" + str(k), CSRStorage(32, name="key" + str(k), description="""secret key word {}""".format(k)))
            self.key_re[k].eq(getattr(self, "key" + str(k)).re)

        self.control = CSRStorage(description="Control register for the HMAC block", fields=[
            CSRField("sha_en", size=1, description="Enable the SHA block; disabling resets state"),
            CSRField("endian_swap", size=1, description="Swap the endianness on the input data"),
            CSRField("digest_swap", size=1, description="Swap the endianness on the output digest"),
            CSRField("hmac_en", size=1, description="Latch configuration for HMAC block"),
            CSRField("hash_start", size=1, description="Writing a 1 indicates the beginning of hash data", pulse=True),
            CSRField("hash_process", size=1, description="Writing a 1 digests the hash data", pulse=True),
        ])
        control_latch = Signal(self.control.size)
        ctrl_freeze = Signal()
        self.sync += [
            If(ctrl_freeze,
                control_latch.eq(control_latch)
            ).Else(
                control_latch.eq(self.control.storage)
            )
        ]
        self.status = CSRStatus(fields=[
            CSRField("done", size=1, description="Set when hash is done")
        ])

        self.wipe = CSRStorage(32, description="wipe the secret key using the written value. Wipe happens upon write.")

        for k in range(0, 8):
            setattr(self, "digest" + str(k), CSRStatus(32, name="digest" + str(k), description="""digest word {}""".format(k)))

        self.msg_length = CSRStatus(size=64, description="Length of digested message, in bits")
        self.error_code = CSRStatus(size=32, description="Error code")

        self.submodules.ev = EventManager()
        self.ev.err_valid = EventSourcePulse(description="Error flag was generated")
        self.ev.fifo_full = EventSourcePulse(description="FIFO is full")
        self.ev.hash_done = EventSourcePulse(description="Hash is done")
        self.ev.finalize()
        err_valid=Signal()
        err_valid_r=Signal()
        fifo_full=Signal()
        fifo_full_r=Signal()
        hash_done=Signal()
        self.sync += [
            err_valid_r.eq(err_valid),
            fifo_full_r.eq(fifo_full),
            hash_done.eq(self.status.fields.done),
        ]
        self.comb += [
            self.ev.err_valid.trigger.eq(~err_valid_r & err_valid),
            self.ev.fifo_full.trigger.eq(~fifo_full_r & fifo_full),
            self.ev.hash_done.trigger.eq(~hash_done & self.status.fields.done),
        ]

        # At a width of 32 bits, an 36kiB fifo is 1024 entries deep
        fifo_wvalid=Signal()
        fifo_wready=Signal()
        fifo_wdata=Signal(32)
        fifo_rvalid=Signal()
        fifo_rready=Signal()
        fifo_rdata=Signal(32)
        self.fifo = CSRStatus(description="FIFO status", fields=[
            CSRField("read_count", size=10, description="read pointer"),
            CSRField("write_count", size=10, description="write pointer"),
            CSRField("read_error", size=1, description="read error occurred"),
            CSRField("write_error", size=1, description="write error occurred"),
            CSRField("almost_full", size=1, description="almost full"),
            CSRField("almost_empty", size=1, description="almost empty"),
        ])
        self.specials += Instance("FIFO_SYNC_MACRO",
            p_DEVICE="7SERIES",
            p_FIFO_SIZE="36Kb",
            p_DATA_WIDTH=32,
            p_ALMOST_EMPTY_OFFSET=8,
            p_ALMOST_FULL_OFFSET=(1024 - 8),
            p_DO_REG=0,
            i_CLK=ClockSignal("clk50"),
            i_RST=ResetSignal("clk50"),
            o_FULL=~fifo_wready,
            i_WREN=fifo_wvalid,
            i_DI=fifo_wdata,
            o_EMPTY=~fifo_rvalid,
            i_RDEN=fifo_rready & ~fifo_rvalid,
            o_DO=fifo_rdata,
            o_RDCOUNT=self.fifo.fields.read_count,
            o_RDERR=self.fifo.fields.read_error,
            o_WRCOUNT=self.fifo.fields.write_count,
            o_WRERR=self.fifo.fields.write_error,
            o_ALMOSTFULL=self.fifo.fields.almost_full,
            o_ALMOSTEMPTY=self.fifo.fields.almost_empty,
        )

        key_re_50 = Signal()
        self.submodules.keyre = BlindTransfer("sys", "clk50", data_width=1)
        self.comb += [ self.keyre.i.eq(self.key_re), key_re_50.eq(self.keyre.o) ]

        hash_start_50 = Signal()
        self.submodules.hashstart = BlindTransfer("sys", "clk50", data_width=1)
        self.comb += [ self.hashstart.i.eq(self.control.fields.hash_start), hash_start_50.eq(self.hashstart.o) ]

        hash_proc_50 = Signal()
        self.submodules.hashproc = BlindTransfer("sys", "clk50", data_width=1)
        self.comb += [ self.hashproc.i.eq(self.control.fields.hash_process), hash_proc_50.eq(self.hashproc.o) ]

        wipe_50 = Signal()
        self.submodules.wipe50 = BlindTransfer("sys", "clk50", data_width=1)
        self.comb += [ self.wipe50.i.eq(self.wipe.re), wipe_50.eq(self.wipe50.o) ]

        self.specials += Instance("sha2_litex",
            i_clk_i = ClockSignal("clk50"),
            i_rst_ni = ~ResetSignal("clk50"),

            i_secret_key_0=self.key0.storage,
            i_secret_key_1=self.key1.storage,
            i_secret_key_2=self.key2.storage,
            i_secret_key_3=self.key3.storage,
            i_secret_key_4=self.key4.storage,
            i_secret_key_5=self.key5.storage,
            i_secret_key_6=self.key6.storage,
            i_secret_key_7=self.key7.storage,
            i_secret_key_re=key_re_50,

            i_reg_hash_start=hash_start_50,
            i_reg_hash_process=hash_proc_50,

            o_ctrl_freeze=ctrl_freeze,
            i_sha_en=control_latch[0],
            i_endian_swap=control_latch[1],
            i_digest_swap=control_latch[2],
            i_hmac_en=control_latch[3],

            o_reg_hash_done=self.status.fields.done,

            i_wipe_secret_re=wipe_50,
            i_wipe_secret_v=self.wipe.storage,

            o_digest_0=self.digest0.status,
            o_digest_1=self.digest1.status,
            o_digest_2=self.digest2.status,
            o_digest_3=self.digest3.status,
            o_digest_4=self.digest4.status,
            o_digest_5=self.digest5.status,
            o_digest_6=self.digest6.status,
            o_digest_7=self.digest7.status,

            o_msg_length=self.msg_length.status,
            o_error_code=self.error_code.status,

            i_msg_fifo_wdata=wdata,
            i_msg_fifo_write_mask=wmask,
            i_msg_fifo_we=wdata_we,
            i_msg_fifo_req=wdata_avail,
            o_msg_fifo_gnt=wdata_ready,

            o_local_fifo_wvalid=fifo_wvalid,
            i_local_fifo_wready=fifo_wready,
            o_local_fifo_wdata=fifo_wdata,
            i_local_fifo_rvalid=fifo_rvalid,
            o_local_fifo_rready=fifo_rready,
            i_local_fifo_rdata=fifo_rdata,

            o_err_valid=err_valid,
            i_err_valid_pending=self.ev.err_valid.pending,
            o_fifo_full_event=fifo_full,
        )

        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "hmac", "rtl", "hmac_pkg.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "hmac", "rtl", "sha2.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "hmac", "rtl", "sha2_pad.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "prim", "rtl", "prim_packer.sv"))
        platform.add_source(os.path.join("deps", "opentitan", "hw", "ip", "hmac", "rtl", "hmac_core.sv"))
        platform.add_source(os.path.join("deps", "gateware", "gateware", "sha2_litex.sv"))

# System constants ---------------------------------------------------------------------------------

boot_offset    = 0x500000 # enough space to hold 2x FPGA bitstreams before the firmware start
bios_size      = 0x8000
SPI_FLASH_SIZE = 128 * 1024 * 1024

# BetrustedSoC -------------------------------------------------------------------------------------

class BetrustedSoC(SoCCore):
    # I/O range: 0x80000000-0xfffffffff (not cacheable)
    SoCCore.mem_map = {
        "rom":             0x00000000,
        "sram":            0x10000000, # Should this be 0x0100_0000 ???
        "spiflash":        0x20000000,
        "sram_ext":        0x40000000,
        "memlcd":          0xb0000000,
        "audio":           0xe0000000,
        "sha":             0xe0001000,
        "vexriscv_debug":  0xefff0000,
        "csr":             0xf0000000,
    }

    def __init__(self, platform, revision, sys_clk_freq=int(100e6), legacy_spi=False, xous=False, **kwargs):
        assert sys_clk_freq in [int(12e6), int(100e6)]
        global bios_size

        # CPU cluster
        ## For dev work, we're booting from SPI directly. However, for enhanced security
        ## we will eventually want to move to a bitstream-ROM based bootloader that does
        ## a signature verification of the external SPI code before running it. The theory is that
        ## a user will burn a random AES key into their FPGA and encrypt their bitstream to their
        ## unique AES key, creating a root of trust that offers a defense against trivial patch attacks.

        if xous == False:  # raw firmware boots from SPINOR directly; xous boots from default Litex internal ROM
            reset_address = self.mem_map["spiflash"]+boot_offset
            bios_size = 0
        else:
            reset_address = self.mem_map["rom"]

        # SoCCore ----------------------------------------------------------------------------------
        SoCCore.__init__(self, platform, sys_clk_freq, csr_data_width=32,
            integrated_rom_size  = bios_size,
            integrated_sram_size = 0x20000,
            ident                = "betrusted.io LiteX Base SoC",
            cpu_type             = "vexriscv",
            csr_paging           = 4096,  # increase paging to 1 page size
            csr_address_width    = 16,    # increase to accommodate larger page size
            uart_name            = "crossover", # use UART-over-wishbone for debugging
            cpu_reset_address    = reset_address,
            **kwargs)

        # CPU --------------------------------------------------------------------------------------
        self.cpu.use_external_variant("deps/gateware/gateware/cpu/VexRiscv_BetrustedSoC_Debug.v")
        self.cpu.add_debug()
        self.submodules.reboot = WarmBoot(self, reset_address)
        self.add_csr("reboot")
        warm_reset = Signal()
        self.comb += warm_reset.eq(self.reboot.do_reset)
        self.cpu.cpu_params.update(i_externalResetVector=self.reboot.addr.storage)

        # Debug cluster ----------------------------------------------------------------------------
        from litex.soc.cores.uart import UARTWishboneBridge
        self.submodules.uart_bridge = UARTWishboneBridge(platform.request("debug"), sys_clk_freq, baudrate=115200)
        self.add_wb_master(self.uart_bridge.wishbone)
        self.register_mem("vexriscv_debug", 0xefff0000, self.cpu.debug_bus, 0x100)

        # Clockgen cluster -------------------------------------------------------------------------
        self.submodules.crg = CRG(platform, sys_clk_freq, spinor_edge_delay_ns=2.2)
        self.add_csr("crg")
        self.comb += self.crg.warm_reset.eq(warm_reset)

        # XADC analog interface---------------------------------------------------------------------

        from litex.soc.cores.xadc import analog_layout
        analog_pads = Record(analog_layout)
        analog = platform.request("analog")
        self.comb += [
            analog_pads.vp.eq(analog.ana_vp),
            analog_pads.vn.eq(analog.ana_vn),
        ]
        if revision == 'evt':
            # NOTE - if part is changed to XC7S25, the pin-to-channel mappings change
            analog_pads.vauxp.eq(Cat(analog.noise0,       # 0
                                     Signal(7, reset=0),  # 1,2,3,4,5,6,7
                                     analog.noise1, analog.vbus_div, analog.usbc_cc1, analog.usbc_cc2, # 8,9,10,11
                                     Signal(4, reset=0),  # 12,13,14,15
                                )),
            analog_pads.vauxn.eq(Cat(analog.noise0_n, Signal(15, reset=0))),  # PATCH
        else:
            # DVT is solidly an xc7s50-only build
            analog_pads.vauxp.eq(Cat(Signal(3, reset=0),  # 0,1,2,3
                                     analog.usbc_cc2,       # 4
                                     Signal(1, reset=0),   # 5
                                     analog.vbus_div, analog.noise1, # 6,7
                                     Signal(4, reset=0),  # 8,9,10,11
                                     analog.usbc_cc1,      # 12
                                     Signal(2, reset=0),  # 13,14
                                     analog.noise0
                                )),
            analog_pads.vauxn.eq(Cat(Signal(3, reset=0),  # 0,1,2,3
                                     analog.usbc_cc2_n,     # 4
                                     Signal(1, reset=0),   # 5
                                     analog.vbus_div_n, analog.noise1_n, # 6,7
                                     Signal(4, reset=0),  # 8,9,10,11
                                     analog.usbc_cc1_n,    # 12
                                     Signal(2, reset=0),  # 13,14
                                     analog.noise0_n
                                )),

        self.submodules.info = info.Info(platform, self.__class__.__name__, analog_pads)
        self.add_csr("info")
        self.platform.add_platform_command('create_generated_clock -name dna_cnt -source [get_pins {{dna_count_reg[0]/Q}}] -divide_by 2 [get_pins {{DNA_PORT/CLK}}]')

        # External SRAM ----------------------------------------------------------------------------
        # Note that page_rd_timing=2 works, but is a slight overclock on RAM. Cache fill time goes from 436ns to 368ns for 8 words.
        self.submodules.sram_ext = sram_32.SRAM32(platform.request("sram"), rd_timing=7, wr_timing=6, page_rd_timing=3)  # this works with 2:nbits page length with Rust firmware...
        #self.submodules.sram_ext = sram_32.SRAM32(platform.request("sram"), rd_timing=7, wr_timing=6, page_rd_timing=5)  # this worked with 3:nbits page length in C firmware
        self.add_csr("sram_ext")
        self.register_mem("sram_ext", self.mem_map["sram_ext"], self.sram_ext.bus, size=0x1000000)
        # A bit of a bodge -- the path is actually async, so what we are doing is trying to constrain intra-channel skew by pushing them up against clock limits
        self.platform.add_platform_command("set_input_delay -clock [get_clocks sys_clk] -min -add_delay 4.0 [get_ports {{sram_d[*]}}]")
        self.platform.add_platform_command("set_input_delay -clock [get_clocks sys_clk] -max -add_delay 9.0 [get_ports {{sram_d[*]}}]")
        self.platform.add_platform_command("set_output_delay -clock [get_clocks sys_clk] -min -add_delay 0.0 [get_ports {{sram_adr[*] sram_d[*] sram_ce_n sram_oe_n sram_we_n sram_zz_n sram_dm_n[*]}}]")
        self.platform.add_platform_command("set_output_delay -clock [get_clocks sys_clk] -max -add_delay 3.0 [get_ports {{sram_adr[*] sram_d[*] sram_ce_n sram_oe_n sram_we_n sram_zz_n sram_dm_n[*]}}]")
        # ODDR falling edge ignore
        self.platform.add_platform_command("set_false_path -fall_from [get_clocks sys_clk] -through [get_ports {{sram_d[*] sram_adr[*] sram_ce_n sram_oe_n sram_we_n sram_zz_n sram_dm_n[*]}}]")
        self.platform.add_platform_command("set_false_path -fall_to [get_clocks sys_clk] -through [get_ports {{sram_d[*]}}]")
        self.platform.add_platform_command("set_false_path -fall_from [get_clocks sys_clk] -through [get_nets sram_ext_load]")
        self.platform.add_platform_command("set_false_path -fall_to [get_clocks sys_clk] -through [get_nets sram_ext_load]")
        self.platform.add_platform_command("set_false_path -rise_from [get_clocks sys_clk] -fall_to [get_clocks sys_clk]")  # sort of a big hammer but should be OK
        # reset ignore
        self.platform.add_platform_command("set_false_path -through [get_nets sys_rst]")
        # relax OE driver constraint (it's OK if it is a bit late, and it's an async path from fabric to output so it will be late)
        self.platform.add_platform_command("set_multicycle_path 2 -setup -through [get_pins sram_ext_sync_oe_n_reg/Q]")
        self.platform.add_platform_command("set_multicycle_path 1 -hold -through [get_pins sram_ext_sync_oe_n_reg/Q]")

        # LCD interface ----------------------------------------------------------------------------
        self.submodules.memlcd = memlcd.MemLCD(platform.request("lcd"))
        self.add_csr("memlcd")
        self.register_mem("memlcd", self.mem_map["memlcd"], self.memlcd.bus, size=self.memlcd.fb_depth*4)

        # COM SPI interface ------------------------------------------------------------------------
        self.submodules.com = spi.SPIMaster(platform.request("com"))
        self.add_csr("com")
        # 20.83ns = 1/2 of 24MHz clock, we are doing falling-to-rising timing
        # up5k tsu = -0.5ns, th = 5.55ns, tpdmax = 10ns
        # in reality, we are measuring a Tpd from the UP5K of 17ns. Routed input delay is ~3.9ns, which means
        # the fastest clock period supported would be 23.9MHz - just shy of 24MHz, with no margin to spare.
        # slow down clock period of SPI to 20MHz, this gives us about a 4ns margin for setup for PVT variation
        self.platform.add_platform_command("set_input_delay -clock [get_clocks spi_clk] -min -add_delay 0.5 [get_ports {{com_miso}}]") # could be as low as -0.5ns but why not
        self.platform.add_platform_command("set_input_delay -clock [get_clocks spi_clk] -max -add_delay 17.5 [get_ports {{com_miso}}]")
        self.platform.add_platform_command("set_output_delay -clock [get_clocks spi_clk] -min -add_delay 6.0 [get_ports {{com_mosi com_csn}}]")
        self.platform.add_platform_command("set_output_delay -clock [get_clocks spi_clk] -max -add_delay 16.0 [get_ports {{com_mosi com_csn}}]")  # could be as large as 21ns but why not
        # cross domain clocking is handled with explicit software barrires, or with multiregs
        self.platform.add_false_path_constraints(self.crg.cd_sys.clk, self.crg.cd_spi.clk)
        self.platform.add_false_path_constraints(self.crg.cd_spi.clk, self.crg.cd_sys.clk)

        # I2C interface ----------------------------------------------------------------------------
        self.submodules.i2c = i2c.RTLI2C(platform, platform.request("i2c", 0))
        self.add_csr("i2c")
        self.add_interrupt("i2c")

        # Event generation for I2C and COM ---------------------------------------------------------
        self.submodules.btevents = BtEvents(platform.request("com_irq", 0), platform.request("rtc_irq", 0))
        self.add_csr("btevents")
        self.add_interrupt("btevents")

        # Messible for debug -----------------------------------------------------------------------
        self.submodules.messible = messible.Messible()
        self.add_csr("messible")

        # Tick timer -------------------------------------------------------------------------------
        self.submodules.ticktimer = ticktimer.TickTimer(1000, sys_clk_freq, bits=64)
        self.add_csr("ticktimer")

        # Power control pins -----------------------------------------------------------------------
        self.submodules.power = BtPower(platform.request("power"), revision)
        self.add_csr("power")

        # SPI flash controller ---------------------------------------------------------------------
        if legacy_spi:
            self.submodules.spinor = spinor.SPINOR(platform, platform.request("spiflash_1x"), size=SPI_FLASH_SIZE)
        else:
            sclk_instance_name="SCLK_ODDR"
            iddr_instance_name="SPI_IDDR"
            miso_instance_name="MISO_FDRE"
            spiread=False
            self.submodules.spinor = S7SPIOPI(platform.request("spiflash_8x"),
                    sclk_name=sclk_instance_name, iddr_name=iddr_instance_name, miso_name=miso_instance_name, spiread=spiread)
            # reminder to self: the {{ and }} overloading is because Python treats these as special in strings, so {{ -> { in actual constraint
            # NOTE: ECSn is deliberately not constrained -- it's more or less async (0-10ns delay on the signal, only meant to line up with "block" region

            # constrain DQS-to-DQ input DDR delays
            self.platform.add_platform_command("create_clock -name spidqs -period 10 [get_ports spiflash_8x_dqs]")
            self.platform.add_platform_command("set_input_delay -clock spidqs -max 0.6 [get_ports {{spiflash_8x_dq[*]}}]")
            self.platform.add_platform_command("set_input_delay -clock spidqs -min -0.6 [get_ports {{spiflash_8x_dq[*]}}]")
            self.platform.add_platform_command("set_input_delay -clock spidqs -max 0.6 [get_ports {{spiflash_8x_dq[*]}}] -clock_fall -add_delay")
            self.platform.add_platform_command("set_input_delay -clock spidqs -min -0.6 [get_ports {{spiflash_8x_dq[*]}}] -clock_fall -add_delay")

            # derive clock for SCLK - clock-forwarded from DDR see Xilinx answer 62488 use case #4
            self.platform.add_platform_command("create_generated_clock -name spiclk_out -multiply_by 1 -source [get_pins {}/Q] [get_ports spiflash_8x_sclk]".format(sclk_instance_name))
            # if using CCLK output and not DDR forwarded clock, these are the commands used to define the clock
            #self.platform.add_platform_command("create_generated_clock -name spiclk_out -source [get_pins STARTUPE2/USRCCLKO] -combinational [get_pins STARTUPE2/USRCCLKO]")
            #self.platform.add_platform_command("set_clock_latency -min 0.5 [get_clocks spiclk_out]")  # define the min/max delay of the STARTUPE2 buffer
            #self.platform.add_platform_command("set_clock_latency -max 7.5 [get_clocks spiclk_out]")

            # constrain MISO SDR delay -- WARNING: -max is 'actually' 5.0ns, but design can't meet timing @ 5.0 tPD from SPIROM. There is some margin in the timing closure tho, so 4.5ns is probably going to work....
            self.platform.add_platform_command("set_input_delay -clock [get_clocks spiclk_out] -clock_fall -max 4.5 [get_ports spiflash_8x_dq[1]]")
            self.platform.add_platform_command("set_input_delay -clock [get_clocks spiclk_out] -clock_fall -min 1 [get_ports spiflash_8x_dq[1]]")
            # corresponding false path on MISO DDR input when clocking SDR data
            self.platform.add_platform_command("set_false_path -from [get_clocks spiclk_out] -to [get_pin {}/D ]".format(iddr_instance_name + "1"))
            # corresponding false path on MISO SDR input from DQS strobe, only if the MISO path is used
            if spiread:
                self.platform.add_platform_command("set_false_path -from [get_clocks spidqs] -to [get_pin {}/D ]".format(miso_instance_name))

            # constrain CLK-to-DQ output DDR delays; MOSI uses the same rules
            self.platform.add_platform_command("set_output_delay -clock [get_clocks spiclk_out] -max 1 [get_ports {{spiflash_8x_dq[*]}}]")
            self.platform.add_platform_command("set_output_delay -clock [get_clocks spiclk_out] -min -1 [get_ports {{spiflash_8x_dq[*]}}]")
            self.platform.add_platform_command("set_output_delay -clock [get_clocks spiclk_out] -max 1 [get_ports {{spiflash_8x_dq[*]}}] -clock_fall -add_delay")
            self.platform.add_platform_command("set_output_delay -clock [get_clocks spiclk_out] -min -1 [get_ports {{spiflash_8x_dq[*]}}] -clock_fall -add_delay")
            # constrain CLK-to-CS output delay. NOTE: timings require one dummy cycle insertion between CS and SCLK (de)activations. Not possible to meet timing for DQ & single-cycle CS due to longer tS/tH reqs for CS
            self.platform.add_platform_command("set_output_delay -clock [get_clocks spiclk_out] -min -1 [get_ports spiflash_8x_cs_n]") # -3 in reality
            self.platform.add_platform_command("set_output_delay -clock [get_clocks spiclk_out] -max 1 [get_ports spiflash_8x_cs_n]")  # 4.5 in reality
            # unconstrain OE path - we have like 10+ dummy cycles to turn the bus on wr->rd, and 2+ cycles to turn on end of read
            self.platform.add_platform_command("set_false_path -through [ get_pins s7spiopi_dq_mosi_oe_reg/Q ]")
            self.platform.add_platform_command("set_false_path -through [ get_pins s7spiopi_dq_oe_reg/Q ]")

        self.register_mem("spiflash", self.mem_map["spiflash"], self.spinor.bus, size=SPI_FLASH_SIZE)
        self.add_csr("spinor")

        # Keyboard module --------------------------------------------------------------------------
        self.submodules.keyboard = ClockDomainsRenamer(cd_remapping={"kbd":"lpclk"})(keyboard.KeyScan(platform.request("kbd")))
        self.add_csr("keyboard")
        self.add_interrupt("keyboard")

        # GPIO module ------90f63ac2678aed36813c9ecb1de9a245b7ef137a------------------------------------------------------------------------
        # self.submodules.gpio = BtGpio(platform.request("gpio"))
        # self.add_csr("gpio")
        # self.add_interrupt("gpio")

        # Build seed -------------------------------------------------------------------------------
        self.submodules.seed = BtSeed(reproduceable=False)
        self.add_csr("seed")

        # ROM test ---------------------------------------------------------------------------------
        self.submodules.romtest = RomTest(platform)
        self.add_csr("romtest")

        # Audio interfaces -------------------------------------------------------------------------
        self.submodules.audio = S7I2SSlave(platform.request("i2s", 0))
        self.add_wb_slave(self.mem_map["audio"], self.audio.bus, 4)
        self.add_memory_region("audio", self.mem_map["audio"], 4, type='io')
        self.add_csr("audio")
        self.add_interrupt("audio")

        self.comb += platform.request("au_mclk", 0).eq(self.crg.clk12_bufg)

        # Ring Oscillator TRNG ---------------------------------------------------------------------
        self.submodules.trng_osc = trng.TrngRingOsc(platform, target_freq=1e6, make_pblock=True)
        self.add_csr("trng_osc")
        # ignore ring osc paths
        self.platform.add_platform_command("set_false_path -through [get_nets trng_osc_ena]")
        self.platform.add_platform_command("set_false_path -through [get_nets trng_osc_ring_cw_1]")
        # MEMO: diagnostic option, need to turn off GPIO
        # gpio_pads = platform.request("gpio")
        #### self.comb += gpio_pads[0].eq(self.trng_osc.trng_fast)  # this one rarely needs probing
        # self.comb += gpio_pads[1].eq(self.trng_osc.trng_slow)
        # self.comb += gpio_pads[2].eq(self.trng_osc.trng_raw)

        # AES block --------------------------------------------------------------------------------
        self.submodules.aes = Aes(platform)
        self.add_csr("aes")

        # SHA block --------------------------------------------------------------------------------
        self.submodules.sha = Hmac(platform)
        self.add_csr("sha")
        self.add_interrupt("sha")
        self.add_wb_slave(self.mem_map["sha"], self.sha.bus, 4)
        self.add_memory_region("sha", self.mem_map["sha"], 4, type='io')

        # JTAG self-provisioning block -------------------------------------------------------------
        if revision != 'evt': # these pins don't exist on EVT
            self.submodules.jtag = jtag_phy.BtJtag(platform.request("jtag"))
            self.add_csr("jtag")

        # Lock down both ICAPE2 blocks -------------------------------------------------------------
        # this attempts to make it harder to partially reconfigure a bitstream that attempts to use
        # the ICAP block. An ICAP block can read out everything inside the FPGA, including key ROM,
        # even when the encryption fuses are set for the configuration stream.
        platform.toolchain.attr_translate["icap0"] = ("LOC", "ICAP_X0Y0")
        platform.toolchain.attr_translate["icap1"] = ("LOC", "ICAP_X0Y1")
        self.specials += [
            Instance("ICAPE2", i_I=0, i_CLK=0, i_CSIB=1, i_RDWRB=1,
                     attr={"KEEP", "DONT_TOUCH", "icap0"}
                     ),
            Instance("ICAPE2", i_I=0, i_CLK=0, i_CSIB=1, i_RDWRB=1,
                     attr={"KEEP", "DONT_TOUCH", "icap1"}
                     ),
        ]

# Build --------------------------------------------------------------------------------------------

def main():
    global _io

    if os.environ['PYTHONHASHSEED'] != "1":
        print( "PYTHONHASHEED must be set to 1 for consistent validation results. Failing to set this results in non-deterministic compilation results")
        return 1

    parser = argparse.ArgumentParser(description="Build the Betrusted SoC")
    parser.add_argument(
        "-D", "--document-only", default=False, action="store_true", help="Build docs only"
    )
    parser.add_argument(
        "-e", "--encrypt", help="Format output for encryption using the specified dummy key. Image is re-encrypted at sealing time with a secure key.", type=str
    )
    parser.add_argument(
        "-x", "--xous", help="Build for the Xous runtime environment. Defaults to `fw` validation image.", default=False, action="store_true"
    )
    parser.add_argument(
        "-r", "--revision", choices=['evt', 'dvt'], help="Build for a particular revision. Defaults to 'evt'", default='evt', type=str,
    )

    ##### extract user arguments
    args = parser.parse_args()
    compile_gateware = True
    compile_software = True

    if args.document_only:
        compile_gateware = False
        compile_software = False

    if args.encrypt == None:
        encrypt = False
    else:
        encrypt = True

    if args.revision == 'evt':
        io = _io_evt
    elif args.revision == 'dvt':
        io = _io_dvt
    else:
        print("Invalid hardware revision specified: {}; aborting.".format(args.revision))
        sys.exit(1)

    ##### setup platform
    platform = Platform(io, encrypt=encrypt)
    platform.add_extension(_io_uart_debug)  # specify the location of the UART pins, we can swap them to some reserved GPIOs

    ##### define the soc
    soc = BetrustedSoC(platform, args.revision, xous=args.xous)

    ##### setup the builder and run it
    builder = Builder(soc, output_dir="build", csr_csv="build/csr.csv", csr_svd="build/software/soc.svd", compile_software=compile_software, compile_gateware=compile_gateware)
    builder.software_packages = [
        ("bios", os.path.abspath(os.path.join(os.path.dirname(__file__), "loader")))
    ]
    vns = builder.build()

    ##### post-build routines
    soc.do_exit(vns)
    lxsocdoc.generate_docs(soc, "build/documentation", note_pulses=True)

    # generate the rom-inject library code
    if ~args.document_only:
        if not os.path.exists('fw/rom-inject/src'): # make rom-inject/src if it doesn't exist, e.g. on clean checkout
            os.mkdir('fw/rom-inject/src')
        with open('fw/rom-inject/src/lib.rs', 'w+') as libfile:
            subprocess.call([sys.executable, './key2bits.py', '-c', '-k../../keystore.bin', '-r../../rom.db'], cwd='deps/rom-locate', stdout=libfile)

    # now re-encrypt the binary if needed
    if encrypt and not args.document_only:
        # check if we need to re-encrypt to a set key
        # my.nky -- indicates the fuses have been burned on the target device, and needs re-encryption
        # keystore.bin -- indicates we want to initialize the on-chip key ROM with a set of known values
        if Path(args.encrypt).is_file():
            print('Found {}, re-encrypting binary to the specified fuse settings.'.format(args.encrypt))
            if Path('keystore.bin').is_file():
                print('Found keystore.bin, patching bitstream to contain specified keystore values.')
                with open('keystore.patch', 'w') as patchfile:
                    subprocess.call([sys.executable, './key2bits.py', '-k../../keystore.bin', '-r../../rom.db'], cwd='deps/rom-locate', stdout=patchfile)
                    keystore_args = '-pkeystore.patch'
                    enc = ['deps/encrypt-bitstream-python/encrypt-bitstream.py', '-fbuild/gateware/top.bin', '-idummy.nky', '-k' + args.encrypt, '-obuild/gateware/encrypted'] + [keystore_args]
            else:
                enc = ['deps/encrypt-bitstream-python/encrypt-bitstream.py', '-fbuild/gateware/top.bin', '-idummy.nky', '-k' + args.encrypt, '-obuild/gateware/encrypted']
            subprocess.call(enc)
        else:
            print('Specified key file {} does not exist'.format(args.encrypt))
            return 1

    return 0

if __name__ == "__main__":
    from datetime import datetime
    start = datetime.now()
    ret = main()
    print("Run completed in {}".format(datetime.now()-start))

    sys.exit(ret)
