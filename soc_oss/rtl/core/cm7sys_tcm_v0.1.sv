
module cm7sys_tcm
  #(
    parameter bit itcm = 0, 
    parameter sram_pkg::sramcfg_t thecfg={
        AW: 13,
        DW: 32,
        KW: 32,
        PW: 4,
        WCNT: 2**13,
        AWX: 5,
        isBWEN: '1,
        isSCMB: '0,
        isPRT:  '1,
        EVITVL: 15
    },
    parameter RC=1,
    parameter RDW = 2 // ready_counter/wait_cyc width 
   )
   (input  logic                     clk,
    input  logic                     clktop,
    input  logic                     clken,
    input  logic                     cmsatpg,
    input  logic                     cmsbist,
    input  logic [RDW-1:0]           waitcyc,
    input  logic [2:0]               sramtrm,
    input  logic                     resetn,

    input  logic [thecfg.AW-1:0]     addr_i,
    input  logic [thecfg.DW-1:0]     wd_i,
    input  logic                     cs_i,
    input  logic [thecfg.DW/8-1:0]   we_i,
    output logic [thecfg.DW-1:0]     rd_o,
    output logic                     wait_o,
    output logic                     err_o,
    output logic                    retry_o
   );

    ramif #(.RAW(thecfg.AW),.DW(thecfg.DW))rams();

    logic verifyerr, prerr, ramready;

    wire2ramm #(.AW(thecfg.AW),.DW(thecfg.DW)) RM(
        .ramm_ramen      (1'b1),     
        .ramm_ramcs      (cs_i),     
        .ramm_ramaddr    (addr_i),     
        .ramm_ramwr      (we_i),     
        .ramm_ramwdata   (wd_i),     
        .ramm_ramrdata   (rd_o),
        .ramm_ramready   (ramready),
        .ramm (rams)
    );

    assign wait_o = ~ramready;
    assign err_o = verifyerr | prerr;
    assign retry_o = 1'b0;

`ifdef FPGA

    generate
    if(itcm) begin: genitcm
        uram_cas #( .XX (1), .YY (8)) tcmram (
          .clk          (clk),
          .resetn,
          .waitcyc      (waitcyc|4'h0),
          .rams         (rams)
        );
    end
    else begin:gendtcm
        uram_none tcmram (
          .clk          (clk),
          .resetn,
          .waitcyc      (waitcyc|4'h0),
          .rams         (rams)
        );
    end
    endgenerate

`else
    ramif #(.RAW(thecfg.AW),.DW(thecfg.DW+thecfg.PW),.BW(9))tcmrams();

    gnrl_sramc #(
        .thecfg(thecfg)
    )uramc(
        .clk,
        .resetn,
        .cmsatpg,
        .cmsbist,
        .scmben     ('0       ),
        .scmbkey    ('0       ),
        .prerr      (prerr    ),
        .verifyerr  (verifyerr),
        .ramslave   (rams     ),
        .rammaster  (tcmrams  )        
    );

    tcmram #(.itcm(itcm),.thecfg(thecfg),.RC(RC))tcm
    (
        .clk    (clktop),
        .clken  (clken),
        .resetn (resetn),
        .cmsatpg(cmsatpg),
        .waitcyc(waitcyc),
        .sramtrm(sramtrm),
        .rams   (tcmrams)
    );

`endif

endmodule : cm7sys_tcm

/*
module dummytb_cm7sys_tcm();


  assign clkcm7in = fclken;

  cm7sys_tcm 
   #(
      .itcm   ('1),
      .thecfg (daric_cfg::itcmcfg),
      .RC     (daric_cfg::itcmrc)
    )
  u_itcm_ram
    (.clk        (clkcm7in),
     .clktop     (clktop), 
     .clken      (fclken), 
     .cmsatpg    (cmsatpg),
     .cmsbist    (cmsbist),
     .waitcyc    (cm7cfg_itcmwaitcyc),
     .resetn     (resetn),

     .addr_i     (sys_itcmaddr[daric_cfg::itcmcfg.AW+3-1:3]), 
     .wd_i       (sys_itcmwdata[63:0]),
     .cs_i       (sys_itcmcs),
     .we_i       (sys_itcmbytewr[7:0]),
     .rd_o       (sys_itcmrdata0[63:0]),
     .wait_o     (sys_itwait),
     .err_o      (sys_iterr),
     .retry_o    (sys_itretry)
     );

  cm7_ik_tcm_ram 
   #(
      .itcm   ('0),
      .thecfg (daric_cfg::dtcmcfg),
      .RC     (daric_cfg::dtcmrc)
    )
  u_d0tcm_ram
    (.clk        (clkcm7in),
     .clktop     (clktop), 
     .clken      (fclken), 
     .cmsatpg    (cmsatpg),
     .cmsbist    (cmsbist),
     .waitcyc    (cm7cfg_dtcmwaitcyc),
     .resetn     (resetn),

     .addr_i     (sys_d0tcmaddr[daric_cfg::dtcmcfg.AW+3-1:3]),
     .wd_i       (sys_d0tcmwdata[31:0]),
     .cs_i       (sys_d0tcmcs),
     .we_i       (sys_d0tcmbytewr[3:0]),
     .rd_o       (sys_d0tcmrdata[31:0]),
     .wait_o     (sys_d0wait),
     .err_o      (sys_d0err),
     .retry_o    (sys_d0retry)
     );

    `maintest(dummytb_cm7sys_tcm,dummytb_cm7sys_tcm)
        #105 ;

        #(1 `MS);
    `maintestend

endmodule
*/

