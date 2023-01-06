`timescale 1ns/1ps

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
always @(posedge clk12) begin
    trst <= reset;
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

    .sim_coreuser(coreuser),
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
initial #4_000_000 $finish;

parameter RAM_DATA_WIDTH = 32;
parameter RAM_ADDR_WIDTH = 17; // 22 is the full width, but slightly smaller to accelerate the simulation

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
