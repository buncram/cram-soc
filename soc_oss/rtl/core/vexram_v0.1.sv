
`resetall
`timescale 1ns / 1ps
`default_nettype none
`include "rtl/model/artisan_ram_def_v0.1.svh"

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
    output logic  [rdDataWidth - 1:0]         rd_data,
    input  wire [2:0]    sramtrm,
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


`ifdef FPGA

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
always @(posedge rd_clk) begin
    if (rd_en) begin
        rd_data <= mem[rd_addr];
    end
end

`else

    logic clka, clkb, cena, cenb;

    ICG icga(.CK(rd_clk),.EN(rd_en),.SE(CMATPG),.CKG(clka));
    ICG icgb(.CK(wr_clk),.EN(wr_en),.SE(CMATPG),.CKG(clkb));
    assign #0.5 cena = ( ~rd_en );
    assign #0.5 cenb = ( ~wr_en );

    logic [WORD_WIDTH-1:0][WORD_SIZE-1:0] wenb ;
    generate
        if(ramname=="RAM_DP_32_16_WM"||ramname=="RAM_DP_32_16_MM") begin: gen_RAM_DP_32_16

            for (genvar i = 0; i < WORD_WIDTH; i++) begin: gwenb
                assign wenb[i] = ~{WORD_SIZE{wr_mask[i]}};
            end
                rdram32x16 m(
                .clka   (clka),
                .cena   (cena),
                .aa     (rd_addr),
                .qa     (rd_data),
                .clkb   (clkb),
                .cenb   (cenb),
                .wenb   (wenb),
                .ab     (wr_addr),
                .db     (wr_data),
                `rf_2p_hdc_inst_vex
                );
        end
        if(ramname=="RAM_DP_1024_32"&&wrMaskEnable==0) begin: gen_RAM_DP_1024_32_wrmask0
                rdram1kx32 m(
                .clka   (clka),
                .cena   (cena),
                .aa     (rd_addr),
                .qa     (rd_data),
                .clkb   (clkb),
                .cenb   (cenb),
                .wenb   ('0),
                .ab     (wr_addr),
                .db     (wr_data),
                `rf_2p_hdc_inst_vex
                );
        end
        if(ramname=="RAM_DP_1024_32"&&wrMaskEnable==1) begin: gen_RAM_DP_1024_32_wrmask1

            for (genvar i = 0; i < WORD_WIDTH; i++) begin: gwenb
                assign wenb[i] = ~{WORD_SIZE{wr_mask[i]}};
            end
                rdram1kx32 m(
                .clka   (clka),
                .cena   (cena),
                .aa     (rd_addr),
                .qa     (rd_data),
                .clkb   (clkb),
                .cenb   (cenb),
                .wenb   (wenb),
                .ab     (wr_addr),
                .db     (wr_data),
                `rf_2p_hdc_inst_vex
                );
        end
        if(ramname=="RAM_DP_512_64") begin: gen_RAM_DP_512_64
                rdram512x64 m(
                .clka   (clka),
                .cena   (cena),
                .aa     (rd_addr),
                .qa     (rd_data),
                .clkb   (clkb),
                .cenb   (cenb),
                .ab     (wr_addr),
                .db     (wr_data),
                `rf_2p_hdc_inst_vex
                );
        end
        if(ramname=="RAM_DP_128_22") begin: gen_RAM_DP_128_22
                rdram128x22 m(
                .clka   (clka),
                .cena   (cena),
                .aa     (rd_addr),
                .qa     (rd_data),
                .clkb   (clkb),
                .cenb   (cenb),
                .ab     (wr_addr),
                .db     (wr_data),
                `rf_2p_hdc_inst_vex
                );
        end


    endgenerate



`endif


endmodule

`resetall