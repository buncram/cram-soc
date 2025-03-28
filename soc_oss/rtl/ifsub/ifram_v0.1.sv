`ifndef SIM
`include "rtl/model/artisan_ram_def_v0.1.svh"
`endif

module ifram #(
    parameter sram_pkg::sramcfg_t thecfg = '{
        AW: 17-2,
        DW: 32,
        KW: 32,
        PW: 4,
        WCNT: 2**(17-2),
        AWX: 5,
        isBWEN: '1,
        isSCMB: '1,
        isPRT:  '1,
        EVITVL:  15
    }
    )(

    input logic             clk,
    input logic             resetn,
    input logic             cmsatpg,
    input logic             cmsbist,
//    input logic             scmben,
    input logic [thecfg.KW-1:0] scmbkey,
    output logic            prerr,
    output logic            verifyerr,

    ramif.slave             rams
);

    localparam AW = thecfg.AW;
    localparam DW = thecfg.DW;
    localparam PW = thecfg.PW;
    localparam DW0 = DW+PW;
    localparam BW0 = 9;
    localparam BC  = DW0/BW0;
    localparam RCW = $clog2(1);
    localparam AW0 = AW-$clog2(1);

    ramif #(.RAW(AW),.DW(DW0),.BW(BW0)) ram0();
    logic                   rams_ramen      ;     
    logic                   rams_ramcs      ;     
    logic [AW-1:0]          rams_ramaddr    ;     
    logic [DW0/BW0-1:0]     rams_ramwr      ;     
    logic [DW0-1:0]         rams_ramwdata   ;     
    logic [DW0-1:0]         rams_ramrdata   ;
    logic                   rams_ramready   ;
    logic [RCW-1:0]         bsel, bselreg   ;
    logic [DW0-1:0]         bd              ;
    logic [DW0-1:0] bq              ;
    logic           bcen, bgwen     ;
    logic [BC-1:0][BW0-1:0]   bwen        ;
//    logic [RC-1:0][DW0/BW0-1:0] bwen        ;
    logic [AW-RCW-1:0]        ba              ;
    logic           clkb, clkben    ;

    gnrl_sramc #(.thecfg(thecfg))dut
    (
        .clk,
        .resetn,
        .cmsatpg,
        .cmsbist,
//        .scmben(thecfg.isSCMB),
        .scmben(1'b0),
        .scmbkey,
        .prerr,
        .verifyerr,
        .ramslave(rams),
        .rammaster(ram0) 
    );

    rams2wire
    #(
        .AW(AW),
        .DW(DW0),
        .BW(BW0)
    )r2w(
        .rams            (ram0            ),
        .rams_ramen      (rams_ramen      ),     
        .rams_ramcs      (rams_ramcs      ),     
        .rams_ramaddr    (rams_ramaddr    ),     
        .rams_ramwr      (rams_ramwr      ),     
        .rams_ramwdata   (rams_ramwdata   ),     
        .rams_ramrdata   (rams_ramrdata   ),
        .rams_ramready   (rams_ramready   )    
    );

    assign bsel = 0;// rams_ramaddr[AW-1:AW-RCW] ;
    `theregrn(bselreg) <= ( rams_ramcs & rams_ramready ) ? bsel : bselreg;
    assign rams_ramready = '1;
    assign rams_ramrdata = bq[bselreg];
    assign #0.5 bd = rams_ramwdata;
    assign #0.5 ba = rams_ramaddr;

    assign #0.5 bcen = ~rams_ramcs;
    assign #0.5 bgwen =  ~( |rams_ramwr & rams_ramcs ); // was this the typo?
//    assign #0.5 bwen[i]  =  rams_ramcs & (bsel==i) ? ~rams.ramwr : '1;

    ifram32kx36  m (
         .clk         (clkb),
         .q           (bq),
         .cen         (bcen),
         .gwen        (bgwen),
         .wen         (bwen),
         .a           (ba),
         .d           (bd),
        `sram_sp_uhde_inst
         );

    ICG icg(.CK(clk),.EN(clkben),.SE(cmsatpg),.CKG(clkb));
    assign clkben = ~bcen ;

    generate
    for (genvar gvj = 0; gvj < BC; gvj++) begin: genwe
        assign bwen[gvj] = rams_ramcs & rams_ramwr[gvj] ? '0 : '1;
    end
    endgenerate

endmodule

module dummytb_ifram();

    parameter RC = 1;
    parameter sram_pkg::sramcfg_t thecfg = '{
        AW: 17-2,
        DW: 32,
        KW: 32,
        PW: 4,
        WCNT: 2**(17-2),
        AWX: 5,
        isBWEN: '1,
        isSCMB: '1,
        isPRT:  '1,
        EVITVL:  15
    };

    logic             clk;
    logic             resetn;
    logic             cmsatpg;
    logic             cmsbist;
    logic [thecfg.KW-1:0] scmbkey;
    logic            prerr;
    logic            verifyerr;

    ramif #(.RAW(thecfg.AW),.DW(thecfg.DW)) rams();

    ifram #(RC,thecfg) u0(
    .clk           , 
    .resetn        , 
    .cmsatpg       , 
    .cmsbist       , 
    .scmbkey       , 
    .prerr         , 
    .verifyerr     , 
    .rams            
    );

endmodule
