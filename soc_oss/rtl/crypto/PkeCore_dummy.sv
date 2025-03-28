
module PkeCore(
            Clk              ,
            Resetn           ,
            PkeIR            ,
            NLen             ,
            ELen             ,
            PkeStart         ,
            RamPkeDat0       ,
            RamPkeDat1       ,
            PkeInt           ,
            ModInvRdy	     ,

            N0Dat	     ,

            PkeRamRd0        ,
            PkeRamWr0        ,
            PkeRamAddr0      ,
            PkeRamDat0       ,
            PkeRamRd1        ,
            PkeRamWr1        ,
            PkeRamAddr1      ,
            PkeRamDat1       
          );

input  	        Clk;
input           Resetn;   
input  [7:0]    PkeIR;
input  [12:0]   NLen;
input  [12:0]   ELen;
input           PkeStart;
input  [63:0]   RamPkeDat0;
input  [63:0]   RamPkeDat1;
   
input  [63:0]   N0Dat;
output          PkeInt;         
output          ModInvRdy;         
output          PkeRamRd0;
output          PkeRamWr0;
output [7:0]    PkeRamAddr0;
output [63:0]   PkeRamDat0;
output          PkeRamRd1;
output          PkeRamWr1;
output [7:0]    PkeRamAddr1;
output [63:0]   PkeRamDat1;
    
wire 	        Clk;
wire            Resetn;         
wire   [7:0]    PkeIR;
wire            PkeStart;
wire   [63:0]   RamPkeDat0;
wire   [63:0]   RamPkeDat1;
    
wire            PkeInt;         
wire            ModInvRdy;         
wire            PkeRamRd0;
wire            PkeRamWr0;
wire   [7:0]    PkeRamAddr0;
wire   [63:0]   PkeRamDat0;
wire            PkeRamRd1;
wire            PkeRamWr1;
wire   [7:0]    PkeRamAddr1;
wire   [63:0]   PkeRamDat1;


assign PkeInt = '0;
assign ModInvRdy = '0;
assign PkeRamRd0 = '0;
assign PkeRamWr0 = '0;
assign PkeRamAddr0 = '0;
assign PkeRamDat0 = '0;
assign PkeRamRd1 = '0;
assign PkeRamWr1 = '0;
assign PkeRamAddr1 = '0;
assign PkeRamDat1 = '0;

endmodule
//-----------------------------End---------------------------------//

