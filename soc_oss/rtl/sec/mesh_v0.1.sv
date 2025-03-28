`include "template.sv"

module mesh #(
    parameter LC = 32,
    parameter PC = 32
)(

    input bit   clk,    
    input bit   resetn,
    input bit   cmsatpg,

    apbif.slavein   apbs,
    apbif.slave     apbx,
    output logic irq
);	


parameter GC = 8;
parameter GW = PC/GC;

	logic [LC-1:0] cr_mldrv;
	logic [LC-1:0] cr_mlie;
	logic [LC-1:0][GC-1:0] sr_mlsr;
	logic [LC-1:0] mldrv;
	logic [LC-1:0][PC-1:0] mlapt, apt, aptreg, apterr;

    logic sfrlock;
    logic apbrd, apbwr;
    logic pclk;
    assign pclk = clk;
    `theregrn( sfrlock ) <= '0;
    `apbs_common;
    assign apbx.prdata = '0
                        | sfr_mldrv.prdata32 
                        | sfr_mlie.prdata32 | sfr_mlsr.prdata32
                        ;

    apb_cr #(.A('h00),  	.DW(32), .REVY(1), .SFRCNT(LC/32))  		sfr_mldrv   (.cr(cr_mldrv), .prdata32(),.*);
    apb_cr #(.A('h00+LC/8), .DW(32), .REVY(1), .SFRCNT(LC/32))  		sfr_mlie    (.cr(cr_mlie),  .prdata32(),.*);
    apb_sr #(.A('h00+LC/4), .DW(32), .REVY(1), .SFRCNT(LC*GC/32))  		sfr_mlsr    (.sr(sr_mlsr),  .prdata32(),.*);


generate
	for (genvar i = 0; i < LC; i++) begin: gl
		meshlinedrv ud(.A(cr_mldrv[i]),.Z(mldrv[i]));
		meshline #(.PC(PC)) ml(.DRV(mldrv[i]),.APT(mlapt[i]));
		for (genvar j = 0; j < PC; j++) begin: gp
			meshlinebuf ua(.A(mlapt[i][j]),.IE(cr_mlie[i]),.Z(apt[i][j]));
			`theregrn( aptreg[i][j] ) <= apt[i][j];
			assign apterr[i][j] = cr_mlie[i] & ( aptreg[i][j] != cr_mldrv[i] );
		end
		for (genvar k = 0; k < GC; k++) begin: gg
			assign sr_mlsr[i][k] = |apterr[i][GW*k+GW-1:GW*k];
		end
	end
endgenerate

	`theregrn( irq ) <= | sr_mlsr;

endmodule


module meshlinedrv (
	input logic A,
	output logic Z
);

	`ifdef SYN
	`else
		assign Z = A;
	`endif

endmodule


module meshlinebuf (
	input logic A,
	input logic IE,
	output logic Z
);

	`ifdef SYN
	`else
		assign Z = A & IE;
	`endif

endmodule


module meshline #(
	parameter PC=32
)(
	input  logic 		DRV,
	output logic [31:0] APT
);

	`ifdef SYN
	`else

		always@(*)begin
			APT = 'X;
			APT[0] = #( 30 `US) DRV;
			APT[PC-1:1] = APT[PC-2:0];
		end

	`endif

endmodule

module dummytb_mesh ();

    parameter LC = 32;
    parameter PC = 32;

    bit   clk;
    bit   resetn;
    bit   cmsatpg;
    apbif   apbs();
    apbif   apbx();
    logic irq;
	mesh u(.*);

endmodule

