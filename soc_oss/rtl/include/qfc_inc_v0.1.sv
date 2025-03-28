
`define SDVT_NO_BLACKBOX

`include "ips/smartdv/spi_flash_controller_iip/hdl/include/sdvt_spi_master_defines.vh"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_apb_slave.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_async_blk.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_async_fifo_ctrl.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_async_fifo_ff.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_axi_slave.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_csr.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_dft_mux_cell.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_fsm.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_nedge_cell.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_prescaler.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_sync_cell.v"
//`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_sync_dpram.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_sync_fifo.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_xip.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/sdvt_spi_master_core_v0.2.v"
`include "ips/smartdv/spi_flash_controller_iip/hdl/src/qfc_socbus_aes_v0.1.sv"

`ifndef __TRNGAES
`include "rtl/crypto/trng/aes_cipher_top.v"
`include "rtl/crypto/trng/aes_key_expand_128.v"
`include "rtl/crypto/trng/aes_rcon.v"
`include "rtl/crypto/trng/aes_sbox.v"
`include "rtl/crypto/trng/aes_update.v"
`include "rtl/crypto/trng/ctr_aes.v"
`define __TRNGAES
`endif

`include "rtl/core/qfc_aes_v0.1.sv"
`include "rtl/core/qfc_v0.2.sv"
