`include "template.sv"

module sys_top_tb ();

    bit clk,resetn;
    integer i=0, j=0, k=0, errcnt=0, warncnt=0;

    bit [0:2]   cmspad;
    bit         padresetn;
    bit         clkswd;
    bit         dbgtxd;

  //
  //  dut
  //  ==

    ioif iopad_A[0: 7]();
    ioif iopad_B[0:15]();
    ioif iopad_C[0:13]();
    ioif iopad_D[0:15]();
    ioif iopad_E[0:15]();
    ioif iopad_F[0: 5]();

    soc_top dut(
/*        input logic       */   .clkxtl     ( clk          ),          
/*        input logic [0:2] */   .cmspad     ( cmspad       ),          
/*        input logic       */   .chipresetn ( resetn       ),              
/*        input logic       */   .padresetn  ( padresetn    ),             
                                 .dbgtxd     ( dbgtxd       ),
/*        input logic       */   .clkswd     ( clkswd       ),
            .*
    );

  //
  //  monitor and clk
  //  ==

    `genclk( clk, 20 )
    `timemarker2

  //
  //  subtitle
  //  ==

    initial begin
        #(30 `MS);
    `maintestend

    `maintest(sys_top_tb,sys_top_tb)
        #105 resetn = 0; padresetn = 1;

//        #(15 `US) resetn = 0;
        #(1 `US) resetn = 1;
    
        #(30 `MS);
    `maintestend

    initial resetn = 0;
    initial clk = 0;
    initial clkswd = 0;

    integer fd;
    string MEMFILE = "reram_simcode.bin";
    parameter MEMDEPTH = 2**14;
    parameter DATAWIDTH = 64;
    localparam DATA_BYTES = 8;

    reg  [DATAWIDTH-1:0]    data;
    integer r;

    initial begin
          fd = $fopen(MEMFILE,"rb");

//          for(i = 0; (i < MEMDEPTH) && ($fread(data,fd) != -1); i = i + 1)
          for(i = 0; (i < MEMDEPTH )  && ($fread(data,fd) != 0 ) ; i = i + 1)
          begin
//            r = $fread(data,fd);
            dut.soc_coresub.rrc.emuram.ramdat[i] = swizzle(data);
            $display("%d, %016x, %d",i,dut.soc_coresub.rrc.emuram.ramdat[i],r);
          end
/*
          r = 1;
          i = 0;
          while( (i < MEMDEPTH ) && ( r != 0 ) )
          begin
            r = $fread(data,fd) ;
//            r = $fread(data,fd);
            dut.soc_coresub.rrc.emuram.ramdat[i] = swizzle(data);
            $display("%d, %016x, %d",i,dut.soc_coresub.rrc.emuram.ramdat[i],r);

            i = i + 1;
          end
*/
          $fclose(fd);
          $write("RAM: image max    = %d lines\n",i);
    end
//      dut.soc_coresub.rrc.emuram.sramc.ramdat

  //Function to account for the Big Endianness of $fread
  function [DATAWIDTH-1:0] swizzle;
      input [DATAWIDTH-1:0] data_in;
      integer i;
      begin
        for (i=0; i<DATA_BYTES; i=i+1)
          swizzle[i*8 +:8] = data_in[(DATA_BYTES-1-i)*8 +:8];
      end
   endfunction

endmodule
