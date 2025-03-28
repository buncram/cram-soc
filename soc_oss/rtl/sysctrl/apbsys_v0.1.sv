
  cmsdk_apb_timer timer (
    .PCLK              (pclk),     // PCLK for timer operation
    .PCLKG             (apb_pclken),    // Gated PCLK for bus
    .PRESETn           (coreresetn),  // Reset
    // APB interface inputs
    .PSEL              (timer0_psel),
    .PADDR             (i_paddr[11:2]),
    .PENABLE           (i_penable),
    .PWRITE            (i_pwrite),
    .PWDATA            (i_pwdata),

    .ECOREVNUM         (4'h0),// Engineering-change-order revision bits

      // APB interface outputs
    .PRDATA            (timer0_prdata),
    .PREADY            (timer0_pready),
    .PSLVERR           (timer0_pslverr),

    .EXTIN             (timer0_extin),  // Extenal input
    .TIMERINT          (timer0_int)     // interrupt output
  );
  end else
  begin : gen_no_apb_timer_0
    assign timer0_prdata  = {32{1'b0}};
    assign timer0_pready  = 1'b1;
    assign timer0_pslverr = 1'b0;
    assign timer0_int     = 1'b0;
  end endgenerate

  generate if (INCLUDE_APB_TIMER1 == 1) begin : gen_apb_timer_1
  cmsdk_apb_timer u_apb_timer_1 (
    .PCLK              (PCLK),     // PCLK for timer operation
    .PCLKG             (PCLKG),    // Gated PCLK for bus
    .PRESETn           (PRESETn),  // Reset
    // APB interface inputs
    .PSEL              (timer1_psel),
    .PADDR             (i_paddr[11:2]),
    .PENABLE           (i_penable),
    .PWRITE            (i_pwrite),
    .PWDATA            (i_pwdata),

    .ECOREVNUM         (4'h0),// Engineering-change-order revision bits

      // APB interface outputs
    .PRDATA            (timer1_prdata),
    .PREADY            (timer1_pready),
    .PSLVERR           (timer1_pslverr),

    .EXTIN             (timer1_extin),  // Extenal input
    .TIMERINT          (timer1_int)     // interrupt output
  );
  end else
  begin : gen_no_apb_timer_1
    assign timer1_prdata  = {32{1'b0}};
    assign timer1_pready  = 1'b1;
    assign timer1_pslverr = 1'b0;
    assign timer1_int     = 1'b0;
  end endgenerate

  // -----------------------------------------------------------------
  // Dual Timers
  generate if (INCLUDE_APB_DUALTIMER0 == 1) begin : gen_apb_dualtimers_2
  cmsdk_apb_dualtimers u_apb_dualtimers_2 (
   // Inputs
    .PCLK              (PCLKG),
    .PRESETn           (PRESETn),
    .PENABLE           (i_penable),
    .PSEL              (dualtimer2_psel),
    .PADDR             (i_paddr[11:2]),
    .PWRITE            (i_pwrite),
    .PWDATA            (i_pwdata),

    .TIMCLK            (PCLK),
    .TIMCLKEN1         (1'b1), // simple case:the timer 0 clock always enable
    .TIMCLKEN2         (1'b1), // simple case:the timer 1 clock always enable

    .ECOREVNUM         (4'h0),// Engineering-change-order revision bits

   // Outputs
    .PRDATA            (dualtimer2_prdata),

    .TIMINT1           (dualtimer2a_int), // not used
    .TIMINT2           (dualtimer2b_int), // not used
    .TIMINTC           (dualtimer2_comb_int)

  );
  end else
  begin : gen_no_apb_dualtimers_2
    assign dualtimer2_prdata   = {32{1'b0}};
    assign dualtimer2_comb_int = 1'b0;
    assign dualtimer2a_int     = 1'b0;
    assign dualtimer2b_int     = 1'b0;
  end endgenerate

  // When using peripherals with APB (AMBA 2.0), the PREADY and PSLVERR
  // signals are not required. So we connect PREADY to 1 and PSLVERR to 0.
  assign dualtimer2_pslverr = 1'b0;
  assign dualtimer2_pready  = 1'b1;

  // -----------------------------------------------------------------
  // Watchdog

  generate if (INCLUDE_APB_WATCHDOG == 1) begin : gen_apb_watchdog
  cmsdk_apb_watchdog u_apb_watchdog (
   // Inputs
    .PCLK              (PCLKG),
    .PRESETn           (PRESETn),
    .PENABLE           (i_penable),
    .PSEL              (watchdog_psel),
    .PADDR             (i_paddr[11:2]),
    .PWRITE            (i_pwrite),
    .PWDATA            (i_pwdata),

    .WDOGCLK           (PCLK),
    .WDOGCLKEN         (1'b1),
    .WDOGRESn          (PRESETn),

    .ECOREVNUM         (4'h0),// Engineering-change-order revision bits

   // Outputs
    .PRDATA            (watchdog_prdata),

    .WDOGINT           (watchdog_int),  // connect to NMI
    .WDOGRES           (watchdog_rst)   // connect to reset generator

  );


