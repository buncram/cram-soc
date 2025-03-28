`include "template.sv"

module tb();

    bit clk,resetn;
    integer i=0, j=0, k=0, errcnt=0, warncnt=0;

  //
  //  dut
  //  ==


    
function automatic integer clog2(input integer value);
  integer result;
  result = (value > 1) ? clog2(value >> 1) + 1 : 0;
  return result;
endfunction : clog2

  //
  //  monitor and clk
  //  ==

    `genclk( clk, 100 )
    `timemarker2

  //
  //  subtitle
  //  ==

    `maintest(dumpfile,tb)
        #105 resetn = 1;
        for ( i = 0; i < 65; i++) begin
        	j = clog2(i);
        	$display("%d,clog2=%d,$clog2=%d",i,j,$clog2(i));
        end

    
    `maintestend

endmodule
