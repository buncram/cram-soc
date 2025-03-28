`default_nettype none
`include "template.sv"
`include "rtl/include/common_cell_inc_v0.1.sv"
`include "rtl/sysctrl/cms_v0.1.sv"
`include "rtl/sysctrl/nvrcfgs_v0.1.sv"

`include "rtl/sysctrl/brc_v0.1.sv"
`include "rtl/sysctrl/sysctrl_v0.2.sv"
`include "rtl/sysctrl/cgucore_v0.2.sv"
`include "rtl/sysctrl/cgudyncswt_v0.2.sv"
`include "rtl/sysctrl/cgufdsync_v0.1.sv"
`include "rtl/sysctrl/gearbox_v0.3.sv"
`include "rtl/sysctrl/freqmeter_v0.1.sv"

`ifdef FPGA
//    `include "rtl/sysctrl/cgufpgadrp_v0.1.sv"
    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/dyna_clk_vup4.v"
    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/mmcm4_drp_core.v"
    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/mmcm4_drp_vup.v"
//    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/mmcme4_drp_func.h"
//    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/mmcme4_drp_func.human.readable.h"
    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/mmcm_simple_vup.v"
//    `include "lib/fpga/clock_e4_drp_nostep_20221107/rtl/mmcm_usp_drp_tables.vh"
`else
    `include "rtl/sysctrl/cgupll_v0.1.sv"
`endif

//`endif
