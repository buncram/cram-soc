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

module top_tb_fpga();

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

wire sclk;
wire scs;
wire si;
wire coreuser;

wire spi_sclk;
wire [7:0] sio;
wire dqs;
wire ecsb;
wire csn;

MX66UM1G45G rom(
  .SCLK(spi_sclk),
  .CS(csn),
  .SIO(sio),
  .DQS(dqs),
  .ECSB(ecsb),
  .RESET(~reset)
);

reg fpga_reset;
initial begin
  fpga_reset = 1'b1;  // fpga reset is extra-long to get past init delays of SPINOR; in reality, this is all handled by the config engine
  #40_000;
  fpga_reset = 1'b0;
end

wire [21:0] sram_adr;
wire sram_ce_n;
wire sram_oe_n;
wire sram_we_n;
wire sram_zz_n;
wire [31:0] sram_d;
wire [3:0] sram_dm_n;

wire [7:0] uart_kernel;
wire uart_kernel_valid;
wire [7:0] uart_log;
wire uart_log_valid;
wire [7:0] uart_app;
wire uart_app_valid;
wire clk;

cram_fpga dut (
    .clk12(clk12),
    .lpclk(lpclk),
    .reset(fpga_reset),

    .jtag_cpu_tck(tck),
    .jtag_cpu_tms(tms),
    .jtag_cpu_tdi(tdi),
    .jtag_cpu_tdo(tdo),
    .jtag_cpu_trst(trst),

    .spiflash_8x_cs_n(csn),
    .spiflash_8x_dq(sio),
    .spiflash_8x_dqs(dqs),
    .spiflash_8x_ecs_n(ecsb),
    .spiflash_8x_sclk(spi_sclk),

    .sram_adr(sram_adr),
    .sram_ce_n(sram_ce_n),
    .sram_oe_n(sram_oe_n),
    .sram_we_n(sram_we_n),
    .sram_zz_n(sram_zz_n),
    .sram_d(sram_d),
    .sram_dm_n(sram_dm_n),

    .serial_tx(serial_tx),
    .serial_rx(serial_rx),
    .lcd_sclk(sclk),
    .lcd_si(si),
    .lcd_scs(scs),

    .sim_uart_kernel(uart_kernel),
    .sim_uart_kernel_valid(uart_kernel_valid),
    .sim_uart_log(uart_log),
    .sim_uart_log_valid(uart_log_valid),
    .sim_uart_app(uart_app),
    .sim_uart_app_valid(uart_app_valid),
    .sim_sysclk(clk),
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
initial #750_000_000 $finish;

parameter RAM_DATA_WIDTH = 32;
parameter RAM_ADDR_WIDTH = 22; // could reduce to accelerate the simulation

reg [RAM_DATA_WIDTH-1:0] mem[(2**RAM_ADDR_WIDTH)-1:0];
reg [31:0] rd_data;

integer i, j;

initial begin
    for (i = 0; i < 2**RAM_ADDR_WIDTH; i = i + 2**(RAM_ADDR_WIDTH/2)) begin
        for (j = i; j < i + 2**(RAM_ADDR_WIDTH/2); j = j + 1) begin
            mem[j] = 0;
        end
    end
end

always @* begin
    rd_data = mem[sram_adr];
end
assign sram_d = (sram_oe_n || sram_ce_n) ? 32'hzzzz_zzzz : rd_data;

always @(posedge sram_we_n) begin
    // dm_n is ignored in this implementation, because we always write full words
    if (sram_ce_n == 1'b0) begin
        mem[sram_adr] <= sram_d;
    end
end

endmodule
