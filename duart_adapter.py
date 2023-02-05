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

# AXI to AXI-Lite Adapter --------------------------------------------------------------------------

class DuartAdapter(Module):
    def __init__(self, platform, s_ahb, pads, sel_addr = 0x1000,
        address_width = 12,
    ):
        self.logger = logging.getLogger("DuartAdapter")

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
            i_PADDR                = s_ahb.addr[:address_width],
            i_PENABLE              = 1,
            i_PWRITE               = s_ahb.write,
            i_PSTRB                = 0xf,
            i_PPROT                = s_ahb.prot,
            i_PWDATA               = s_ahb.wdata,
            i_PSEL                 = s_ahb.addr[12:28] == (sel_addr >> 12),
            i_APBACTIVE            = 1,
            o_PRDATA               = s_ahb.rdata,
            o_PREADY               = s_ahb.readyout,
            o_PSLVERR              = Open(),
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
        rtl_dir = os.path.join(os.path.dirname(__file__), "do_not_checkin", "rtl")
        platform.add_source(os.path.join(rtl_dir, "duart_v0.1.sv"))
