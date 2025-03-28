module coresub_sramtrm (
        apbif.slavein apbs,
        apbif.slave   apbx,
        input logic clk,
        input logic resetn,
        input logic [3:0] srambankerr,
        output logic [1:0] itcmwaitcyc,
        output logic [1:0] dtcmwaitcyc,
        output logic [1:0] sram0waitcyc,
        output logic [1:0] sram1waitcyc,
        output logic [2:0] cachesramtrm,
        output logic [2:0] itcmsramtrm,
        output logic [2:0] dtcmsramtrm,
        output logic [2:0] sram0sramtrm,
        output logic [2:0] sram1sramtrm,
        output logic [2:0] vexsramtrm
);

	logic pclk;
	assign pclk = clk;

    logic apbrd, apbwr;
    `apbs_common;
    logic sfrlock;
    assign sfrlock = '0;
    assign apbx.prdata = '0
                | sfr_cache.prdata32 |  sfr_itcm.prdata32 | sfr_dtcm.prdata32
                | sfr_sram0.prdata32 | sfr_sram1.prdata32
                | sfr_vexram.prdata32
                | sfr_sramerr.prdata32
                ;

	logic [2:0] cr_cache, cr_vexram;
	logic [4:0] cr_itcm ;
	logic [4:0] cr_dtcm ;
	logic [4:0] cr_sram0;
	logic [4:0] cr_sram1;

	apb_cr #(.A('h00), .DW(3), .IV('h4))  sfr_cache   (.cr(cr_cache  ),   .prdata32(),.*);
	apb_cr #(.A('h04), .DW(5), .IV('h4))  sfr_itcm    (.cr(cr_itcm   ),   .prdata32(),.*);
	apb_cr #(.A('h08), .DW(5), .IV('h4))  sfr_dtcm    (.cr(cr_dtcm   ),   .prdata32(),.*);
	apb_cr #(.A('h0C), .DW(5), .IV('h2))  sfr_sram0   (.cr(cr_sram0  ),   .prdata32(),.*);
	apb_cr #(.A('h10), .DW(5), .IV('h4))  sfr_sram1   (.cr(cr_sram1  ),   .prdata32(),.*);
	apb_cr #(.A('h14), .DW(3), .IV('h4))  sfr_vexram  (.cr(cr_vexram ),   .prdata32(),.*);
	apb_fr #(.A('h20), .DW(4))            sfr_sramerr (.fr(srambankerr ),   .prdata32(),.*);

	assign { cachesramtrm } = cr_cache;
	assign { itcmwaitcyc,itcmsramtrm } = cr_itcm;
	assign { dtcmwaitcyc,dtcmsramtrm } = cr_dtcm;
	assign { sram0waitcyc,sram0sramtrm } = cr_sram0;
	assign { sram1waitcyc,sram1sramtrm } = cr_sram1;
	assign { vexsramtrm } = cr_vexram;

endmodule : coresub_sramtrm
