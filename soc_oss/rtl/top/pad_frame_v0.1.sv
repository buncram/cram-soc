module pad_frame (

// workmode pads
    input wire PAD_WMS0,
    input wire PAD_WMS1,
    input wire PAD_WMS2,

// external resetn
    input wire PAD_XRSTn,

    input  wire XTAL_IN,
    output wire XTAL_OUT,

// qspiflash
// USB
// ADC

// swd, d-uart
    output wire PAD_DUART,
    input  wire PAD_SWDCK,
    inout  wire PAD_SWDIO,

    inout wire PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, 
    inout wire PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, 
    inout wire PC0, PC1, PC2, PC3, PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, 
    inout wire PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, 
    inout wire PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7, PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15, 

    input wire PAD_SDCLK,
    input wire PAD_SDCMD,
    inout wire PAD_SDDAT0,
    inout wire PAD_SDDAT1,
    inout wire PAD_SDDAT2,
    inout wire PAD_SDDAT3,

// always on
    input  wire XTAL32K_IN,
    output wire XTAL32K_OUT,
    input  wire PAD_AOXRSTn,
    inout  wire PF0, PF1, PF2, PF3, PF4, PF5,

// to Soc_Top

    output logic        clkxtl,
    output logic [0:2]  cmspad,
    output logic        padresetn,

    input  logic        dbgtxd,
    output logic        swdclk,
    ioif.load           swdio,

    ioif.load           iopad_A[0: 7],
    ioif.load           iopad_B[0:15],
    ioif.load           iopad_C[0:13],
    ioif.load           iopad_D[0:15],
    ioif.load           iopad_E[0:15],

    ioif.load           sddc_clk,
    ioif.load           sddc_cmd,
//    ioif.load           sddc_dat[3:0],
    ioif.load           sddc_dat0,
    ioif.load           sddc_dat1,
    ioif.load           sddc_dat2,
    ioif.load           sddc_dat3,
// to AO_Top
    
    output logic        clkxtl32k,
    output logic        ao_padresetn,
    ioif.load           ao_iopad_F[0: 5]

);


    padcell_xtal pad_xtal( .padxin(XTAL_IN), .padxout(XTAL_OUT), .pc(clkxtl) );
    padcell_i #(.pu('0), .pd('1)) pad_cmspad0( .pad( PAD_WMS0 ), .pi( cmspad[0] ));
    padcell_i #(.pu('0), .pd('1)) pad_cmspad1( .pad( PAD_WMS1 ), .pi( cmspad[1] ));
    padcell_i #(.pu('0), .pd('1)) pad_cmspad2( .pad( PAD_WMS2 ), .pi( cmspad[2] )); 

    padcell_i  #(.pu(1'b1)) pad_xrstn  ( .pad( PAD_XRSTn ), .pi( padresetn )); 
    padcell_i  #(.pu(1'b1)) pad_swdck  ( .pad( PAD_SWDCK ), .pi( swdclk )); 
    padcell_o  pad_duart  ( .pad( PAD_DUART ), .po( dbgtxd )); 

    padcell_io pad_swdio  ( .pad( PAD_SWDIO ), .pio( swdio )); 

    padcell_io pad_PA0  ( .pad( PA0  ), .pio( iopad_A[0 ] ));
    padcell_io pad_PA1  ( .pad( PA1  ), .pio( iopad_A[1 ] ));
    padcell_io pad_PA2  ( .pad( PA2  ), .pio( iopad_A[2 ] ));
    padcell_io pad_PA3  ( .pad( PA3  ), .pio( iopad_A[3 ] ));
    padcell_io pad_PA4  ( .pad( PA4  ), .pio( iopad_A[4 ] ));
    padcell_io pad_PA5  ( .pad( PA5  ), .pio( iopad_A[5 ] ));
    padcell_io pad_PA6  ( .pad( PA6  ), .pio( iopad_A[6 ] ));
    padcell_io pad_PA7  ( .pad( PA7  ), .pio( iopad_A[7 ] ));
    padcell_io pad_PB0  ( .pad( PB0  ), .pio( iopad_B[0 ] ));
    padcell_io pad_PB1  ( .pad( PB1  ), .pio( iopad_B[1 ] ));
    padcell_io pad_PB2  ( .pad( PB2  ), .pio( iopad_B[2 ] ));
    padcell_io pad_PB3  ( .pad( PB3  ), .pio( iopad_B[3 ] ));
    padcell_io pad_PB4  ( .pad( PB4  ), .pio( iopad_B[4 ] ));
    padcell_io pad_PB5  ( .pad( PB5  ), .pio( iopad_B[5 ] ));
    padcell_io pad_PB6  ( .pad( PB6  ), .pio( iopad_B[6 ] ));
    padcell_io pad_PB7  ( .pad( PB7  ), .pio( iopad_B[7 ] ));
    padcell_io pad_PB8  ( .pad( PB8  ), .pio( iopad_B[8 ] ));
    padcell_io pad_PB9  ( .pad( PB9  ), .pio( iopad_B[9 ] ));
    padcell_io pad_PB10 ( .pad( PB10 ), .pio( iopad_B[10] ));
    padcell_io pad_PB11 ( .pad( PB11 ), .pio( iopad_B[11] ));
    padcell_io pad_PB12 ( .pad( PB12 ), .pio( iopad_B[12] ));
    padcell_io pad_PB13 ( .pad( PB13 ), .pio( iopad_B[13] ));
    padcell_io pad_PB14 ( .pad( PB14 ), .pio( iopad_B[14] ));
    padcell_io pad_PB15 ( .pad( PB15 ), .pio( iopad_B[15] ));
    padcell_io pad_PC0  ( .pad( PC0  ), .pio( iopad_C[0 ] ));
    padcell_io pad_PC1  ( .pad( PC1  ), .pio( iopad_C[1 ] ));
    padcell_io pad_PC2  ( .pad( PC2  ), .pio( iopad_C[2 ] ));
    padcell_io pad_PC3  ( .pad( PC3  ), .pio( iopad_C[3 ] ));
    padcell_io pad_PC4  ( .pad( PC4  ), .pio( iopad_C[4 ] ));
    padcell_io pad_PC5  ( .pad( PC5  ), .pio( iopad_C[5 ] ));
    padcell_io pad_PC6  ( .pad( PC6  ), .pio( iopad_C[6 ] ));
    padcell_io pad_PC7  ( .pad( PC7  ), .pio( iopad_C[7 ] ));
    padcell_io pad_PC8  ( .pad( PC8  ), .pio( iopad_C[8 ] ));
    padcell_io pad_PC9  ( .pad( PC9  ), .pio( iopad_C[9 ] ));
    padcell_io pad_PC10 ( .pad( PC10 ), .pio( iopad_C[10] ));
    padcell_io pad_PC11 ( .pad( PC11 ), .pio( iopad_C[11] ));
    padcell_io pad_PC12 ( .pad( PC12 ), .pio( iopad_C[12] ));
    padcell_io pad_PC13 ( .pad( PC13 ), .pio( iopad_C[13] ));
    padcell_io pad_PD0  ( .pad( PD0  ), .pio( iopad_D[0 ] ));
    padcell_io pad_PD1  ( .pad( PD1  ), .pio( iopad_D[1 ] ));
    padcell_io pad_PD2  ( .pad( PD2  ), .pio( iopad_D[2 ] ));
    padcell_io pad_PD3  ( .pad( PD3  ), .pio( iopad_D[3 ] ));
    padcell_io pad_PD4  ( .pad( PD4  ), .pio( iopad_D[4 ] ));
    padcell_io pad_PD5  ( .pad( PD5  ), .pio( iopad_D[5 ] ));
    padcell_io pad_PD6  ( .pad( PD6  ), .pio( iopad_D[6 ] ));
    padcell_io pad_PD7  ( .pad( PD7  ), .pio( iopad_D[7 ] ));
    padcell_io pad_PD8  ( .pad( PD8  ), .pio( iopad_D[8 ] ));
    padcell_io pad_PD9  ( .pad( PD9  ), .pio( iopad_D[9 ] ));
    padcell_io pad_PD10 ( .pad( PD10 ), .pio( iopad_D[10] ));
    padcell_io pad_PD11 ( .pad( PD11 ), .pio( iopad_D[11] ));
    padcell_io pad_PD12 ( .pad( PD12 ), .pio( iopad_D[12] ));
    padcell_io pad_PD13 ( .pad( PD13 ), .pio( iopad_D[13] ));
    padcell_io pad_PD14 ( .pad( PD14 ), .pio( iopad_D[14] ));
    padcell_io pad_PD15 ( .pad( PD15 ), .pio( iopad_D[15] ));
    padcell_io pad_PE0  ( .pad( PE0  ), .pio( iopad_E[0 ] ));
    padcell_io pad_PE1  ( .pad( PE1  ), .pio( iopad_E[1 ] ));
    padcell_io pad_PE2  ( .pad( PE2  ), .pio( iopad_E[2 ] ));
    padcell_io pad_PE3  ( .pad( PE3  ), .pio( iopad_E[3 ] ));
    padcell_io pad_PE4  ( .pad( PE4  ), .pio( iopad_E[4 ] ));
    padcell_io pad_PE5  ( .pad( PE5  ), .pio( iopad_E[5 ] ));
    padcell_io pad_PE6  ( .pad( PE6  ), .pio( iopad_E[6 ] ));
    padcell_io pad_PE7  ( .pad( PE7  ), .pio( iopad_E[7 ] ));
    padcell_io pad_PE8  ( .pad( PE8  ), .pio( iopad_E[8 ] ));
    padcell_io pad_PE9  ( .pad( PE9  ), .pio( iopad_E[9 ] ));
    padcell_io pad_PE10 ( .pad( PE10 ), .pio( iopad_E[10] ));
    padcell_io pad_PE11 ( .pad( PE11 ), .pio( iopad_E[11] ));
    padcell_io pad_PE12 ( .pad( PE12 ), .pio( iopad_E[12] ));
    padcell_io pad_PE13 ( .pad( PE13 ), .pio( iopad_E[13] ));
    padcell_io pad_PE14 ( .pad( PE14 ), .pio( iopad_E[14] ));
    padcell_io pad_PE15 ( .pad( PE15 ), .pio( iopad_E[15] ));

    padcell_io pad_SDCLK  ( .pad( PAD_SDCLK  ), .pio( sddc_clk ));
    padcell_io pad_SDCMD  ( .pad( PAD_SDCMD  ), .pio( sddc_cmd ));
    padcell_io pad_SDDAT0 ( .pad( PAD_SDDAT0 ), .pio( sddc_dat0 ));
    padcell_io pad_SDDAT1 ( .pad( PAD_SDDAT1 ), .pio( sddc_dat1 ));
    padcell_io pad_SDDAT2 ( .pad( PAD_SDDAT2 ), .pio( sddc_dat2 ));
    padcell_io pad_SDDAT3 ( .pad( PAD_SDDAT3 ), .pio( sddc_dat3 ));

// always on

    padcell_xtal pad_xtal32k( .padxin(XTAL32K_IN), .padxout(XTAL32K_OUT), .pc(clkxtl32k) );
    padcell_i #(.pu(1))  pad_aoxrstn  ( .pad( PAD_AOXRSTn ), .pi( ao_padresetn )); 
    padcell_io pad_PF0  ( .pad( PF0  ), .pio( ao_iopad_F[0 ] ));
    padcell_io pad_PF1  ( .pad( PF1  ), .pio( ao_iopad_F[1 ] ));
    padcell_io pad_PF2  ( .pad( PF2  ), .pio( ao_iopad_F[2 ] ));
    padcell_io pad_PF3  ( .pad( PF3  ), .pio( ao_iopad_F[3 ] ));
    padcell_io pad_PF4  ( .pad( PF4  ), .pio( ao_iopad_F[4 ] ));
    padcell_io pad_PF5  ( .pad( PF5  ), .pio( ao_iopad_F[5 ] ));

endmodule
