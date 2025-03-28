module blake3_tb;

reg clk;
reg [0:15][31:0] msgin;
reg [0:15][31:0] iv16;
wire [0:7][31:0] stout;

// Instantiate the blake3 module

    blake3 dut (

        .clk(clk),
        .msgin(msgin),
        .iv16(iv16),
        .stout(stout)
    );

    initial begin
        clk =0;
        forever #5 clk = ~clk;
    end

    initial begin   
        iv16[0:7] = {32'h6b08e647, 32'hbb67ae85, 32'h3c6ef372, 32'ha54ff53a, 32'h510e527f, 32'h9b05688c, 32'h1f83d9ab, 32'h5be0cd19};//6a09e667 
        iv16[ 8  ] = 32'h0 ;
        iv16[ 9  ] = 32'h0 ;
        iv16[ 10 ] = 32'h0 ;
        iv16[ 11 ] = 32'h0 ;
        iv16[ 12 ] = 32'h0 ^ 32'h3;
        iv16[ 13 ] = 32'h0 ^ 32'h0;
        iv16[ 14 ] = '1 ;
        iv16[ 15 ] = 32'h0 ;

        msgin[0:7] = {32'h61626300, 32'h0, 32'h0, 32'h0, 32'h0, 32'h0, 32'h0, 32'h0};
        msgin[8:15] = {32'h0, 32'h0, 32'h0, 32'h0, 32'h0, 32'h0, 32'h0, 32'h0};

        #100
        $display("stout[0]: %x", stout[0]);
        $display("stout: %x", stout);
        $finish;
    end


endmodule

