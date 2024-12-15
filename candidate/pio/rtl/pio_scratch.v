// (c) Copyright CrossBar, Inc. 2024.
//
// This documentation describes Open Hardware and is licensed under the [CERN-OHL-W-2.0].
//
// You may redistribute and modify this documentation under the terms of the
// [CERN-OHL- W-2.0 (http://ohwr.org/cernohl)]. This documentation is
// distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY, INCLUDING OF
// MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A PARTICULAR PURPOSE.
// Please see the [CERN-OHL- W-2.0] for applicable conditions.

// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// `default_nettype none
module pio_scratch (
  input         clk,
  input         penable,
  input         reset,
  input         stalled,
  input [31:0]  din,
  input         set,
  input         dec,
  output [31:0] dout
);

  reg [31:0] val;

  assign dout = val;

  always @(posedge clk) begin
    if (reset)
      val <= 0;
    else if (penable && !stalled) begin
      if (set)
        val <= din;
      else if (dec)
        val <= val - 1;
    end
  end

endmodule
 
