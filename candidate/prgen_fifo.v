//------------------------------------------------------------------
//-- File generated by RobustVerilog parser
//-- RobustVerilog version 1.2 (limited free version)
//-- Invoked Mon Feb 06 01:29:27 2023
//-- Source file: prgen_fifo.v
//-- Parent file: axi2ahb_cmd.v
//-- Run directory: F:/largework/rust-win/code/robust_axi2ahb/
//-- Target directory: out/
//-- Command flags: .\src\base\axi2ahb.v -od out -I .\src\gen\ -list list.txt -listpath -header
//-- www.provartec.com/edatools ... info@provartec.com
//------------------------------------------------------------------




module prgen_fifo(clk,
                  reset,
                  push,
                  pop,
                  din,
                  dout,
                  empty,
                  full
);
    parameter                  WIDTH      = 8;
    parameter                  DEPTH_FULL = 8;

    parameter               SINGLE = DEPTH_FULL == 1;
    parameter               DEPTH  = SINGLE ? 1 : DEPTH_FULL -1;
    parameter               DEPTH_BITS =
    (DEPTH <= 2)   ? 1 :
    (DEPTH <= 4)   ? 2 :
    (DEPTH <= 8)   ? 3 :
    (DEPTH <= 16)  ? 4 :
    (DEPTH <= 32)  ? 5 :
    (DEPTH <= 64)  ? 6 :
    (DEPTH <= 128) ? 7 :
    (DEPTH <= 256) ? 8 :
    (DEPTH <= 512) ? 9 : 0; //0 is ilegal

    parameter               LAST_LINE = DEPTH-1;



    input                      clk;
    input                      reset;

    input               push;
    input               pop;
    input [WIDTH-1:0]           din;
    output [WIDTH-1:0]           dout;
    output               empty;
    output               full;


    wire               reg_push;
    wire               reg_pop;
    wire               fifo_push;
    wire               fifo_pop;

    // TODO: fix these initializers so they are ASIC-friendly.
    reg [DEPTH-1:0]           full_mask_in = 0;
    reg [DEPTH-1:0]           full_mask_out = 0;
    reg [DEPTH-1:0]           full_mask = 0;
    reg [WIDTH-1:0]           fifo [DEPTH-1:0];
    wire               fifo_empty;
    wire               next;
    reg [WIDTH-1:0]           dout = 0;
    reg                   dout_empty;
    reg [DEPTH_BITS-1:0]       ptr_in = 0;
    reg [DEPTH_BITS-1:0]       ptr_out = 0;

    // TODO: on an ASIC, the RAM powers up as X.
    integer j;
    initial begin
        for (j = 0; j < DEPTH; j = j + 1) begin
            fifo[j] = 0;
        end
    end

    assign               reg_push  = push & fifo_empty & (dout_empty | pop);
    assign               reg_pop   = pop & fifo_empty;
    assign               fifo_push = !SINGLE & push & (~reg_push);
    assign               fifo_pop  = !SINGLE & pop & (~reg_pop);


    always @(posedge clk or posedge reset)
        if (reset) begin
            dout       <= {WIDTH{1'b0}};
            dout_empty <= 1'b1;
        end else if (reg_push) begin
            dout       <= din;
            dout_empty <= 1'b0;
        end else if (reg_pop) begin
          dout       <= {WIDTH{1'b0}};
          dout_empty <= 1'b1;
        end else if (fifo_pop) begin
          dout       <= fifo[ptr_out];
          dout_empty <= 1'b0;
        end

    always @(posedge clk or posedge reset)
        if (reset)
            ptr_in <= {DEPTH_BITS{1'b0}};
        else if (fifo_push)
            ptr_in <= ptr_in == LAST_LINE ? 0 : ptr_in + 1'b1;

    always @(posedge clk or posedge reset)
        if (reset)
            ptr_out <= {DEPTH_BITS{1'b0}};
        else if (fifo_pop)
            ptr_out <= ptr_out == LAST_LINE ? 0 : ptr_out + 1'b1;

    always @(posedge clk)
        if (fifo_push)
            fifo[ptr_in] <= din;


    always @(/*AUTOSENSE*/fifo_push or ptr_in)
    begin
        full_mask_in         = {DEPTH{1'b0}};
        full_mask_in[ptr_in] = fifo_push;
    end

    always @(/*AUTOSENSE*/fifo_pop or ptr_out)
    begin
        full_mask_out          = {DEPTH{1'b0}};
        full_mask_out[ptr_out] = fifo_pop;
    end

    always @(posedge clk or posedge reset)
        if (reset)
            full_mask <= {DEPTH{1'b0}};
        else if (fifo_push | fifo_pop)
            full_mask <= (full_mask & (~full_mask_out)) | full_mask_in;


    assign next       = |full_mask;
    assign fifo_empty = ~next;
    assign empty      = fifo_empty & dout_empty;
    assign full       = SINGLE ? !dout_empty : &full_mask;
endmodule




