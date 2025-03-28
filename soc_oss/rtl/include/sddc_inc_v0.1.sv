
`default_nettype wire
`define SDVT_NO_BLACKBOX

`include "rtl/ifsub/sddc_v0.2.sv"
`include "ips/smartdv/sdio_device_iip/hdl/include/sdvt_sdio_device_defines.vh"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_core.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_ahb_master.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_ahb_slave.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_async_blk.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_async_fifo_ctrl.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_async_fifo.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_cfsm.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_cmd_crc7.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_cshifter.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_csr.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_data_crc16.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dfsm.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dma.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dshifter.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_io_func.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_min_cmd_delay.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_pedge_cell.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_rst_sync_cell.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_suspend_resume_func.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_cell.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_dpram_daric.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_extend_cell.v"

`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_ahb_master.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_ahb_slave.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_async_blk.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_async_fifo_ctrl.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_async_fifo.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_cfsm.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_cmd_crc7.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_core.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_cshifter.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_csr.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_data_crc16.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dfsm.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dft_mux_cell.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dma.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_dshifter.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_io_func.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_lock.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_min_cmd_delay.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_pedge_cell.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_program_csd.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_rst_sync_cell.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_suspend_resume_func.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_cell.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_dpram_daric.v"
//`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_dpram.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_sync_extend_cell.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_voltage_switch_timer.v"
`include "ips/smartdv/sdio_device_iip/hdl/src/sdvt_sdio_device_write_protect_grp.v"




`ifdef FPGA
 //   `default_nettype wire
`endif

`ifdef SYN
    `default_nettype wire
`endif

`ifdef SIM
    `default_nettype none
`endif
