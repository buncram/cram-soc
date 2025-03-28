`default_nettype none
//`define NO_FIXING_PARITY
//`define DETECT_3_BITS

/************************************************/
// pragma protect

`timescale 1ns/10ps
//`define numBankAddrX 10

`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_global_rtl_define.vh"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_user_rtl_define.vh"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_regif_auto_define.vh"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_rtl_define.vh"
//`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_rtl_parameter.vh"

`include "decoder_invless_and.v"
`include "decoder_invless.v"
`include "encoder_new_h.v"
`include "rrn22ull128kx144m32i8r16_d25_shvt_c220530_wrapper.v"
//`include "testchip.v"
`include "trbcx1r32_22ull128kx144m32i8r16d25shvt220530_wrapper.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_col_repair.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_dft.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_generic_synchronizer.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_main.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_regif_auto.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_regif.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_row_repair.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_tap_bypass.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_tap_fsm.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_tap_ir.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_tap.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_top_mux.v"
`include "trbx1r32_22ull128kx144m32i8r16d25shvt220530_bist_top.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_cr72_sector.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_dyn_grp_1bit.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_dyn_grp_8bit.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_dyn_prg_sector.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_macro_cr.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_macro_rr.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_pop_counter_32b.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_row_buf.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_rr_bk.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_signed_adder.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_action_flow.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_dyn_prg.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_generic_dlatn.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_generic_icg.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_generic_mux.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_generic_or.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_if.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_regif_auto_define.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_regif_auto.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_regif.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_rw.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_top.v"
`include "trcx1r32_22ull128kx144m32i8r16d25shvt220530_trc_to_rram_read.v"

`ifdef SIM
`include "lib/rram/16mb/flash/Front_End/verilog/rrn22ull128kx144m32i8r16_d25_shvt_c220530_010c/rram_model_parameter.vh"
`include "lib/rram/16mb/flash/Front_End/verilog/rrn22ull128kx144m32i8r16_d25_shvt_c220530_010c/rrn22ull128kx144m32i8r16_d25_shvt_c220530_010c.v"
`endif
