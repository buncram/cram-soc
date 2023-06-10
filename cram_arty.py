#!/usr/bin/env python3

#
# This file is part of LiteX-Boards.
#
# Copyright (c) 2015-2019 Florent Kermarrec <florent@enjoy-digital.fr>
# Copyright (c) 2020 Antmicro <www.antmicro.com>
# Copyright (c) 2022 Victor Suarez Rovere <suarezvictor@gmail.com>
# SPDX-License-Identifier: BSD-2-Clause

# Note: For now with --toolchain=yosys+nextpnr:
# - DDR3 should be disabled: ex --integrated-main-ram-size=8192
# - Clk Freq should be lowered: ex --sys-clk-freq=50e6

from migen import *

from litex.gen import *

import digilent_arty

from litex.soc.cores.clock import *
from litex.soc.integration.soc import SoCRegion, SoCIORegion
from litex.soc.integration.soc_core import *
from litex.soc.integration.builder import *
from litex.soc.cores.led import LedChaser
from litex.soc.cores.gpio import GPIOIn, GPIOTristate

from litedram.modules import MT41K128M16
from litedram.phy import s7ddrphy

from litex.soc.interconnect.csr import *
from litex.soc.integration.doc import AutoDoc, ModuleDoc
from litex.soc.cores import uart

from litex.soc.interconnect.axi import AXIInterface

from litex.build.generic_platform import *

from cram_common import CramSoC

# CRG ----------------------------------------------------------------------------------------------

class _CRG(LiteXModule):
    def __init__(self, platform, sys_clk_freq, with_dram=True, with_rst=True, sleep_req=None):
        self.rst    = Signal()
        self.cd_sys = ClockDomain()
        self.cd_sys_always_on = ClockDomain()
        self.cd_p   = ClockDomain()
        self.cd_pio = ClockDomain()
        if with_dram:
            self.cd_sys4x     = ClockDomain()
            self.cd_sys4x_dqs = ClockDomain()
            self.cd_idelay    = ClockDomain()

        # # #

        # Clk/Rst.
        clk100 = platform.request("clk100")
        rst    = ~platform.request("cpu_reset") if with_rst else 0

        # PLL.
        self.pll = pll = S7PLL(speedgrade=-1)
        self.comb += pll.reset.eq(rst | self.rst)
        pll.register_clkin(clk100, 100e6)
        pll.create_clkout(self.cd_sys, sys_clk_freq, buf="bufgce", ce=(pll.locked & ~sleep_req))
        pll.create_clkout(self.cd_sys_always_on, sys_clk_freq)
        pll.create_clkout(self.cd_p, 25e6)
        platform.add_false_path_constraints(self.cd_sys.clk, pll.clkin) # Ignore sys_clk to pll.clkin path created by SoC's rst.
        if with_dram:
            pll.create_clkout(self.cd_sys4x,     4*sys_clk_freq)
            pll.create_clkout(self.cd_sys4x_dqs, 4*sys_clk_freq, phase=90)
            pll.create_clkout(self.cd_idelay,    200e6)

        # IdelayCtrl.
        if with_dram:
            self.idelayctrl = S7IDELAYCTRL(self.cd_idelay)

# CramSoC ------------------------------------------------------------------------------------------

def arty_extensions(self,
                    l2_cache_size = 8192,
                    with_led_chaser = True,
                    with_buttons = True,
                    with_jtagbone = False,
                    with_spi_flash = False,
                    with_pmod_gpio = False,
                    ):
    # weird syntax needed otherwise somehow this gets added as a tuple instead of as an integer...
    self.mem_map = {**self.mem_map, **{
        "emu_ram" : self.axi_mem_map["reram"][0]
    }}

    # CRG --------------------------------------------------------------------------------------
    self.crg  = _CRG(self.platform, self.sys_clk_freq, True, sleep_req=self.sleep_req)

    # Various other I/Os
    self.comb += self.platform.request("rgb_led", number=0).g.eq(self.coreuser)

    # DDR3 SDRAM -------------------------------------------------------------------------------
    dram_axi = AXIInterface(data_width=64, address_width=32, id_width=2, bursting=True)
    self.mbus.add_master(name = "reram", m_axi=dram_axi,
                            origin=self.axi_mem_map["reram"][0], size=0x0100_0000 * 2) # maps both ReRAM and SRAM

    self.ddrphy = s7ddrphy.A7DDRPHY(self.platform.request("ddram"),
        memtype        = "DDR3",
        nphases        = 4,
        sys_clk_freq   = self.sys_clk_freq)
    self.add_sdram_emu("sdram",
        mem_bus       = dram_axi,
        phy           = self.ddrphy,
        module        = MT41K128M16(self.sys_clk_freq, "1:4"),
        l2_cache_size = l2_cache_size,
    )

    # Jtagbone ---------------------------------------------------------------------------------
    if with_jtagbone:
        self.add_jtagbone()

    # SPI Flash --------------------------------------------------------------------------------
    if with_spi_flash:
        from litespi.modules import S25FL128L
        from litespi.opcodes import SpiNorFlashOpCodes as Codes
        self.add_spi_flash(mode="4x", module=S25FL128L(Codes.READ_1_1_4), rate="1:2", with_master=True)

    # Leds -------------------------------------------------------------------------------------
    if with_led_chaser:
        self.leds = LedChaser(
            pads         = self.platform.request_all("user_led"),
            sys_clk_freq = self.sys_clk_freq,
        )

    # Buttons ----------------------------------------------------------------------------------
    if with_buttons:
        self.buttons = GPIOIn(
            pads     = self.platform.request_all("user_btn"),
            with_irq = self.irq.enabled
        )

    # GPIOs ------------------------------------------------------------------------------------
    if with_pmod_gpio:
        self.platform.add_extension(digilent_arty.raw_pmod_io("pmoda"))
        self.gpio = GPIOTristate(
            pads     = self.platform.request("pmoda"),
            with_irq = self.irq.enabled
        )

# Build --------------------------------------------------------------------------------------------

def main():
    from litex.build.parser import LiteXArgumentParser
    parser = LiteXArgumentParser(platform=digilent_arty.Platform, description="LiteX SoC on Arty A7.")
    parser.add_target_argument("--flash",        action="store_true",       help="Flash bitstream.")
    parser.add_target_argument("--variant",      default="a7-100",           help="Board variant (a7-35 or a7-100).")
    parser.add_target_argument("--sys-clk-freq", default=50e6, type=float, help="System clock frequency.")
    parser.add_target_argument("--with-xadc",    action="store_true",       help="Enable 7-Series XADC.")
    parser.add_target_argument("--with-dna",     action="store_true",       help="Enable 7-Series DNA.")
    ethopts = parser.target_group.add_mutually_exclusive_group()
    ethopts.add_argument("--with-ethernet",        action="store_true",    help="Enable Ethernet support.")
    ethopts.add_argument("--with-etherbone",       action="store_true",    help="Enable Etherbone support.")
    parser.add_target_argument("--eth-ip",         default="192.168.1.50", help="Ethernet/Etherbone IP address.")
    parser.add_target_argument("--eth-dynamic-ip", action="store_true",    help="Enable dynamic Ethernet IP addresses setting.")
    sdopts = parser.target_group.add_mutually_exclusive_group()
    sdopts.add_argument("--with-spi-sdcard",       action="store_true", help="Enable SPI-mode SDCard support.")
    sdopts.add_argument("--with-sdcard",           action="store_true", help="Enable SDCard support.")
    parser.add_target_argument("--sdcard-adapter",                      help="SDCard PMOD adapter (digilent or numato).")
    parser.add_target_argument("--with-jtagbone",  action="store_true", help="Enable JTAGbone support.")
    parser.add_target_argument("--with-spi-flash", action="store_true", help="Enable SPI Flash (MMAPed).")
    parser.add_target_argument("--with-pmod-gpio", action="store_true", help="Enable GPIOs through PMOD.") # FIXME: Temporary test.
    args = parser.parse_args()

    assert not (args.with_etherbone and args.eth_dynamic_ip)

    platform = digilent_arty.Platform(variant=args.variant, toolchain=args.toolchain)

    # add various platform I/O extensions
    pio = [
        ("pio", 0,
            Subsignal("gpio", Pins(" ".join([f"ck_io:ck_io{i:d}" for i in range(32)]))),
            IOStandard("LVCMOS33"),
        )
    ]
    platform.add_extension(pio)
    duart = [
        ("duart", 0,
            Subsignal("tx", Pins("ck_io:ck_io40")),
            Subsignal("rx", Pins("ck_io:ck_io41")),
            IOStandard("LVCMOS33"),
        )
    ]
    platform.add_extension(duart)
    jtag_pins = [
        ("jtag_cpu", 0,
            Subsignal("tck",  Pins("ck_io:ck_io33")),
            Subsignal("tms",  Pins("ck_io:ck_io34")),
            Subsignal("tdi",  Pins("ck_io:ck_io35")),
            Subsignal("tdo",  Pins("ck_io:ck_io36")),
            Subsignal("trst", Pins("ck_io:ck_io37")),
            Misc("SLEW=SLOW"),
            IOStandard("LVCMOS33"),
        )
    ]
    platform.add_extension(jtag_pins)

    soc = CramSoC(
        platform,
        sys_clk_freq   = args.sys_clk_freq,
        variant        = args.variant,
        **parser.soc_argdict
    )
    if args.sdcard_adapter == "numato":
        soc.platform.add_extension(digilent_arty._numato_sdcard_pmod_io)
    else:
        soc.platform.add_extension(digilent_arty._sdcard_pmod_io)
    if args.with_spi_sdcard:
        soc.add_spi_sdcard()
    if args.with_sdcard:
        soc.add_sdcard()

    CramSoC.arty_extensions = arty_extensions
    soc.arty_extensions(
        l2_cache_size  = 32768,
        with_led_chaser= True,
        with_buttons   = True,
        with_jtagbone  = args.with_jtagbone,
        with_spi_flash = args.with_spi_flash,
        with_pmod_gpio = args.with_pmod_gpio,
    )

    builder = Builder(soc, **parser.builder_argdict)
    if args.build:
        builder.build(**parser.toolchain_argdict)

    if args.load:
        prog = soc.platform.create_programmer()
        prog.load_bitstream(builder.get_bitstream_filename(mode="sram"))

    if args.flash:
        prog = soc.platform.create_programmer()
        prog.flash(0, builder.get_bitstream_filename(mode="flash"))

if __name__ == "__main__":
    main()