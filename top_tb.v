`timescale 1ns/1ps

module uart_print #(
    parameter TYPE="generic"
)(
    input logic [7:0] uart_data,
    input logic uart_data_valid,
    input logic resetn,
    input logic clk
);
    // print debug strings
    `define theregfull( theclk, theresetn, theregname, theinitvalue ) \
        always@( posedge theclk or negedge theresetn ) \
        if( ~theresetn) \
            theregname <= theinitvalue; \
        else \
            theregname

    `define theregrn(theregname) \
        `theregfull( clk, resetn, theregname, '0 )

    localparam CHARLEN = 256;
    logic                       charbufwr, charbuffill, charbufclr;
    bit [$clog2(CHARLEN)-1:0]   charbufidx;
    bit [0:CHARLEN-1][7:0]      charbufdat;
    string charbufstring;
    assign charbufwr = uart_data_valid;
    assign charbuffill = charbufwr & ~(( uart_data[7:0] == 'h0d ) | ( uart_data[7:0] == 'h0a ));
    assign charbufclr  = charbufwr &  (( uart_data[7:0] == 'h0d ) | ( uart_data[7:0] == 'h0a ));

    `theregrn( charbufidx ) <= charbufclr ? '0 : charbuffill ? charbufidx + 1 : charbufidx;
    `theregrn( charbufdat[charbufidx] ) <= charbuffill ? uart_data[7:0] : charbufdat[charbufidx];

    always@( negedge clk )
    if( uart_data_valid )  begin
        if( charbufclr ) begin
            charbufstring = string'(charbufdat);
            $display("[%s] %s", TYPE, charbufstring );
            charbufdat = '0;
        end
    end
endmodule

module top_tb();

/////////// boilerplate in here
`include "common.v"

/////////// DUT code below here

reg aclk;
initial aclk = 0;
always #0.625 aclk = ~aclk;

reg [31:0] trimming_reset;
reg trimming_reset_ena;
reg reset;

initial begin
    // test the trimming reset.
    reset = 0;
    trimming_reset = 32'h6000_0002;
    trimming_reset_ena = 1'b1;

    #20000
    reset = 1;
    #100 reset = 0;
    #10000
    trimming_reset_ena = 1'b1;
    reset = 1;
    #100 reset = 0;
end

reg trst;
reg tck;
reg tms;
reg tdi;
initial tck = 0;
initial tms = 0;
initial tdi = 0;
always @(posedge aclk) begin
    trst <= reset;
end

reg serial_rx;
initial serial_rx = 1;
wire serial_tx;

wire sclk;
wire scs;
wire si;
wire coreuser;

wire [7:0] uart_kernel;
wire uart_kernel_valid;
wire [7:0] uart_log;
wire uart_log_valid;
wire [7:0] uart_app;
wire uart_app_valid;

cram_soc dut (
    .aclk(aclk),
    .reset(reset),

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

    .trimming_reset(trimming_reset),
    .trimming_reset_ena(trimming_reset_ena),

    .sim_uart_kernel(uart_kernel),
    .sim_uart_kernel_valid(uart_kernel_valid),
    .sim_uart_log(uart_log),
    .sim_uart_log_valid(uart_log_valid),
    .sim_uart_app(uart_app),
    .sim_uart_app_valid(uart_app_valid),

    .sim_coreuser(coreuser),
    .sim_success(success),
    .sim_done(done),
    .sim_report(report)
);

uart_print #(
    .TYPE("kernel")
) kernel (
    .uart_data(uart_kernel),
    .uart_data_valid(uart_kernel_valid),
    .resetn(~trst),
    .clk(clk)
);
uart_print #(
    .TYPE("log")
) log (
    .uart_data(uart_log),
    .uart_data_valid(uart_log_valid),
    .resetn(~trst),
    .clk(clk)
);

// extra reporting for CI
initial begin
        $dumpvars(0, sclk);
        $dumpvars(0, si);
        $dumpvars(0, scs);
        $dumpvars(0, uart_kernel);
        $dumpvars(0, uart_kernel_valid);
        $dumpvars(0, uart_log);
        $dumpvars(0, uart_log_valid);
        $dumpvars(0, uart_app);
        $dumpvars(0, uart_app_valid);
        $dumpvars(0, coreuser);
        $dumpvars(0, report);
        $dumpvars(0, success);
        $dumpvars(0, done);
end

// DUT-specific end condition to make sure it eventually stops running for CI mode
initial #40_000_000 $finish;

endmodule
