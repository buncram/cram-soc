
`resetall
`timescale 1ns / 1ps
`default_nettype none

module Ram_1w_1rs #(
    parameter ramname = "undefined",
    parameter wordCount = 32,
    parameter wordWidth = 32,
    parameter clockCrossing = 0,
    parameter technology = "auto", // not used
    parameter readUnderWrite = "dontCare",
    parameter wrAddressWidth = 5,
    parameter wrDataWidth = 32,
    parameter wrMaskWidth = 1,
    parameter wrMaskEnable = 0,
    parameter rdAddressWidth = 5,
    parameter rdDataWidth = 32
)
(
    input  wire                             wr_clk,
    input  wire                             wr_en,
    input  wire [wrMaskWidth -1:0]          wr_mask,
    input  wire [wrAddressWidth - 1:0]      wr_addr,
    input  wire [wrDataWidth - 1:0]         wr_data,
    input  wire                             rd_clk,
    input  wire                             rd_en,
    input  wire [rdAddressWidth - 1:0]      rd_addr,
    output reg  [rdDataWidth - 1:0]         rd_data
);

parameter WORD_WIDTH = wrMaskWidth;
parameter WORD_SIZE = wrDataWidth/WORD_WIDTH;

initial begin
    if (readUnderWrite != "dontCare") begin
        $error("This implementation only handles readUnderWrite == dontCare");
    end
    if (wrDataWidth != rdDataWidth) begin
        $error("This implementation only handles wrDataWidth == rdDataWidth");
    end
    if (wrAddressWidth != rdAddressWidth) begin
        $error("This implementation only handles wrAddressWidth == rdAddressWidth");
    end
    if (clockCrossing != 0) begin
        $error("This implementation only handles clockCrossing == 0");
    end
end

parameter RAM_DATA_WIDTH = wrDataWidth;
parameter RAM_ADDR_WIDTH = wrAddressWidth;

reg [RAM_DATA_WIDTH-1:0] mem[(2**RAM_ADDR_WIDTH)-1:0];

integer i, j;

initial begin
    for (i = 0; i < 2**RAM_ADDR_WIDTH; i = i + 2**(RAM_ADDR_WIDTH/2)) begin
        for (j = i; j < i + 2**(RAM_ADDR_WIDTH/2); j = j + 1) begin
            mem[j] = 0;
        end
    end
end

always @(posedge wr_clk) begin
    for (i = 0; i < WORD_WIDTH; i = i + 1) begin
        if (wr_en & wr_mask[i]) begin
            mem[wr_addr][WORD_SIZE*i +: WORD_SIZE] <= wr_data[WORD_SIZE*i +: WORD_SIZE];
        end
    end
end
always @(posedge rd_clk) begin
    if (rd_en) begin
        rd_data <= mem[rd_addr];
    end
end

endmodule

`resetall