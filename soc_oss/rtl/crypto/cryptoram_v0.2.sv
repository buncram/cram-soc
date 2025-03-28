`include "template.sv"

import sram_pkg::*;

module cryptoram #(
    parameter ramname = "HRAM", // HRAM, PRAM, ARAM, SCERAM
    parameter clrstart = '0,
    parameter sramcfg_t thecfg = sram_pkg::samplecfg
    )(
    input  logic clk, resetn, cmsatpg, cmsbist,
    input  logic [thecfg.AW-1:0] ramaddr,
    input  logic ramen,
    input  logic ramrd,
    input  logic [thecfg.DW/8-1:0] ramwr,
    input  logic [thecfg.DW-1:0] ramwdat,
    output logic [thecfg.DW-1:0] ramrdat,
    output logic ramready,
    output logic [1:0] ramerror
);

    localparam RC = 1;
    localparam RCW = $clog2(RC);
    localparam DW0 = thecfg.DW+thecfg.PW;
    localparam AW = thecfg.AW;


    ramif #(.RAW(thecfg.AW),.DW(thecfg.DW))           rams();
    ramif #(.RAW(thecfg.AW),.DW(thecfg.DW+thecfg.PW)) ramm();

`ifndef FPGA

    wire2ramm
    #(
        .AW(thecfg.AW),
        .DW(thecfg.DW),
        .BW(8)
    )u(
        .ramm_ramen      (ramen),
        .ramm_ramcs      (ramrd | (|ramwr)),
        .ramm_ramaddr    (ramaddr),
        .ramm_ramwr      (ramwr),
        .ramm_ramwdata   (ramwdat),
        .ramm_ramrdata   (ramrdat),
        .ramm_ramready   (ramready),
        .ramm            (rams)
    );

    gnrl_sramc #(.thecfg(thecfg))hram
    (
        .clk,
        .resetn,
        .cmsatpg,
        .cmsbist,
        .scmben('0),
        .scmbkey('0),
        .prerr(ramerror[0]),
        .verifyerr(ramerror[1]),
        .ramslave(rams),
        .rammaster(ramm)
    );

    logic [RCW-1:0]         bsel, bselreg   ;
    logic [DW0-1:0]         bd              ;
    logic [RC-1:0][DW0-1:0] bq              ;
    logic [RC-1:0]          bcen, bgwen     ;
    logic [RC-1:0][DW0-1:0]   bwen        ;
    logic [AW-RCW-1:0]        ba              ;
    logic [RC-1:0]          clkb, clkben    ;

    assign bsel = 0;// rams_ramaddr[AW-1:AW-RCW] ;
    `theregrn(bselreg) <= ( ramm.ramcs & ramm.ramready ) ? bsel : bselreg;
    assign ramm.ramready = '1;
    assign ramm.ramrdata = bq[bselreg];
    assign #0.5 bd = ramm.ramwdata;
    assign #0.5 ba = ramm.ramaddr;

generate
  for (genvar i = 0; i < RC; i++) begin: genram

    assign #0.5 bcen[i] = ~( ramm.ramcs & (bsel==i) );
    assign #0.5 bgwen[i] =  ~( |ramm.ramwr & ramm.ramcs & (bsel==i) );
//    assign #0.5 bwen[i]  =  rams_ramcs & (bsel==i) ? ~({{4{rams.ramwr[thecfg.DW/8-1]}},{4{rams.ramwr[0]}}}) : '1;

    if(ramname=="ARAM")begin:genARAM
       sce_aesram_1k  m (
         .clk         (clkb[i]),
         .q           (bq[i]),
         .cen         (bcen[i]),
         .wen         (bgwen[i]),
         .a           (ba),
         .d           (bd),
         .ema         ('1), 
         .emaw        ('1),
         .emas        ('1),
         .ret1n       (1'b1),
         .wabl        ('1),
         .wablm       ('0),
         .rawl        ('0),
         .rawlm       ('0)
         );
    end
    if(ramname=="PRAM")begin:genPRAM
    assign #0.5 bwen[i]  =  rams.ramcs & (bsel==i) ? ~({{4{rams.ramwr[thecfg.DW/8-1]}},{4{rams.ramwr[0]}}}) : '1;
       sce_pkeram_4k  m (
         .clk         (clkb[i]),
         .q           (bq[i]),
         .cen         (bcen[i]),
         .wen         (bwen[i]),
         .gwen        (bgwen[i]),
         .a           (ba),
         .d           (bd),
         .ema         ('1), 
         .emaw        ('1),
         .emas        ('1),
         .ret1n       (1'b1),
         .wabl        ('1),
         .wablm       ('0),
         .rawl        ('1),
         .rawlm       ('0)
         );
    end
    if(ramname=="HRAM")begin:genHRAM
       sce_hashram_3k  m (
         .clk         (clkb[i]),
         .q           (bq[i]),
         .cen         (bcen[i]),
         .wen         (bgwen[i]),
         .a           (ba),
         .d           (bd),
         .ema         ('1), 
         .emaw        ('1),
         .emas        ('1),
         .ret1n       (1'b1),
         .wabl        ('1),
         .wablm       ('0),
         .rawl        ('1),
         .rawlm       ('0)
         );
    end
    if(ramname=="SCERAM")begin:genSCERAM
       sce_sceram_10k  m (
         .CLK         (clkb[i]),
         .Q           (bq[i]),
         .CEN         (bcen[i]),
         .GWEN        (bgwen[i]),
         .A           (ba),
         .D           (bd),
         .EMA         ('1), 
         .EMAW        ('1),
         .EMAS        ('1),
         .RET1N       (1'b1),
         .STOV        ('0),
         .WABL        ('1),
         .WABLM       ('0),
         .RAWL        ('1),
         .RAWLM       ('0)
         );
    end

    ICG icg(.CK(clk),.EN(clkben[i]),.SE(cmsatpg),.CKG(clkb[i]));
    assign clkben[i] = ~bcen[i] ;
  end
endgenerate
/*
    sce_aesram_1k uram(
               .clk         (clk),
               .q           (ramm.ramrdata),
               .cen         (ramm.ramen),
               .gwen        (ramm.ramwr),
               .wen         (ramm.ramen),
               .a           (ramm.ramen),
               .d           (ramm.ramen),
               .ema         ('0), 
               .emaw        ('0),
               .emas        ('0),
               .ret1n       (1'b1),
               .wabl        ('1),
               .wablm       ('0),
               .rawl        ('0),
               .rawlm       ('0)
               );

    ICG icg(.CK(clk),.EN(clkben[i]),.SE(cmsatpg),.CKG(clkb[i]));
    assign clkben[i] = ~bcen[i] ;

    `ifdef SIM
        sceram_sim #(
            .AW ( thecfg.AW ),
            .DW ( thecfg.DW + thecfg.PW),
            .WCNT ( thecfg.WCNT )
        )m(
            .clk,
            .rams(ramm)
        );
    `endif
    `ifdef SYN

    `endif
 */
`else // ifdef FPGA
    assign ramready = '1;
    assign ramerror = '0;
generate
    if(ramname=="SCERAM") begin: gensceram
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( 8 )
        )br0(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[0]   ),
            .ramwr      ( ramwr[0]  ),
            .ramwdata   ( ramwdat[7:0] ),
            .ramrdata   ( ramrdat[7:0] ) 
        );
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( 8 )
        )br1(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[0]   ),
            .ramwr      ( ramwr[0]  ),
            .ramwdata   ( ramwdat[15:8] ),
            .ramrdata   ( ramrdat[15:8] ) 
        );
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( 8 )
        )br2(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[0]   ),
            .ramwr      ( ramwr[0]  ),
            .ramwdata   ( ramwdat[23:16] ),
            .ramrdata   ( ramrdat[23:16] ) 
        );
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( 8 )
        )br3(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[0]   ),
            .ramwr      ( ramwr[0]  ),
            .ramwdata   ( ramwdat[31:24] ),
            .ramrdata   ( ramrdat[31:24] ) 
        );
    end
    else if( ramname=="PRAM" ) begin: genpram
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( 32 )
        )br0(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[0]   ),
            .ramwr      ( ramwr[0]  ),
            .ramwdata   ( ramwdat[31:0] ),
            .ramrdata   ( ramrdat[31:0] ) 
        );
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( 32 )
        )br1(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[4] ),
            .ramwr      ( ramwr[4]  ),
            .ramwdata   ( ramwdat[63:32] ),
            .ramrdata   ( ramrdat[63:32] ) 
        );        
    end
    else begin: gen1bram
        bramsp #(
            .BS ( "36Kb" ),
            .AW ( thecfg.AW ),
            .DW ( thecfg.DW )
        )br(
            .clk        ( clk      ),
            .ramaddr    ( ramaddr  ),
            .ramcs      ( ramrd | ramwr[0] ),
            .ramwr      ( ramwr[0]  ),
            .ramwdata   ( ramwdat ),
            .ramrdata   ( ramrdat ) 
        );        
    end
endgenerate

`endif



endmodule

module sceram_sim #( 
    parameter AW = 10,
    parameter DW = 36,
    parameter WCNT = 2**AW

)(
    input   bit     clk,
    ramif.slave     rams
);

    bit [0:WCNT-1][DW-1:0]    ramdat;
    bit [DW-1:0] ramrdat;

generate
    for (genvar i = 0; i < DW/9; i++) begin: gg
        always@(posedge clk) if(rams.ramwr[i])  ramdat[rams.ramaddr][i*9+8:i*9] <= rams.ramwdata[i*9+8:i*9];
    end    
endgenerate
    always@(posedge clk) ramrdat <= ramdat[rams.ramaddr];
    assign rams.ramrdata = ramrdat;
    assign rams.ramready = '1;

endmodule : sceram_sim

module dummytb_sceram_sim();
    parameter AW = 10;
    parameter DW = 36;
    parameter WCNT = 2**AW;
    bit     clk;
    ramif #(.RAW(AW),.DW(DW))    rams();

sceram_sim u0(.*);

endmodule
