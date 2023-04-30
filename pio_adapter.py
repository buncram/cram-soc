#
# Adapt PIO to LiteX native bus interface
#
# Copyright (c) 2022 Cramium Inc
# Copyright (c) 2022 Florent Kermarrec <florent@enjoy-digital.fr>
# SPDX-License-Identifier: BSD-2-Clause

import os
import math
import logging

from enum import IntEnum

from migen import *

from litex.soc.interconnect.axi import *
from litex.soc.interconnect import ahb
from axi_common import *

# AHB to APB to PIO --------------------------------------------------------------------------

class PioAdapter(Module):
    def __init__(self, platform, s_ahb, pads, irq0, irq1, sel_addr = 0x2000,
        address_width = 12,
    ):
        self.logger = logging.getLogger("PioAdapter")

        apb_addr = Signal(address_width)
        apb_enable = Signal()
        apb_write = Signal()
        apb_strb = Signal(4)
        apb_prot = Signal(3)
        apb_wdata = Signal(32)
        apb_sel = Signal()
        apb_active = Signal()
        apb_rdata = Signal(32)
        apb_ready = Signal()
        apb_slverr = Signal()

        self.specials += Instance("cmsdk_ahb_to_apb",
            p_ADDRWIDTH            = address_width,

            i_HCLK                 = ClockSignal(),
            i_HRESETn              = ~ResetSignal(),
            i_PCLKEN               = 1,
            i_HSEL                 = s_ahb.addr[12:28] == (sel_addr >> 12),
            i_HADDR                = s_ahb.addr[:address_width],
            i_HTRANS               = s_ahb.trans,
            i_HSIZE                = s_ahb.size,
            i_HPROT                = s_ahb.prot,
            i_HWRITE               = s_ahb.write,
            i_HREADY               = 1, # s_ahb.mastlock, # ??
            i_HWDATA               = s_ahb.wdata,

            o_HREADYOUT            = s_ahb.readyout,
            o_HRDATA               = s_ahb.rdata,
            o_HRESP                = s_ahb.resp,

            o_PADDR                = apb_addr,
            o_PENABLE              = apb_enable,
            o_PWRITE               = apb_write,
            o_PSTRB                = apb_strb,
            o_PPROT                = apb_prot,
            o_PWDATA               = apb_wdata,
            o_PSEL                 = apb_sel,
            o_APBACTIVE            = apb_active,

            i_PRDATA               = apb_rdata,
            i_PREADY               = apb_ready,
            i_PSLVERR              = apb_slverr,
        )

        self.gpio = TSTriple(32)
        self.specials += self.gpio.get_tristate(pads.gpio)

        self.specials += Instance("pio_ahb",
            # Parameters.
            # -----------
            p_AW = 12,

            # Clk / Rst.
            # ----------
            i_clk = ClockSignal(),
            i_resetn = ~ResetSignal(),
            i_cmatpg = Open(),
            i_cmbist = Open(),

            # AHB Slave interface
            # --------------------------
            i_PADDR                = apb_addr,
            i_PENABLE              = apb_enable,
            i_PWRITE               = apb_write,
            i_PSTRB                = apb_strb,
            i_PPROT                = apb_prot,
            i_PWDATA               = apb_wdata,
            i_PSEL                 = apb_sel,
            i_APBACTIVE            = apb_active,
            o_PRDATA               = apb_rdata,
            o_PREADY               = apb_ready,
            o_PSLVERR              = apb_slverr,

            # gpio interfaces
            i_gpio_in              = self.gpio.i,
            o_gpio_out             = self.gpio.o,
            o_gpio_dir             = self.gpio.oe, # 1 is output

            # irq interfaces
            i_irq0                 = irq0,
            i_irq1                 = irq1,
        )

        # Add Sources.
        # ------------
        self.add_sources(platform)

    @staticmethod
    def add_sources(platform):
        rtl_dir = os.path.join(os.path.dirname(__file__), "do_not_checkin", "rtl", "amba")
        platform.add_source(os.path.join(rtl_dir, "template.sv"))
        platform.add_source(os.path.join(rtl_dir, "amba_interface_def_v0.2.sv"))
        platform.add_source(os.path.join(rtl_dir, "apb_sfr_v0.1.sv"))
        platform.add_source(os.path.join(rtl_dir, "cmsdk_ahb_to_apb.v"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "pio")
        platform.add_source(os.path.join(rtl_dir, "pio_ahb.sv"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "pio", "upstream", "src")
        platform.add_source(os.path.join(rtl_dir, "decoder.v"))
        platform.add_source(os.path.join(rtl_dir, "divider.v"))
        platform.add_source(os.path.join(rtl_dir, "fifo.v"))
        platform.add_source(os.path.join(rtl_dir, "isr.v"))
        platform.add_source(os.path.join(rtl_dir, "machine.v"))
        platform.add_source(os.path.join(rtl_dir, "osr.v"))
        platform.add_source(os.path.join(rtl_dir, "pc.v"))
        platform.add_source(os.path.join(rtl_dir, "scratch.v"))
