// SPDX-FileCopyrightText: 2023 Cramium Labs, Inc.
// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// Integration wrapper for bio + dma

module bio_bdma_wrapper #(
    parameter APW = 14,  // APB address width
    parameter DW = 32,
    parameter AHW = 32,  // AHB address width
    parameter IDW = 4,
    parameter UW = 4
)(
    input logic fclk,  // clock of the BIO block itself
    input logic pclk,  // clock of the APB bus
    input logic resetn,
    input logic cmatpg, cmbist,
    input logic [2:0] sramtrm,

    input  wire          [31:0] gpio_in,
    output wire          [31:0] gpio_out,
    output wire          [31:0] gpio_dir,
    output wire           [3:0] irq,

    // AHB master DMA inteface (connected wires)
    output wire  [1:0]     htrans,         // Transfer type
    output wire            hwrite,         // Transfer direction
    output wire  [AHW-1:0] haddr,          // Address bus
    output wire  [2:0]     hsize,          // Transfer size
    output wire  [2:0]     hburst,         // Burst type
    output wire            hmasterlock,    // Locked Sequence
    output bit   [DW-1:0]  hwdata,         // Write data

    input  bit   [DW-1:0]  hrdata,         // Read data bus    // old hready
    input  wire            hready,         // HREADY feedback
    input                  hresp,          // Transfer response
    input        [UW-1:0]  hruser,

    // AHB NC wires
    output wire            hsel,           // Slave Select
    output wire  [3:0]     hprot,          // Protection control
    output wire  [IDW-1:0] hmaster,        //Master select
    input  wire            hreadym,       // Transfer done     // old hreadyin
    output wire  [UW-1:0]  hauser,
    output wire  [UW-1:0]  hwuser,

    // APB configuration interface
    input  wire       [APW-1:0] PADDR,     // APB Address
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

    apbif #(.PAW(APW)) theapb();
    ahbif #(.AW(AHW),.DW(DW),.IDW(IDW),.UW(UW)) dma_ahb32();

    apb_wire2ifm #(
      .AW(APW)
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

    ioif  bio_gpio[31:0]();
    generate
        for (genvar j = 0; j < 32; j++) begin:gp
            assign gpio_out[j] = bio_gpio[j].po;
            assign gpio_dir[j] = bio_gpio[j].oe;
            assign bio_gpio[j].pi = gpio_in[j];
        end
    endgenerate

    bio_bdma bio_bdma(
        .aclk    (fclk),
        .pclk    ,
        .reset_n (resetn),
        .cmatpg  ,
        .cmbist  ,
        .sramtrm ,
        .bio_gpio,
        .irq     (irq),
        .apbs    (theapb),
        .apbx    (theapb),
        .ahbm    (dma_ahb32)
    );
endmodule
