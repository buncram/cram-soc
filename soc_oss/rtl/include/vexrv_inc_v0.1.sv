`include "template.sv"

`ifndef FPGA
`define ASIC_TARGET
`endif

`include "ips/vexriscv/cram-soc/candidate/arbiter.v"
//`include "ips/vexriscv/cram-soc/candidate/axi_adapter_rd.v"
//`include "ips/vexriscv/cram-soc/candidate/axi_adapter.v"
//`include "ips/vexriscv/cram-soc/candidate/axi_adapter_wr.v"
`include "ips/vexriscv/cram-soc/candidate/axi_axil_adapter_rd.v"
`include "ips/vexriscv/cram-soc/candidate/axi_axil_adapter.v"
`include "ips/vexriscv/cram-soc/candidate/axi_axil_adapter_wr.v"
`include "ips/vexriscv/cram-soc/candidate/axi_crossbar_addr.v"
`include "ips/vexriscv/cram-soc/candidate/axi_crossbar_rd.v"
`include "ips/vexriscv/cram-soc/candidate/axi_crossbar.v"
`include "ips/vexriscv/cram-soc/candidate/axi_crossbar_wr.v"
//`include "ips/vexriscv/cram-soc/candidate/axi_ram.v"
`include "ips/vexriscv/cram-soc/candidate/axi_register_rd.v"
`include "ips/vexriscv/cram-soc/candidate/axi_register_wr.v"
`include "ips/vexriscv/cram-soc/candidate/cram_axi.v"
`include "ips/vexriscv/cram-soc/candidate/priority_encoder.v"
//`include "ips/vexriscv/cram-soc/candidate/ram_1w_1ra.v"
//`include "ips/vexriscv/cram-soc/candidate/ram_1w_1rs.v"
`include "ips/vexriscv/cram-soc/candidate/VexRiscv_CramSoC.v"
`include "ips/vexriscv/cram-soc/sim_support/fdre_cosim.v"

`include "rtl/core/vexram_v0.1.sv"
`include "rtl/core/vexsys_v0.1.sv"

//ifdef SYN
// `include "ips/vexriscv/cram-soc/candidate/mbox_blackbox_v0.1.sv"
//`else
 `include "ips/vexriscv/cram-soc/candidate/mbox_client.v"
 `include "ips/vexriscv/cram-soc/candidate/mbox_v0.1.sv"
 `include "ips/vexriscv/cram-soc/sim_support/mbox_wrapper.sv"
//`endif

`ifdef SIM
    `define ARM_UD_MODEL
    `define ARM_DISABLE_EMA_CHECK
	`include "lib/arm_sram_macro/rdram128x22/rdram128x22.v"
	`include "lib/arm_sram_macro/rdram1kx32/rdram1kx32.v"
	`include "lib/arm_sram_macro/rdram32x16/rdram32x16.v"
	`include "lib/arm_sram_macro/rdram512x64/rdram512x64.v"
`endif
