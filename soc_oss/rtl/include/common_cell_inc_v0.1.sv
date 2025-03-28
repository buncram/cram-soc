`timescale 1 ns/1 ps

`ifndef _COMMON_CELLS
`define _COMMON_CELLS

// for axi_xbar
`define VCS


`include "rtl/include/ipbox_include_v0.1.sv"
//`include "template.sv"
`include "template.sv"
`include "amba_interface_def_v0.2.sv"
`include "io_interface_def_v0.1.sv"
`include "ram_interface_def_v0.3.sv"
`include "jtag_interface_def_v0.1.sv"
`include "rtl/model/artisan_ram_def_v0.1.svh"
`include "rtl/model/icg_v0.2.v"
`include "rtl/common/scresetgen_v0.1.sv"
`include "rtl/ifsub/utmi_def_v0.1.sv"

`include "rtl/amba/apb_sfr_v0.1.sv"
`include "rtl/amba/ahb_sfr_v0.1.sv"
`include "rtl/amba/amba_components_v0.2.sv"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync/verilog/cmsdk_ahb_to_ahb_sync.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync_down/verilog/cmsdk_ahb_to_ahb_sync_down.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync_down/verilog/cmsdk_ahb_to_ahb_sync_wb.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync_down/verilog/cmsdk_ahb_to_ahb_sync_error_canc.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync_down/verilog/cmsdk_ahb_to_ahb_sync_down_core.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync_up/verilog/cmsdk_ahb_to_ahb_sync_up.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_ahb_sync_up/verilog/cmsdk_ahb_to_ahb_sync_up_core.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_apb/verilog/cmsdk_ahb_to_apb.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_to_sram/verilog/cmsdk_ahb_to_sram.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_downsizer64/verilog/cmsdk_ahb_downsizer64.v"
`include "ips/ambabuilder/logical/cmsdk_ahb_master_mux/verilog/cmsdk_ahb_master_mux.v"

`include "ips/common_cells/src/cf_math_pkg.sv"
`include "ips/common_cells/src/addr_decode.sv"
`include "ips/common_cells/src/spill_register.sv"
`include "ips/common_cells/src/rr_arb_tree.sv"
`include "ips/common_cells/src/fifo_v3.sv"
`include "ips/common_cells/src/counter.sv"
`include "ips/common_cells/src/delta_counter.sv"
`include "ips/common_cells/src/spill_register_flushable.sv"
`include "ips/common_cells/src/lzc.sv"

`include "ips/axi-master/src/axi_pkg.sv"
`include "ips/axi-master/src/axi_intf.sv"
`include "ips/axi-master/src/axi_xbar.sv"
`include "ips/axi-master/src/axi_mux.sv"
`include "ips/axi-master/src/axi_err_slv.sv"
`include "ips/axi-master/src/axi_demux_nointf.sv"
`include "ips/axi-master/src/axi_id_prepend.sv"

`include "rtl/amba/aab_intf_v0.1.sv"
//`include "rtl/amba/ahb_axi_bdg_v0.1.sv"
`include "rtl/include/nic400_hxb32_inc_v0.1.sv"
`include "rtl/amba/ahb_demux_v0.1.sv"
`include "rtl/amba/axitrans_v0.1.sv"

`include "ips/cortexm7/logical/cm7aab/verilog/CM7AAB.v"
`include "ips/cortexm7/logical/cm7aab/verilog/cm7aab_ahb.v"
`include "ips/cortexm7/logical/cm7aab/verilog/cm7aab_axi.v"

`include "rtl/general/gnrl_sramc_pkg_v0.2.sv"
`include "rtl/general/gnrl_sramc_v0.2.sv"
`include "rtl/general/pulp_icg_v0.1.sv"
`include "rtl/general/gnrl_sync_v0.1.sv"

`include "rtl/common/sram.sv"

`include "rtl/general/dummytb_v0.1.sv"

//`ifdef SIM
`include "rtl/model/osc_sim_v0.1.sv"

`include "rtl/common/insauth_v0.2.v" // for fpga or sim

`ifdef FPGA
`include "lib/fpga/ram/uram_cas_v0.1.sv"
`include "lib/fpga/ram/bram_v0.1.sv"
`endif
`ifdef FPGASIM
`include "xilinx_glbl.v"
`include "lib/fpga/ram/URAM288.v"
`include "lib/fpga/ram/URAM288_BASE.v"
`include "lib/fpga/ram/BRAM_SINGLE_MACRO.v"
`include "lib/fpga/ram/BRAM_SDP_MACRO.v"
`include "lib/fpga/ram/RAMB18E1.v"
`include "lib/fpga/ram/RAMB36E1.v"
`endif
  `ifdef FPGASIM
    `include "lib/fpga/mmcm4/MMCME4_ADV.v"
    `include "lib/fpga/mmcm4/MMCME4_BASE.v"
    `include "lib/fpga/mmcm4/BUFG.v"
    `include "lib/fpga/mmcm4/BUFGCE.v"
  `endif

module _____dummytb_commoncell__just_ignore_it();
    dummytb_for_ambainterfaces__just_ignore_it u1();
    tb_aab_intf u2();
       tb_ahbaxi_bdg_intf u3();
       dummytb_ahb_demux u4();
       dummytb_axitrans u5();
       dummytb_axi_xbar u6();
       dummytb_axi_mux u7();
       dummytb_ahbsfr u8();
       dummytb_apbsfr u9();
       ioif ioifa();
       ramif ramifa();
endmodule : _____dummytb_commoncell__just_ignore_it

`ifdef SIM
    `define ARM_UD_MODEL
    `define ARM_DISABLE_EMA_CHECK
    `include "lib/arm_sram_macro/fifo128x32/fifo128x32.v"
    `include "lib/arm_sram_macro/fifo32x19/fifo32x19.v"
`endif

`endif // `ifndef _COMMON_CELLS

