
    module ip_gluecell (
    //	input logic ana_laser_in,
    //	inout wire vddd,
    //	inout wire vssd,
        input wire ana_test_use_only,
    	input logic d2a_glue_in,
    	input logic d2a_nrst,
    	output logic a2d_glue_out
    );
    `ifndef SYN
    	logic n0, n1;
    	assign n0 = ~( d2a_nrst & n1 );
    	assign n1 = ~n0;

        `ifdef FPGA
            assign a2d_glue_out = '0;
        `else
    	    assign a2d_glue_out = n1 | d2a_glue_in;
        `endif
    `endif
    endmodule
