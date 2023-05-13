// SPDX-FileCopyrightText: 2023 Cramium Labs, Inc.
// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// Integration wrapper for rp_pio

module pio_ahb #(
    parameter AW = 12
)(
    input logic clk,   // clock of the PIO block itself
    input logic pclk,  // clock of the AHB bus
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

    ioif  pio_gpio[0:31]();
    generate
        for (genvar j = 0; j < 32; j++) begin:gp
            assign gpio_out[j] = pio_gpio[j].po;
            assign gpio_dir[j] = pio_gpio[j].oe;
            assign pio_gpio[j].pi = gpio_in[j];
        end
    endgenerate

    rp_pio rp_pio(
        .clk     ,
        .pclk    ,
        .resetn  ,
        .cmatpg  ,
        .cmbist  ,
        .pio_gpio,
        .irq0    (irq0),
        .irq1    (irq1),
        .apbs    (theapb),
        .apbx    (theapb)
    );
endmodule
