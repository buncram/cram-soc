// old `include "ips/udc/Innosilicon_2022_11_17/rtl_u2_dev/design_lib.v"
`include "ips/udc/Innosilicon/rtl_u2_dev/design_lib.v"

`ifdef SYN
	`include "ips/udc/Innosilicon/rtl_u2_dev/xhci_top_syn.vp"
`else
	`include "ips/udc/Innosilicon/rtl_u2_dev/xhci_top.vp"
`endif

`include "rtl/ifsub/utmi_def_v0.1.sv"
`include "rtl/ifsub/udc_v0.1.sv"

`ifdef SIM
    `define ARM_UD_MODEL
    `define ARM_DISABLE_EMA_CHECK
    `include "lib/arm_sram_macro/udcmem_256x64/udcmem_256x64.v"
	`include "lib/arm_sram_macro/udcmem_1088x64/udcmem_1088x64.v"
`endif
