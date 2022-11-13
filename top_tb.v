`timescale 1ns/1ps

module top_tb();

/////////// boilerplate in here
`include "common.v"

/////////// DUT code below here

reg lpclk;
initial lpclk = 1'b0;
//always #15258.789 lpclk = ~lpclk;
always #400 lpclk = ~lpclk;   // speed up faster than real-time, but still much slower than main clocks

reg trst;
initial begin
    trst = 1;
    #100 trst = 0;
end

reg tck;
reg tms;
reg tdi;
initial tck = 0;
initial tms = 0;
initial tdi = 0;

reg serial_rx;
initial serial_rx = 1;
wire serial_tx;

wire sclk;
wire scs;
wire si;

cram_soc dut (
    .clk12(clk12),
    .lpclk(lpclk),

    .jtag_cpu_tck(tck),
    .jtag_cpu_tms(tms),
    .jtag_cpu_tdi(tdi),
    .jtag_cpu_tdo(tdo),
    .jtag_cpu_trst(trst),

    .serial_tx(serial_tx),
    .serial_rx(serial_rx),
    .lcd_sclk(sclk),
    .lcd_si(si),
    .lcd_scs(scs),

    .sim_success(success),
    .sim_done(done),
    .sim_report(report)
);

// extra reporting for CI
initial begin
        $dumpvars(0, sclk);
        $dumpvars(0, si);
        $dumpvars(0, scs);
        $dumpvars(0, serial_tx);
        $dumpvars(0, serial_rx);
        $dumpvars(0, tck);
        $dumpvars(0, tms);
        $dumpvars(0, tdi);
        $dumpvars(0, tdo);
        $dumpvars(0, trst);
end

// DUT-specific end condition to make sure it eventually stops running for CI mode
initial #600_000 $finish;

endmodule
