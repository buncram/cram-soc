module syncpulse(
    input  bit clka,
    input  bit clkb,
    input  bit pin,
    output bit pout
    );

    bit ptoga, ptogb, ptogbreg;

    always@(posedge clka) ptoga <= ptoga ^ pin;

    syncbit u1(
        .clk(clkb),
        .sin(ptoga),
        .sout(ptogb)
        );

    always@(posedge clkb) ptogbreg <= ptogb;
    assign pout = ptogbreg ^ ptogb;

endmodule

module syncbit#(
    parameter RC = 2
)(
    input  bit clk,
    input  bit sin,
    output bit sout
    );

    bit [RC-1:0] sreg;

    always@(posedge clk) sreg <= { sreg, sin };
    assign sout = sreg[RC-1];

endmodule
