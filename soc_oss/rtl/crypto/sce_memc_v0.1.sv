`include "template.sv"
import scedma_pkg::*;


module sce_memc #(

    parameter SEGCNT = 32,
    parameter INCNT  = 8,
    parameter RAMCNT = 4,

    parameter ramcfg_t [0:RAMCNT-1] RAMCFGs = '0,
    parameter segcfg_t [0:SEGCNT-1] SEGCFGs = '0,
    parameter ramcfg_t [0:RAMCNT-1] RAMCFGs = '0,



)(

    input   bit             clk,
    input   bit             resetn,

// fifo ctrl/status
    input   bit        [0:SEGCNT-1]     segfifoen,
    input   bit        [0:SEGCNT-1]     segfifoclr,
    output  adr_t      [0:SEGCNT-1]     segfifocnt,
    output  bit        [0:SEGCNT-1]     segfifo_full,
    output  bit        [0:SEGCNT-1]     segfifo_almf,
    output  bit        [0:SEGCNT-1]     segfifo_empt,
    output  bit        [0:SEGCNT-1]     segfifo_alme,

// chnl port
    input   porttype_e [0:INCNT-1]      porttype,
    input   segcfg_t   [0:INCNT-1]      rams_segcfg,
    input   adr_t      [0:INCNT-1]      rams_ptr,
    input   bit        [0:INCNT-1]      rams_rd,
    input   bit        [0:INCNT-1]      rams_wr,
    input   dat_t      [0:INCNT-1]      rams_wdat,
    output  dat_t      [0:INCNT-1]      rams_rdat,
    output  bit        [0:INCNT-1]      rams_ready,

// ram port
    output  adr_t      [0:RAMCNT-1]     ramm_addr,
    output  bit        [0:RAMCNT-1]     ramm_cs,
    output  bit        [0:RAMCNT-1]     ramm_wr,
    output  dat_t      [0:RAMCNT-1]     ramm_wdat,
    input   dat_t      [0:RAMCNT-1]     ramm_rdat,

    output  bit        [7:0]            intr
);

    logic [0:INCNT-1][7:0] thesegid;
    logic [0:INCNT-1][7:0] theramsel;
    logic adr_t [0:INCNT-1] ramsegaddr;
    logic [0:INCNT-1] ramrd0, ramwr0, rams_ready, theramselreg;
    logic adr_t [0:INCNT-1] ramptr0;
    logic dat_t [0:INCNT-1] rams_rdat;
    logic [0:INCNT-1] arbs_vld;

genvar  gvi, gvj, gvk;

generate
    for(gvi=0; gvi<INCNT; gvi++) begin : genRAMSEG

    // extract seg cfg, static during trans
    assign thesegid[gvi]   = rams_segcfg[gvi].segid;
    assign theramsel[gvi]  = rams_segcfg[gvi].ramsel;
    assign ramsegaddr[gvi] = rams_segcfg[gvi].segaddr;

    // if fifo is enabled, only when fifo is valid
    assign ramrd0[gvi]  = ffen[thesegid[gvi]] ? rams_rd[gvi] & ~fo_empt[thesegid[gvi]] : rams_rd[gvi];
    assign ramwr0[gvi]  = ffen[thesegid[gvi]] ? rams_wr[gvi] & ~fi_full[thesegid[gvi]] : rams_wr[gvi];
    assign ramptr0[gvi] = ffen[thesegid[gvi]] ? fframptr[thesegid[gvi]]                : rams_ptr[gvi];

    // if fifo is enabled, only when fifo is valid
    assign rams_ready[gvi] = arbs_gnt_2d[theramsel[gvi]][gvi] &
                             ( ffen[thesegid[gvi]] ? 
                                     (( porttype[gvi] = PT_RO ) & ~fo_empt[thesegid[gvi]])
                                    |(( porttype[gvi] = PT_RW ) & ~fo_full[thesegid[gvi]])
                                    : 1'b1 ) ;

    // rdat only need to 
    `theregrn( theramselreg[gvi] ) <= theramsel[gvi];
    assign rams_rdat[gvi] = ramm_rdat[theramselreg[gvi]];

    // output for arb, axi style
    assign arbs_vld[gvi] = ramrd0[gvi] | ramwr0[gvi];
    assign arbs_dat[gvi] = '0{ 
                segaddr:    ramsegaddr[gvi], 
                segptr:     ramptr0[gvi], 
                ramrd:      ramrd0[gvi], 
                ramwr:      ramwr0[gvi], 
                ramwdat:    rams_wdat[gvi]
             };

    end
endgenerate

generate
    for(gvj=0; gvj<SEGCNT; gvj++) begin : genSEG

    assign ffen[gvj]  = SEGCFGs[gvj].isfifo & segfifoen[gvj];
    assign ffclr[gvj] = SEGCFGs[gvj].isfifo & segfifoclr[gvj];
    assign segfifocnt[gvi] = ffcnt[gvi];

    for(gvi=0; gvi<INCNT; gvi++) begin : genIN
        assign fi_rd_2d[gvj][gvi] = ( theramsel[gvi] == gvj ) & ramrd0[gvi] ;
        assign fi_wr_2d[gvj][gvi] = ( theramsel[gvi] == gvj ) & ramwr0[gvi] ;
    end

    assign fi_rd[gvj] = |fi_rd_2d[gvj];
    assign fi_wr[gvj] = |fi_wr_2d[gvj];

    if(SEGCFGs.isfifo)begin: genisfifo
        scedma_simplefifo #(
            parameter AW = 8,
            parameter ALMF = 2,
            parameter ALME = 2,
            parameter bit OPT_STREAMING = SEGCFGs[gvj].isfifostream,
        )segff(
            /* input   bit           */  .clk           (clk            ),
            /* input   bit           */  .resetn        (resetn         ),
            /* input   scefifocfg_t  */  .thecfg        (SEGCFGs[gvj]   ),
            /* input   bit           */  .ffen          (ffen[gvj]      ),
            /* input   bit           */  .ffclr         (ffclr[gvj]     ),
            /* output [AW:0]         */  .ffcnt         (ffcnt[gvj]     ),
            /* input   bit           */  .fi_wr         (fi_wr[gvj]     ),    // fifo in write
            /* output  bit           */  .fi_full       (fi_full[gvj]   ),    // fifo in full
            /* output  bit           */  .fi_almf       (fi_almf[gvj]   ),    // fifo in almost full
            /* input   bit           */  .fo_rd         (fo_rd[gvj]     ),   // fifo out read, should be pulse
            /* output  bit           */  .fo_empt       (fo_empt[gvj]   ),   // fifo out empty
            /* output  bit           */  .fo_alme       (fo_alme[gvj]   ),   // fifo out almost empty
            /* output  bit [RAW-1:0] */  .ramptr        (fframptr[gvj]  ),
            /* output  bit [7:0]     */  .intr          (ffint[gvj]     )
        );

        end
    else begin: gennotfifo
            assign ffcnt[gvj] = '0;
            assign { fi_full[gvj], fi_almf[gvj], fo_empt[gvj], fo_alme[gvj] } = '0;
            assign fframptr[gvj] = '0;
            assign ffint[gvj] = '0;
        end
    end
endgenerate

generate
    for(gvk=0; gvk<RAMCNT; gvk++) begin : genRAM

    rr_arb_tree #(
      .NumIn    ( INCNT         ),
      .DataType ( arbdat_t      ),
      .AxiVldRdy( 1'b1          ),
      .LockIn   ( 1'b1          )
    ) i_aw_arbiter (
      .clk_i  ( clk             ),
      .rst_ni ( resetn          ),
      .flush_i( 1'b0            ),
      .rr_i   ( '0              ),
      .req_i  ( arbs_req_2d[gvk]        ),
      .gnt_o  ( arbs_gnt_2d[gvk]        ),
      .data_i ( arbs_dat_2d[gvk]        ),
      .req_o  ( arbm_req[gvk]           ),
      .gnt_i  ( arbm_gnt[gvk]           ),
      .data_o ( arbm_dat[gvk]           ),
      .idx_o  (                         )
    );


    for(gvi=0; gvi<INCNT; gvi++) begin : genRAMSEG
        assign arbs_req_2d[gvk][gvi] = ( theramsel[gvi] == gvk ) & arbs_vld[gvi] ;
        assign arbs_dat_2d[gvk][gvi] = arbs_dat[gvi];
    end


//    assign arbs_dat[gvk] = { ramsegaddr[gvk], ramptr0[gvk], ramrd[gvk], ramwr[gvk], ramwdat[gvk] };
    assign arbm_gnt[gvk] = 1'b1;

    assign ramm_addr = arbm_dat[gvk].segaddr + arbm_dat[gvk].segptr;
    assign ramm_cs = arbm_dat[gvk].ramrd | arbm_dat[gvk].ramwr;
    assign ramm_wr =  arbm_dat[gvk].ramwr;
    assign ramm_wdat = arbm_dat[gvk].ramwdat;

    end
endgenerate

endmodule : sce_memc
