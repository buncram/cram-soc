`include "template.sv"

import scedma_pkg::*;



module scedma_chnl_tb ();
    parameter TCW = scedma_pkg::TRANSCNTW; // trans size width
    parameter DW = 32;

    bit             clk;
    bit             resetn;
    chnlcfg_t       thecfg;
    bit             start;
    bit             busy;
    bit             done;
    adr_t           rpseg;
    adr_t           rpptr;
    bit [DW-1:0]    rprdat;
    bit             rprd;
    bit             rpwr;
    bit [DW-1:0]    rpwdat;
    bit             rpready='1;
    adr_t           wpseg;
    adr_t           wpptr;
    bit [DW-1:0]    wprdat;
    bit             wprd;
    bit             wpwr;
    bit [DW-1:0]    wpwdat;
    bit             wpready='1;
    bit [7:0]       intr;
    integer i,j,k,errcnt=0,warncnt=0;
    bit             rprdatvld, wprdatvld;
    chnlreq_t       rpreq;
    chnlres_t       rpres;
    chnlreq_t       wpreq;
    chnlres_t       wpres;

    assign rpseg    = rpreq.segaddr ;
    assign rpptr    = rpreq.segptr  ;
    assign rprd     = rpreq.segrd   ;
    assign rpwr     = rpreq.segwr   ;
    assign rpwdat   = rpreq.segwdat ;
    assign rpres.segrdat  = rprdat  ;
    assign rpres.segready = rpready ;
    assign rpres.segrdatvld = rprdatvld   ;

    assign wpseg   = wpreq.segaddr      ;
    assign wpptr   = wpreq.segptr       ;
    assign wprd    = wpreq.segrd        ;
    assign wpwr    = wpreq.segwr        ;
    assign wpwdat  = wpreq.segwdat      ;
    assign wpres.segrdat  = wprdat      ;
    assign wpres.segready = wpready     ;
    assign wpres.segrdatvld = wprdatvld   ;

scedma_chnl dut(
        .clk,
        .resetn,
        .thecfg (thecfg),
        .start  (start),
        .busy   (busy),
        .done   (done),
        .*
    );

    assign rprdatvld = 1'b1;
    assign wprdatvld = 1'b1;


  //
  //  monitor and clk
  //  ==

    `genclk( clk, 100 )
    `timemarker2

    initial begin
        #(10 `MS);
    `maintestend

    parameter scedma_pkg::segcfg_t [0:7] thesegcfg = '{
        '{ 8'h0, ST_NONE, 8'h0, 'h0  , 'h100, '0, '0, '0 },
        '{ 8'h1, ST_BI,   8'h0, 'h100, 'h100, '0, '0, '0 },
        '{ 8'h2, ST_BO,   8'h0, 'h200, 'h100, '0, '0, '0 },
        '{ 8'h3, ST_SI,   8'h0, 'h300, 'h100, '0, '0, '0 },
        '{ 8'h4, ST_SO,   8'h1, 'h400, 'h100, '1, '0, '0 },
        '{ 8'h5, ST_BI,   8'h1, 'h500, 'h100, '1, '0, 'h1 },
        '{ 8'h6, ST_BO,   8'h2, 'h600, 'h100, '1, '1, 'h2 },
        '{ 8'h7, ST_SO,   8'h2, 'h700, 'h100, '1, '1, 'h3 }
    };

    parameter [255:0] theprime = 0 - 135;

    parameter scedma_pkg::chnlcfg_t thechnlcfg =
        '{ 8'h5, thesegcfg[0], thesegcfg[1], '0, '0, 'h10, '0, '0, '0,  theprime };


    `theregrn( rprdat ) <= 'h33221100 + rpptr;
    `theregrn( wprdat ) <= 'h55667700 + wpptr;


  //
  //  subtitle
  //  ==

    `maintest(scedma_chnl_tb,scedma_chnl_tb)
        #105 resetn = 1;

        thecfg = thechnlcfg;

        #(1 `US);

//        $display("ss");
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);

//        $display("ss2");
//        $display("ss3");

        thecfg.opt_ltx = 4'b1;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);
        thecfg.opt_ltx = 4'b10;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);
        thecfg.opt_ltx = 4'b100;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);
        thecfg.opt_ltx = 4'b1000;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);



//        rprdat = 'h33221100;
//        wprdat = 'h55667700;
        thecfg.opt_xor = '1;
        thecfg.opt_ltx = 4'b1;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);
        thecfg.opt_ltx = 4'b10;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);
        thecfg.opt_ltx = 4'b100;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);
        thecfg.opt_ltx = 4'b1000;
        @(negedge clk)start=1;@(negedge clk)start=0;#(10 `US);

        #(1 `MS);
    `maintestend

endmodule
