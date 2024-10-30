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

module fdre_cosim(
    output wire Q,
    input wire R,
    input wire C,
    input wire CE,
    input wire D
);

    reg inner_Q;
    // force the asic synthesis tool to infer an async, active-low reset FF
    always @(posedge C or posedge R) begin
        if(R) begin
            inner_Q <= 0;
        end else begin
            if(CE) begin
                inner_Q <= D;
            end else begin
                inner_Q <= Q;
            end
        end
    end
    assign Q = inner_Q;

endmodule