`include "template.sv"
`include "io_interface_def_v0.1.sv"

import daric_cfg::*;
import rrc_pkg::*;
//import cms_pkg::*;

module soc_coresub #(
    parameter BRC  = daric_cfg::BRC,
    parameter BRCW = daric_cfg::BRCW,
    parameter BRDW = daric_cfg::BRDW,
    parameter IRQCNT = daric_cfg::IRQCNT-16
)(
//    input   logic   clktop,

    input   logic   fclk,
    input   logic   aclk,
    input   logic   hclk,
    input   logic   iclk,
//    input   logic   pclk,
    input   logic   clktop,
    input   logic   clktopen,
    input   logic   fclken,
//    input   logic   pclken,

    input   logic   clksys,
    input   logic   clk1M,


//    input   logic   clkcore,
    input   logic   clkdma,
    input   logic   clksce,
    input   logic   clkqfc,
    input   logic   clkvex,

    input   logic   aximclken,
    input   logic   ahbpclken,
    input   logic   ahbsclken,

    input   logic   sysresetn,
    input   logic   coreresetn,

`ifdef FPGA
    input   logic   coresel_cm7,
`endif
    output  logic   cm7sleep,
    output  logic   cm7resetreq,
    input   logic [IRQCNT-1:0]  cm7_irq,
    input   logic   cm7_nmi, 
    input   logic   cm7_rxev,
    output  logic [31:0]    coresubevo,
    output  logic           coresuberro,
    output  logic [31:0]    sceevo,
    output  logic           sceerro,

// cms
//    output cmsdata_e            cmsdata,
//    output logic                cmsdatavld,

    input  logic [3:0]          brready,
    output logic                brvld,
    output logic [BRCW-1:0]     bridx,
    output logic [BRDW-1:0]     brdat,
    output logic                brdone,

    input   cms_pkg::cmscode_e   cmscode,
    input   logic       cmsatpg,
    input   logic       cmsbist,

// qfc
    output padcfg_arm_t  padcfg_qfc_sck,
    output padcfg_arm_t  padcfg_qfc_qds,
    output padcfg_arm_t  padcfg_qfc_ss,
    output padcfg_arm_t  padcfg_qfc_sio,
//    output padcfg_arm_t  padcfg_qfc_rwds,
    output padcfg_arm_t  padcfg_qfc_int,
    output padcfg_arm_t  padcfg_qfc_rst,

    ioif.drive qfc_sck,
    ioif.drive qfc_sckn,
    ioif.drive qfc_dqs,
    ioif.drive qfc_ss[1:0],
    ioif.drive qfc_sio[7:0],
//    ioif.drive qfc_rwds,
    ioif.drive qfc_rstm[1:0],
    ioif.drive qfc_rsts[1:0],
    ioif.drive qfc_int,

// 
    jtagif.slave jtagvex,

// bus
    ahbif.master        bmxif_ahb32,
    ahbif.master        coreahb_sys,
    ahbif.master        coreahb_sec,
    ahbif.master        coreahb_ao,

    input   logic       clkswd,
    ioif.drive          swdio

// interfaces

);

    typedef axi_pkg::xbar_rule_32_t       rule32_t; // Has to be the same width as axi addr

    logic [3:0] clkcm7stenregs;
    logic clkcm7sten_1M;
    logic cm7cfg_dev, vexcfg_dev, cm7cfg_en, vexcfg_en ;
    logic [31:0] cm7cfg_iv, vexcfg_iv;
//    logic cmsatpg, cmsbist;
//    logic [IRQCNT-1:0]  cm7_irq;
//    logic cm7_nmi, cm7_rxev;
//    logic aximclken, ahbpclken, ahbsclken;
    axiif #(.AW(32),.DW(64),.IDW(8),.LENW(8),.UW(8)) 
        cm7_axim(), rrc_axi64(), sram_axi64[0:1](), qfc_axi64();
    axiif #(.AW(32),.DW(32),.IDW(8),.LENW(8),.UW(8)) 
        sce_axi32();
    ahbif #(.AW(32),.DW(32),.IDW(4),.UW(4)) 
        cm7_ahbp(), cm7_ahbs(), mdma_ahb32(), core_ahb32(), coreahbmux[0:6]();

    ramif #(.RAW(20-3), .DW(64)//, .BW(8)
        ) sramc[0:1]();

    apbif #(.PAW(16)) coresubapb();
    apbif #(.PAW(12)) coresubapbs[0:15]();

    axiif #(.DW(64)) vex_iaxi();
    axiif #(.DW(32)) vex_daxi();
    ahbif #(.AW(32),.DW(32),.IDW(4),.UW(4)) 
        vex_ahbp();

// parameters


// clk/resetn
    logic clkaxi, clkahb, clkif, resetn;
    logic rrcint;
    logic pclk, pclken;

    assign pclk = hclk;
    assign pclken = 1'b1; // coresub apb has same freq
    
//    assign clkcore = fclk;
    assign clkaxi = aclk;
    assign clkahb = hclk;
//    assign clksce = hclk;
    assign clkif  = iclk;
    assign resetn = sysresetn;

    `theregfull(fclk, resetn, clkcm7stenregs, '0) <= { clkcm7stenregs, clk1M };
    `theregfull(fclk, resetn, clkcm7sten_1M, '0) <= clkcm7stenregs[3] & ~clkcm7stenregs[2];

// ■■■■■■■■■■■
// corecfg
// ■■■■■■■■■■■

    localparam RERAMUSERCNT = 0;
    localparam PM_COREUSERCNT = daric_cfg::CODEMEMCNT + RERAMUSERCNT;
    localparam rule32_t [daric_cfg::CODEMEMCNT-1:0] code_mem_map = daric_cfg::code_mem_map;
    localparam rule32_t [6:0] coreahb_demux_map = daric_cfg::coreahb_demux_map;

    rule32_t    [PM_COREUSERCNT-1:0] coreusermap;
    bit         [PM_COREUSERCNT-1:0] coreuser, sceuser;
    bit corecfg_devreg, cm7cfg_enreg, vexcfg_enreg;

    assign cm7cfg_iv    = 32'h6000_0000;
    assign vexcfg_iv    = 32'h6010_0000;
    assign cm7cfg_dev   = corecfg_devreg & cm7cfg_en;
    assign vexcfg_dev   = corecfg_devreg & vexcfg_en;

    `theregfull( hclk, resetn, cm7cfg_enreg   , '0 ) <= '1;
    `theregfull( hclk, resetn, vexcfg_enreg   , '0 ) <= '0;
    `theregfull( hclk, resetn, corecfg_devreg , '0 ) <= '1;

`ifdef FPGA
    assign cm7cfg_en =  coresel_cm7;
    assign vexcfg_en = ~coresel_cm7;
`else
    assign cm7cfg_en = ~vexcfg_enreg | cm7cfg_enreg;
    assign vexcfg_en =  vexcfg_enreg;
`endif

// ■■■■■■■■■■■
// cm7sys/vexsys
// ■■■■■■■■■■■


    logic fclk_cm7, clktop_cm7, clkswd_cm7, sysresetn_cm7, coreresetn_cm7;
    logic clk_vex0, resetn_vex;

generate
    if(1) begin: __coresys

    ICG cm7en_icg0 (.CK (fclk),   .EN (cm7cfg_en),.SE(cmsatpg),.CKG(fclk_cm7));
    ICG cm7en_icg1 (.CK (clktop), .EN (cm7cfg_en),.SE(cmsatpg),.CKG(clktop_cm7));
    ICG cm7en_icg2 (.CK (clkswd), .EN (cm7cfg_en),.SE(cmsatpg),.CKG(clkswd_cm7));
    assign sysresetn_cm7  = cmsatpg ? 1'b1 : sysresetn  & cm7cfg_en;
    assign coreresetn_cm7 = cmsatpg ? 1'b1 : coreresetn & cm7cfg_en;

    ICG vexen_icg0 (.CK (clkvex),   .EN (~resetn | vexcfg_en),.SE(cmsatpg),.CKG(clk_vex0));
    assign resetn_vex  = cmsatpg ? 1'b1 : resetn  & vexcfg_en;

    cm7sys #(
        .PM_COREUSERCNT    (PM_COREUSERCNT),
        .PM_CFGITCMSZ      (daric_cfg::CFGITCMSZ),
        .PM_CFGDTCMSZ      (daric_cfg::CFGDTCMSZ),
        .FPU        (daric_cfg::CM7CFG.FPU        ),
        .ICACHE     (daric_cfg::CM7CFG.ICACHE     ),
        .DCACHE     (daric_cfg::CM7CFG.DCACHE     ),
        .CACHEECC   (daric_cfg::CM7CFG.CACHEECC   ),
        .MPU        (daric_cfg::CM7CFG.MPU        ),
        .IRQNUM     (daric_cfg::CM7CFG.IRQNUM     ),
        .IRQLVL     (daric_cfg::CM7CFG.IRQLVL     ),
        .ICACHESIZE (daric_cfg::CM7CFG.ICACHESIZE ),
        .DCACHESIZE (daric_cfg::CM7CFG.DCACHESIZE ),
        .dtcmrc     (daric_cfg::dtcmrc            ),
        .dtcmcfg    (daric_cfg::dtcmcfg           ),
        .itcmrc     (daric_cfg::itcmrc            ),
        .itcmcfg    (daric_cfg::itcmcfg           ),
        .AXIMID4    (daric_cfg::AMBAID4_CM7A),
        .AHBPID4    (daric_cfg::AMBAID4_CM7P)
    )
    cm7sys
    (
    // system ctrl
        .clk            (fclk_cm7       ),
        .clktop         (clktop_cm7     ),
        .fclken,

        .resetn         (sysresetn_cm7  ),
        .coreresetn     (coreresetn_cm7 ),
        .cm7_resetreq   (cm7resetreq    ),
        .cm7_sleep      (cm7sleep       ),
        .clkcm7sten_1M  (clkcm7sten_1M  ),
    // cfg
        .cm7cfg_dev     (cm7cfg_dev),
        .cm7cfg_iv      (cm7cfg_iv),// = 32'h6000_0000;
        .cm7cfg_itcmwaitcyc('0),
        .cm7cfg_dtcmwaitcyc('0),
    // test mode
        .cmsatpg,
        .cmsbist,
    //    mbist.master                mbistif,
    // interrupt, nmi, events
        .cm7_irq,
        .cm7_nmi,
        .cm7_rxev,
    // amba
        .aximclken      ,       // axi clk enable
        .ahbpclken      ,
        .ahbsclken      ,
        .axim           (cm7_axim       ),
        .ahbp           (cm7_ahbp       ),
        .ahbs           (cm7_ahbs       ),
    // coreuser 
        .coreusermap    (coreusermap    ),
        .coreuser       (coreuser       ),
    // debug
        .swclk          (clkswd_cm7     ),
        .swdio          (swdio          )
    );

// vexsys

    vexsys #(
        .PM_COREUSERCNT(1),
        .IRQNUM        (IRQCNT+16),
        .AXIIID4       (daric_cfg::AMBAID4_VEXI),
        .AXIDID4       (daric_cfg::AMBAID4_VEXD),
        .AHBPID4       (daric_cfg::AMBAID4_VEXP)
        )vexsys(
        /*input   logic             */  .clk            (clk_vex0),            // Free running clock
        /*input   logic             */  .resetn         (resetn_vex),
        /*input   logic             */  .ahbpclken      ,
        /*input   logic             */  .cmsatpg        ,
        /*input   logic             */  .cmsbist        ,
        /*input   logic             */  .vexcfg_dev     ,
        /*input   logic [31:0]      */  .vexcfg_iv      ,// = 32'h6000_0000;
        /*input   logic [IRQNUM-1:0]*/  .vex_irq        ({cm7_irq,16'h0}),
        /*axiif.master              */  .iaxim          (vex_iaxi),
        /*axiif.master              */  .daxim          (vex_daxi),
        /*ahbif.master              */  .ahbp           (vex_ahbp),
        /*output  logic             */  .coreuser       (),//##
        .jtags (jtagvex)
    );


    end
endgenerate

// security crypto engine

//    axim_null usce_axis_null(.aximaster(sce_axi32));
//    ahbs_null usce_ahbs_null(.ahbslave(coreahbmux[2]));

    logic [7:0] sceintr, sceerrs;
    logic secmode;

    assign sceevo[31:0] = sceintr|'0;
    assign sceerro = |sceerrs;

    sce #(
        .AXID ( daric_cfg::AMBAID4_SCEA ),
        .COREUSERCNT ( PM_COREUSERCNT ),
    //    parameter type coreuser_t = bit[0:COREUSERCNT-1],
        .INTC ( 8 ),
        .ERRC ( 8 )
    )sce(
        .clk        (clksce),
        .resetn,
        .cmsatpg, .cmsbist,

        .coreuser   ( coreuser ),
        .sceuser    ( sceuser  ),
        .secmode    ( secmode  ),

        .ahbs       ( coreahbmux[2] ),
        .axim       ( sce_axi32 ),

        .intr       ( sceintr ),
        .err        ( sceerrs )
    );

// main dma

//    ahbm_null mdma_ahb32_null(.ahbmaster(mdma_ahb32));
//    apbs_null mdma_apbs_null(.apbslave(coresubapbs[1]));

    logic mdmairq, mdmaerr;

    mdma #(
        .CHNLC      ( 8 ),
        .AHBMID4    ( daric_cfg::AMBAID4_MDMA ),
        .EVC        ( IRQCNT + 16 )
    )mdma(
        .clk        (clkdma),
        .resetn,

        .evin       ({cm7_irq,16'h0}),
        .irq        (mdmairq),
        .err        (mdmaerr),

        .ahbm       (mdma_ahb32),
        .apbs_dma   (coresubapbs[1]),
        .apbs       (coresubapbs[2]),
        .apbx       (coresubapbs[2])
    );

// bus matrix

generate
    if(1) begin: __bmx

    ahbif #(.AW(32),.DW(32),.IDW(4),.UW(4)) 
        core_ahbp3[0:2](), core_ahbp();

     bmxcore bmxcore
    (
/*    input bit           */.aclk           (aclk        ),
/*    input bit           */.hclk           (hclk        ),
/*    input bit           */.resetn         (resetn      ),
                            .cmsatpg        (cmsatpg     ),
/*    axiif.slave         */.cm7_axim       (cm7_axim    ),
                            .vex_iaxi,
                            .vex_daxi,
/*    ahbif.slave         */.cm7_ahbp       (core_ahbp   ),
/*    axiif.slave         */.sce_axi32      (sce_axi32  ),
/*    ahbif.slave         */.mdma_ahb32     (mdma_ahb32  ),
/*    axiif.master        */.rrc_axi64      (rrc_axi64   ),
/*    axiif.master        */.sram0_axi64    (sram_axi64[0] ),
/*    axiif.master        */.sram1_axi64    (sram_axi64[1] ),
/*    axiif.master        */.qfc_axi64      (qfc_axi64   ),
/*    ahbif.master        */.cm7_ahbs       (cm7_ahbs    ),
/*    ahbif.master        */.core_ahb32     (core_ahb32  ),
/*    ahbif.master        */.bmxif_ahb32    (bmxif_ahb32 )
    );

    ahb_thru  _ahbp0( .ahbslave( cm7_ahbp ), .ahbmaster( core_ahbp3[0] ) );
    ahb_thru  _ahbp1( .ahbslave( vex_ahbp ), .ahbmaster( core_ahbp3[1] ) );
    ahbm_null _ahbp2( .ahbmaster( core_ahbp3[2] ) );

    ahb_mux3 #(
          .AW(32),
          .DW(32)
    )coreahbpmux(
          .hclk,
          .resetn,
          .ahbslave     (core_ahbp3),
          .ahbmaster    (core_ahbp)
    );

    ahb_demux_map #(
        .SLVCNT                 ( 7  ),
        .DW                     ( 32 ),
        .AW                     ( 32 ),
        .UW                     ( 4 ),
        .ADDRMAP                ( coreahb_demux_map )
    ) coreahb_mux (
        .hclk                   ( hclk    ),
        .resetn                 ( resetn  ),
        .ahbslave               ( core_ahb32 ),
        .ahbmaster              ( coreahbmux )
    );


    ahbs_null coreahbmux3_null(.ahbslave(coreahbmux[3]));

//    ahbs_null uapb_ahb_null(.ahbslave(coreahbmux[7]));
    apb_bdg #(.PAW(16)) u1(.ahbslave(coreahbmux[1]),.apbmaster(coresubapb),.hclk(hclk),.resetn(resetn),.pclken(pclken));

    ahb_thru coreahb4(.ahbslave(coreahbmux[4]), .ahbmaster(coreahb_sys));
    ahb_thru coreahb5(.ahbslave(coreahbmux[5]), .ahbmaster(coreahb_sec));
    ahb_thru coreahb6(.ahbslave(coreahbmux[6]), .ahbmaster(coreahb_ao));

    apb_mux #(.PAW(16),.DECAW(4)) coresubapbmux(
        .apbslave (coresubapb),
        .apbmaster(coresubapbs)
    );

    apbs_nulls #(.SLVCNT(16-3)) coresubapbs_null(.apbslave(coresubapbs[3:15]));

    end
endgenerate

generate
    if(1) begin: __sram
// 
//sram0_axi64
    axisramc64 sramc0 (
        .clk                    ( aclk     ),
        .resetn                 ( resetn  ),
        .axislave               ( sram_axi64[0] ),
        .rammaster              ( sramc[0] )
    );

    axisramc64 sramc1 (
        .clk                    ( aclk     ),
        .resetn                 ( resetn  ),
        .axislave               ( sram_axi64[1] ),
        .rammaster              ( sramc[1] )
    );
`ifdef FPGA

    uram_cas #( .XX (4), .YY (8)) sram0 (
      .clk          (aclk),
      .resetn,
      .waitcyc      ('0),
      .rams         (sramc[0])
    );

    uram_cas #( .XX (4), .YY (8)) sram1 (
      .clk          (aclk),
      .resetn,
      .waitcyc      ('0),
      .rams         (sramc[1])
    );

`else
    core_srambank #(
        .RC     (daric_cfg::coresrammacrocnt0),
        .thecfg (daric_cfg::coresramcfg0)
    )sram0(
        .clk                    ( aclk ),
        .resetn,
        .cmsatpg,
        .cmsbist,
        .scmbkey                ('0),
        .prerr                  (),
        .verifyerr              (),
        .rams                   ( sramc[0] )
    );

    core_srambank #(
        .RC     (daric_cfg::coresrammacrocnt1),
        .thecfg (daric_cfg::coresramcfg1)
    )sram1(
        .clk                    ( aclk ),
        .resetn,
        .cmsatpg,
        .cmsbist,
        .scmbkey                ('0),
        .prerr                  (),
        .verifyerr              (),
        .rams                   ( sramc[1] )
    );
`endif

    end
endgenerate

// rrc

    jtagif jtagrrc[0:1]();

rrc #(
        .BRC  (BRC ),
        .BRCW (BRCW),
        .BRDW (BRDW)
    )rrc(
/*    input logic              */   .clk            (aclk           ),
/*    input logic              */   .clktop         (clktop         ),
/*    input logic              */   .clksys         (clksys         ),
/*    input logic              */   .clken          (aximclken&fclken),
/*    input logic              */   .hclk           (hclk           ),
/*    input logic              */   .sysresetn      (sysresetn      ),
/*    input logic              */   .coreresetn     (coreresetn     ),
/*    axiif.slave              */   .axis           (rrc_axi64      ),
/*    ahbif.slave              */   .ahbs           (coreahbmux[0]  ),  //ahb -acram, ->apb sfr
/*    input  logic [3:0]       */   .brready        (brready        ),
/*    output logic             */   .brvld          (brvld          ),
/*    output logic [BRCW-1:0]  */   .bridx          (bridx          ),
/*    output logic [BRDW-1:0]  */   .brdat          (brdat          ),
/*    output logic             */   .brdone         (brdone         ),
/*    output logic             */   .rrcint         (rrcint         ),
                                    .rri                             ,
                                    .rro                             ,

                                    .cmsatpg        (cmsatpg        ),        
                                    .scan_resetn    ('0             ),
                                    .scan_test      ('0             ),
                                    .scan_en        ('0             ),
                                    .test_resetn    ('0             ),
                                    .test_en        ('0             ),
                                    .jtag           (jtagrrc        )      // need add new jtagrrc.
);


// reram x2
    rrc_pkg::rri_t [1:0] rri;
    rrc_pkg::rro_t [1:0] rro;

    generate
        for (genvar i = 0; i < 2; i++) begin:greram
             rerammacro reram(.rri(rri[i]),.rro(rro[i]),.ANALOG_0());
        end
    endgenerate

//bistrd ##

// qspi flash controller
    logic qfcirq;

    qfc qfc(

    /*    input bit*/   .clk(clkqfc),    
    /*    input bit*/   .pclk,    
    /*    input bit*/   .resetn,
    /*    input bit*/   .cmsatpg,
    /*    input bit*/   .cmsbist,

        .axis ( qfc_axi64 ),
        .apbs ( coresubapbs[0] ),
        .apbx ( coresubapbs[0] ),

        .qfc_sck,
        .qfc_sckn,
        .qfc_dqs,
        .qfc_ss,
        .qfc_sio,
//        .qfc_rwds,
        .qfc_rstm,
        .qfc_rsts,
        .qfc_int,

        .padcfg_qfc_sck,
        .padcfg_qfc_qds,
        .padcfg_qfc_ss,
        .padcfg_qfc_sio,
//        .padcfg_qfc_rwds,
        .padcfg_qfc_int,
        .padcfg_qfc_rst,

        .irq ( qfcirq )
    );


// apb system


//cm7resetreq

// ev/err
    assign coresubevo[15:0] = '0; // first 16 are internal.
    assign coresubevo[16] = qfcirq;
    assign coresubevo[17] = mdmairq;
    assign coresubevo[31:18] = '0;
    assign coresuberro = mdmaerr;

// system ctrl

    assign coreusermap = code_mem_map;//{};
//    assign cmsatpg = cmscode == CMS_ATPG;
//    assign cmsbist = cmscode == CMS_TEST;

// apb security


endmodule : soc_coresub


module dummytb_soc_coresub();
    ioif swdio();
//    ahbif bmxif_ahb32();
//    bit clk,resetn;
    parameter BRC  = daric_cfg::BRC;
    parameter BRCW = daric_cfg::BRCW;
    parameter BRDW = daric_cfg::BRDW;
    parameter IRQCNT = daric_cfg::IRQCNT;

    logic   clktop, clktopen;
    logic   clksys;
    logic   clk1M;
    logic   fclk;
    logic   aclk;
    logic   hclk;
    logic   iclk;
//    logic   clkcore;
    logic   clkdma;
    logic   clksce;
    logic   clkqfc;
    logic   clkvex;
    logic   aximclken;
    logic   ahbpclken;
    logic   ahbsclken;
    logic   sysresetn;
    logic   coreresetn;
    logic   cm7sleep;
    logic   cm7resetreq;
    cms_pkg::cmscode_e   cmscode;
    logic       cmsatpg, cmsbist;
    logic       clkswd;
    logic [3:0]          brready;
    logic                brvld;
    logic [BRCW-1:0]     bridx;
    logic [BRDW-1:0]     brdat;
    logic                brdone;
    //logic   clktop;
    logic   fclken;
    logic [IRQCNT-1-16:0]  cm7_irq;
    logic cm7_nmi;
    logic cm7_rxev;
    logic [31:0]    coresubevo;
    logic           coresuberro;
    logic [31:0]    sceevo;
    logic           sceerro;

// qfc
    padcfg_arm_t  padcfg_qfc_sck;
    padcfg_arm_t  padcfg_qfc_qds;
    padcfg_arm_t  padcfg_qfc_ss;
    padcfg_arm_t  padcfg_qfc_sio;
    padcfg_arm_t  padcfg_qfc_rwds;
    padcfg_arm_t  padcfg_qfc_int;
    padcfg_arm_t  padcfg_qfc_rst;
    ioif qfc_sck();
    ioif qfc_sckn();
    ioif qfc_dqs();
    ioif qfc_ss[1:0]();
    ioif qfc_sio[7:0]();
    ioif qfc_rwds();
    ioif qfc_rstm[1:0]();
    ioif qfc_rsts[1:0]();
    ioif qfc_int();

// 
    jtagif jtagvex();



    ahbif #(.AW(32),.DW(32),.IDW(4),.UW(4)) 
        bmxif_ahb32(), coreahb_sys(), coreahb_sec(), coreahb_ao();

    soc_coresub u1(.bmxif_ahb32(bmxif_ahb32),.swdio      (swdio),.*);
endmodule
