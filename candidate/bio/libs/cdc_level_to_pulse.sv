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

// Assumes: clk_a is slower than clk_faster

module cdc_level_to_pulse (
    input  wire          reset,
    input  wire          clk_a,
    input  wire          clk_faster,
    input  wire          in_a,
    output wire          out_b
);

logic in_a_d;
always_ff @(posedge clk_a) begin
    in_a_d <= in_a;
end

logic [2:0] pulse;
always_ff @(posedge clk_faster) begin
    pulse[2] <= ~in_a_d & in_a;
    pulse[1] <= pulse[2];
    pulse[0] <= pulse[1];
end

assign out_b = ~pulse[0] & pulse[1];
endmodule