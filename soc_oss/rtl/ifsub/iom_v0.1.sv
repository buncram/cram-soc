    `include "amba_interface_def_v0.1.sv"
    `include "io_interface_def_v0.1.sv"
    `include "template.sv"

module iom #(
    parameter IFCNT=32,
    parameter IOCNT=16,
    parameter INTCNT=4
    )(
        input       pclk,
        input       resetn,
        input       syscfglock,
        apbif.slave   apbslave,

        ioif.load   ifpad  [0:IFCNT-1],       // 0 would be ignored by gpio func
        ioif.drive  iopad  [0:IOCNT-1],
        output bit  iomint            ,
        output bit  iopi   [0:IOCNT-1]
    );

    bit     clk;
    assign clk = pclk;

  //
  //  parameter
  //  ==


    localparam IFCNTW = $clog2( IFCNT );
    localparam IOCNTW = $clog2( IOCNT );
    localparam INTCNTW = $clog2( INTCNT );
    localparam SFRGPIODRCNT = IOCNTW / 32 + 1;

  //
  //  mux
  //  ==


    bit [0:IOCNT-1][IFCNTW-1:0]    cfg_padmux;
    bit [0:IFCNT-1][0:IOCNT-1]     muxsel, mux2d_pi;
    bit [0:IOCNT-1]                gpio_po, gpio_pi, gpio_oe, gpio_pu;
    bit [0:IFCNT-1][2:0]            ifpad_drive;
    genvar i, j;
    generate
    	for( i = 0; i < IOCNT; i = i + 1) begin: geni
        	for( j = 0; j < IFCNT; j = j + 1) begin: genj
                assign muxsel[j][i] = ( cfg_padmux[i]==j );
                assign mux2d_pi[j][i] = muxsel[j][i] & iopad[i].pi;
        	end

            assign iopad[i].po = cfg_padmux[i] ? ifpad_drive[cfg_padmux[i]][2] : gpio_po[i];
            assign iopad[i].oe = cfg_padmux[i] ? ifpad_drive[cfg_padmux[i]][1] : gpio_oe[i];
            assign iopad[i].pu = cfg_padmux[i] ? ifpad_drive[cfg_padmux[i]][0] : gpio_pu[i];

            assign gpio_pi[i] = ( cfg_padmux[i] == 0 ) ? iopad[i].pi : 0;
            assign iopi[i] = iopad[i].pi;
    	end
    	for( j = 0; j < IFCNT; j = j + 1) begin: genj2
            assign ifpad[j].pi = |mux2d_pi[j];
        	assign ifpad_drive[j] = { ifpad[j].po, ifpad[j].oe, ifpad[j].pu };  
    	end
    endgenerate

  //
  //  int
  //  ==

    bit [0:INTCNT-1][IOCNTW-1:0]    ctl_intsel;
    bit [0:INTCNT-1][1:0]           ctl_intmode;
    bit [0:INTCNT-1]                ctl_inten, ctl_intvld;

    localparam INTMD_RISE = 2'h0;
    localparam INTMD_FALL = 2'h1;
    localparam INTMD_HIGH = 2'h2;
    localparam INTMD_LOW  = 2'h3;

    bit [0:INTCNT-1]    intsrc, intsrcreg0, intsrcreg, intsrcrise, intsrcfall, intvldpre;

    generate
    	for( i = 0; i < INTCNT; i = i + 1) begin: genint
    
        `theregrn( intsrc[i] ) <= iopi[ctl_intsel[i]];
        `theregrn( intsrcreg0[i] ) <= intsrc[i];
        `theregrn( intsrcreg[i] ) <= intsrcreg0[i];
        `theregrn( intsrcrise[i] ) <= intsrcreg0[i] & ~intsrcreg[i];
        `theregrn( intsrcfall[i] ) <= ~intsrcreg0[i] & intsrcreg[i];

        assign intvldpre[i] =
                 ( ctl_intmode[i] == INTMD_HIGH ) ? intsrc[i] :
                 ( ctl_intmode[i] == INTMD_LOW  ) ? ~intsrc[i] :
                 ( ctl_intmode[i] == INTMD_RISE ) ? intsrcrise[i] :
                 ( ctl_intmode[i] == INTMD_FALL ) ? intsrcfall[i] : 1'b0;
        `theregrn( ctl_intvld[i] ) <= ctl_inten[i] & intvldpre[i]; 

    	end
    endgenerate



  //
  //  apb
  //  ==


    localparam GPIO_OE_DEFAULT = 1'b0;
    localparam GPIO_PU_DEFAULT = 1'b0;

    localparam APBADDR_INTCR      = 16'h0010;
    localparam APBADDR_INTSR      = 16'h001f;
    localparam APBADDR_PADCFG     = 16'h1000;
    localparam APBADDR_PADCFGVLD  = 16'h1fff;
    localparam APBADDR_GPIODR     = 16'h2000;

    bit [0:IOCNT-1][31:0]       apbsfr_padcfg   ;
    bit [0:SFRGPIODRCNT-1][31:0]       apbsfr_gpiodr   ;
    bit [0:INTCNT-1][31:0]      apbsfr_intcr    ;
    bit [31:0]                  apbsfr_intsr    ;
    bit [31:0]                  prdata_intsr, prdata_intcr, prdata_gpiodr, prdata_padcfg;

    bit                         padcfgvld;

    bit apbrd, apbwr;
    `apbslave_common;
    assign apbslave.prdata = prdata_intsr | prdata_intcr | prdata_gpiodr | prdata_padcfg;
    assign iomint = |apbsfr_intsr;

    // padcfg : padmux, gpio pu/oe
    apb_sfr #(
            .AW		     ( 16            ),
            .DW		     ( 32            ),
            .IV		     ( { ( 16'h0 | { GPIO_OE_DEFAULT, GPIO_PU_DEFAULT} ), 16'h0 } ),
            .SFRCNT      ( IOCNT        ),
            .SRMASK      ( 32'h0003_0000 | {IFCNTW{1'b1}}  ),      // set write 1 to clr ( for status reg )
            .RMASK       ( 32'hffff_ffff ),      // read mask to remove undefined bit
            .REXTMASK    ( 32'h0         )       // read ext mask
         )uapbsfr_padcfg(
            .pclk        ,
            .resetn      ,
            .apbslave    ,
            .sfrlock     (syscfglock        ),
            .sfrpaddr    (APBADDR_PADCFG    ),
            .sfrprdataext(),
            .sfrsr       (),
            .sfrprdata   (prdata_padcfg     ),
            .sfrdata     (apbsfr_padcfg     )
         );

    // padcfgvld : write non-zero to enable padcfg
    `theregrn( padcfgvld ) <= apbwr & ( apbslave.paddr == APBADDR_PADCFGVLD ) & ~( apbslave.pwdata == 0 ) & ~syscfglock;

    generate
    	for( i = 0; i < IOCNT; i = i + 1) begin: genapbsfr_padcfg
    	    `theregrn( cfg_padmux[i] ) <= padcfgvld ? apbsfr_padcfg[i][IFCNTW-1:0] : cfg_padmux[i];
    	    `theregfull( pclk, resetn, {gpio_oe[i],gpio_pu[i]}, { GPIO_OE_DEFAULT, GPIO_PU_DEFAULT} ) <=  padcfgvld ? apbsfr_padcfg[i][17:16] : {gpio_oe[i],gpio_pu[i]};
    	end
    endgenerate

    // padcfgvld : write non-zero to enable padcfg
    apb_sfr #(
            .AW		     ( 16            ),
            .DW		     ( 32            ),
            .IV		     ( 32'h0         ),
            .SFRCNT      ( SFRGPIODRCNT  ),
            .SRMASK      ( 32'h0         ),      // set write 1 to clr ( for status reg )
            .RMASK       ( 32'hffff_ffff ),      // read mask to remove undefined bit
            .REXTMASK    ( 32'hffff_ffff )       // read ext mask
         )uapbsfr_gpiodr(
            .pclk        (pclk           ),
            .resetn      (resetn         ),
            .apbslave    (apbslave       ),
            .sfrlock     (1'b0           ),
            .sfrpaddr    (APBADDR_GPIODR ),
            .sfrprdataext( {SFRGPIODRCNT{32'h0}} | gpio_pi        ),     // ## little endian
            .sfrsr       (32'h0          ),
            .sfrprdata   (prdata_gpiodr  ),
            .sfrdata     (apbsfr_gpiodr  )
         );

    `theregrn( gpio_po ) <= apbsfr_gpiodr;//##gpio po, little endian

    // intcr, intsr

    apb_sfr #(
            .AW		     ( 16            ),
            .DW		     ( 32            ),
            .IV		     ( 32'h0         ),
            .SFRCNT      ( INTCNT        ),
            .SRMASK      ( 32'h0         ),      // set write 1 to clr ( for status reg )
            .RMASK       ( 32'h8003_0000 | {IOCNTW{1'b1}} ),      // read mask to remove undefined bit
            .REXTMASK    ( 32'hffff_ffff )       // read ext mask
         )uapbsfr_intcr(
            .pclk        (pclk           ),
            .resetn      (resetn         ),
            .apbslave    (apbslave       ),
            .sfrlock     (syscfglock     ),
            .sfrpaddr    (APBADDR_INTCR  ),
            .sfrprdataext(),
            .sfrsr       (),
            .sfrprdata   (prdata_intcr   ),
            .sfrdata     (apbsfr_intcr   )
         );

    apb_sfr #(
            .AW		     ( 16            ),
            .DW		     ( 32            ),
            .IV		     ( 32'h0         ),
            .SFRCNT      ( 1             ),
            .SRMASK      ( 32'h0 | {INTCNT{1'b1}} ),      // set write 1 to clr ( for status reg )
            .RMASK       ( 32'h0 | {INTCNT{1'b1}} ),      // read mask to remove undefined bit
            .REXTMASK    ( 32'h0         )          // read ext mask
         )uapbsfr_intsr(
            .pclk        (pclk           ),
            .resetn      (resetn         ),
            .apbslave    (apbslave       ),
            .sfrlock     (1'b0           ),
            .sfrpaddr    (APBADDR_INTSR  ),
            .sfrprdataext(32'h0          ),
            .sfrsr       (32'h0 | ctl_intvld ),
            .sfrprdata   (prdata_intsr   ),
            .sfrdata     (apbsfr_intsr   )
         );

    generate
    	for( i = 0; i < INTCNT; i = i + 1) begin: genapbsfr_intcr
    	    assign ctl_intsel[i]  = apbsfr_intcr[i][IOCNTW-1:0];
    	    assign ctl_intmode[i] = apbsfr_intcr[i][17:16];
    	    `theregrn( ctl_inten[i] ) <= apbsfr_intcr[i][31];
    	end
    endgenerate
endmodule


`ifdef SIM

module iopad(
    ioif.load   ioload,
    inout       PAD
    );

    wire    padup;

    pullup ( padup );
    bufif1 ( PAD, padup, ioload.pu );

    assign PAD = ioload.oe ? ioload.po : 1'bz;
    assign ioload.pi = PAD;

endmodule

`endif

`ifdef FPGA

module iopad(
    ioif.load   ioload,
    inout       PAD
    );

    wire    PAD;

    assign PAD = ioload.oe ? ioload.po : 1'bz;
    assign ioload.pi = PAD;

endmodule
`endif


module dummytb_iom();
        apbif   apbslave();
        bit pclk,resetn,syscfglock;
        wire pad;


        ioif ifpad[0:31]() ;
        ioif iopad[0:15]() ;
        bit iomint;
        bit iopi    [0:15];

        iom u0( .* );

        iopad u1( .ioload(iopad[0]), .PAD(pad));




endmodule
