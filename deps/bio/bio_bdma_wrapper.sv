// SPDX-FileCopyrightText: 2023 Cramium Labs, Inc.
// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// Integration wrapper for bio + dma

module bio_bdma_wrapper #(
    parameter APW = 12,  // APB address width
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
    output wire                 PSLVERR,   // Error state for each APB slave

    input  wire       [APW-1:0] IM0_PADDR     ,
    input  wire                 IM0_PENABLE   ,
    input  wire                 IM0_PWRITE    ,
    input  wire           [3:0] IM0_PSTRB     ,
    input  wire           [2:0] IM0_PPROT     ,
    input  wire          [31:0] IM0_PWDATA    ,
    input  wire                 IM0_PSEL      ,
    input  wire                 IM0_APBACTIVE ,
    output wire          [31:0] IM0_PRDATA    ,
    output wire                 IM0_PREADY    ,
    output wire                 IM0_PSLVERR   ,

    input  wire       [APW-1:0] IM1_PADDR     ,
    input  wire                 IM1_PENABLE   ,
    input  wire                 IM1_PWRITE    ,
    input  wire           [3:0] IM1_PSTRB     ,
    input  wire           [2:0] IM1_PPROT     ,
    input  wire          [31:0] IM1_PWDATA    ,
    input  wire                 IM1_PSEL      ,
    input  wire                 IM1_APBACTIVE ,
    output wire          [31:0] IM1_PRDATA    ,
    output wire                 IM1_PREADY    ,
    output wire                 IM1_PSLVERR   ,

    input  wire       [APW-1:0] IM2_PADDR     ,
    input  wire                 IM2_PENABLE   ,
    input  wire                 IM2_PWRITE    ,
    input  wire           [3:0] IM2_PSTRB     ,
    input  wire           [2:0] IM2_PPROT     ,
    input  wire          [31:0] IM2_PWDATA    ,
    input  wire                 IM2_PSEL      ,
    input  wire                 IM2_APBACTIVE ,
    output wire          [31:0] IM2_PRDATA    ,
    output wire                 IM2_PREADY    ,
    output wire                 IM2_PSLVERR   ,

    input  wire       [APW-1:0] IM3_PADDR     ,
    input  wire                 IM3_PENABLE   ,
    input  wire                 IM3_PWRITE    ,
    input  wire           [3:0] IM3_PSTRB     ,
    input  wire           [2:0] IM3_PPROT     ,
    input  wire          [31:0] IM3_PWDATA    ,
    input  wire                 IM3_PSEL      ,
    input  wire                 IM3_APBACTIVE ,
    output wire          [31:0] IM3_PRDATA    ,
    output wire                 IM3_PREADY    ,
    output wire                 IM3_PSLVERR
);

    apbif #(.PAW(APW)) theapb();
    ahbif #(.AW(AHW),.DW(DW),.IDW(IDW),.UW(UW)) dma_ahb32();
    apbif #(.PAW(APW)) apb_imem[4]();

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

    apb_wire2ifm #(
      .AW(APW)
    )apbtrans_im0(
        .apbmaster    (apb_imem[0]),
        .psel         (IM0_PSEL       ),
        .paddr        (IM0_PADDR      ),
        .penable      (IM0_PENABLE    ),
        .pwrite       (IM0_PWRITE     ),
        .pstrb        (IM0_PSTRB      ),
        .pprot        (IM0_PPROT      ),
        .pwdata       (IM0_PWDATA     ),
        .apbactive    (IM0_APBACTIVE  ),
        .prdata       (IM0_PRDATA     ),
        .pready       (IM0_PREADY     ),
        .pslverr      (IM0_PSLVERR    )
    );

    apb_wire2ifm #(
      .AW(APW)
    )apbtrans_im1(
        .apbmaster    (apb_imem[1]),
        .psel         (IM1_PSEL       ),
        .paddr        (IM1_PADDR      ),
        .penable      (IM1_PENABLE    ),
        .pwrite       (IM1_PWRITE     ),
        .pstrb        (IM1_PSTRB      ),
        .pprot        (IM1_PPROT      ),
        .pwdata       (IM1_PWDATA     ),
        .apbactive    (IM1_APBACTIVE  ),
        .prdata       (IM1_PRDATA     ),
        .pready       (IM1_PREADY     ),
        .pslverr      (IM1_PSLVERR    )
    );

    apb_wire2ifm #(
      .AW(APW)
    )apbtrans_im2(
        .apbmaster    (apb_imem[2]),
        .psel         (IM2_PSEL       ),
        .paddr        (IM2_PADDR      ),
        .penable      (IM2_PENABLE    ),
        .pwrite       (IM2_PWRITE     ),
        .pstrb        (IM2_PSTRB      ),
        .pprot        (IM2_PPROT      ),
        .pwdata       (IM2_PWDATA     ),
        .apbactive    (IM2_APBACTIVE  ),
        .prdata       (IM2_PRDATA     ),
        .pready       (IM2_PREADY     ),
        .pslverr      (IM2_PSLVERR    )
    );

    apb_wire2ifm #(
      .AW(APW)
    )apbtrans_im3(
        .apbmaster    (apb_imem[3]),
        .psel         (IM3_PSEL       ),
        .paddr        (IM3_PADDR      ),
        .penable      (IM3_PENABLE    ),
        .pwrite       (IM3_PWRITE     ),
        .pstrb        (IM3_PSTRB      ),
        .pprot        (IM3_PPROT      ),
        .pwdata       (IM3_PWDATA     ),
        .apbactive    (IM3_APBACTIVE  ),
        .prdata       (IM3_PRDATA     ),
        .pready       (IM3_PREADY     ),
        .pslverr      (IM3_PSLVERR    )
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
        .apbs_imem(apb_imem),
        .apbx_imem(apb_imem),
        .ahbm    (dma_ahb32)
    );
endmodule
