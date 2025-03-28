`ifndef _RAM_INTERFACE_DEFINE

interface ramif #(
    parameter RAW=14,
    parameter BW=8,
    parameter DW=32
)();

    wire            ramen       ;
    wire            ramcs       ;
    wire  [RAW-1:0]  ramaddr     ;
    wire  [DW/BW-1:0]          ramwr       ;
    wire  [DW-1:0]  ramwdata    ;
    wire  [DW-1:0]  ramrdata    ;
    wire            ramready    ;

  modport slave ( 
    input  ramen      ,     
    input  ramcs      ,     
    input  ramaddr    ,     
    input  ramwr      ,     
    input  ramwdata   ,     
    output ramrdata   ,
    output ramready        
    );
 
  modport master ( 
    output ramen      ,     
    output ramcs      ,     
    output ramaddr    ,     
    output ramwr      ,     
    output ramwdata   ,     
    input  ramrdata   ,
    input  ramready    
    );

endinterface

module rams2wire
#(
    parameter AW=14,
    parameter DW=32,
    parameter BW=8
)(
    ramif.slave                  rams            ,
    output logic                 rams_ramen      ,     
    output logic                 rams_ramcs      ,     
    output logic [AW-1:0] rams_ramaddr    ,     
    output logic [DW/BW-1:0]               rams_ramwr      ,     
    output logic [DW-1:0] rams_ramwdata   ,     
    input  logic [DW-1:0] rams_ramrdata   ,
    input  logic                 rams_ramready       
);

    assign rams_ramen    = rams.ramen    ;
    assign rams_ramcs    = rams.ramcs    ;
    assign rams_ramaddr  = rams.ramaddr  ;
    assign rams_ramwr    = rams.ramwr    ;
    assign rams_ramwdata = rams.ramwdata ;
    assign rams.ramrdata = rams_ramrdata ;
    assign rams.ramready = rams_ramready ;

endmodule : rams2wire

module wire2ramm
#(
    parameter AW=14,
    parameter DW=32,
    parameter BW=8
)(
    input  logic                  ramm_ramen      ,     
    input  logic                  ramm_ramcs      ,     
    input  logic [AW-1:0]  ramm_ramaddr    ,     
    input  logic [DW/BW-1:0]                 ramm_ramwr      ,     
    input  logic [DW-1:0]  ramm_ramwdata   ,     
    output logic [DW-1:0]  ramm_ramrdata   ,
    output logic                  ramm_ramready   ,
    ramif.master                  ramm
);

    assign ramm.ramen    = ramm_ramen    ;
    assign ramm.ramcs    = ramm_ramcs    ;
    assign ramm.ramaddr  = ramm_ramaddr  ;
    assign ramm.ramwr    = ramm_ramwr    ;
    assign ramm.ramwdata = ramm_ramwdata ;
    assign ramm_ramrdata = ramm.ramrdata ;
    assign ramm_ramready = ramm.ramready ;

endmodule : wire2ramm

module __dummytb_ramif#(
    parameter AW=14,
    parameter DW=32
)();

//    parameter sram_pkg::sramcfg_t thecfg=sram_pkg::samplecfg;

    bit                    ramm_ramen    , rams_ramen    ;
    bit                    ramm_ramcs    , rams_ramcs    ;
    bit   [AW-1:0]  ramm_ramaddr  , rams_ramaddr  ;
    bit   [DW/8-1:0]                 ramm_ramwr    , rams_ramwr    ;
    bit   [DW-1:0]  ramm_ramwdata , rams_ramwdata ;
    bit   [DW-1:0]  ramm_ramrdata , rams_ramrdata ;
    bit                    ramm_ramready , rams_ramready ;
    ramif                  theramif()      ;

    wire2ramm #(.AW(AW),.DW(DW)) u0(.ramm(theramif),.*);
    rams2wire #(.AW(AW),.DW(DW)) u1(.rams(theramif),.*);

endmodule

`endif //`ifndef _INTERFACE_DEFINE

`define _RAM_INTERFACE_DEFINE

