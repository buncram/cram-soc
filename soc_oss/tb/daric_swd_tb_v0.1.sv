`include "template.sv"

module daric_top_tb ();

    bit clk,resetn;
    bit clk1m;
    integer i=0, j=0, k=0, errcnt=0, warncnt=0;

    bit [0:2]   cmspad;
    bit         padresetn;
    bit         clkswd;
    bit         dbgtxd;

    reg pulse1M = 0;

    wire pulse1Mwire; assign pulse1Mwire = pulse1M;
  //
  //  dut
  //  ==
/*
    ioif iopad_A[0: 7]();
    ioif iopad_B[0:15]();
    ioif iopad_C[0:13]();
    ioif iopad_D[0:15]();
    ioif iopad_E[0:15]();
    ioif iopad_F[0: 5]();
*/
    bit clk32k, clk32m;
    OSC_SIM #(31.25*1000)   osc32k ( .EN('1), .CFG('0),      .CKO( clk32k  ) );
    OSC_SIM #(31.25)        osc32M ( .EN('1), .CFG('0),      .CKO( clk32m  ) );

    wire PAD_SWDIO;
    wire SWCLK;

    daric_top dut(
        .XTAL_IN    (clk32m),
`ifndef FPGA
        .XTAL32K_IN (clk32k),
`endif
        .PAD_SWDCK  (SWCLK),
        .PAD_SWDIO  (PAD_SWDIO),
        .PA0 (pulse1Mwire)
/*        input logic       */  // .clkxtl     ( clk          ),          
/*        input logic [0:2] */  // .cmspad     ( cmspad       ),          
/*        input logic       */  // .chipresetn ( resetn       ),              
/*        input logic       */  // .padresetn  ( padresetn    ),             
                                // .dbgtxd     ( dbgtxd       ),
/*        input logic       */  // .clkswd     ( clkswd       ),
//            .*
    );

    swd uswd(.clkin('0), .SWCLK(SWCLK), .SWDIO(PAD_SWDIO));


  //
  //  monitor and clk
  //  ==

    `genclk( clk, 20 )
    `genclk( clk1m, 1000 )
    `timemarker2


    initial forever begin
         pulse1M = 0;
        #( 500 `US ) pulse1M = 1;
        #( 1 `US ) pulse1M = 1;
    end

//    assign dut.PA0 = pulse1Mwire;

  //
  //  subtitle
  //  ==

    initial begin
        #(10 `MS);
    `maintestend

    `maintest(daric_top_tb,daric_top_tb)
        #105 resetn = 0; padresetn = 1;

//        #(15 `US) resetn = 0;
        #(1 `US) resetn = 1;
    
        #(10 `MS);
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
          #(10 `US );

//          for(i = 0; (i < MEMDEPTH) && ($fread(data,fd) != -1); i = i + 1)
          for(i = 0; (i < MEMDEPTH )  && ($fread(data,fd) != 0 ) ; i = i + 1)
          begin
//            r = $fread(data,fd);
`ifdef FPGA
            if( i < 4096 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[0].FIRST.u0.mem[i] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[0].FIRST.u0.mem[i],r);
            end
            else if( i < 4096*2 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[1].MIDDLE.um.mem[i&4095] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[1].MIDDLE.um.mem[i&4095],r);
            end
            else if( i < 4096*3 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[2].MIDDLE.um.mem[i&4095] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[2].MIDDLE.um.mem[i&4095],r);
            end
            else if( i < 4096*4 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[3].MIDDLE.um.mem[i&4095] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[3].MIDDLE.um.mem[i&4095],r);
            end


`else
            dut.soc.soc_coresub.rrc.emuram.ramdat[i] = swizzle(data);
            $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.ramdat[i],r);
`endif

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
