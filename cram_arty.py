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
from litex.soc.cores.xadc import XADC
from litex.soc.cores.dna  import DNA

from litedram.modules import MT41K128M16
from litedram.phy import s7ddrphy

from litex.soc.interconnect.csr import *
from litex.soc.integration.doc import AutoDoc, ModuleDoc
from litex.soc.cores import uart

from litex.soc.interconnect.axi import AXIInterface, AXILiteInterface
from litex.soc.interconnect.axi import AXILite2Wishbone
from litex.soc.interconnect import axi
from axi_axil_adapter import AXI2AXILiteAdapter
from axi_crossbar import AXICrossbar
from axil_crossbar import AXILiteCrossbar
from axi_adapter import AXIAdapter
from axi_ram import AXIRAM
from axi_common import *

from axil_ahb_adapter import AXILite2AHBAdapter
from litex.soc.interconnect import ahb

# CRG ----------------------------------------------------------------------------------------------

class _CRG(LiteXModule):
    def __init__(self, platform, sys_clk_freq, with_dram=True, with_rst=True):
        self.rst    = Signal()
        self.cd_sys = ClockDomain()
        self.cd_eth = ClockDomain()
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
        pll.create_clkout(self.cd_sys, sys_clk_freq)
        pll.create_clkout(self.cd_eth, 25e6)
        self.comb += platform.request("eth_ref_clk").eq(self.cd_eth.clk)
        platform.add_false_path_constraints(self.cd_sys.clk, pll.clkin) # Ignore sys_clk to pll.clkin path created by SoC's rst.
        if with_dram:
            pll.create_clkout(self.cd_sys4x,     4*sys_clk_freq)
            pll.create_clkout(self.cd_sys4x_dqs, 4*sys_clk_freq, phase=90)
            pll.create_clkout(self.cd_idelay,    200e6)

        # IdelayCtrl.
        if with_dram:
            self.idelayctrl = S7IDELAYCTRL(self.cd_idelay)

# BaseSoC ------------------------------------------------------------------------------------------

class BaseSoC(SoCCore):
    def __init__(self, variant="a7-100", toolchain="vivado", sys_clk_freq=100e6,
        bios_path       = None,
        with_led_chaser = True,
        with_jtagbone   = True,
        with_spi_flash  = False,
        with_buttons    = False,
        with_pmod_gpio  = False,
        **kwargs):

        VEX_VERILOG_PATH = "VexRiscv/VexRiscv_CramSoC.v"
        axi_map = {
            "spiflash"  : 0x20000000,
            "reram"     : 0x6000_0000, # +3M
            "sram"      : 0x6100_0000, # +2M
            "p_bus"     : 0x4000_0000, # +256M
            "memlcd"    : 0x4200_0000,
            "vexriscv_debug": 0xefff_0000,
        }

        platform = digilent_arty.Platform(variant=variant, toolchain=toolchain)

        # CRG --------------------------------------------------------------------------------------
        self.crg  = _CRG(platform, sys_clk_freq, True)

        # SoCCore ----------------------------------------------------------------------------------
        platform.add_source("build/gateware/cram_axi.v")
        platform.add_source(VEX_VERILOG_PATH)
        platform.add_source("sim_support/ram_1w_1rs.v")
        platform.add_source("sim_support/prims.v")
        # this must be pulled in manually because it's instantiated in the core design, but not in the SoC design
        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "verilog-axi", "rtl")
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter.v"))
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter_wr.v"))
        platform.add_source(os.path.join(rtl_dir, "axi_axil_adapter_rd.v"))

        # SoCMini ----------------------------------------------------------------------------------
        SoCMini.__init__(self,
            platform,
            clk_freq=int(sys_clk_freq),
            csr_paging           = 4096,  # increase paging to 1 page size
            csr_address_width    = 16,    # increase to accommodate larger page size
            bus_standard         = "axi-lite",
            # bus_timeout          = None,         # use this if regular_comb=True on the builder
            with_ctrl            = False,
            ident                = "Cramium SoC on Arty A7",
            io_regions           = {
                # Origin, Length.
                0x4000_0000 : 0x2000_0000,
                0xa000_0000 : 0x6000_0000,
            },
        )
        self.add_memory_region(name="sram", origin=axi_map["sram"], length=16*1024*1024)
        # Wire up peripheral SoC busses
        p_axi = axi.AXILiteInterface(name="pbus")

        # Add simulation "output pins" -----------------------------------------------------
        self.sim_report = CSRStorage(32, name = "report", description="A 32-bit value to report sim state")
        self.sim_success = CSRStorage(1, name = "success", description="Determines the result code for the simulation. 0 means fail, 1 means pass")
        self.sim_done = CSRStorage(1, name ="done", description="Set to `1` if the simulation should auto-terminate")
        # test that caching is OFF for the I/O regions
        self.sim_coherence_w = CSRStorage(32, name= "wdata", description="Write values here to check cache coherence issues")
        self.sim_coherence_r = CSRStatus(32, name="rdata", description="Data readback derived from coherence_w")
        self.sim_coherence_inc = CSRStatus(32, name="rinc", description="Every time this is read, the base value is incremented by 3", reset=0)
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

        # 3) Add AXICrossbar  (2 Slave / 1 Master).
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
        dram_axi = AXIInterface(data_width=32, address_width=32, id_width=2, bursting=True)
        mbus.add_master(name = "reram", m_axi=dram_axi, origin=axi_map["reram"], size=0x0100_0000 * 2) # maps both ReRAM and SRAM

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
        self.bus.add_master(name="pbus", master=testbench_axil)

        # DDR3 SDRAM -------------------------------------------------------------------------------
        self.cpu.memory_buses = [dram_axi]
        self.ddrphy = s7ddrphy.A7DDRPHY(platform.request("ddram"),
            memtype        = "DDR3",
            nphases        = 4,
            sys_clk_freq   = sys_clk_freq)
        self.add_sdram("sdram",
            phy           = self.ddrphy,
            module        = MT41K128M16(sys_clk_freq, "1:4"),
            l2_cache_size = kwargs.get("l2_size", 8192)
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
                pads         = platform.request_all("user_led"),
                sys_clk_freq = sys_clk_freq,
            )

        # Buttons ----------------------------------------------------------------------------------
        if with_buttons:
            self.buttons = GPIOIn(
                pads     = platform.request_all("user_btn"),
                with_irq = self.irq.enabled
            )

        # GPIOs ------------------------------------------------------------------------------------
        if with_pmod_gpio:
            platform.add_extension(digilent_arty.raw_pmod_io("pmoda"))
            self.gpio = GPIOTristate(
                pads     = platform.request("pmoda"),
                with_irq = self.irq.enabled
            )

# Build --------------------------------------------------------------------------------------------

def main():
    from litex.build.parser import LiteXArgumentParser
    parser = LiteXArgumentParser(platform=digilent_arty.Platform, description="LiteX SoC on Arty A7.")
    parser.add_target_argument("--flash",        action="store_true",       help="Flash bitstream.")
    parser.add_target_argument("--variant",      default="a7-100",           help="Board variant (a7-35 or a7-100).")
    parser.add_target_argument("--sys-clk-freq", default=100e6, type=float, help="System clock frequency.")
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

    soc = BaseSoC(
        variant        = args.variant,
        toolchain      = args.toolchain,
        sys_clk_freq   = args.sys_clk_freq,
        with_xadc      = args.with_xadc,
        with_dna       = args.with_dna,
        with_ethernet  = args.with_ethernet,
        with_etherbone = args.with_etherbone,
        eth_ip         = args.eth_ip,
        eth_dynamic_ip = args.eth_dynamic_ip,
        with_jtagbone  = args.with_jtagbone,
        with_spi_flash = args.with_spi_flash,
        with_pmod_gpio = args.with_pmod_gpio,
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