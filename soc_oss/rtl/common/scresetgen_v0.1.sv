module scresetgen #(
        parameter ICNT = 4,
        parameter EXTCNT = 4096,
        parameter ECW = $clog2(EXTCNT)
    )(
        input   logic               clk,
        input   logic               cmsatpg,
        input   logic               resetn,
        input   logic [0:ICNT-1]    resetnin,
        output  logic               resetnout
    );

    bit [ECW-1:0] resetextcnt;
    logic resetextcnthit;
    logic resetext;
    logic resetninx;

    assign resetninx = &resetnin & resetn;

    `theregfull(clk, resetninx, resetextcnt, '0) <= resetextcnthit ? resetextcnt : resetextcnt + 1;
    `theregfull(clk, resetninx, resetext,    '0) <= resetextcnthit ;

    assign resetextcnthit = resetextcnt == EXTCNT-1;

`ifdef FPGA
    BUFG u0 (.I(resetext), .O(resetnout));
`else 
    assign resetnout = cmsatpg ? 1'b1 : resetext;
`endif
endmodule