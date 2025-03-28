`include "template.sv"

import daric_cfg::*;
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
    input   logic   clktop,
    input   logic   clktopen,
    input   logic   fclken,

    input   logic   clksys,
    input   logic   clk1M,


//    input   logic   clkcore,
    input   logic   clkdma,
    input   logic   clksce,

    input   logic   aximclken,
    input   logic   ahbpclken,
    input   logic   ahbsclken,

    input   logic   sysresetn,
    input   logic   coreresetn,

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
    logic cm7cfg_dev;
    logic [31:0] cm7cfg_iv;
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

// parameters


// clk/resetn
    logic clkaxi, clkahb, clkif, resetn;
    logic rrcint;
    
//    assign clkcore = fclk;
    assign clkaxi = aclk;
    assign clkahb = hclk;
//    assign clksce = hclk;
    assign clkif  = iclk;
    assign resetn = sysresetn;

    `theregfull(fclk, resetn, clkcm7stenregs, '0) <= { clkcm7stenregs, clk1M };
    `theregfull(fclk, resetn, clkcm7sten_1M, '0) <= clkcm7stenregs[3] & ~clkcm7stenregs[2];

// coreahb mapping

    localparam RERAMUSERCNT = 0;
    localparam PM_COREUSERCNT = daric_cfg::CODEMEMCNT + RERAMUSERCNT;
    localparam rule32_t [daric_cfg::CODEMEMCNT-1:0] code_mem_map = daric_cfg::code_mem_map;
    localparam rule32_t [6:0] coreahb_demux_map = daric_cfg::coreahb_demux_map;


    rule32_t    [PM_COREUSERCNT-1:0] coreusermap;
    bit         [PM_COREUSERCNT-1:0] coreuser, sceuser;


// cm7sys

    assign cm7cfg_iv = 32'h6000_0000;

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
        .itcmcfg    (daric_cfg::itcmcfg           )
    )
    cm7sys
    (
    // system ctrl
        .clk            (fclk           ),
        .clktop         (clktop),
        .fclken,

        .resetn         (sysresetn      ),
        .coreresetn     (coreresetn     ),
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
        .swclk          (clkswd         ),
        .swdio          (swdio          )
    );

// security crypto engine

//    axim_null usce_axis_null(.aximaster(sce_axi32));
//    ahbs_null usce_ahbs_null(.ahbslave(coreahbmux[2]));

    logic [7:0] sceintr, sceerrs;
    logic secmode;

    assign sceevo[31:0] = sceintr|'0;
    assign sceerro = |sceerrs;

sce #(
    .AXID ( 5 ),
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
    apbif   mdma_apbs();

    ahbm_null mdma_ahb32_null(.ahbmaster(mdma_ahb32));
    apbs_null mdma_apbs_null(.apbslave(mdma_apbs));
    apb_bdg u3(.ahbslave(coreahbmux[3]),.apbmaster(mdma_apbs),.hclk(hclk),.resetn(resetn),.pclken(1'b1));

// bus matrix

     bmxcore bmxcore
    (
/*    input bit           */.aclk           (aclk        ),
/*    input bit           */.hclk           (hclk        ),
/*    input bit           */.resetn         (resetn      ),
                            .cmsatpg        (cmsatpg     ),
/*    axiif.slave         */.cm7_axim       (cm7_axim    ),
/*    ahbif.slave         */.cm7_ahbp       (cm7_ahbp    ),
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

//    ahbs_null uapb_ahb_null(.ahbslave(coreahbmux[7]));

    ahb_thru coreahb4(.ahbslave(coreahbmux[4]), .ahbmaster(coreahb_sys));
    ahb_thru coreahb5(.ahbslave(coreahbmux[5]), .ahbmaster(coreahb_sec));
    ahb_thru coreahb6(.ahbslave(coreahbmux[6]), .ahbmaster(coreahb_ao));

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
        .RC     (daric_cfg::coresrammacrocnt),
        .thecfg (daric_cfg::coresramcfg)
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
        .RC     (daric_cfg::coresrammacrocnt),
        .thecfg (daric_cfg::coresramcfg)
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
// reram
/*
    bit [3:0]   cmsdatavldregs;
    assign cmsdatavld = cmsdatavldregs[3];
    assign cmsdata = CMSDAT_USERMODE;
    `theregfull(clksys, sysresetn, cmsdatavldregs, 4'h1 ) <= cmsdatavldregs * 2;
    axis_null urcc_axis_null(.axislave(rrc_axi64));
    ahbs_null urcc_ahbs_null(.ahbslave(coreahbmux[0]));
*/

//    logic [3:0]          brready;
//    logic                brvld;
//    logic [BRCW-1:0]     bridx;
//    logic [BRDW-1:0]     brdat;
////    logic                brdone;


rrc #(
        .BRC  (BRC ),
        .BRCW (BRCW),
        .BRDW (BRDW)
    )rrc(
/*    input logic              */   .clk            (aclk           ),
/*    input logic              */   .clktop         (clktop         ),
/*    input logic              */   .clksys         (clksys         ),
/*    input logic              */   .clktopen       (clktopen       ),
/*    input logic              */   .sysresetn      (sysresetn      ),
/*    input logic              */   .coreresetn     (coreresetn     ),
/*    axiif.slave              */   .axis           (rrc_axi64      ),
/*    ahbif.slave              */   .ahbs           (coreahbmux[0]  ),
/*    input  logic [3:0]       */   .brready        (brready        ),
/*    output logic             */   .brvld          (brvld          ),
/*    output logic [BRCW-1:0]  */   .bridx          (bridx          ),
/*    output logic [BRDW-1:0]  */   .brdat          (brdat          ),
/*    output logic             */   .brdone         (brdone         ),
/*    output logic             */   .rrcint         (rrcint         )

);

//bistrd ##

// qspi flash controller

    axis_null uqfc_axis_null(.axislave(qfc_axi64));
    ahbs_null uqfc_ahbs_null(.ahbslave(coreahbmux[1]));

// apb system


//cm7resetreq

// ev/err
    assign coresubevo[31:0] = '0;
    assign coresuberro = '0;

// system ctrl

    assign coreusermap = code_mem_map;//{};
//    assign cmsatpg = cmscode == CMS_ATPG;
//    assign cmsbist = cmscode == CMS_TEST;
    assign cm7cfg_dev = 1'b1;

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

    ahbif #(.AW(32),.DW(32),.IDW(4),.UW(4)) 
        bmxif_ahb32(), coreahb_sys(), coreahb_sec(), coreahb_ao();

    soc_coresub u1(.bmxif_ahb32(bmxif_ahb32),.swdio      (swdio),.*);
endmodule
