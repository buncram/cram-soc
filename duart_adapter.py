#
# Adapt AXILite to AHB. Derived from files in the verilog-axi test directory
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

# AHB to APB to DUART --------------------------------------------------------------------------

class DuartAdapter(Module):
    def __init__(self, platform, s_ahb, pads, sel_addr = 0x1000,
        address_width = 12,
    ):
        self.logger = logging.getLogger("DuartAdapter")

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
        sel_fullwidth = Signal(12, reset=((sel_addr & 0xFF_FFFF) >> 12))

        self.specials += Instance("cmsdk_ahb_to_apb",
            p_ADDRWIDTH            = address_width,

            i_HCLK                 = ClockSignal(),
            i_HRESETn              = ~ResetSignal(),
            i_PCLKEN               = 1,
            i_HSEL                 = s_ahb.addr[12:24] == sel_fullwidth,
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


        self.specials += Instance("duart_top",
            # Parameters.
            # -----------
            p_AW = address_width,
            p_INITETU = 32,

            # Clk / Rst.
            # ----------
            i_clk = ClockSignal(),
            i_resetn = ~ResetSignal(),
            i_sclk = ClockSignal(),

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
            o_txd                  = pads.tx,
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
        rtl_dir = os.path.join(os.path.dirname(__file__), "do_not_checkin", "rtl")
        platform.add_source(os.path.join(rtl_dir, "duart_v0.1.sv"))
