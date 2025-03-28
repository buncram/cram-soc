
module sce_sec #(
    parameter COREUSERCNT = 8,
    parameter type coreuser_t = bit[0:COREUSERCNT-1]
)(
  	input 	bit 					clk,
  	input 	bit 					resetn,
  	input 	bit [1:0] 				scemode,
  	input 	coreuser_t	coreuser,
  	output 	coreuser_t	sceuser,
  	output  bit mode_non,
  	output  bit mode_xls,
  	output  bit mode_sec,

  	output  bit ahbs_lock,

  	input 	bit ar_reset,
  	input 	bit ar_clrram,
  	output  bit sceresetnin,
  	output  bit sceramclr

);

    assign mode_non = ( scemode == 0 );
    assign mode_xls = ( scemode == 1 );
    assign mode_sec = ( scemode[1] == 1 );

    bit [1:0] scemodereg;
    bit sceuserlock, modequit;
    bit [3:0] initregs;

    assign sceuserlock = ( scemodereg == 0 ) && ~( scemode == scemodereg ) ;
    `theregrn( scemodereg ) <= scemode;
    `theregrn( sceuser ) <= sceuserlock ? coreuser : sceuser;

	assign ahbs_lock = ( mode_xls || mode_sec ) && ~( coreuser == sceuser );



    assign modequit = ~( scemodereg == 0 ) && ~( scemode == scemodereg ) ;

	assign sceresetnin = ~( modequit | ar_reset );

	`theregrn( initregs ) <= { initregs, 1'b1 };

	assign sceramclr = ( initregs == 4'h7 ) | ar_clrram ;

endmodule
