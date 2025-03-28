module pulp_clock_gating (
   input  logic clk_i,
   input  logic en_i,
   input  logic test_en_i,
   output logic clk_o
);

    ICG icg(.CK(clk_i),.EN(en_i),.SE(test_en_i),.CKG(clk_o));

endmodule