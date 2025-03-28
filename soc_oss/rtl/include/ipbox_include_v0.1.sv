
`ifndef _IPBOX_INC
`define _IPBOX_INC

// usbphy / pll / adc
// ■■■■■■■■■■■■■■■

	`include "rtl/ifsub/utmi_def_v0.1.sv"
	`ifdef SIM
		`ifdef PHYSIM
	//		`include "lib/INNO_PKG_U2_PRJ2210CBS1_S2210_T22ULL_V1P0_R20221020/FRONTEND/MODEL/presim_model/src/short_time_model/vcs/inno_usb_phy.vp"
	//		`include "lib/innosilico/INNO_PKG_U2_PRJ2210CBS1_S2210_T22ULL_V2P0_R20230726/FRONTEND/MODEL/presim_model/src/real_time_model/vcs/inno_usb_phy.vp"
			`include "lib/innosilico/u2p/FRONTEND/MODEL/presim_model/src/real_time_model/vcs/inno_usb_phy.vp"
		`else
			`include "rtl/model/udphydummy.sv"
		`endif
	`endif

	`ifdef SIM
	//`include "lib/INNO_PKG_PLL_PRJ2210CBS1_S2210_T22_V1P1_R20221020/model/presim/sim_vcs/rtl/INNO_PLL_TOP.vp"
	//`include "lib/innosilico/INNO_PKG_PLL_PRJ2210CBS1_S2210_T22_V2P0_R20230726/FRONTEND/MODEL/presim_model/src/vcs/INNO_FNPLL_TOP.vp"
	`include "lib/innosilico/pll/FRONTEND/MODEL/presim_model/src/vcs/INNO_FNPLL_TOP.vp"
	`else
	//`include "lib/INNO_PKG_PLL_PRJ2210CBS1_S2210_T22_V1P1_R20221020/model/presim/sim_vcs/rtl/INNO_PLL_TOP_blackbox.v"
	`endif

	`ifndef FPGA
	`ifdef SIM
	`default_nettype wire
	//`include "lib/innosilico/INNO_PKG_TVSENSOR_PRJ2210CBS1_S2210_T22_V2P0_R20230726/FRONTEND/MODEL/presim_model/src/vcs/inno_tsensor_ip.vp"
	`include "lib/innosilico/tvsensor/FRONTEND/MODEL/presim_model/src/vcs/inno_tsensor_ip.vp"
	`endif
	`endif

// io
// ■■■■■■■■■■■■■■■

`ifdef SIM
    //`include "lib/io/tphn28hpcgv2od3_fast.v"
    //`include "lib/io/tphn22ullgv2od3.v"
    `include "lib/io/io_gppr_cln22ul_t25_mv09_mv33_fs33_svt_dr_fast.v"
    `include "template.sv"
`endif


// self design
// pmu/adcmux/osc32m/osc32k/rng/ld/gluecell
// ■■■■■■■■■■■■■■■

`ifdef SYN
//	`include "rtl/top/pmu_top_v0.2.sv"
//	`include "rtl/model/adcmux_sim_v0.1.sv"
//	`include osc32m
//	`include osc32k
//	`include "rtl/model/rng_cell_v0.1.sv"
//	`include "rtl/model/ld_v0.1.sv"
//	`include "rtl/model/gluecell_v0.1.sv"
`else
	`include "rtl/top/pmu_top_v0.2.sv"
	`include "rtl/model/adcmux_sim_v0.1.sv"
//	`include osc32m
//	`include osc32k
	`include "rtl/model/rng_cell_v0.1.sv"
	`include "rtl/model/ld_v0.1.sv"
	`include "rtl/model/gluecell_v0.1.sv"
`endif


`endif