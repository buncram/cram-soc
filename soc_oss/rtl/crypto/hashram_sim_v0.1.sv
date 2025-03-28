
module hashram #( 
    parameter AW = 10,
    parameter string INITFILE,
    parameter DW = 32

)(
    input   bit             clk,    // Clock
    input   bit             resetn,  // Asynchronous reset active low

    input bit   [AW-1:0]    ramaddr,
    output  bit [DW-1:0]    ramrdat,
    input bit               ramwr,
    input bit   [DW-1:0]    ramwdat
);

    bit [DW-1:0]    ramdat[0:2**AW-1];
    bit [0:2**AW-1][DW-1:0]    vramdat;

    initial $readmemh(INITFILE, ramdat);

    always@(posedge clk) if(ramwr) ramdat[ramaddr] <= ramwdat;
    always@(posedge clk) ramrdat <= ramdat[ramaddr];

    genvar i;
    generate
        for(i=0;i<2**AW;i++)
        assign vramdat[i] = ramdat[i];
    endgenerate


endmodule : hashram
