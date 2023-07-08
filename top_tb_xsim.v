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

module top_tb_xsim();

/////////// boilerplate in here
`include "common.v"

/////////// DUT code below here

reg lpclk;
initial lpclk = 1'b0;
//always #15258.789 lpclk = ~lpclk;
always #400 lpclk = ~lpclk;   // speed up faster than real-time, but still much slower than main clocks

reg reset;

initial begin
    reset = 0;
    #200
    reset = 1;
    #100
    reset = 0;
end

reg trst;
reg tck;
reg tms;
reg tdi;
initial tck = 0;
initial tms = 0;
initial tdi = 0;
initial begin
    trst = 0;
    #100000; // pulse this after the first DQS cycle happens, for some reason this triggers a false timing violation in the FIFO18E block if it's done early.
    trst = 1;
    #200;
    trst = 0;
end

reg serial_rx;
initial serial_rx = 1;
wire serial_tx;

wire coreuser;

reg fpga_reset;
initial begin
  fpga_reset = 1'b1;  // fpga reset is extra-long to get past init delays of SPINOR; in reality, this is all handled by the config engine
  #40_000;
  fpga_reset = 1'b0;
end

wire [7:0] uart_kernel;
wire uart_kernel_valid;
wire [7:0] uart_log;
wire uart_log_valid;
wire [7:0] uart_app;
wire uart_app_valid;
wire clk;

cram_soc dut (
    .clk12(clk12),
    .lpclk(lpclk),
    .reset(fpga_reset),

    .jtag_cpu_tck(tck),
    .jtag_cpu_tms(tms),
    .jtag_cpu_tdi(tdi),
    .jtag_cpu_tdo(tdo),
    .jtag_cpu_trst_n(~trst),

    .serial_tx(serial_tx),
    .serial_rx(serial_rx),

    .sim_uart_kernel(uart_kernel),
    .sim_uart_kernel_valid(uart_kernel_valid),
    .sim_uart_log(uart_log),
    .sim_uart_log_valid(uart_log_valid),
    .sim_uart_app(uart_app),
    .sim_uart_app_valid(uart_app_valid),
    .simio_sysclk(clk),
    .simio_coreuser(coreuser),
    .simio_success(success),
    .simio_done(done),
    .simio_report(report)
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
initial #750_000_000 $finish;
endmodule
