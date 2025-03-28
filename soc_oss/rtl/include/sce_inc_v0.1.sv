`include "template.sv"


`include "rtl/include/common_cell_inc_v0.1.sv"
`include "rtl/crypto/hash_pkg_v0.1.sv"
`include "rtl/crypto/scedma_pkg_v0.3.sv"

`include "rtl/crypto/hashcore_v0.2.sv"
`include "rtl/crypto/hashcore_blk_v0.1.sv"
`include "rtl/crypto/combohash_v0.2.sv"
`include "rtl/crypto/sce_dmachnl_v0.1.sv"
`include "rtl/crypto/scedma_amba_v0.2.sv"

`include "rtl/crypto/sce_memc_v0.2.sv"
`include "rtl/crypto/scedma_ac_v0.1.sv"
`include "rtl/crypto/scedma_v0.1.sv"

`include "rtl/crypto/pke_v0.1.sv"
//`include "rtl/crypto/PkeCore_dummy.sv"
//`include "rtl/crypto/pke/cmsdk_ahb_to_sram.v"
`include "rtl/crypto/pke/com_alg.v"
//`include "rtl/crypto/pke/emb_v2.v"
`include "rtl/crypto/pke/mgmr_mul.v"
`include "rtl/crypto/pke/PkeCore.v"
`include "rtl/crypto/pke/PkeCtrl.v"
`include "rtl/crypto/pke/PkeRamMux.v"
`include "rtl/crypto/pke/QRegCal.v"
//`include "rtl/crypto/pke/sram128X32C2V4.v"
//`include "rtl/crypto/pke/sram128X32C2V4_wrp.v"

`include "rtl/crypto/aes_v0.2.sv"
//`include "rtl/crypto/AesCore_dummy.sv"

`include "rtl/crypto/aes/AesCore.v"
`include "rtl/crypto/aes/AesCtrl.v"
`include "rtl/crypto/aes/AesDataPath.v"
`include "rtl/crypto/aes/AesMixCol.v"
`include "rtl/crypto/aes/AesSbox.v"
`include "rtl/crypto/aes/GfFunctions.v"

`include "rtl/crypto/cryptoram_v0.2.sv"

`include "rtl/crypto/sce_glbsfr_v0.1.sv"

`include "rtl/crypto/sce_v0.1.sv"

`ifdef SIM
`include "lib/arm_sram_macro/sce_aesram_1k/sce_aesram_1k.v"
`include "lib/arm_sram_macro/sce_pkeram_4k/sce_pkeram_4k.v"
`include "lib/arm_sram_macro/sce_hashram_3k/sce_hashram_3k.v"
`include "lib/arm_sram_macro/sce_sceram_10k/sce_sceram_10k.v"
`endif