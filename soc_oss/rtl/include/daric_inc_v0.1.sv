`timescale 1 ns/1 ps
`define MPW


`include "rtl/include/ipbox_include_v0.1.sv"

`include "template.sv"

`ifdef SYN
//		`include "rtl/model/udphydummy.sv"
//	`include "rtl/rrc/rerammacro_blackbox_v0.1.sv"
`endif

`include "rtl/include/common_cell_inc_v0.1.sv"
`include "rtl/top/daric_cfg_pkg_v0.1.sv"
`include "rtl/include/sysctrl_inc_v0.1.sv"
`include "rtl/rrc/rrc_pkg_v0.1.sv"
`include "rtl/include/bmxcore_inc_v0.1.sv"
`include "rtl/include/cm7sys_inc_v0.1.sv"
`include "rtl/include/sce_inc_v0.3.sv"
`include "rtl/include/mdma_inc_v0.1.sv"
`include "rtl/include/vexrv_inc_v0.1.sv"
`include "rtl/include/qfc_inc_v0.1.sv"

//`include "rtl/ifsub/soc_ifsub_v0.1.sv"
`include "rtl/include/soc_ifsub_inc_v0.1.sv"
`include "rtl/include/sec_inc_v0.1.sv"

//`include "ips/cortexm7/logical/testbench/execution_tb/verilog/example_sys/cm7_ik_ahb_sram_bridge_64.v"
`include "rtl/amba/ahb_sram_bridge_64_waitcyc.v"
`include "rtl/core/coresub_sramtrm_v0.1.sv"

`include "rtl/general/axisramc_v0.1.sv"
`include "rtl/top/soc_coresub_v0.2.sv"
`include "rtl/include/jtagtap_inc_v0.1.sv"
`include "rtl/top/soc_top_v0.2.sv"
`ifdef SIM
`include "lib/arm_sram_macro/acram2kx64/acram2kx64.v"
`endif

`include "rtl/core/core_srambank_v0.1.try8k.sv"
`ifdef FPGA
    `include "rtl/rrc/rrc_emu_v0.1.sv"
`else
    `include "rtl/include/reram_inc_v0.1.sv"
    `include "rtl/rrc/rerammacro_v0.1.sv"
    `include "rtl/rrc/trbcx1r32_daric_wrapper.sv"
    `include "rtl/rrc/rrc_v0.2.sv"
`endif

`include "rtl/core/duart_v0.1.sv"
`include "rtl/sysctrl/apbsys_intf_v0.1.sv"
`include "rtl/sysctrl/evc_v0.1.sv"
`include "ips/pulp_soc/rtl/pulp_soc/soc_event_queue.sv"
`include "ips/pulp_soc/rtl/pulp_soc/soc_event_arbiter.sv"
`include "ips/ambabuilder/logical/cmsdk_apb_watchdog/verilog/cmsdk_apb_watchdog.v"
`include "ips/ambabuilder/logical/cmsdk_apb_watchdog/verilog/cmsdk_apb_watchdog_frc.v"
//`include "ips/ambabuilder/logical/cmsdk_apb_uart/verilog/cmsdk_apb_uart.v"
`include "ips/pulp_soc/rtl/components/apb_timer_unit.sv"
`include "ips/timer_unit/rtl/timer_unit_counter.sv"

`include "rtl/model/padcell_v0.3_arm.sv"

// ao

`include "rtl/sysctrl/aoperi_v0.1.sv"
`include "rtl/include/rtc_inc_v0.1.sv"

//`ifndef FPGA
// top
    `include "rtl/top/daric_top_v0.3.sv"
//`endif

`include "rtl/top/ao_top_v0.2.sv"
`include "rtl/top/pad_frame_v0.3_arm.sv"
`include "rtl/top/powerpad_v0.4.sv"

`ifdef FPGASIM
`include "lib/fpga/PULLUP.v"
`include "lib/fpga/PULLDOWN.v"
`endif

`ifdef SIM
`include "rtl/model/sim_mon_v0.1.sv"
`include "rtl/model/usbipmon.sv"
`endif


// for less dummytb on the top level
module _____dummytb_soc_coresub__just_ignore_it();
       _____dummytb_commoncell__just_ignore_it u1();
       dummytb_ahb_bmx33 u2();
       dummytb_bmxcore u3();
//       cm7dpu_alu_sbitx u4();
//       cm7_pmu_sync_reset u5();
//       cm7_rst_send_set u6();
//       cm7_pmu_sync_set u7();
//       cm7_cdc_random u8();
//       cm7_pmu_cdc_send_reset u9();
//       cm7_rst_sync uc();
//       cortexm7_ecc_repair64 ua();
       __dummy_tb_cm7sys_ ub();
//       dummytb_soc_coresub u();
endmodule : _____dummytb_soc_coresub__just_ignore_it
