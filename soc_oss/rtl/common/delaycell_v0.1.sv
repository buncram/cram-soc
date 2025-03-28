`include "template.sv"

module delaycell
#(
    parameter DW = 32,
    parameter DH = 8
)
(
    input   bit clk,
    input   bit resetn,
    input   bit [DW-1:0]    datain,
    output  bit [DW-1:0]    dataout    
);
    bit [0:DH][DW-1:0]  datareg;
    
genvar i;
generate
	for( i = 0; i < DH; i = i + 1) begin: GenRnd
	    `thereg(datareg[i+1]) <= datareg[i];
	end
endgenerate

    `thereg(datareg[0]) <= datain;
    
    assign dataout = datareg[DH];
endmodule
