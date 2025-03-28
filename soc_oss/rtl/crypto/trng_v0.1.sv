module trng #(
        parameter RAW = 8,
        parameter ERRCNT = 8,
        parameter INTCNT = 8,
        parameter ETPC = 8
    )(

    input  logic clk, resetn, cmsatpg, cmsbist,

    apbif.slavein           apbs,
    apbif.slave             apbx,    
    output bit busy, 
    output bit done,
    output  chnlreq_t       chnl_wpreq   ,
    input   chnlres_t       chnl_wpres   ,

    output logic [0:ERRCNT-1]      err,
    output logic [0:INTCNT-1]      intr
);

    assign chnl_wpreq = '0;
    apbs_null u0(.apbslave(apbx));
    `theregrn( intr ) <= '0;
    `theregrn( err  ) <= '0;

    assign busy = '0;
    assign done = '0;





rngcore #(
        .ETPC  (ETPC)
    )(
/*        input  logic              */ .clk,
/*        input  logic              */ .resetn,
/**/ 
/*    // entropy*/ 
/*        input  logic [ETPC-1:0]   */ .clk_ana,
/*        input  logic [ETPC-1:0]   */ .ana_en,
/*        input  logic [ETPC-1:0]   */ .ana_data,
/*        input  logic [ETPC-1:0]   */ .ana_vld,
/**/ 
/*    // ctrl*/ 
/*        input  logic              */ .rngcore_en,  // enable
/*    */ 
/*    // cr    */ 
/*        input  logic              */ .partityfilter_en,
/*        input  logic              */ .healthtest_en,
/*        input  logic              */ .trng_drng_sel, // 0,trng;1,drng
/*        input  logic [1:0]        */ .postprocess_opt,
/*        input  logic [5:0]        */ .healthtest_length,
/*        input  logic [1:0]        */ .generate_interval,
/*        input  logic [1:0]        */ .reseed_interval,
/*        input  logic [255:0]      */ .personalization_string,
/*        input  logic              */ .additional_input_gen_en,
/*        input  logic [255:0]      */ .additional_input_generate,
/*        input  logic [255:0]      */ .additional_input_reseed,
/*    */ 
/*    // sr    */ 
/*        output logic              */ .drng_reseed_req, // drng req for sw
/*        output logic              */ .buf_ready, // sr
/*        output logic              */ .healthtest_err,
/*    */ 
/*    // buf    */ 
/*        input  logic [2:0]        */ .buf_addr, // addr can keep inc1 mode
/*        input  logic              */ .buf_read,
/*        input  logic              */ .buf_write,
/*        input  logic [31:0]       */ .buf_datain,
/*        output logic [31:0]       */ .buf_dataout,
/*    */ 
/*    // channel    */ 
/*        output logic [127:0]      */ .rngcore_dataout,
/*        output logic              */ .rngcore_dataout_vld,
/*        input  logic              */ .rngcore_rddone
);




endmodule
