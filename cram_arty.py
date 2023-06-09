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
from axil_cdc import AXILiteCDC

from axil_ahb_adapter import AXILite2AHBAdapter
from litex.soc.interconnect import ahb

from litex.build.generic_platform import *

from math import log2, ceil

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

# BaseSoC ------------------------------------------------------------------------------------------

class BaseSoC(SoCCore):
    def __init__(self, platform,
        variant = "fpga",
        sys_clk_freq=100e6,
        bios_path       = None,
        with_led_chaser = True,
        with_jtagbone   = True,
        with_spi_flash  = False,
        with_buttons    = False,
        with_pmod_gpio  = False,
        **kwargs):

        VEX_VERILOG_PATH = "VexRiscv/VexRiscv_CramSoC.v"
        axi_mem_map = {
            "reram"          : [0x6000_0000, 4 * 1024 * 1024], # +4M
            "sram"           : [0x6100_0000, 2 * 1024 * 1024], # +2M
            "xip"           :  [0x7000_0000, 128 * 1024 * 1024], # up to 128MiB of XIP
            "vexriscv_debug" : [0xefff_0000, 0x1000],
        }
        # Firmware note:
        #    - entire region from 0x4000_0000 through 0x4010_0000 is VM-mapped in test bench
        #    - entire region from 0x5012_0000 through 0x5013_0000 is VM-mapped in test bench
        axi_peri_map = {
            "testbench" : [0x4008_0000, 0x1_0000], # 64k
            "duart"     : [0x4004_2000, 0x0_1000],
            "pio"       : [0x5012_3000, 0x0_1000],
        }
        self.mem_map = {**SoCCore.mem_map, **{
            "emu_ram": axi_mem_map["reram"][0],
            "csr": axi_peri_map["testbench"][0], # save bottom 0x10_0000 for compatibility with Cramium native registers
        }}

        # CRG --------------------------------------------------------------------------------------
        sleep_req = Signal()
        self.crg  = _CRG(platform, sys_clk_freq, True, sleep_req=sleep_req)

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
            with_uart            = True,
        )
        # populate regions for SVD export
        # not used in the FPGA version because it's all mapped to one big RAM
        # for (name, region) in axi_mem_map.items():
        #    self.add_memory_region(name=name, origin=region[0], length=region[1])

        # Wire up peripheral SoC busses
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
        mbus.add_master(name = "reram", m_axi=dram_axi, origin=axi_mem_map["reram"][0], size=0x0100_0000 * 2) # maps both ReRAM and SRAM

        # 4) Add peripherals
        # build the controller port for the peripheral crossbar
        self.submodules.pxbar = pxbar = AXILiteCrossbar(platform)
        p_axil = AXILiteInterface(name="pbus", bursting = False)
        pxbar.add_slave(
            name = "p_axil", s_axil = p_axil,
        )
        # This region is used for testbench elements (e.g., does not appear in the final SoC):
        # these are peripherals that are inferred by LiteX in this module such as the UART to facilitate debug
        for (name, region) in axi_peri_map.items():
            setattr(self, name + "_region", SoCIORegion(region[0], region[1], mode="rw", cached=False))
            setattr(self, name + "_axil", AXILiteInterface(name=name + "_axil"))
            pxbar.add_master(
                name = name,
                m_axil = getattr(self, name + "_axil"),
                origin = region[0],
                size = region[1],
            )
            if name == "testbench":
                # connect the testbench master
                self.bus.add_master(name="pbus", master=self.testbench_axil)
            else:
                # connect the SoC via AHB adapters
                setattr(self, name + "_slower_axil", AXILiteInterface(clock_domain="p", name=name + "_slower_axil"))
                setattr(self.submodules, name + "_slower_axi",
                        AXILiteCDC(platform,
                                   getattr(self, name + "_axil"),
                                   getattr(self, name + "_slower_axil"),
                        ))
                setattr(self, name + "_ahb", ahb.Interface())
                self.submodules += ClockDomainsRenamer({"sys" : "p"})(
                    AXILite2AHBAdapter(platform,
                                       getattr(self, name + "_slower_axil"),
                                       getattr(self, name + "_ahb")
                ))
                # wire up the specific subsystems
                if name == "pio":
                    from pio_adapter import PioAdapter
                    pio_irq0 = Signal()
                    pio_irq1 = Signal()
                    self.submodules += ClockDomainsRenamer({"sys" : "p", "pio": "sys"})(PioAdapter(platform,
                        getattr(self, name +"_ahb"), platform.request("pio"), pio_irq0, pio_irq1, sel_addr=region[0],
                        sim=True # this will cause some funky stuff to appear on the GPIO for simulation frameworking/testbenching
                    ))
                elif name == "duart":
                    from duart_adapter import DuartAdapter
                    self.submodules += ClockDomainsRenamer({"sys" : "p"})(DuartAdapter(platform,
                        getattr(self, name + "_ahb"), pads=platform.request("duart"), sel_addr=region[0]
                    ))
                else:
                    print("Missing binding for peripheral block {}".format(name))
                    exit(1)

        # add interrupt handler
        interrupt = Signal(32)
        self.cpu.interrupt = interrupt
        self.irq.enable()

        # add JTAG pins to header
        jtag_cpu = platform.request("jtag_cpu")

        # DDR3 SDRAM -------------------------------------------------------------------------------
        self.ddrphy = s7ddrphy.A7DDRPHY(platform.request("ddram"),
            memtype        = "DDR3",
            nphases        = 4,
            sys_clk_freq   = sys_clk_freq)
        self.add_sdram_emu("sdram",
            mem_bus       = dram_axi,
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
        wfi_loopback = Signal(20)
        wfi_delay = Signal(10, reset=512) # coded as a one-shot
        self.sync.sys_always_on += [
            If(sleep_req & (wfi_delay > 0),
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
            i_always_on           = ClockSignal("sys_always_on"),
            i_cmatpg             = 0,
            i_cmbist             = 0,
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
            i_jtag_trst           = jtag_cpu.trst  | ResetSignal("sys"), # integration note: this needs to be wired up

            o_coreuser            = platform.request("rgb_led", number=0).g      ,
            i_irqarray_bank0      = self.irqtest0.fields.trigger | irq0_wire_or,
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

            o_sleep_req            = sleep_req,
        )

    def add_sdram_emu(self, name="sdram", mem_bus=None, phy=None, module=None, origin=None, size=None,
        l2_cache_size           = 8192,
        l2_cache_min_data_width = 128,
        l2_cache_reverse        = False,
        l2_cache_full_memory_we = True,
        **kwargs):

        # Imports.
        from litedram.common import LiteDRAMNativePort
        from litedram.core import LiteDRAMCore
        from litedram.frontend.wishbone import LiteDRAMWishbone2Native
        from litex.soc.interconnect import wishbone

        # LiteDRAM core.
        self.check_if_exists(name)
        sdram = LiteDRAMCore(
            phy             = phy,
            geom_settings   = module.geom_settings,
            timing_settings = module.timing_settings,
            clk_freq        = self.sys_clk_freq,
            **kwargs)
        self.add_module(name=name, module=sdram)

        # Save SPD data to be able to verify it at runtime.
        if hasattr(module, "_spd_data"):
            # Pack the data into words of bus width.
            bytes_per_word = self.bus.data_width // 8
            mem = [0] * ceil(len(module._spd_data) / bytes_per_word)
            for i in range(len(mem)):
                for offset in range(bytes_per_word):
                    mem[i] <<= 8
                    if self.cpu.endianness == "little":
                        offset = bytes_per_word - 1 - offset
                    spd_byte = i * bytes_per_word + offset
                    if spd_byte < len(module._spd_data):
                        mem[i] |= module._spd_data[spd_byte]
            self.add_rom(
                name     = f"{name}_spd",
                origin   = self.mem_map.get(f"{name}_spd", None),
                size     = len(module._spd_data),
                contents = mem,
            )

        # Compute/Check SDRAM size.
        sdram_size = 2**(module.geom_settings.bankbits +
                         module.geom_settings.rowbits +
                         module.geom_settings.colbits)*phy.settings.nranks*phy.settings.databits//8
        if size is not None:
            sdram_size = min(sdram_size, size)

        # Add SDRAM region.
        main_ram_region = SoCRegion(
            origin = self.mem_map.get("emu_ram", origin),
            size   = sdram_size,
            mode   = "rwx")
        self.bus.add_region("emu_ram", main_ram_region)

        # Down-convert width by going through a wishbone interface. also gets us a cache maybe?
        mem_wb  = wishbone.Interface(
            data_width = mem_bus.data_width,
            adr_width  = 32-log2_int(mem_bus.data_width//8))
        mem_a2w = axi.AXI2Wishbone(
            axi          = mem_bus,
            wishbone     = mem_wb,
            base_address = 0)
        self.submodules += mem_a2w

        # Insert L2 cache inbetween Wishbone bus and LiteDRAM
        l2_cache_size = max(l2_cache_size, int(2*mem_bus.data_width/8)) # Use minimal size if lower
        l2_cache_size = 2**int(log2(l2_cache_size))                  # Round to nearest power of 2
        l2_cache_data_width = max(mem_bus.data_width, l2_cache_min_data_width)
        l2_cache = wishbone.Cache(
            cachesize = l2_cache_size//4,
            master    = mem_wb,
            slave     = wishbone.Interface(l2_cache_data_width),
            reverse   = l2_cache_reverse)
        if l2_cache_full_memory_we:
            l2_cache = FullMemoryWE()(l2_cache)
        self.l2_cache = l2_cache
        litedram_wb = self.l2_cache.slave
        self.add_config("L2_SIZE", l2_cache_size)

        # Request a LiteDRAM native port.
        port = sdram.crossbar.get_port()
        self.submodules += LiteDRAMWishbone2Native(
            wishbone     = litedram_wb,
            port         = port,
            base_address = self.bus.regions["emu_ram"].origin)

def arty_extensions(self, **kwargs):
    # Clockgen cluster -------------------------------------------------------------------------
    self.crg  = _CRG(self.platform, self.sys_clk_freq, True, sleep_req=self.sleep_req)

    # Memory regions ---------------------------------------------------------------------------
    # All the memory regions are emulated with a single, large RAM port that is big enough to overlap
    # both ReRAM and SRAM.
    self.mem_map["emu_ram"] = self.axi_mem_map["reram"][0]

    dram_axi = AXIInterface(data_width=32, address_width=32, id_width=2, bursting=True)
        # maps both ReRAM and SRAM
    self.mbus.add_master(name = "reram", m_axi=dram_axi, origin=self.axi_mem_map["reram"][0], size=0x0100_0000 * 2)

    # DDR3 SDRAM -------------------------------------------------------------------------------
    self.ddrphy = s7ddrphy.A7DDRPHY(self.platform.request("ddram"),
        memtype        = "DDR3",
        nphases        = 4,
        sys_clk_freq   = self.sys_clk_freq)
    self.add_sdram_emu("sdram",
        mem_bus       = dram_axi,
        phy           = self.ddrphy,
        module        = MT41K128M16(self.sys_clk_freq, "1:4"),
        l2_cache_size = kwargs.get("l2_size", 8192)
    )

    # Various other I/Os
    self.comb += self.platform.request("rgb_led", number=0).g.eq(self.coreuser)

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

    soc = BaseSoC(
        platform,
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