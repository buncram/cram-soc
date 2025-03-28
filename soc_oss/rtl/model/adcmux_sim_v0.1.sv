
module mux_4to1 (
	input wire [3:0] VIN,
	input wire MSB,
	input wire LSB,
	output wire VOUT
);
`ifdef SIM

logic [1:0] sel;
assign sel = { MSB, LSB };
assign VOUT = VIN[sel];
`endif

endmodule