`include "template.sv"

module rtc_intf (
	input logic 	clk,
	input logic 	clk32k,
	input logic 	resetn,
	input logic 	cmsatpg,
    apbif.slave     apbs,
    output logic 	irq
);
`ifdef SIM
logic [3:0] clk1hzcnt;
`else
logic [13:0] clk1hzcnt;
`endif

`theregrn( clk1hzcnt ) <= clk1hzcnt + 1;
`theregrn( clk1hz ) <= clk1hz ^ ( clk1hzcnt == '1 );

Rtc u(
	/*       */ .PCLK           (clk              ),            // APB clock
	/*       */ .CLK1HZ         (clk1hz           ),            // 1 HZ clock
	/*       */ .nRTCRST        (resetn           ),            // RTC reset signal
	/*       */ .nPOR           (resetn           ),            // RTC power on reset
	/*       */ .PRESETn        (resetn           ),            // APB reset
	/*       */ .PSEL           (apbs.psel        ),            // APB select
	/*       */ .PENABLE        (apbs.penable     ),            // APB enable
	/*       */ .PWRITE         (apbs.pwrite      ),            // APB write
	/*[11:2] */ .PADDR          (apbs.paddr[11:2] ),            // APB Address
	/*[31:0] */ .PWDATA         (apbs.pwdata      ),            // APB write data
	/* [31:0]*/ .PRDATA         (apbs.prdata      ),            // APB  read data
	/*       */ .SCANENABLE     (cmsatpg 		  ),            // Test mode enable
	/*       */ .SCANINPCLK     (clk 			  ),            // PCLK Scan chain input
	/*       */ .SCANINCLK1HZ   (clk 			  ),            // CLK1HZ Scan chain input
	/*       */ .SCANOUTPCLK    (				  ),            // PCLK Scan chain output
	/*       */ .SCANOUTCLK1HZ  ( 				  ),            // CLK1HZ Scan chain output
	/*       */ .RTCINTR        (irq              )             // RTC interrupt
);

endmodule

