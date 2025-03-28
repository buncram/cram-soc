module test_param (
	input clk,    // Clock
	input clk_en, // Clock Enable
	input rst_n  // Asynchronous reset active low
	
);

    parameter ADCCNT = 6;
    parameter ADCPAD_NUMs= {0,1,2,3,4,5};

    parameter bit pad_ANA[0:16*5-1] = '0;
    generate
        for (genvar i = 0; i < ADCCNT; i++) begin:g
            defparam pad_ANA[ADCPAD_NUMs[i]] = 1'b1;
        end
    endgenerate

    initial 
	    $display("%b", pad_ANA);
	    $finish;
	end

endmodule
