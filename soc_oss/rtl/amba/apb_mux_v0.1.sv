`include "template.sv"

module apb_mux #(
  parameter DW=32,
  parameter PAW=4
 )
 (
    input   bit [PAW-1:0]           paddrdec,
    input   bit                     pselin,

    input   bit [0:2**PAW-1]        preadyin,
    input   bit [0:2**PAW-1]        pslverrin,
    input   bit [0:2**PAW-1][DW-1:0]prdatain,

    output  bit [0:2**PAW-1]        psel,
    output  bit                     pready,
    output  bit                     pslverr,
    output  bit [DW-1:0]            prdata
);
 

    localparam SLVCNT = 2**PAW;
    bit [0:SLVCNT] preadyall;

    genvar i;
    generate
    	for( i = 0; i < SLVCNT; i = i + 1) begin: GenRnd
        assign psel[i] =  pselin & ( paddrdec == i );
    	end
    endgenerate

    assign pready = pselin ? |preadyall : 1'b1;
    assign preadyall = preadyin & psel;
    assign pslverr = |( psel & pslverrin );

    function bit [DW-1:0] fnprdata(input bit [0:SLVCNT-1][DW-1:0]    fnprdatain, input bit [0:SLVCNT-1] fnpsel);
        bit [DW-1:0] fnmux;
        int fni;
        for(fni = 0; fni < SLVCNT; fni = fni + 1 ) fnmux = fnmux | ( fnprdatain[fni] & {DW{fnpsel[fni]}} );
        fnprdata = fnmux;
    endfunction
 
 
 endmodule
 
