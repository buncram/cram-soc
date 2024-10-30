// (c) Copyright CrossBar, Inc. 2024.
//
// This documentation describes Open Hardware and is licensed under the
// [CERN-OHL-W-2.0].
//
// You may redistribute and modify this documentation under the terms of the
// [CERN-OHL- W-2.0 (http://ohwr.org/cernohl)]. This documentation is
// distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY, INCLUDING OF
// MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A PARTICULAR PURPOSE.
// Please see the [CERN-OHL- W-2.0] for applicable conditions.

// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// `default_nettype none
module pio_divider (
  input         clk,
  input         reset,
  input [15:0]  div_int,
  input [7:0]   div_frac,
  output        penable
);
  wire use_divider;
  wire divint_1;
  assign use_divider = !((div_int == 0) && (div_frac == 0));
  assign divint_1 = (div_int == 16'd1);

  wire [23:0] div;
  reg [23:0] div_counter;
  reg pen;
  reg old_pen;

  assign div = {div_int, div_frac};
  always @(posedge clk) begin
    if (reset) begin
      div_counter <= 0;
      pen <= 1;
      old_pen <= 0;
    end else begin
      if (use_divider) begin
        old_pen <= pen;
        div_counter <= div_counter + 256;
        if (div_counter >= div - 256)
          div_counter <= div_counter - (div - 256);
        pen <= div_counter < (div >> 1);
      end
    end
  end

  assign penable = ((pen & ~old_pen) || !use_divider) ^ divint_1;
endmodule
