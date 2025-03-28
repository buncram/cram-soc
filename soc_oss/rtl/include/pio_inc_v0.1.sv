`default_nettype wire
`include "ips/vexriscv/cram-soc/candidate/pio/rp_pio.sv"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_decoder.v"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_divider.v"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_fifo.v"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_isr.v"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_machine.v"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_osr.v"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_pc.v"
//`include "ips/vexriscv/cram-soc/candidate/pio/pio_ahb.sv"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_apb.sv"
`include "ips/vexriscv/cram-soc/candidate/pio/pio_scratch.v"
`include "ips/vexriscv/cram-soc/sim_support/cdc_blinded.v"

`ifdef FPGA
//    `default_nettype wire
`endif

`ifdef SYN
    `default_nettype wire
`endif

`ifdef SIM
    `default_nettype none
`endif
