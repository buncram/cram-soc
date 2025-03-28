`include "rtl/include/common_cell_inc_v0.1.sv"

`include "rtl/include/bmxcore_inc_v0.1.sv"
`include "rtl/include/cm7sys_inc_v0.1.sv"

`include "ips/cortexm7/logical/testbench/execution_tb/verilog/example_sys/cm7_ik_ahb_sram_bridge_64.v"
`include "rtl/general/axisramc_v0.1.sv"
`include "rtl/top/daric_cfg_pkg_v0.1.sv"
`include "rtl/top/soc_coresub_v0.1.sv"


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
