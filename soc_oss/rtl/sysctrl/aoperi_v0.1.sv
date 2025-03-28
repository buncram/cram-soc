module ao_peri (
            input logic           pclk   ,
            input logic           presetn,
            input logic           pwrite ,
            input logic           penable ,
            input logic [1:0]     psel   ,
            input logic [11:2]    paddr  ,
            input logic [31:0]    pwdata ,
            output logic [31:0]   prdata ,

            input logic clk32k ,
            input logic clk1hz ,

            output logic wdtintr,
            output logic tmrintr ,
            output logic rtcintr,
            output logic wdtrst
);

    localparam PAW = 12;

    logic psel0, psel1, psel2, psel3;
    assign psel0 = ( psel == 'h0 );
    assign psel1 = ( psel == 'h1 );
    assign psel2 = ( psel == 'h2 );
    assign psel3 = ( psel == 'h3 );

    logic [3:0][31:0] prdatas;  //eco
    assign prdata = prdatas[psel];

    logic tmrintrl, tmrintrh;
    logic [4:0] clk1kcnt;
    logic clk1k, clk1ken;

    `theregfull( clk32k, presetn, clk1kcnt, '0 ) <= clk1kcnt + 1;
    `theregfull( clk32k, presetn, clk1ken, '0 ) <= ( clk1kcnt == '1 ) ;

    ICG uclk1k ( .CK (clk32k), .SE('0), .EN (clk1ken), .CKG(clk1k));

  Rtc  urtc (

    .PCLK            (pclk),
    .PRESETn         (presetn),
    .PENABLE         (penable),
    .PSEL            (psel3),
    .PADDR           (paddr[11:2]),
    .PWRITE          (pwrite),
    .PWDATA          (pwdata),
    .PRDATA          (prdatas[3]),    //eco

    .CLK1HZ          (clk1hz),
    .nRTCRST         (presetn),
    .nPOR            (presetn),

    .SCANINPCLK      ('0),
    .SCANINCLK1HZ    ('0),
    .SCANOUTPCLK     (),
    .SCANOUTCLK1HZ   (),
    .SCANENABLE      ('0),

    .RTCINTR         (rtcintr)

    );

  cmsdk_apb_watchdog uwdt (
   // Inputs
    .PCLK            (pclk),
    .PRESETn         (presetn),
    .PENABLE         (penable),
    .PSEL            (psel1),
    .PADDR           (paddr[11:2]),
    .PWRITE          (pwrite),
    .PWDATA          (pwdata),
    .PRDATA          (prdatas[1]),

    .WDOGCLK           (clk1k),
    .WDOGCLKEN         (1'b1),
    .WDOGRESn          (presetn),
    .ECOREVNUM         (4'h0),// Engineering-change-order revision bits
   // Outputs
    .WDOGINT           (wdtintr),  // connect to NMI
    .WDOGRES           (wdtrst)   // connect to reset generator
  );

    apb_timer_unit #(.APB_ADDR_WIDTH(PAW)) utmr (
        .HCLK       ( pclk          ),
        .HRESETn    ( presetn       ),
        .PADDR      ( { paddr[11:2],2'b00 } ),
        .PWDATA     ( pwdata  ),
        .PWRITE     ( pwrite  ),
        .PSEL       ( psel2    ),
        .PENABLE    ( penable ),
        .PRDATA     ( prdatas[2]  ),
        .PREADY     ( ),
        .PSLVERR    ( ),
        .ref_clk_i  ( clk1k         ),
        .event_lo_i ( '0 ),
        .event_hi_i ( '0 ),
        .irq_lo_o   ( tmrintrl    ),
        .irq_hi_o   ( tmrintrh    ),
        .busy_o     (            )
    );

    assign tmrintr = tmrintrl | tmrintrh;

endmodule : ao_peri