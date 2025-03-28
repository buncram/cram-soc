module core_srambank #(
    parameter RC = 4,
    parameter sram_pkg::sramcfg_t thecfg = {
        AW: 20-3,
        DW: 64,
        KW: 64,
        PW: 8,
        WCNT: 2**(20-3),
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
    localparam RCW = $clog2(RC);
    localparam AW0 = AW-$clog2(RC);

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
    logic [RC-1:0][DW0-1:0] bq              ;
    logic [RC-1:0]          bcen, bgwen     ;
    logic [RC-1:0][BC-1:0][BW0-1:0]   bwen        ;
//    logic [RC-1:0][DW0/BW0-1:0] bwen        ;
    logic [AW-RCW-1:0]        ba              ;
    logic [RC-1:0]          clkb, clkben    ;
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

    assign bsel = rams_ramaddr[AW-1:AW-RCW] ;
    `theregrn(bselreg) <= ( rams_ramcs & rams_ramready ) ? bsel : bselreg;
    assign rams_ramready = '1;
    assign rams_ramrdata = bq[bselreg];
    assign #0.5 bd = rams_ramwdata;
    assign #0.5 ba = rams_ramaddr;
generate
    for (genvar i = 0; i < RC; i++) begin: genram

    assign #0.5 bcen[i] = ~( rams_ramcs & (bsel==i) );
    assign #0.5 bgwen[i] =  ~( |rams.ramwr & rams_ramcs & (bsel==i) );
//    assign #0.5 bwen[i]  =  rams_ramcs & (bsel==i) ? ~rams.ramwr : '1;

    ram32kx72  m (
         .clk         (clkb[i]),
         .q           (bq[i]),
         .cen         (bcen[i]),
         .gwen        (bgwen[i]),
         .wen         (bwen[i]),
         .a           (ba),
         .d           (bd),
         .ema         ('0), 
         .emaw        ('0),
         .emas        ('0),
         .ret1n       (1'b1),
         .stov        ('0),
         .wabl        ('1),
         .wablm       ('0),
         .rawl        ('1),
         .rawlm       ('0)
         );

    ICG icg(.CK(clk),.EN(clkben[i]),.SE(cmsatpg),.CKG(clkb[i]));
    assign clkben[i] = ~bcen[i] ;

    for (genvar gvj = 0; gvj < BC; gvj++) begin: genwe
        assign bwen[i][gvj] = rams_ramcs & (bsel==i) & rams_ramwr[gvj] ? '0 : '1;
    end

    end
endgenerate

endmodule

module dummytb_srambank();

    parameter RC = 4;
    parameter sram_pkg::sramcfg_t thecfg = {
        AW: 20-3,
        DW: 64,
        KW: 64,
        PW: 8,
        WCNT: 2**(20-3),
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

    core_srambank #(RC,thecfg) u0(
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
