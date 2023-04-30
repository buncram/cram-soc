module pio_ahb #(
    parameter AW = 12
)(
    input logic clk,
    input logic resetn,
    input logic cmatpg, cmbist,

    input  wire          [31:0] gpio_in,
    output wire          [31:0] gpio_out,
    output wire          [31:0] gpio_dir,
    output wire                 irq0,
    output wire                 irq1,

    input  wire        [AW-1:0] PADDR,     // APB Address
    input  wire                 PENABLE,   // APB Enable
    input  wire                 PWRITE,    // APB Write
    input  wire           [3:0] PSTRB,     // APB Byte Strobe
    input  wire           [2:0] PPROT,     // APB Prot
    input  wire          [31:0] PWDATA,    // APB write data
    input  wire                 PSEL,      // APB Select
    input  wire                 APBACTIVE, // APB bus is active, for clock gating
                                           // of APB bus
                                           // APB Input
    output wire          [31:0] PRDATA,    // Read data for each APB slave
    output wire                 PREADY,    // Ready for each APB slave
    output wire                 PSLVERR   // Error state for each APB slave
);

    apbif #(.PAW(AW)) theapb();

    apb_wire2ifm #(
      .AW(AW)
     )apbtrans(
        .apbmaster    (theapb),
        .psel         (PSEL),
        .paddr        (PADDR),
        .penable      (PENABLE),
        .pwrite       (PWRITE),
        .pstrb        (PSTRB),
        .pprot        (PPROT),
        .pwdata       (PWDATA),
        .apbactive    (APBACTIVE),
        .prdata       (PRDATA),
        .pready       (PREADY),
        .pslverr      (PSLVERR)
    );

    pio_wrap pio_wrap(
        .clk     ,
        .resetn  ,
        .cmatpg  ,
        .cmbist  ,
        .gpio_in (gpio_in),
        .gpio_out(gpio_out),
        .gpio_dir(gpio_dir),
        .irq0    (irq0),
        .irq1    (irq1),
        .apbs    (theapb),
        .apbx    (theapb)
    );
endmodule

module pio_wrap(
    input logic         clk,
    input logic         resetn,
    input logic cmatpg, cmbist,

    input  logic [31:0] gpio_in,
    output logic [31:0] gpio_out,
    output logic [31:0] gpio_dir,
    output logic        irq0,
    output logic        irq1,

    apbif.slavein       apbs,
    apbif.slave         apbx
);
    // ---- apb -> peripheral wires ----
    wire [1:0]       mindex;
    wire [31:0]      din;
    wire [4:0]       index;
    wire [3:0]       action;
    wire [7:0]       irq_force_pulse;
    wire [11:0]      irq0_inte;
    wire [11:0]      irq0_intf;
    wire [11:0]      irq1_inte;
    wire [11:0]      irq1_intf;
    wire             sync_bypass;
    wire [31:0]      dout;
    wire  [11:0]     irq0_ints;
    wire  [11:0]     irq1_ints;
    wire  [3:0]      tx_empty;
    wire  [3:0]      tx_full;
    wire  [3:0]      rx_empty;
    wire  [3:0]      rx_full;
    wire  [2:0]      rx_level0;
    wire  [2:0]      rx_level1;
    wire  [2:0]      rx_level2;
    wire  [2:0]      rx_level3;
    wire  [2:0]      tx_level0;
    wire  [2:0]      tx_level1;
    wire  [2:0]      tx_level2;
    wire  [2:0]      tx_level3;
    wire  [3:0]      pclk; // unused, this is mostly for debug purposes

    // ---- SFR bank ----
    logic pclk;
    assign pclk = clk;
    logic apbrd, apbwr, sfrlock;
    assign sfrlock = '0;

    `apbs_common;
    assign  apbx.prdata = '0 |
            sfr_din.prdata32 |
            sfr_op.prdata32 |
            // sfr_exec.prdata32 |
            sfr_dout.prdata32 |
            sfr_bypass.prdata32 |
            sfr_irq_force_pulse.prdata32 |
            sfr_irq0_inte.prdata32 |
            sfr_irq0_intf.prdata32 |
            sfr_irq1_inte.prdata32 |
            sfr_irq1_intf.prdata32 |
            sfr_irq_ints.prdata32 |
            sfr_fstat.prdata32 |
            sfr_flevel.prdata32
            ;

    bit do_action;

    apb_cr #(.A('h00), .DW(32))      sfr_din              (.cr(din), .prdata32(),.*);
    apb_cr #(.A('h04), .DW(11))      sfr_op               (.cr({mindex, index, action}), .prdata32(),.*);
    apb_ar #(.A('h08), .AR(32'hD100353C)) sfr_exec        (.ar(do_action), .*);
    apb_sr #(.A('h0C), .DW(32))      sfr_dout             (.sr(out), .prdata32(),.*);
    apb_cr #(.A('h10), .DW(1))       sfr_bypass           (.cr(sync_bypass), .prdata32(),.*);
    apb_cr #(.A('h14), .DW(8))       sfr_irq_force_pulse  (.cr(irq_force_pulse), .prdata32(),.*);
    apb_cr #(.A('h18), .DW(12))      sfr_irq0_inte        (.cr(irq0_inte), .prdata32(),.*);
    apb_cr #(.A('h1C), .DW(12))      sfr_irq0_intf        (.cr(irq0_intf), .prdata32(),.*);
    apb_cr #(.A('h20), .DW(12))      sfr_irq1_inte        (.cr(irq1_inte), .prdata32(),.*);
    apb_cr #(.A('h24), .DW(12))      sfr_irq1_intf        (.cr(irq1_intf), .prdata32(),.*);
    apb_sr #(.A('h28), .DW(32))      sfr_irq_ints         (.sr({4'd0, irq1_ints, 4'd0, irq0_ints}), .prdata32(),.*);
    apb_sr #(.A('h2C), .DW(32))      sfr_fstat            (.sr({4'd0, tx_empty, 4'd0, tx_full, 4'd0, rx_empty, 4'd0, rx_full}), .prdata32(),.*);
    apb_sr #(.A('h30), .DW(32))      sfr_flevel           (.sr({rx_level3, tx_level3, rx_level2, tx_level2, rx_level1, tx_level1, rx_level0, tx_level0}), .prdata32(),.*);

    // ----- peripheral module ------
    pio pio (
        .clk            (clk            ),
        .reset          (!resetn        ),
        .mindex         (mindex         ),
        .din            (din            ),
        .index          (index          ),
        .action         (action         ),
        .irq_force_pulse(irq_force_pulse),
        .irq0_inte      (irq0_inte      ),
        .irq0_intf      (irq0_intf      ),
        .irq1_inte      (irq1_inte      ),
        .irq1_intf      (irq1_intf      ),
        .sync_bypass    (sync_bypass    ),
        .gpio_in        (gpio_in        ),
        .gpio_out       (gpio_out       ),
        .gpio_dir       (gpio_dir       ),
        .dout           (dout           ),
        .irq0           (irq0           ),
        .irq1           (irq1           ),
        .irq0_ints      (irq0_ints      ),
        .irq1_ints      (irq1_ints      ),
        .tx_empty       (tx_empty       ),
        .tx_full        (tx_full        ),
        .rx_empty       (rx_empty       ),
        .rx_full        (rx_full        ),
        .rx_level0      (rx_level0      ),
        .rx_level1      (rx_level1      ),
        .rx_level2      (rx_level2      ),
        .rx_level3      (rx_level3      ),
        .tx_level0      (tx_level0      ),
        .tx_level1      (tx_level1      ),
        .tx_level2      (tx_level2      ),
        .tx_level3      (tx_level3      ),
        .pclk           (pclk           )
    )
endmodule

module apb_wire2ifm #(
    parameter AW=16,
    parameter DW=32
    )(
    apbif.master            apbmaster,
    input  logic            psel         ,
    input  logic [AW-1:0]   paddr        ,
    input  logic            penable      ,
    input  logic            pwrite       ,
    input  logic [3:0]      pstrb        ,
    input  logic [2:0]      pprot        ,
    input  logic [31:0]     pwdata       ,
    input  logic            apbactive    ,
    output logic [DW-1:0]   prdata       ,
    output logic            pready       ,
    output logic            pslverr
);

    assign apbmaster.psel      = psel          ;
    assign apbmaster.paddr     = paddr         ;
    assign apbmaster.penable   = penable       ;
    assign apbmaster.pwrite    = pwrite        ;
    assign apbmaster.pstrb     = pstrb         ;
    assign apbmaster.pprot     = pprot         ;
    assign apbmaster.pwdata    = pwdata        ;
    assign apbmaster.apbactive = apbactive     ;
    assign prdata       = apbmaster.prdata       ;
    assign pready       = apbmaster.pready       ;
    assign pslverr      = apbmaster.pslverr      ;

endmodule
