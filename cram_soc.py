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

from litex.build.generic_platform import *

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

from deps.gateware.gateware import memlcd

from axi_crossbar import AXICrossbar
from axi_adapter import AXIAdapter
from axi_ram import AXIRAM
from axi_common import *

import subprocess


VEX_VERILOG_PATH = "deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_CramSoC.v"

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
    ("clk12", 0, Pins("R3"), IOStandard("LVCMOS18")),
    ("lpclk", 0, Pins("N15"), IOStandard("LVCMOS18")),
    ("reset", 0, Pins(1)),

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
     ("sim", 0,
        Subsignal("success", Pins(1)),
        Subsignal("done", Pins(1)),
        Subsignal("report", Pins(32)),
        Subsignal("coreuser", Pins(1)),
     ),

    # Trimming bits
     ("trimming", 0,
        Subsignal("reset", Pins(32)),
        Subsignal("reset_ena", Pins(1)),
     )
]

# CRG ----------------------------------------------------------------------------------------------

class CRG(Module):
    def __init__(self, platform, sys_clk_freq, spinor_edge_delay_ns=2.5):
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
        self.comb += reset_combo.eq(self.warm_reset | (~mmcm.locked & ~self.ignore_locked) | platform.request("reset"))
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

# CramSoC ------------------------------------------------------------------------------------------

class CramSoC(SoCMini):
    mem_map = {**SoCCore.mem_map, **{
        "csr": 0x4000_0000,
    }}
    def __init__(self, platform, bios_path=None, sys_clk_freq=75e6, sim=False, litex_axi=False):
        axi_map = {
            "reram"     : 0x6000_0000, # +3M
            "sram"      : 0x6100_0000, # +2M
            "p_bus"     : 0x4000_0000, # +256M
            "memlcd"    : 0xb0000000,
            "vexriscv_debug": 0xefff_0000,
        }
        self.platform = platform

        # Clockgen cluster -------------------------------------------------------------------------
        warm_reset = Signal()
        self.submodules.crg = CRG(platform, sys_clk_freq, spinor_edge_delay_ns=2.5)
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

        # Wire up peripheral SoC busses
        p_axi = axi.AXILiteInterface(name="pbus")
        jtag_cpu = platform.request("jtag_cpu")

        # Add simulation "output pins" -----------------------------------------------------
        if sim:
            self.sim_report = CSRStorage(32, name = "report", description="A 32-bit value to report sim state")
            self.sim_success = CSRStorage(1, name = "success", description="Determines the result code for the simulation. 0 means fail, 1 means pass")
            self.sim_done = CSRStorage(1, name ="done", description="Set to `1` if the simulation should auto-terminate")

            sim = platform.request("sim")
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
        self.submodules.axi_reram = AXIRAM(platform, reram_axi, size=0x1_0000, name="reram", init=bios_data)

        sram_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
        self.submodules.axi_sram = AXIRAM(platform, sram_axi, size=0x1_0000, name="sram")

        # 3) Add AXICrossbar  (2 Slave / 2 Master).
        if not litex_axi:
            mbus = AXICrossbar(platform=platform)
            self.submodules += mbus
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
        else:
            self.mbus = SoCBusHandler(
                name                  = "CachedMemoryXbar",
                standard              = "axi",
                data_width            = 64,
                address_width         = 32,
                bursting              = True,
                interconnect          = "crossbar",
                interconnect_register = True,
            )

            # Add AXI Buses.
            self.mbus.add_master(name="ibus", master=ibus64_axi)
            self.mbus.add_master(name="dbus", master=dbus64_axi)

            self.mbus.add_slave(name="reram", slave=reram_axi, region=SoCRegion(origin=axi_map["reram"], size=0x1_0000, mode="rwx", cached=True))
            self.mbus.add_slave(name="sram", slave=sram_axi, region=SoCRegion(origin=axi_map["sram"], size=0x1_0000, mode="rwx", cached=True))

        # 4) Add peripherals
        # setup p_axi as the local bus master
        self.bus.add_master(name="pbus", master=p_axi)

        # add interrupt handler
        interrupt = Signal(32)
        self.cpu.interrupt = interrupt
        self.irq.enable()

        # Muxed UARTS ---------------------------------------------------------------------------
        self.gpio = CSRStorage(fields=[
            CSRField("uartsel", description="Select the UART", size=2, reset=0)
        ])
        uart_pins = platform.request("serial")
        serial_layout = [("tx", 1), ("rx", 1)]
        kernel_pads = Record(serial_layout)
        console_pads = Record(serial_layout)
        app_uart_pads = Record(serial_layout)
        self.comb += [
            If(self.gpio.fields.uartsel == 0,
                uart_pins.tx.eq(kernel_pads.tx),
                kernel_pads.rx.eq(uart_pins.rx),
            ).Elif(self.gpio.fields.uartsel == 1,
                uart_pins.tx.eq(console_pads.tx),
                console_pads.rx.eq(uart_pins.rx),
            ).Else(
                uart_pins.tx.eq(app_uart_pads.tx),
                app_uart_pads.rx.eq(uart_pins.rx),
            )
        ]
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
        self.submodules.memlcd = ClockDomainsRenamer({"sys":"sys_always_on"})(memlcd.MemLCD(platform.request("lcd"), interface="axi-lite"))
        self.add_csr("memlcd")
        self.bus.add_slave("memlcd", self.memlcd.bus, SoCRegion(origin=axi_map["memlcd"], size=self.memlcd.fb_depth*4, mode="rw", cached=False))

        # Internal reset ---------------------------------------------------------------------------
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
        trimming = platform.request("trimming")
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

        # Pull in DUT IP ---------------------------------------------------------------------------
        self.specials += Instance("cram_axi",
            i_aclk                = ClockSignal("sys"),
            i_rst                 = ResetSignal("sys"),
            i_always_on           = ClockSignal("sys"),
            i_trimming_reset      = trimming.reset,
            i_trimming_reset_ena  = trimming.reset_ena,
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

            o_coreuser            = sim.coreuser      ,
            i_irqarray_bank0      = self.irqtest0.fields.trigger,
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

class Platform(XilinxPlatform):
    def __init__(self, io, toolchain="vivado", programmer="vivado", part="50", encrypt=False, make_mod=False, bbram=False, strategy='default'):
        part = "xc7s" + part + "-csga324-1il"
        XilinxPlatform.__init__(self, part, io, toolchain=toolchain)

        if strategy != 'default':
            self.toolchain.vivado_route_directive = strategy
            self.toolchain.vivado_post_route_phys_opt_directive = "Explore"  # always explore if we're in a non-default strategy

        # NOTE: to do quad-SPI mode, the QE bit has to be set in the SPINOR status register. OpenOCD
        # won't do this natively, have to find a work-around (like using iMPACT to set it once)
        self.add_platform_command(
            "set_property CONFIG_VOLTAGE 1.8 [current_design]")
        self.add_platform_command(
            "set_property CFGBVS GND [current_design]")
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

            if args.sim:
                subprocess.run(["cargo", "xtask", "boot-image", "--feature", "sim"], check=True, cwd="boot")
            else:
                subprocess.run(["cargo", "xtask", "boot-image"], check=True, cwd="boot")
            bios_path = 'boot{}boot.bin'.format(os.path.sep)
    else:
        bios_path=None

    ##### second pass to build the actual chip. Note any changes below need to be reflected into the first pass...might be a good idea to modularize that
    ##### setup platform
    platform = Platform(io, encrypt=encrypt, bbram=bbram, strategy=args.strategy)

    ##### define the soc
    soc = CramSoC(
        platform,
        bios_path=bios_path,
        sys_clk_freq=75e6,
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
        SimRunner(args.ci, [], vex_verilog_path=VEX_VERILOG_PATH)

    return 0

if __name__ == "__main__":
    from datetime import datetime
    start = datetime.now()
    ret = main()
    print("Run completed in {}".format(datetime.now()-start))

    sys.exit(ret)
