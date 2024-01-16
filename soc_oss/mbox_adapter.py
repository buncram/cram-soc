#
# Adapt Mbox AHB client to LiteX native bus interface
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
from soc_oss.axi_common import *

# AHB to APB to PIO --------------------------------------------------------------------------

class MboxAdapter(Module):
    def __init__(self, platform, s_ahb, mbox, irq_available, irq_abort_init, irq_abort_done, irq_error, sel_addr = 0x4000,
        address_width = 12, sim=False,
    ):
        self.logger = logging.getLogger("MboxAdapter")

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

            i_HCLK                 = ClockSignal("pclk"),
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

        self.specials += Instance("mbox_wrapper",
            # Parameters.
            # -----------
            p_AW = 12,

            # Clk / Rst.
            # ----------
            i_aclk = ClockSignal(),
            i_pclk = ClockSignal("pclk"),
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

            o_mbox_w_dat           = mbox.w_dat,
            o_mbox_w_valid         = mbox.w_valid,
            i_mbox_w_ready         = mbox.w_ready,
            o_mbox_w_done          = mbox.w_done,
            i_mbox_r_dat           = mbox.r_dat,
            i_mbox_r_valid         = mbox.r_valid,
            o_mbox_r_ready         = mbox.r_ready,
            i_mbox_r_done          = mbox.r_done,
            o_mbox_w_abort         = mbox.w_abort,
            i_mbox_r_abort         = mbox.r_abort,

            o_irq_available        = irq_available,
            o_irq_abort_init       = irq_abort_init,
            o_irq_abort_done       = irq_abort_done,
            o_irq_error            = irq_error,
        )

        # Add Sources.
        # ------------
        self.add_sources(platform)

    @staticmethod
    def add_sources(platform):
        rtl_dir = os.path.join(os.path.dirname(__file__), "..", "soc_oss", "rtl", "common")
        platform.add_source(os.path.join(rtl_dir, "template.sv"))
        platform.add_source(os.path.join(rtl_dir, "amba_interface_def_v0.2.sv"))
        platform.add_source(os.path.join(rtl_dir, "io_interface_def_v0.1.sv"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "..", "soc_oss", "rtl", "amba")
        platform.add_source(os.path.join(rtl_dir, "apb_sfr_v0.1.sv"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "..", "sim_support")
        platform.add_source(os.path.join(rtl_dir, "mbox_v0.1.sv"))
        platform.add_source(os.path.join(rtl_dir, "mbox_client.v"))
        platform.add_source(os.path.join(rtl_dir, "mbox_wrapper.sv"))

        print(f"TODO: clean up this contamination! {__file__}")
        rtl_dir = os.path.join(os.path.dirname(__file__), "..", "soc_mpw", "ips", "ambabuilder", "logical", "cmsdk_ahb_to_apb", "verilog")
        platform.add_source(os.path.join(rtl_dir, "cmsdk_ahb_to_apb.v"))
