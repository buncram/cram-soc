`timescale 1ns / 1ps

// contains "cosim" models that encapsulate some key differences between ASIC/FPGA behaviors
// particularly around reset conditions.

module finisher(
    input wire [31:0] report,
    input wire success,
    input wire done
);

reg [31:0] rep_state;

always @(report) begin
    if (report != rep_state) begin
        rep_state <= report;
        $display("Report update: %h", report);
    end
end

always @(*) begin
    if (done == 1'b1) begin
        if (success == 1'b1) begin
            $display("Simulation success: %h", report);
        end else begin
            $display("Simulation failure: %h", report);
        end
        $dumpflush;
        $dumpoff;
        $finish;
    end
end

// make sure we eventually stop
// initial #750_000_000 $finish;

endmodule
