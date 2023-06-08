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
        address_width = 12, sim=False,
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

        gpio_i = Signal(32)
        gpio_o = Signal(32)
        gpio_oe = Signal(32)
        if sim:
            FAILING_ADDRESS = 0x17
            i2c_scl = Signal()
            i2c_sda = Signal()
            i2c_scl_d = Signal()
            i2c_sda_d = Signal()
            i2c_ctr = Signal(4)
            i2c_adr_in = Signal(8)
            i2c_dout = Signal(8)
            zero = Signal()
            self.sync += [
                i2c_sda_d.eq(i2c_sda),
                i2c_scl_d.eq(i2c_scl),
            ]
            i2c_sda_controller_drive_low = Signal()
            i2c_sda_peripheral_drive_low = Signal()
            # crappy I2C peripheral emulator just for testing purposes. Because it was faster
            # to write this than try to find a verilog model and adapt it into this purpose.
            # Note: this would never work in any real situation because it doesn't handle noise, spurious edges, etc.
            self.submodules.i2c_peri = i2c_p = FSM(reset_state="IDLE")
            i2c_p.act("IDLE",
                If(i2c_sda_d & ~i2c_sda & i2c_scl & i2c_scl_d, # start condition
                   NextValue(i2c_ctr, 8),
                   NextState("START_A")
                )
            )
            i2c_p.act("START_A",
                If(i2c_sda_d & ~i2c_sda & i2c_scl & i2c_scl_d, # start condition
                   NextValue(i2c_ctr, 8),
                   NextState("START_A")
                ).Elif(~i2c_sda_d & i2c_sda & i2c_scl & i2c_scl_d, # stop condition
                   NextState("IDLE")
                ).Elif(i2c_scl & ~i2c_scl_d, # rising edge
                    NextValue(i2c_ctr, i2c_ctr - 1),
                    If(i2c_ctr != 0,
                        NextValue(i2c_adr_in, Cat(i2c_sda, i2c_adr_in[:-1]))
                    )
                ).Elif(~i2c_scl & i2c_scl_d, # falling edge
                    If(i2c_ctr == 0,
                        NextState("ACK_A")
                    )
                )
            )
            i2c_p.act("ACK_A",
                If(i2c_adr_in != FAILING_ADDRESS, # simulate a failure to ACK on the failing address
                    i2c_sda_peripheral_drive_low.eq(1),
                ),
                If(~i2c_sda_d & i2c_sda & i2c_scl & i2c_scl_d, # stop condition
                    NextState("IDLE")
                ).Elif(~i2c_scl & i2c_scl_d, # falling edge
                    NextValue(i2c_dout, ~i2c_adr_in), # reflect the inverse of the address back for testing
                    If(i2c_adr_in != FAILING_ADDRESS,
                        NextState("RESP_D"),
                        NextValue(i2c_ctr, 8)
                    ).Else(
                        # on the failing case, just go back to idle because the cycle aborts here
                        NextState("IDLE")
                    )
                )
            )
            i2c_p.act("RESP_D",
                If(~i2c_scl & i2c_scl_d, # falling edge
                    NextValue(i2c_ctr, i2c_ctr - 1),
                    If(i2c_ctr != 0,
                        NextValue(i2c_dout, Cat(zero, i2c_dout[:-1]))
                    )
                ).Elif(i2c_scl & ~i2c_scl_d, # rising edge
                    If(i2c_ctr == 0,
                        NextState("ACK_D")
                    )
                ),
                i2c_sda_controller_drive_low.eq(~i2c_dout[7])
            )
            i2c_p.act("ACK_D",
                If(~i2c_scl & i2c_scl_d, # falling edge
                   NextState("IDLE")
                ),
                # host drives it here
                # i2c_sda_peripheral_drive_low.eq(1),
            )

        for i in range(32):
            self.gpio = TSTriple()
            self.specials += self.gpio.get_tristate(pads.gpio[i])
            self.comb += [
                self.gpio.o.eq(gpio_o[i]),
                self.gpio.oe.eq(gpio_oe[i]),
            ]
            if sim:
                if (i == 2): # SDA
                    self.comb += [
                        i2c_sda_controller_drive_low.eq(gpio_oe[i]),
                        i2c_sda.eq(~(i2c_sda_controller_drive_low | i2c_sda_peripheral_drive_low)), # fake I2C wire-OR
                        gpio_i[i].eq(i2c_sda)
                    ]
                    # self.comb += gpio_i[i].eq(0) # for NAK testing
                elif (i == 3): # SCL
                    self.comb += gpio_i[i].eq(~gpio_oe[i]) # funky setup to try and "fake" some I2C-ish pullups
                    self.comb += i2c_scl.eq(~gpio_oe[i])
                elif (i == 31): # for register tests
                    self.comb += gpio_i[i].eq(gpio_oe[i])
                else:
                    self.comb += gpio_i[i].eq(gpio_o[i]) # loopback for testing
            else:
                self.comb += gpio_i[i].eq(self.gpio.i)

        self.specials += Instance("pio_ahb",
            # Parameters.
            # -----------
            p_AW = 12,

            # Clk / Rst.
            # ----------
            i_clk = ClockSignal("pio"),
            i_pclk = ClockSignal(),
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
            i_gpio_in              = gpio_i,
            o_gpio_out             = gpio_o,
            o_gpio_dir             = gpio_oe, # 1 is output

            # irq interfaces
            o_irq0                 = irq0,
            o_irq1                 = irq1,
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
        platform.add_source(os.path.join(rtl_dir, "io_interface_def_v0.1.sv"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "pio")
        platform.add_source(os.path.join(rtl_dir, "pio_apb.sv"))
        platform.add_source(os.path.join(rtl_dir, "rp_pio.sv"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "deps", "pio", "upstream", "src")
        platform.add_source(os.path.join(rtl_dir, "pio_decoder.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_divider.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_fifo.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_isr.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_machine.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_osr.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_pc.v"))
        platform.add_source(os.path.join(rtl_dir, "pio_scratch.v"))

        rtl_dir = os.path.join(os.path.dirname(__file__), "sim_support")
        platform.add_source(os.path.join(rtl_dir, "cdc_blinded.v"))
