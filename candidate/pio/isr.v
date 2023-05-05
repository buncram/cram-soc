// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// `default_nettype none
module isr (
  input         clk,
  input         penable,
  input         reset,
  input         stalled,
  input [31:0]  din,
  input [4:0]   shift,
  input         dir,
  input         set,
  input         do_shift,
  input [5:0]   bit_count,
  output [31:0] dout,
  output [31:0] push_dout,
  output [5:0]  shift_count
);

  reg [31:0] shift_reg;
  reg [6:0]  count;

  // A shift value of 0 means shift 32
  wire [6:0] shift_val = shift == 0 ? 32 : shift;
  // Left align the input value and concatenate it with the shift register to produce a 64-bit value
  wire [63:0] new_shift = dir ? {din, shift_reg} >> shift_val
                              : {shift_reg, din << (32 - shift_val)} << shift_val;
  // wire [63:0] new_shift = dir ? {din, set ? din : shift_reg} >> shift_val
  //                            : {set ? din : shift_reg, din << (32 - shift_val)} << shift_val;

  always @(posedge clk) begin
    if (reset) begin
      shift_reg <= 0;
      count <= 0;  // Empty
    end else if (penable && !stalled) begin
       if (set) begin
         shift_reg <= din;
         count <= 0;
       end else if (do_shift) begin
         shift_reg <= dir ? new_shift[31:0] : new_shift[63:32];
         count <= (count + shift_val) > 7'd32 ? 7'd32 : count + shift_val;
       end
    end
  end

  assign dout = shift_reg;
  assign push_dout = do_shift ? (dir ? new_shift[31:0] : new_shift[63:32]) : shift_reg;
  // count must be made available "early" to comply with timing
  assign shift_count = ((count + shift_val) > 7'd32) ? 7'd32 : count + shift_val;

endmodule

