`include "template.sv"

module sce #(
    parameter AXID = 5,
    parameter COREUSERCNT = 8,
    parameter type coreuser_t = bit[0:COREUSERCNT-1],
    parameter INTC = 8,
    parameter ERRC = 8,
    parameter TSC = 16
)(
    input wire        ana_rng_0p1u,
    input logic clk,
    input logic clktop,
    input logic clksceen,
//    input logic clkpke,
    input logic clkpkeen,
    input logic resetn,
    input logic cmsatpg, cmsbist,

    input  coreuser_t   coreuser,
    output coreuser_t   sceuser,
    output logic        secmode,

    ahbif.slave ahbs,
    axiif.master axim[0:1],

    output logic [TSC-1:0] truststate,
    output logic [INTC-1:0] intr,
    output logic [ERRC-1:0] err
);

// cfg

    parameter axi_pkg::xbar_rule_32_t [2:0] ADDRMAP = '{
        '{idx: 32'd2 , start_addr: 32'h0000_4000, end_addr: 32'h0000_6000}, // pkeram
        '{idx: 32'd1 , start_addr: 32'h0000_0000, end_addr: 32'h0000_2800}, // ram
        '{idx: 32'd0 , start_addr: 32'h0000_8000, end_addr: 32'h0000_FFFF}  // ctrl
    };
    parameter CHNLCNT = 17;
    parameter SEGCNT = scedma_pkg::SEGCNT;
    parameter SUBCNT = 5;
    parameter SUBCNT2 = 8;
    parameter CHNLACCNT = scedma_pkg::CHNLACCNT;
    parameter RAMCNT = scedma_pkg::RAMCNT;
    parameter AW = scedma_pkg::AW;
    parameter segcfg_t [0:SEGCNT-1] SEGCFGS = scedma_pkg::SEGCFGS;
    parameter sram_pkg::sramcfg_t [0:RAMCNT-1] RAMCFGS = scedma_pkg::SCERAMCFGS;

    bit [4:0]       cr_ahbsopt;
    logic [0:FFCNT-1]       segfifoen;
    logic [0:FFCNT-1][3:0]  segfifosr;
    adr_t [0:FFCNT-1]       segfifocnt;
    logic [0:FFCNT-1]       segfifoclr;
    chnlreq_t [0:CHNLACCNT-1] chnlreq;
    chnlres_t [0:CHNLACCNT-1] chnlres;
    chnlreq_t [0:CHNLCNT-1] ramsreq;
    chnlres_t [0:CHNLCNT-1] ramsres;
    logic [1:0]         glb_scemode;
    logic [0:SUBCNT-1]  cr_suben, clksub;
    logic [0:SUBCNT2-1+1] glb_busy;
    logic [0:SUBCNT2-1] glb_done;
    logic [0:SUBCNT2-1] glb_err;
    logic               ar_reset, ar_clrram;
    logic               alu_busy, hash_busy, pke_busy, aes_busy, trng_busy;
    logic               alu_done, hash_done, pke_done, aes_done, trng_done;
    logic [0:7]         alu_err , hash_err , pke_err , aes_err , trng_err ;
    logic [2:0]         sdma_busy;
    logic [2:0]         sdma_done;
    logic [2:0]         sdma_err;
    logic               acenable;
    chnlreq_t [0:CHNLCNT-1] ramreq;
    chnlres_t [0:CHNLCNT-1] ramres;
    logic   [0:CHNLACCNT-1] acerr;
    adr_t      [0:RAMCNT-1] ramaddr ;
    bit        [0:RAMCNT-1] ramrd   ;
    bit        [0:RAMCNT-1] ramwr   ;
    dat_t      [0:RAMCNT-1] ramwdat ;
    dat_t      [0:RAMCNT-1] ramrdat ;
    bit        [0:RAMCNT-1][0:1] ramerr   ;

    logic mode_non, mode_xls, mode_sec;
    logic sceresetn;
    logic ahbs_lock;
    logic sceresetnin, sceramclr;
    logic ramclrbusy;
    logic clkpke;
//
// ==
    assign secmode = mode_sec;
    assign intr = {             sdma_done, alu_done, hash_done, pke_done, aes_done, trng_done };
    assign err = glb_err;

// amba
// ■■■■■■■■■■■■■■■

    ahbif #(.AW(32),.UW(8)) ahbs0();

    ahbif #(.AW(32),.UW(8)) ahbmux[0:2]();
    apbif #(.PAW(15)) apb0();
    apbif #(.PAW(12)) apbs[0:7]();

    ahb_sync #(.AW(32),.DW(32),.SYNCDOWN(0),.SYNCUP(0))ahbs_sync(
        .hclken   ('1),
        .hclk     (clk),
        .resetn   (sceresetn),
        .ahbslave (ahbs),
        .ahbmaster(ahbs0)
        );

    ahb_demux_map #(
        .SLVCNT(3),
        .AW(16),
        .UW(8),
        .ADDRMAP(ADDRMAP)
    )hmux(
        .hclk     (clk),
        .resetn   (sceresetn),
        .ahbslave (ahbs0),
        .ahbmaster(ahbmux)
    );

    apb_bdg #(.PAW(15)) pbdg(
        .hclk     (clk),
        .resetn   (sceresetn),
        .pclken   (1'b1),
        .ahbslave (ahbmux[0]),
        .apbmaster(apb0)
    );

    apb_mux #(.PAW(15),.DECAW(3)) pmux(
        .apbslave (apb0),
        .apbmaster(apbs)
    );

    apbs_null u0(apbs[2]);

// chnl - ahbs
// ■■■■■■■■■■■■■■■

    scedmachnl_ahbs #(
//        .CHID      ( 0 ),
//        .AW        ( scedma_pkg::AW ),
//        .DW        ( scedma_pkg::DW ),
//        .FFCNT     ( scedma_pkg::FFCNT ),
//        .BA        ( '0 ),
//        .SEGCNT    ( scedma_pkg::SEGCNT ),
//        .SEGCFGS   ( scedma_pkg::SEGCFGS )
    )ahbc(
    /*    input logic           */  .clk,
    /*    input logic           */  .resetn         (sceresetn),
    /*    ahbif.slave           */  .ahbs           (ahbmux[1]),
    /*    input bit [4:0]       */  .cr_opt         (cr_ahbsopt),
    /*    input bit [0:FFCNT-1] */  .segfifoen,
    /*    output chnlreq_t      */  .rpreq          ( chnlreq[0] ),
    /*    input  chnlres_t      */  .rpres          ( chnlres[0] ),
    /*    output chnlreq_t      */  .wpreq          ( chnlreq[1] ),
    /*    input  chnlres_t      */  .wpres          ( chnlres[1] ),
    /*    output bit [7:0]      */  .intr           (),
    /*    output bit [7:0]      */  .err            ()
    );

// sfr
// ■■■■■■■■■■■■■■■

    sce_glbsfr #(
        .SUBCNT (SUBCNT),
        .SRCNT  ( SUBCNT2 + 1 ),
        .FRCNT  ( SUBCNT2 ),
        .ERRCNT ( SUBCNT2 ),
        .CHNLACCNT (CHNLACCNT)
    )glbsfr
    (
                .clk,
                .resetn(sceresetn),
                .cmsatpg, .cmsbist,
                .apbs           (apbs[0]),
                .apbx           (apbs[0]),

    /* output logic [1:0]             */  .cr_scemode (glb_scemode),
                                          .cr_suben   (cr_suben),
                                          .cr_ahbsopt (cr_ahbsopt),
    /* input  logic [0:SRCNT-1]       */  .sr_busy    (glb_busy),
    /* input  logic [0:FRCNT-1]       */  .fr_done    (glb_done),
    /* input  logic [0:ERRCNT-1]      */  .fr_err     (glb_err),
    /* output  logic                  */  .ar_reset,
    /* output  logic                  */  .ar_clrram,
    /* output  logic [0:FFCNT-1]      */  .cr_ffen    (segfifoen),
    /* input   logic [0:FFCNT-1][3:0] */  .sr_ffsr    (segfifosr),
    /* input   adr_t [0:FFCNT-1]      */  .sr_ffcnt   (segfifocnt),
    /* output  logic [0:FFCNT-1]      */  .ar_ffclr   (segfifoclr),
                                          .fr_acerr   (acerr),
                                          .cr_ts      (truststate)
    );

    assign glb_busy = { ramclrbusy, sdma_busy, alu_busy, hash_busy, pke_busy, aes_busy, trng_busy } | '0;
    assign glb_done = {             sdma_done, alu_done, hash_done, pke_done, aes_done, trng_done } | '0;
    assign glb_err  = { ramerr[0][0:1] , hash_err[0:1] , pke_err[0:1] , aes_err[0:1] } | '0;//{ sdma_err , alu_err , hash_err , pke_err , aes_err , trng_err  };

    generate
        for (genvar gvi = 0; gvi < SUBCNT; gvi++) begin: gensub
        `ifdef FPGA
            assign clksub[gvi] = clk;
        `else
            ICG uclksub ( .CK ( clk ), .EN ( cr_suben[gvi] ), .SE(cmsatpg), .CKG ( clksub[gvi] ));
        `endif
        end
    endgenerate

        `ifdef FPGA
            assign clkpke = clk;
        `else
            ICG uclkpke ( .CK ( clktop ), .EN ( cr_suben[2] && clkpkeen ), .SE(cmsatpg), .CKG ( clkpke ));
        `endif

// sec
// ■■■■■■■■■■■■■■■

    sce_sec #(
        .COREUSERCNT (COREUSERCNT),
        .coreuser_t  (coreuser_t)
    )sec(
        .clk,
        .resetn(sceresetn),
        .scemode(glb_scemode),
        .coreuser,
        .sceuser,
        .mode_non,
        .mode_xls,
        .mode_sec,
        .ahbs_lock,
        .ar_reset,
        .ar_clrram,
        .sceresetnin,
        .sceramclr
    );

// reset

    sceresetgen #(.ICNT(1),.EXTCNT(16))sceresetgen(
        .clk         ( clk ),
        .cmsatpg     ( cmsatpg ),
        .resetn      ( resetn ),
        .resetnin    ( sceresetnin ),
        .resetnout   ( sceresetn )
    );

// dma
// ■■■■■■■■■■■■■■■

    scedma #(
        .AXID       ( AXID )//,
//        .AW         ( scedma_pkg::AW ),
//        .DW         ( scedma_pkg::DW ),
//        .FFCNT      ( scedma_pkg::FFCNT ),
//        .BA         ( SCERAM_BA ),
//        .SEGCNT     ( scedma_pkg::SEGCNT ),
//        .SEGCFGS    ( scedma_pkg::SEGCFGS ),
//        .TRANSCNTW  ( 16 )
    )dma(
/*    input logic          */  .clk             (clksub[4]),
/*    input logic          */  .resetn          (sceresetn),
/*    apbif.slavein        */  .apbs            ( apbs[1] ),
/*    apbif.slave          */  .apbx            ( apbs[1] ),
/*    axiif.master         */  .axim,
/*    output chnlreq_t     */  .xchrpreq        ( chnlreq[2] ),
/*    input  chnlres_t     */  .xchrpres        ( chnlres[2] ),
/*    output chnlreq_t     */  .xchwpreq        ( chnlreq[3] ),
/*    input  chnlres_t     */  .xchwpres        ( chnlres[3] ),
/*    output chnlreq_t     */  .schrpreq        ( chnlreq[4] ),
/*    input  chnlres_t     */  .schrpres        ( chnlres[4] ),
/*    output chnlreq_t     */  .schwpreq        ( chnlreq[5] ),
/*    input  chnlres_t     */  .schwpres        ( chnlres[5] ),
/*    output chnlreq_t     */  .ichrpreq        ( chnlreq[6] ),
/*    input  chnlres_t     */  .ichrpres        ( chnlres[6] ),
/*    output chnlreq_t     */  .ichwpreq        ( chnlreq[7] ),
/*    input  chnlres_t     */  .ichwpres        ( chnlres[7] ),
/*    input bit [0:FFCNT-1]*/  .segfifoen       (segfifoen),
/*    output bit[3:0]      */  .sr_sdma         (sdma_busy),
/*    output bit[3:0]      */  .fr_sdma         (sdma_done),
/*    output bit [7:0]     */  .intr            (),
/*    output bit [7:0]     */  .err             ()
    );
// interconnection with access control

    assign acenable = mode_sec;

    scedma_ac dmaac(
        .clk,
        .resetn             (sceresetn),
        .acenable           ( acenable      ),
        .chnlinreq          ( chnlreq[0:CHNLACCNT-1]  ),
        .chnlinres          ( chnlres[0:CHNLACCNT-1]  ),
        .chnloutreq         ( ramreq[0:CHNLACCNT-1]   ),
        .chnloutres         ( ramres[0:CHNLACCNT-1]   ),
        .acerr              ( acerr )
    );

// memc
// ■■■■■■■■■■■■■■■

    sce_memc #(
        .AW          (AW),
        .SEGCNT      (SEGCNT),
        .INCNT       (CHNLCNT),
        .RAMCNT      (RAMCNT),
        .FFCNT       (FFCNT),
        .SEGCFGs     (SEGCFGS),
        .RAMCFGs     (RAMCFGS)
    )dmamemc(
        .clk,
        .resetn      (sceresetn),
        /*input   bit        [0:FFCNT-1]   */   .segfifoen     (segfifoen     ),
        /*input   bit        [0:FFCNT-1]   */   .segfifoclr    (segfifoclr    ),
        /*input   adr_t      [0:FFCNT-1]   */   .segfifocnt    (segfifocnt    ),
        /*output  bit        [0:FFCNT-1]   */   .segfifosr     (segfifosr     ),
        /*input   chnlreq_t  [0:INCNT-1]   */   .ramsreq       (ramreq        ),
        /*input   chnlres_t  [0:INCNT-1]   */   .ramsres       (ramres        ),
        /*output  adr_t      [0:RAMCNT-1]  */   .ramm_addr     (ramaddr       ),
        /*output  bit        [0:RAMCNT-1]  */   .ramm_rd       (ramrd         ),
        /*output  bit        [0:RAMCNT-1]  */   .ramm_wr       (ramwr         ),
        /*output  dat_t      [0:RAMCNT-1]  */   .ramm_wdat     (ramwdat       ),
        /*input   dat_t      [0:RAMCNT-1]  */   .ramm_rdat     (ramrdat       ),
        /*output  bit        [7:0]         */   .intr          (      )
    );

    generate
        for(genvar gvk=0; gvk<RAMCNT; gvk++) begin : genRAM
            cryptoram #(
                .ramname    ("SCERAM"), // HRAM, PRAM, ARAM, SCERAM
                .thecfg     (RAMCFGS[gvk])
            )m(
                .clk,
                .clkram(clk), .clkramen('1),
                .resetn(resetn),
                .cmsatpg, .cmsbist,
                .ramclr (sceramclr),
                .ramaddr (ramaddr[gvk] ),
                .ramen('1),
                .ramrd(ramrd[gvk]),
                .ramwr({4{ramwr[gvk]}}),
                .ramwdat(ramwdat[gvk]),
                .ramrdat(ramrdat[gvk]),
                .ramready(),
                .ramerror(ramerr[gvk]),
                .ramclren(ramclrbusy)
            );
        end
    endgenerate

// crypto
// ■■■■■■■■■■■■■■■

    combohash hash(
            .clk            (clksub[1]),
            .resetn         (sceresetn),
            .clkram         (clk),
            .ramclr         (sceramclr),
            .cmsatpg, .cmsbist,
            .apbs           (apbs[3]),
            .apbx           (apbs[3]),
            .chnl_rpreq     (ramreq[8]),
            .chnl_rpres     (ramres[8]),
            .chnl_wpreq     (ramreq[9]),
            .chnl_wpres     (ramres[9]),
            .busy           (hash_busy),
            .done           (hash_done),
            .err            (hash_err),
            .intr           ()
        );

    pke pke(
            .clk            (clksub[2]),
            .clktop         (clktop),
            .clksceen       ,
            .clkpke         ,
            .clkpkeen       ,
            .resetn         (sceresetn),
            .ramclr         (sceramclr),
            .cmsatpg, .cmsbist,
            .apbs           (apbs[4]),
            .apbx           (apbs[4]),
            .ahbs           (ahbmux[2]),
            .chnl_rpreq     (ramreq[10]),
            .chnl_rpres     (ramres[10]),
            .chnl_wpreq     (ramreq[11]),
            .chnl_wpres     (ramres[11]),
            .busy           (pke_busy),
            .done           (pke_done),
            .err            (pke_err),
            .intr           ()
        );

    logic [31:0] aesmask;
    assign aesmask = '0;

    aes aes(
            .clk            (clksub[3]),
            .resetn         (sceresetn),
            .sysresetn      (resetn),
            .clkram         (clk),
            .ramclr         (sceramclr),
            .cmsatpg, .cmsbist,
            .apbs           (apbs[5]),
            .apbx           (apbs[5]),
            .chnl_rpreq     (ramreq[12]),
            .chnl_rpres     (ramres[12]),
            .chnl_wpreq     (ramreq[13]),
            .chnl_wpres     (ramres[13]),
            .maskin         (aesmask),
            .busy           (aes_busy),
            .done           (aes_done),
            .err            (aes_err),
            .intr           ()
        );

    trng trng(
            .ana_rng_0p1u(ana_rng_0p1u),
            .clk            (clk),
            .resetn         (sceresetn),
            .cmsatpg, .cmsbist,
            .apbs           (apbs[6]),
            .apbx           (apbs[6]),
//            .chnl_rpreq     (ramreq[11]),
//            .chnl_rpres     (ramres[11]),
            .chnl_wpreq     (ramreq[14]),
            .chnl_wpres     (ramres[14]),
            .busy           (trng_busy),
            .done           (trng_done),
            .err            (trng_err),
            .intr           ()
        );

    alu alu(
            .clk            (clksub[0]),
            .resetn         (sceresetn),
            .cmsatpg, .cmsbist,
            .apbs           (apbs[7]),
            .apbx           (apbs[7]),
            .chnl_rpreq     (ramreq[15]),
            .chnl_rpres     (ramres[15]),
            .chnl_wpreq     (ramreq[16]),
            .chnl_wpres     (ramres[16]),
            .busy           (alu_busy),
            .done           (alu_done),
            .err            (alu_err),
            .intr           ()
        );

endmodule

// dummy crypto
// ■■■■■■■■■■■■■■■
/*
module aes #(
        parameter RAW = 8, //## or 9??
        parameter ERRCNT = 8,
        parameter INTCNT = 8
    )(

    input  logic clk, resetn, cmsatpg, cmsbist,

    apbif.slavein           apbs,
    apbif.slave             apbx,
    output  chnlreq_t       chnl_rpreq, chnl_wpreq   ,
    input   chnlres_t       chnl_rpres, chnl_wpres   ,

    output logic [0:ERRCNT-1]      err,
    output logic [0:INTCNT-1]      intr
);

    assign chnl_rpreq = '0;
    assign chnl_wpreq = '0;
    apbs_null u0(.apbslave(apbx));
    `theregrn( intr ) <= '0;
    `theregrn( err  ) <= '0;

endmodule
*/
/*
module trng #(
        parameter RAW = 8, //## or 9??
        parameter ERRCNT = 8,
        parameter INTCNT = 8
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

endmodule
*/
module alu #(
        parameter RAW = 8, //## or 9??
        parameter ERRCNT = 8,
        parameter INTCNT = 8
    )(

    input  logic clk, resetn, cmsatpg, cmsbist,

    apbif.slavein           apbs,
    apbif.slave             apbx,
    output bit busy,
    output bit done,
    output  chnlreq_t       chnl_rpreq, chnl_wpreq   ,
    input   chnlres_t       chnl_rpres, chnl_wpres   ,

    output logic [0:ERRCNT-1]      err,
    output logic [0:INTCNT-1]      intr
);

    assign chnl_rpreq = '0;
    assign chnl_wpreq = '0;
    apbs_null u0(.apbslave(apbx));
    `theregrn( intr ) <= '0;
    `theregrn( err  ) <= '0;

    assign busy = '0;
    assign done = '0;

endmodule

module dummytb_sce ();
    parameter COREUSERCNT = 8;
    parameter type coreuser_t = bit[0:COREUSERCNT-1];
    parameter INTC = 8;
    parameter ERRC = 8;
    logic clk;
    logic resetn;
    logic cmsatpg, cmsbist;
    coreuser_t   coreuser;
    coreuser_t   sceuser;
    logic        secmode;
    ahbif ahbs();
    axiif axim[0:1]();
    logic [INTC-1:0] intr;
    logic [ERRC-1:0] err;
    logic clktop, clksceen, clkpke, clkpkeen;
    logic [15:0] truststate;
    logic ana_rng_0p1u;
    sce u0(.*);

endmodule

module sceresetgen #(
        parameter ICNT = 4,
        parameter EXTCNT = 4096,
        parameter ECW = $clog2(EXTCNT)
    )(
        input   logic               clk,
        input   logic               cmsatpg,
        input   logic               resetn,
        input   logic [0:ICNT-1]    resetnin,
        output  logic               resetnout
    );

    bit [ECW-1:0] resetextcnt;
    logic resetextcnthit;
    logic resetext;
    logic resetninx;

    assign resetninx = &resetnin & resetn;

    `theregfull(clk, resetninx, resetextcnt, '0) <= resetextcnthit ? resetextcnt : resetextcnt + 1;
    `theregfull(clk, resetninx, resetext,    '0) <= resetextcnthit ;

    assign resetextcnthit = resetextcnt == EXTCNT-1;

`ifdef FPGA
    BUFG u0 (.I(resetext), .O(resetnout));
`else
    assign resetnout = cmsatpg ? 1'b1 : resetext;
`endif
endmodule