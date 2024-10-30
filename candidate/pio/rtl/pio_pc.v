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
module pio_pc (
  input        clk,
  input        penable,
  input        reset,
  input [4:0]  din,
  input        jmp,
  input [4:0]  pend,
  input        stalled,
  input [4:0]  wrap_target,
  input        imm,
  output [4:0] dout
);

  reg [4:0] index = 0;

  assign dout = ((penable || imm) && !stalled) ?
        (jmp ?
          din // PC will get jmp target, even if IMM
          : imm ?
              index  // PC does not increment if it's an IMM and not a JMP
              : (index == pend) ? wrap_target : index + 1)
        : index;

  always @(posedge clk) begin
    if (reset)
      index <= 0;
    else if (imm) begin
      if (jmp)
        index <= din;
    end else if (penable && !stalled) begin
      if (jmp)
        index <= din;
      else
        index <= index == pend ? wrap_target : index + 1;
    end
  end

endmodule
