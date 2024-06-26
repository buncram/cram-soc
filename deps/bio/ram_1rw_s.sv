
`resetall
`timescale 1ns / 1ps
`default_nettype none

// This is modeled on Single-Port High Density Register File for 22ULL spec
// Clock speed target = 800MHz, Min Cycle clk ~0.6ns @ typical
module Ram_1rw_s #(
    parameter ramname = "undefined",
    parameter wordCount = 1024,
    parameter wordWidth = 32,
    parameter technology = "auto", // not used
    parameter AddressWidth = 10,
    parameter DataWidth = 32,
    parameter wrMaskWidth = 4,
    parameter wrMaskEnable = 1
)
(
    input  wire                             clk,
    input  wire [AddressWidth - 1:0]        addr,
    input  wire [DataWidth - 1:0]           d,
    output reg  [DataWidth - 1:0]           q,
    input  wire                             wr_n,    // gwen on RAM maacro
    input  wire                             ce_n,
    input  wire [wrMaskWidth -1:0]          wr_mask_n, // wen[n-1] on RAM macro
    input  wire                             cmbist, // dummy pins for test insertion
    input  wire                             cmatpg, // dummy pins for test insertion
    input  wire [2:0]                       sramtrm // dummy pins for trim insertion
);

parameter WORD_WIDTH = wrMaskWidth;
parameter WORD_SIZE = DataWidth/WORD_WIDTH;

parameter RAM_DATA_WIDTH = DataWidth;
parameter RAM_ADDR_WIDTH = AddressWidth;

reg [RAM_DATA_WIDTH-1:0] mem[(2**RAM_ADDR_WIDTH)-1:0];

integer i, j;

initial begin
    for (i = 0; i < 2**RAM_ADDR_WIDTH; i = i + 2**(RAM_ADDR_WIDTH/2)) begin
        for (j = i; j < i + 2**(RAM_ADDR_WIDTH/2); j = j + 1) begin
            mem[j] = 'X;
        end
    end
end

always @(posedge clk) begin
    if (!ce_n) begin
        q <= mem[addr];
        for (i = 0; i < WORD_WIDTH; i = i + 1) begin: writes
            if (!(wr_n | (wr_mask_n[i])) & wrMaskEnable) begin
                mem[addr][WORD_SIZE*i +: WORD_SIZE] <= d[WORD_SIZE*i +: WORD_SIZE];
            end
        end
    end else begin
        q <= q;
    end
end

endmodule

`resetall