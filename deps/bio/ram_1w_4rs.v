
`resetall
`timescale 1ns / 1ps
`default_nettype none

module Ram_1w_4rs #(
    parameter ramname = "undefined",
    parameter wordCount = 1024,
    parameter wordWidth = 32,
    parameter clockCrossing = 0,
    parameter technology = "auto", // not used
    parameter readUnderWrite = "dontCare",
    parameter wrAddressWidth = 10,
    parameter wrDataWidth = 32,
    parameter wrMaskWidth = 4,
    parameter wrMaskEnable = 0,
    parameter rdAddressWidth = 10,
    parameter rdDataWidth = 32
)
(
    input  wire                             wr_clk,
    input  wire                             wr_en,
    input  wire [wrMaskWidth -1:0]          wr_mask,
    input  wire [wrAddressWidth - 1:0]      wr_addr,
    input  wire [wrDataWidth - 1:0]         wr_data,
    input  wire                             rd_clk,
    input  wire                             rd_en[4],
    input  wire [rdAddressWidth - 1:0]      rd_addr[4],
    output reg  [rdDataWidth - 1:0]         rd_data[4],
    input  wire                             CMBIST, // dummy pins for test insertion
    input  wire                             CMATPG // dummy pins for test insertion
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
        if (wr_en & (wr_mask[i] | !wrMaskEnable)) begin
            mem[wr_addr][WORD_SIZE*i +: WORD_SIZE] <= wr_data[WORD_SIZE*i +: WORD_SIZE];
        end
    end
end
generate
    genvar j;
    for(j=0; j<4; j=j+1) begin; ports
        always @(posedge rd_clk) begin
            if (rd_en[j]) begin
                rd_data[j] <= mem[rd_addr[j]];
            end
        end
    end
endgenerate

endmodule

`resetall