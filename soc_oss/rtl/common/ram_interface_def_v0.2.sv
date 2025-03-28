`ifndef _RAM_INTERFACE_DEFINE

interface ramif #(
    parameter RAW=14,
    parameter DW=32
)();

    wire            ramen       ;
    wire            ramcs       ;
    wire  [RAW-1:0]  ramaddr     ;
    wire  [DW/8-1:0]          ramwr       ;
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

`endif //`ifndef _INTERFACE_DEFINE

`define _RAM_INTERFACE_DEFINE

