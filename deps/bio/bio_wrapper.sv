// SPDX-FileCopyrightText: 2023 Cramium Labs, Inc.
// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// Integration wrapper for bio

module bio_apb #(
    parameter AW = 13
)(
    input logic aclk,  // clock of the BIO block itself
    input logic pclk,  // clock of the APB bus
    input logic resetn,
    input logic cmatpg, cmbist,

    input  wire          [31:0] gpio_in,
    output wire          [31:0] gpio_out,
    output wire          [31:0] gpio_dir,
    output wire           [3:0] irq,

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

    ioif  bio_gpio[0:31]();
    generate
        for (genvar j = 0; j < 32; j++) begin:gp
            assign gpio_out[j] = bio_gpio[j].po;
            assign gpio_dir[j] = bio_gpio[j].oe;
            assign bio_gpio[j].pi = gpio_in[j];
        end
    endgenerate

    bio bio(
        .aclk    ,
        .pclk    ,
        .reset_n (resetn),
        .cmatpg  ,
        .cmbist  ,
        .bio_gpio,
        .irq     (irq),
        .apbs    (theapb),
        .apbx    (theapb)
    );
endmodule
