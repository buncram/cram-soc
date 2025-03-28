module rtc_intf (
	input bit clk,    // Clock
	input bit resetn,
	input bit clk1hz,
	apbif.slave  apbs,
    output logic intr
);

  Rtc  uut (

        .PRESETn         (resetn),
        .PCLK            (clk),
        .PWRITE          (apbs.pwrite),
        .PENABLE         (apbs.psel]),
        .PSEL            (apbs.psel),
        .PADDR           (apbs.paddr[11:2]),
        .PWDATA          (apbs.pwdata),
        .PRDATA          (apbs.prdata),
        .CLK1HZ          (clk1hz),
        .nRTCRST         (resetn),
        .nPOR            (resetn),

        .SCANINPCLK      ('0),
        .SCANINCLK1HZ    ('0),
        .SCANOUTPCLK     (),
        .SCANOUTCLK1HZ   (),
        .SCANENABLE      ('0),   

        .RTCINTR         (intr)

        );

endmodule
