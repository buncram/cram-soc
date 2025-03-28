module ip_lightdet(
	input  wire analog_test_only,
	input  wire d2a_clk,
	input  wire d2a_self_test_en,
	output wire light_out
);

`ifndef SYN

	bit ldreg;
	assign light_out = ldreg | d2a_self_test_en;

`endif


endmodule
