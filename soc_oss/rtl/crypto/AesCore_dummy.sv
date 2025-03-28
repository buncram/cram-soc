
module AesCore(
        Clk,
		Resetn,
		StartAes,
		AesIR,
		AesLen,
		AesMode,

                IVector0 ,
                IVector1 ,
                IVector2 ,
                IVector3 ,
 
		
		AesRamRd,
		AesRamWr,
		AesRamAdr,
		AesRamDat,
		AesRamDat1,
		RamAesDat,

		AesDone);


input bit         Clk	;
input bit         Resetn	;
input bit         StartAes	;
input bit  [1:0]  AesIR	;
input bit  [1:0]  AesLen	;
input bit  [2:0]  AesMode	;

input bit[31:0] IVector0 ;
input bit[31:0] IVector1 ;
input bit[31:0] IVector2 ;
input bit[31:0] IVector3 ;

output bit        AesRamRd	;
output bit        AesRamWr	;
output bit [7:0]  AesRamAdr;
output bit [31:0] AesRamDat;
output bit [31:0] AesRamDat1;
input  bit [31:0] RamAesDat;
output bit        AesDone	;

assign AesRamRd = '0;
assign AesRamWr = '0;
assign AesRamAdr = '0;
assign AesRamDat = '0;
assign AesRamDat1 = '0;
assign AesDone = '0;


endmodule
