`include "template.sv"

module gluechain #(
	parameter GCX = 32,
	parameter GCY = 32
)(

    input bit   clk,
    input bit   clksys,
    input bit   resetn,
    input bit   cmsatpg,

    apbif.slavein   apbs,
    apbif.slave     apbx,
    output logic irq
);


	logic [GCX-1:0][GCY:0] gluenet;
	logic [GCX-1:0] gluereg, cr_gcmask, gluerst, gluetest, glueresetn ;


    logic sfrlock;
    logic apbrd, apbwr;
    logic pclk;
    assign pclk = clk;
    `theregrn( sfrlock ) <= '0;
    `apbs_common;
    assign apbx.prdata = '0
                        | sfr_gcmask.prdata32 | sfr_gcsr.prdata32 | sfr_gcrst.prdata32 | sfr_gctest.prdata32
                        ;

    apb_cr #(.A('h00), .DW(GCX) )  		sfr_gcmask    (.cr(cr_gcmask), .prdata32(),.*);
    apb_sr #(.A('h04), .DW(GCX) )  		sfr_gcsr      (.sr(gluereg),   .prdata32(),.*);
    apb_cr #(.A('h08), .DW(GCX) )       sfr_gcrst     (.cr(gluerst),   .prdata32(),.*);
    apb_cr #(.A('h0C), .DW(GCX) )       sfr_gctest    (.cr(gluetest),  .prdata32(),.*);

	generate
	for (genvar i = 0; i < GCX; i++) begin: gx
		assign gluenet[i][0] = gluetest[i];
        `theregfull( clksys, resetn, gluereg[i], '0 ) <= gluenet[i][GCY] & ~cr_gcmask[i];
        assign glueresetn[i] = ~gluerst[i] & resetn;
		for (genvar j = 0; j < GCY; j++) begin: gy
			ip_gluecell u(.ana_test_use_only(),.d2a_nrst( glueresetn[i]), .d2a_glue_in(gluenet[i][j]), .a2d_glue_out(gluenet[i][j+1]));
		end
	end
	endgenerate

	`theregrn( irq ) <= |gluereg;


endmodule

module dummytb_gc ();
    parameter VDC = 8;
    parameter LDC = 4;
    bit   clk;
    bit   clksys;
    bit   resetn;
    bit   cmsatpg;
    apbif   apbs();
    apbif   apbx();
    logic irq;

    gluechain u(.*);

endmodule

