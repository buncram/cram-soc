
module  sram128X32C2V4_wrp(
   Q,
   CLK,
   CEN,
   WEB,
   A,
   D,
   OEN
);
   parameter BITS = 32;
   //zxjian,20220320,changed from 128*32bit to 256*32bit
   //parameter addr_width = 7;
   parameter addr_width = 8;

   output [BITS-1:0] Q;
   input CLK;
   input CEN;
   input [BITS-1:0] WEB;
   input [addr_width-1:0] A;
   input [BITS-1:0] D;
   input OEN;

`ifdef  FPGA
  sram128X32C2V4 PkeRam(
     .clka      ( CLK    ),
     .ena       ( ~CEN    ),
     .wea       ( {~WEB[0],~WEB[0],~WEB[0],~WEB[0]}),
     .addra     ( A      ),
     .dina      ( D      ),
     .douta     ( Q      ) 
  );

`else 
  sram128X32C2V4 PkeRam(
     .Q		( Q	),
     .CLK       ( CLK   ),
     .CEN       ( CEN   ),
     .WEB       ( WEB   ),
     .A		( A	),
     .D		( D	),
     .OEN       (1'b0   )
  );
`endif

endmodule
