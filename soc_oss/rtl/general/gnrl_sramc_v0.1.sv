`include "template_v0.1.sv"

module gnrl_sramc
#(

	parameter AW = 8,
	parameter DW = 32,
	parameter KW = DW,
	parameter PW = DW/8,

)(

    input        clk,
    input        resetn,
    input		 scmben,
    input [KW-1:0] scmbkey,

    ramif.slave  ramslave,
    ramif.master rammaster
);

    localparam HEIGHT = AW**2;
    localparam DWW = $clog2(DW);
    localparam WRW = DW/8;

	localparam PM_parinitwordfsmcnt = 2;

//  initial parity

	`theregrn( parinitdone ) <= ( interaddr == HEIGHT - 1 ) & parinitworddone ? 1'b1 : parinitdone;
	`theregrn( parinitwordfsm ) <= ( ramslave.ramen | parinitworddone ) ? 0 : parinitwordfsm + 1;
	assign parinit = ~parinitdone;
	assign parinitworddone = ( parinitwordfsm == PM_parinitwordfsmcnt );

	`theregrn( interaddr ) <= parinit ? ( parinitworddone ? interaddr + 1 : parinitaddr ) :
							  extverf ? ( extverfworddone ? extverfaddnext : extverf )
	assign extverfworddone = ( extverfwordfsm == extverfwordfsmcnt );
extverfaddnext
extverfwordfsmcnt
extverf


	assign parinit_ramrd = parinit & ( parinitwordfsm == 1 );
	assign parinit_ramwr = parinit & ( parinitwordfsm == 2 );

	assign parinit_ramwdata = ##;



rammaster.ramen
rammaster.ramcs
ramaddr
ramwr
ramwdata


ramenc uramrdata( 
	.key	( scmbkey ),
	.addr 	( ramslave.ramaddr ),
	.din 	( ramslave.ramwdata ),
	.dout 	( ramwdata_enc  )
 );





ramrdata 

//


endmodule : gnrl_sramc


input SLP;
input SD;
input CLK;
input CEB;
input WEB;
input [14:0] A;
input [31:0] D;
input [31:0] BWEB;
