`ifndef _RAMBIST_INTERFACE_DEFINE

interface rbif #(
    parameter AW=14,
    parameter DW=32,
    parameter TW=16
)();

    wire                ramclk      ;
    wire                ramcen      ;
    wire  [AW-1:0]      ramaddr     ;
    wire  [DW-1:0]      ramwen      ;
    wire                ramgwen     ;
    wire  [DW-1:0]      ramwdata    ;
    wire  [DW-1:0]      ramrdata    ;

    wire                ramclkb     ;
    wire                ramcenb     ;
    wire  [AW-1:0]      ramaddrb    ;
    wire  [DW-1:0]      ramwenb     ;
    wire                ramgwenb    ;
    wire  [DW-1:0]      ramwdatab   ;
    wire  [DW-1:0]      ramrdatab   ;

    wire  [TW-1:0]      ramtrm;

  modport slave (
    input   ramclk      ,
    input   ramcen      ,
    input   ramaddr     ,
    input   ramwen      ,
    input   ramgwen     ,
    input   ramwdata    ,
    output  ramrdata    ,
    input   ramtrm
    );

  modport master (
    output  ramclk      ,
    output  ramcen      ,
    output  ramaddr     ,
    output  ramwen      ,
    output  ramgwen     ,
    output  ramwdata    ,
    input   ramrdata    ,
    output  ramtrm
    );

  modport slavedp (
    input   ramclk      ,
    input   ramcen      ,
    input   ramaddr     ,
    input   ramwen      ,
    input   ramgwen     ,
    input   ramwdata    ,
    output  ramrdata    ,

    input   ramclkb     ,
    input   ramcenb     ,
    input   ramaddrb    ,
    input   ramwenb     ,
    input   ramgwenb    ,
    input   ramwdatab   ,
    output  ramrdatab   ,

    input   ramtrm
    );

  modport masterdp (
    output  ramclk      ,
    output  ramcen      ,
    output  ramaddr     ,
    output  ramwen      ,
    output  ramgwen     ,
    output  ramwdata    ,
    input   ramrdata    ,

    output  ramclkb     ,
    output  ramcenb     ,
    output  ramaddrb    ,
    output  ramwenb     ,
    output  ramgwenb    ,
    output  ramwdatab   ,
    input   ramrdatab   ,

    output  ramtrm
    );

endinterface

module rbifs2wire
#(
    parameter AW=14,
    parameter DW=32
)(
    rbif.slave                  rbifs           ,
    output logic                rbifs_ramclk    ,
    output logic                rbifs_ramcen    ,
    output logic                rbifs_ramgwen   ,
    output logic [AW-1:0]       rbifs_ramaddr   ,
    output logic [DW-1:0]       rbifs_ramwen    ,
    output logic [DW-1:0]       rbifs_ramwdata  ,
    input  logic [DW-1:0]       rbifs_ramrdata
);

    assign rbifs_ramclk   = rbifs.ramclk   ;
    assign rbifs_ramcen   = rbifs.ramcen   ;
    assign rbifs_ramgwen  = rbifs.ramgwen  ;
    assign rbifs_ramaddr  = rbifs.ramaddr  ;
    assign rbifs_ramwen   = rbifs.ramwen   ;
    assign rbifs_ramwdata = rbifs.ramwdata ;
    assign rbifs.ramrdata = rbifs_ramrdata ;

endmodule : rbifs2wire

module wire2rbifm
#(
    parameter AW=14,
    parameter DW=32
)(
    input  logic                rbifm_ramclk    ,
    input  logic                rbifm_ramcen    ,
    input  logic                rbifm_ramgwen   ,
    input  logic [AW-1:0]       rbifm_ramaddr   ,
    input  logic [DW-1:0]       rbifm_ramwen    ,
    input  logic [DW-1:0]       rbifm_ramwdata  ,
    output logic [DW-1:0]       rbifm_ramrdata  ,
    rbif.master                 rbifm
);

    assign rbifm.ramclk   = rbifm_ramclk    ;
    assign rbifm.ramcen   = rbifm_ramcen    ;
    assign rbifm.ramgwen  = rbifm_ramgwen   ;
    assign rbifm.ramaddr  = rbifm_ramaddr   ;
    assign rbifm.ramwen   = rbifm_ramwen     ;
    assign rbifm.ramwdata = rbifm_ramwdata  ;
    assign rbifm_ramrdata = rbifm.ramrdata  ;

endmodule : wire2rbifm

module rbifs2wiredp
#(
    parameter AW=14,
    parameter DW=32
)(
    rbif.slavedp                rbifsdp           ,
    output logic                rbifsdp_ramclk    ,
    output logic                rbifsdp_ramcen    ,
    output logic                rbifsdp_ramgwen   ,
    output logic [AW-1:0]       rbifsdp_ramaddr   ,
    output logic [DW-1:0]       rbifsdp_ramwen    ,
    output logic [DW-1:0]       rbifsdp_ramwdata  ,
    input  logic [DW-1:0]       rbifsdp_ramrdata  ,

    output logic                rbifsdp_ramclkb    ,
    output logic                rbifsdp_ramcenb    ,
    output logic                rbifsdp_ramgwenb   ,
    output logic [AW-1:0]       rbifsdp_ramaddrb   ,
    output logic [DW-1:0]       rbifsdp_ramwenb    ,
    output logic [DW-1:0]       rbifsdp_ramwdatab  ,
    input  logic [DW-1:0]       rbifsdp_ramrdatab
);

    assign rbifsdp_ramclk   = rbifsdp.ramclk   ;
    assign rbifsdp_ramcen   = rbifsdp.ramcen   ;
    assign rbifsdp_ramgwen  = rbifsdp.ramgwen  ;
    assign rbifsdp_ramaddr  = rbifsdp.ramaddr  ;
    assign rbifsdp_ramwen   = rbifsdp.ramwen   ;
    assign rbifsdp_ramwdata = rbifsdp.ramwdata ;
    assign rbifsdp.ramrdata = rbifsdp_ramrdata ;

    assign rbifsdp_ramclkb   = rbifsdp.ramclkb   ;
    assign rbifsdp_ramcenb   = rbifsdp.ramcenb   ;
    assign rbifsdp_ramgwenb  = rbifsdp.ramgwenb  ;
    assign rbifsdp_ramaddrb  = rbifsdp.ramaddrb  ;
    assign rbifsdp_ramwenb   = rbifsdp.ramwenb   ;
    assign rbifsdp_ramwdatab = rbifsdp.ramwdatab ;
    assign rbifsdp.ramrdatab = rbifsdp_ramrdatab ;

endmodule : rbifs2wiredp

module wire2rbifmdp
#(
    parameter AW=14,
    parameter DW=32
)(
    input  logic                rbifmdp_ramclk    ,
    input  logic                rbifmdp_ramcen    ,
    input  logic                rbifmdp_ramgwen   ,
    input  logic [AW-1:0]       rbifmdp_ramaddr   ,
    input  logic [DW-1:0]       rbifmdp_ramwen    ,
    input  logic [DW-1:0]       rbifmdp_ramwdata  ,
    output logic [DW-1:0]       rbifmdp_ramrdata  ,

    input  logic                rbifmdp_ramclkb    ,
    input  logic                rbifmdp_ramcenb    ,
    input  logic                rbifmdp_ramgwenb   ,
    input  logic [AW-1:0]       rbifmdp_ramaddrb   ,
    input  logic [DW-1:0]       rbifmdp_ramwenb    ,
    input  logic [DW-1:0]       rbifmdp_ramwdatab  ,
    output logic [DW-1:0]       rbifmdp_ramrdatab  ,

    rbif.masterdp               rbifmdp
);

    assign rbifmdp.ramclk   = rbifmdp_ramclk    ;
    assign rbifmdp.ramcen   = rbifmdp_ramcen    ;
    assign rbifmdp.ramgwen  = rbifmdp_ramgwen   ;
    assign rbifmdp.ramaddr  = rbifmdp_ramaddr   ;
    assign rbifmdp.ramwen   = rbifmdp_ramwen    ;
    assign rbifmdp.ramwdata = rbifmdp_ramwdata  ;
    assign rbifmdp_ramrdata = rbifmdp.ramrdata  ;

    assign rbifmdp.ramclkb   = rbifmdp_ramclkb    ;
    assign rbifmdp.ramcenb   = rbifmdp_ramcenb    ;
    assign rbifmdp.ramgwenb  = rbifmdp_ramgwenb   ;
    assign rbifmdp.ramaddrb  = rbifmdp_ramaddrb   ;
    assign rbifmdp.ramwenb   = rbifmdp_ramwenb    ;
    assign rbifmdp.ramwdatab = rbifmdp_ramwdatab  ;
    assign rbifmdp_ramrdatab = rbifmdp.ramrdatab  ;

endmodule : wire2rbifmdp

module __dummytb_rambistif#(
    parameter AW=14,
    parameter DW=32
)();

    bit                 rbifm_ramclk   , rbifs_ramclk   ;
    bit                 rbifm_ramcen   , rbifs_ramcen   ;
    bit                 rbifm_ramgwen  , rbifs_ramgwen  ;
    bit   [AW-1:0]      rbifm_ramaddr  , rbifs_ramaddr  ;
    bit   [DW-1:0]      rbifm_ramwen   , rbifs_ramwen   ;
    bit   [DW-1:0]      rbifm_ramwdata , rbifs_ramwdata ;
    bit   [DW-1:0]      rbifm_ramrdata , rbifs_ramrdata ;
    bit                 rbifm_ramready , rbifs_ramready ;

    bit                 rbifmdp_ramclk   , rbifsdp_ramclk   ;
    bit                 rbifmdp_ramcen   , rbifsdp_ramcen   ;
    bit                 rbifmdp_ramgwen  , rbifsdp_ramgwen  ;
    bit   [AW-1:0]      rbifmdp_ramaddr  , rbifsdp_ramaddr  ;
    bit   [DW-1:0]      rbifmdp_ramwen   , rbifsdp_ramwen   ;
    bit   [DW-1:0]      rbifmdp_ramwdata , rbifsdp_ramwdata ;
    bit   [DW-1:0]      rbifmdp_ramrdata , rbifsdp_ramrdata ;
    bit                 rbifmdp_ramready , rbifsdp_ramready ;

    bit                 rbifmdp_ramclkb   , rbifsdp_ramclkb   ;
    bit                 rbifmdp_ramcenb   , rbifsdp_ramcenb   ;
    bit                 rbifmdp_ramgwenb  , rbifsdp_ramgwenb  ;
    bit   [AW-1:0]      rbifmdp_ramaddrb  , rbifsdp_ramaddrb  ;
    bit   [DW-1:0]      rbifmdp_ramwenb   , rbifsdp_ramwenb   ;
    bit   [DW-1:0]      rbifmdp_ramwdatab , rbifsdp_ramwdatab ;
    bit   [DW-1:0]      rbifmdp_ramrdatab , rbifsdp_ramrdatab ;
    bit                 rbifmdp_ramreadyb , rbifsdp_ramreadyb ;

    rbif                therbif();
    rbif                therbif_dp();

    wire2rbifm #(.AW(AW),.DW(DW)) u0(.rbifm(therbif),.*);
    rbifs2wire #(.AW(AW),.DW(DW)) u1(.rbifs(therbif),.*);

    wire2rbifmdp #(.AW(AW),.DW(DW)) u2(.rbifmdp(therbif_dp),.*);
    rbifs2wiredp #(.AW(AW),.DW(DW)) u3(.rbifsdp(therbif_dp),.*);



    logic cmsatpg;
    logic cmsbist;
    rbif #(.AW(AW),.DW(DW))rbs(), rbdps()     ;
    logic           clk     ,clka     ,clkb     ;
    logic [DW-1:0]  q       ,qa       ,qb       ;
    logic           cen     ,cena     ,cenb     ;
    logic           gwen    ,gwena    ,gwenb    ;
    logic [DW-1:0]  wen     ,wena     ,wenb     ;
    logic [AW-1:0]  a       ,aa       ,ab       ;
    logic [DW-1:0]  d       ,da       ,db       ;
    logic           rb_clk  ,rb_clka  ,rb_clkb  ;
    logic [DW-1:0]  rb_q    ,rb_qa    ,rb_qb    ;
    logic           rb_cen  ,rb_cena  ,rb_cenb  ;
    logic           rb_gwen ,rb_gwena ,rb_gwenb ;
    logic [DW-1:0]  rb_wen  ,rb_wena  ,rb_wenb  ;
    logic [AW-1:0]  rb_a    ,rb_aa    ,rb_ab    ;
    logic [DW-1:0]  rb_d    ,rb_da    ,rb_db    ;

    rbspmux #(.AW(AW),.DW(DW)) um0(.*);
    rbdpmux #(.AW(AW),.DW(DW)) um1(.rbs(rbdps),.*);

endmodule


module rbspmux#(
    parameter AW=14,
    parameter DW=32
) (
         input logic cmsatpg,
         input logic cmsbist,
         rbif.slave rbs     ,
         input  logic           clk     ,
         output logic [DW-1:0]  q       ,
         input  logic           cen     ,
         input  logic           gwen    ,
         input  logic [DW-1:0]  wen     ,
         input  logic [AW-1:0]  a       ,
         input  logic [DW-1:0]  d       ,
         output logic           rb_clk  ,
         input  logic [DW-1:0]  rb_q    ,
         output logic           rb_cen  ,
         output logic           rb_gwen ,
         output logic [DW-1:0]  rb_wen  ,
         output logic [AW-1:0]  rb_a    ,
         output logic [DW-1:0]  rb_d
   );

    CLKCELL_MUX2 c1 (.A(clk), .B(rbs.ramclk), .S(cmsbist), .Z(rb_clk));

    assign rb_cen  = cmsatpg ? '1 : cmsbist ? rbs.ramcen   : cen  ;
    assign rb_gwen = cmsatpg ? '1 : cmsbist ? rbs.ramgwen  : gwen ;
    assign rb_wen  = cmsatpg ? '1 : cmsbist ? rbs.ramwen   : wen  ;
    assign rb_a    = cmsatpg ? '1 : cmsbist ? rbs.ramaddr  : a    ;
    assign rb_d    = cmsatpg ? '1 : cmsbist ? rbs.ramwdata : d    ;

    assign q       = cmsatpg ? ( rb_q ^ wen ^ d ) : rb_q;
    assign rbs.ramrdata = q;

endmodule

module rbdpmux#(
    parameter AW=14,
    parameter DW=32
) (
         input logic cmsatpg,
         input logic cmsbist,
         rbif.slavedp rbs     ,
         input  logic           clka     ,clkb     ,
         output logic [DW-1:0]  qa       ,qb       ,
         input  logic           cena     ,cenb     ,
         input  logic           gwena    ,gwenb    ,
         input  logic [DW-1:0]  wena     ,wenb     ,
         input  logic [AW-1:0]  aa       ,ab       ,
         input  logic [DW-1:0]  da       ,db       ,
         output logic           rb_clka  ,rb_clkb  ,
         input  logic [DW-1:0]  rb_qa    ,rb_qb    ,
         output logic           rb_cena  ,rb_cenb  ,
         output logic           rb_gwena ,rb_gwenb ,
         output logic [DW-1:0]  rb_wena  ,rb_wenb  ,
         output logic [AW-1:0]  rb_aa    ,rb_ab    ,
         output logic [DW-1:0]  rb_da    ,rb_db
   );

    CLKCELL_MUX2 c1a (.A(rbs.ramclk),  .B(clka), .S(cmsbist), .Z(rb_clka));
    CLKCELL_MUX2 c1b (.A(rbs.ramclkb), .B(clkb), .S(cmsbist), .Z(rb_clkb));

    assign rb_cena  = cmsatpg ? '1 : cmsbist ? rbs.ramcen   : rb_cena  ;
    assign rb_gwena = cmsatpg ? '1 : cmsbist ? rbs.ramgwen  : rb_gwena ;
    assign rb_wena  = cmsatpg ? '1 : cmsbist ? rbs.ramwen   : rb_wena  ;
    assign rb_aa    = cmsatpg ? '1 : cmsbist ? rbs.ramaddr  : rb_aa    ;
    assign rb_da    = cmsatpg ? '1 : cmsbist ? rbs.ramwdata : rb_da    ;
    assign qa       = cmsatpg ? ( rb_qa ^ wena ^ da ) : rb_qa;
    assign rbs.ramrdata = qa;

    assign rb_cenb  = cmsatpg ? '1 : cmsbist ? rbs.ramcenb   : rb_cenb  ;
    assign rb_gwenb = cmsatpg ? '1 : cmsbist ? rbs.ramgwenb  : rb_gwenb ;
    assign rb_wenb  = cmsatpg ? '1 : cmsbist ? rbs.ramwenb   : rb_wenb  ;
    assign rb_ab    = cmsatpg ? '1 : cmsbist ? rbs.ramaddrb  : rb_ab    ;
    assign rb_db    = cmsatpg ? '1 : cmsbist ? rbs.ramwdatab : rb_db    ;
    assign qb       = cmsatpg ? ( rb_qb ^ wenb ^ db ) : rb_qb;
    assign rbs.ramrdatab = qb;

endmodule



`endif //`ifndef _RAMBIST_INTERFACE_DEFINE

`define _RAMBIST_INTERFACE_DEFINE
