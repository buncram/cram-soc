module daric_top (

// workmode pads
`ifndef FPGA
    input wire PAD_WMS0,
    input wire PAD_WMS1,
    input wire PAD_WMS2,
`else 
    output wire COREACT,
    output wire SYSACT,
`endif
// external resetn
    input wire PAD_XRSTn,

    input  wire XTAL_IN,
`ifndef FPGA
    inout  wire XTAL_OUT,
`endif
// qspiflash
// USB
// ADC

// swd, d-uart
    output wire PAD_DUART,
    input  wire PAD_SWDCK,
    inout  wire PAD_SWDIO,

    inout wire PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, 
    inout wire PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, 
    inout wire PC0, PC1, PC2, PC3, PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, 
    inout wire PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, 
    inout wire PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7, PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15, 
/*
    inout wire PAD_SDCLK,
    inout wire PAD_SDCMD,
    inout wire PAD_SDDAT0,
    inout wire PAD_SDDAT1,
    inout wire PAD_SDDAT2,
    inout wire PAD_SDDAT3,
*/
`ifdef FPGA
    //`UTMI_IF_DEF
    output wire UTMIPAD_clk0,
    input  wire UTMIPAD_clk1,
//    output wire UTPADMI_databus16_8,
    output wire UTMIPAD_datain0,
    output wire UTMIPAD_datain1,
    output wire UTMIPAD_datain2,
    output wire UTMIPAD_datain3,
    output wire UTMIPAD_datain4,
    output wire UTMIPAD_datain5,
    output wire UTMIPAD_datain6,
    output wire UTMIPAD_datain7,
    input  wire UTMIPAD_dataout0,
    input  wire UTMIPAD_dataout1,
    input  wire UTMIPAD_dataout2,
    input  wire UTMIPAD_dataout3,
    input  wire UTMIPAD_dataout4,
    input  wire UTMIPAD_dataout5,
    input  wire UTMIPAD_dataout6,
    input  wire UTMIPAD_dataout7,
    output wire UTMIPAD_dmpulldown,
    output wire UTMIPAD_dppulldown,
//    output wire UTPADMI_drvvbus,
    input  wire UTMIPAD_hostdisconnect,
    input  wire UTMIPAD_linestate0,
    input  wire UTMIPAD_linestate1,
    output wire UTMIPAD_opmode0,
    output wire UTMIPAD_opmode1,
    input  wire UTMIPAD_rxerror,
    input  wire UTMIPAD_rxvactive,
    input  wire UTMIPAD_rxvalid,
//    input wire UTMPADI_rxvalidh,
    output wire UTMIPAD_suspendm,
    output wire UTMIPAD_termselect,
    output wire UTMIPAD_txready,
    input  wire UTMIPAD_txvalid,
//    wire UTMI_txvaPADlidh,
    output wire UTMIPAD_xcvselect0,
    output wire UTMIPAD_xcvselect1,
`else
    inout wire USB0PN,
    inout wire USB0PP,
`endif
// always on
`ifndef FPGA
    input  wire XTAL32K_IN,
    inout  wire XTAL32K_OUT,
`endif
    input  wire PAD_AOXRSTn,
    inout  wire PF0, PF1, PF2, PF3, PF4, PF5
);

    logic         clkxtl, clkxtl32k;
    logic [0:2]   cmspad;
    logic         chipresetn;
    logic         ao_padresetn;
    logic         padresetn;
    logic         dbgtxd;
    logic         swdclk;
    ioif          swdio();
    ioif          iopad_A[0: 7]();
    ioif          iopad_B[0:15]();
    ioif          iopad_C[0:15]();
    ioif          iopad_D[0:15]();
    ioif          iopad_E[0:15]();
    ioif          iopad_F[0: 5]();
    ioif          ao_iopad_F[0: 5]();
    logic         clkao;
    apbif         apbao();
    ioif          sddc_clk();
    ioif          sddc_cmd();
//    ioif          sddc_dat[3:0]();

    ioif sddc_dat0();
    ioif sddc_dat1();
    ioif sddc_dat2();
    ioif sddc_dat3();




    apbif #(.PAW(12))       apbudp();
    logic         socao_sleep, aopmu_sleep, aowkupvld;

    logic            coreresetn;

`ifdef FPGA
    logic PAD_WMS0; assign PAD_WMS0 = '0;
    logic PAD_WMS1; assign PAD_WMS1 = '0;
    logic PAD_WMS2; assign PAD_WMS2 = '0;
    logic XTAL_OUT;
    logic XTAL32K_IN, XTAL32K_OUT;

    logic sysresetn, clktop;
    logic [4:0] corecnt, syscnt;

    assign sysresetn =  soc.sysresetn;
    assign clktop = soc.clktop;

    assign COREACT = corecnt[4];
    assign SYSACT = syscnt[4];

    `theregfull(clktop, coreresetn, corecnt, '0) <= corecnt + 'h1;
    `theregfull(clktop, sysresetn,  syscnt, '0) <= syscnt + 'h1;

    logic [11:0] clk32kcnt = 0;
    logic clk32kreg = 0;

    always@(posedge XTAL_IN) clk32kcnt <= ( clk32kcnt == 'd750 - 1 ) ? 0 : clk32kcnt + 1;
    always@(posedge XTAL_IN) clk32kreg <= clk32kreg ^ ( clk32kcnt == 'd750 - 1 );
    assign XTAL32K_IN = clk32kreg;

`endif

    logic            utmi_clk;
    logic            u2p0_external_rst;
    logic  [1:0]     utmi_xcvrselect;
    logic            utmi_termselect;
    logic            utmi_suspendm;
    logic  [1:0]     utmi_linestate;
    logic  [1:0]     utmi_opmode;
    logic  [7:0]     utmi_datain7_0;
    logic            utmi_txvalid;
    logic            utmi_txready;
    logic  [7:0]     utmi_dataout7_0;
    logic            utmi_rxvalid;
    logic            utmi_rxactive;
    logic            utmi_rxerror;
    logic            utmi_dppulldown;
    logic            utmi_dmpulldown;
    logic            utmi_hostdisconnect;
    logic            clksys;
    logic            clk24m, pclkudp;

`ifdef FPGA
    `theregfull( clkxtl, porresetn, clk24m, '0 ) <= ~clk24m;
`else
    assign clk24m = clkxtl;
`endif
    


// ░▒▓████████████▓▒░ 
//     soc
// ░▒▓████████████▓▒░ 

    soc_top soc(
    /*        input logic       */  .clkxtl,
    /*        input logic [0:2] */  .cmspad,
    /*        input logic       */  .padresetn,
                                    .clksys    (clksys),
    /*        output logic      */  .dbgtxd,
    /*        input logic       */  .clkswd (swdclk),
                                    .swdio,
    /*        ioif.drive        */  .iopad_A,
    /*        ioif.drive        */  .iopad_B,
    /*        ioif.drive        */  .iopad_C,
    /*        ioif.drive        */  .iopad_D,
    /*        ioif.drive        */  .iopad_E,
                                    .sddc_clk  (sddc_clk),
                                    .sddc_cmd  (sddc_cmd),
                                    .sddc_dat0  (sddc_dat0),
                                    .sddc_dat1  (sddc_dat1),
                                    .sddc_dat2  (sddc_dat2),
                                    .sddc_dat3  (sddc_dat3),
                                    `UTMI_IF_INST
                                    .apbudp,
                                    .pclkudp,
            // ao domain
    /*        input logic       */  .chipresetn,
                                    .coreresetn,
                                    .clkao,
                                    .apbao,
                                    .pmupd(socao_sleep),
    /*        ioif.drive        */  .iopad_F,
                                    .aowkupvld,
                                    .*
    );

// ░▒▓████████████▓▒░ 
//     usbphy
// ░▒▓████████████▓▒░ 


`ifdef FPGA
apbs_null unull(apbudp);

/*    output wire*/ assign UTMIPAD_clk0            = clk24m;
/*    output wire*/ assign UTMIPAD_datain0         = utmi_datain7_0[0];
/*    output wire*/ assign UTMIPAD_datain1         = utmi_datain7_0[1];
/*    output wire*/ assign UTMIPAD_datain2         = utmi_datain7_0[2];
/*    output wire*/ assign UTMIPAD_datain3         = utmi_datain7_0[3];
/*    output wire*/ assign UTMIPAD_datain4         = utmi_datain7_0[4];
/*    output wire*/ assign UTMIPAD_datain5         = utmi_datain7_0[5];
/*    output wire*/ assign UTMIPAD_datain6         = utmi_datain7_0[6];
/*    output wire*/ assign UTMIPAD_datain7         = utmi_datain7_0[7];
/*    output wire*/ assign UTMIPAD_dmpulldown      = utmi_dmpulldown;
/*    output wire*/ assign UTMIPAD_dppulldown      = utmi_dppulldown;
/*    output wire*/ assign UTMIPAD_opmode0         = utmi_opmode[0];
/*    output wire*/ assign UTMIPAD_opmode1         = utmi_opmode[1];
/*    output wire*/ assign UTMIPAD_suspendm        = utmi_suspendm;
/*    output wire*/ assign UTMIPAD_termselect      = utmi_termselect;
/*    output wire*/ assign UTMIPAD_xcvselect0      = utmi_xcvrselect[0];
/*    output wire*/ assign UTMIPAD_xcvselect1      = utmi_xcvrselect[1];
/*    input  wire*/ assign utmi_clk             = UTMIPAD_clk1            ;
/*    input  wire*/ assign utmi_dataout7_0[0]   = UTMIPAD_dataout0        ;
/*    input  wire*/ assign utmi_dataout7_0[1]   = UTMIPAD_dataout1        ;
/*    input  wire*/ assign utmi_dataout7_0[2]   = UTMIPAD_dataout2        ;
/*    input  wire*/ assign utmi_dataout7_0[3]   = UTMIPAD_dataout3        ;
/*    input  wire*/ assign utmi_dataout7_0[4]   = UTMIPAD_dataout4        ;
/*    input  wire*/ assign utmi_dataout7_0[5]   = UTMIPAD_dataout5        ;
/*    input  wire*/ assign utmi_dataout7_0[6]   = UTMIPAD_dataout6        ;
/*    input  wire*/ assign utmi_dataout7_0[7]   = UTMIPAD_dataout7        ;
/*    input  wire*/ assign utmi_hostdisconnect  = UTMIPAD_hostdisconnect  ;
/*    input  wire*/ assign utmi_linestate[0]    = UTMIPAD_linestate0      ;
/*    input  wire*/ assign utmi_linestate[1]    = UTMIPAD_linestate1      ;
/*    input  wire*/ assign utmi_rxerror         = UTMIPAD_rxerror         ;
/*    input  wire*/ assign utmi_rxactive        = UTMIPAD_rxvactive       ;
/*    input  wire*/ assign utmi_rxvalid         = UTMIPAD_rxvalid         ;
/*    input  wire*/ assign utmi_txready         = UTMIPAD_txready         ;
/*    output wire*/ assign UTMIPAD_txvalid         = utmi_txvalid        ;

`else
inno_usb_phy udp(
/*input               */ .utmi_refclk         ( clk24m ),
/*input               */ .POR_reset           ( chipresetn ),
/*input               */ .fss_serialmode_0    ( '0 ),
/*input               */ .fss_txenablez_0     ( '1 ),
/*input               */ .fss_txdata_0        ( '0 ),
/*input               */ .fss_txsezero_0      ( '0 ),
/*output              */ .fss_rxdp_0          ( ),
/*output              */ .fss_rxdm_0          ( ),
/*output              */ .fss_rxrcv_0         ( ),
/*input               */ .utmi_reset_0        ( u2p0_external_rst),
/*output              */ .utmi_clk_0          ( utmi_clk),
/*output  [1:0]       */ .utmi_linestat_0     ( utmi_linestate),
/*output              */ .utmi_rxactive_0     ( utmi_rxactive),
/*output              */ .utmi_rxvalid_l_0    ( utmi_rxvalid),
/*output              */ .utmi_rxerror_0      ( utmi_rxerror),
/*output  [7:0]       */ .utmi_rxdata_0       ( utmi_dataout7_0),
/*output              */ .utmi_txready_0      ( utmi_txready),
/*output              */ .utmi_hostdisc_0     ( utmi_hostdisconnect),
/*input               */ .utmi_txvalid_l_0    ( utmi_txvalid),
/*input   [7:0]       */ .utmi_txdata_0       ( utmi_datain7_0),
/*input   [1:0]       */ .utmi_xcvrselect_0   ( utmi_xcvrselect),
/*input               */ .utmi_termselect_0   ( utmi_termselect), //## bist
/*input   [1:0]       */ .utmi_opmode_0       ( utmi_opmode),
/*input               */ .utmi_suspendm_0     ( utmi_suspendm),
/*input               */ .utmi_dppulldown_0   ( utmi_dppulldown),
/*input               */ .utmi_dmpulldown_0   ( utmi_dmpulldown),
/*input               */ .utmi_biston_0       ( '0),
/*input   [1:0]       */ .utmi_testcontrol_0  ( '0),
/*output              */ .utmi_bistdone_0     ( ),
/*output  [1:0]       */ .utmi_status_0       ( ),
/*input               */ .pclk                ( pclkudp),//##
/*input               */ .penable             ( apbudp.penable ),
/*input               */ .psel                ( apbudp.psel ),
/*input               */ .pwrite              ( apbudp.pwrite ), 
/*input               */ .presetn             ( coreresetn),
/*input        [31:0] */ .paddr               ( apbudp.paddr | 32'h0 ),
/*input        [31:0] */ .pwdata              ( apbudp.pwdata ),
/*output       [31:0] */ .prdata              ( apbudp.prdata ),
/*output              */ .clk48m              ( ),
/*output              */ .clk60m              ( ),
/*output              */ .clk12m              ( ),
/*input               */ .dft_clk             ( '0),
/*input               */ .dft_reset           ( '0),
/*input               */ .dft_mode            ( '0),
/*input               */ .dft_se              ( '0),
/*input       [12:0]  */ .dft_si_0            ( '0),
/*output      [12:0]  */ .dft_so_0            ( ),
/*inout               */ .USB0PN              ( USB0PN ),
/*inout               */ .USB0PP              ( USB0PP )
);
        assign apbudp.pready       = 1        ;
        assign apbudp.pslverr      = 0        ;
`endif
// ░▒▓████████████▓▒░ 
//     pads
// ░▒▓████████████▓▒░ 


    pad_frame pad(

    // workmode pads
        .PAD_WMS0,
        .PAD_WMS1,
        .PAD_WMS2,

    // external resetn
        .PAD_XRSTn,
        .XTAL_IN,
        .XTAL_OUT,

    // qspiflash
    // SDDC
        .PAD_SDCLK,
        .PAD_SDCMD,
        .PAD_SDDAT0,
        .PAD_SDDAT1,
        .PAD_SDDAT2,
        .PAD_SDDAT3,

    // ADC

    // swd, d-uart
        .PAD_DUART,
        .PAD_SWDCK,
        .PAD_SWDIO,

        .PA0, .PA1, .PA2, .PA3, .PA4, .PA5, .PA6, .PA7, 
        .PB0, .PB1, .PB2, .PB3, .PB4, .PB5, .PB6, .PB7, .PB8, .PB9, .PB10, .PB11, .PB12, .PB13, .PB14, .PB15, 
        .PC0, .PC1, .PC2, .PC3, .PC4, .PC5, .PC6, .PC7, .PC8, .PC9, .PC10, .PC11, .PC12, .PC13, .PC14, .PC15, 
        .PD0, .PD1, .PD2, .PD3, .PD4, .PD5, .PD6, .PD7, .PD8, .PD9, .PD10, .PD11, .PD12, .PD13, .PD14, .PD15, 
        .PE0, .PE1, .PE2, .PE3, .PE4, .PE5, .PE6, .PE7, .PE8, .PE9, .PE10, .PE11, .PE12, .PE13, .PE14, .PE15, 

    // always on
        .XTAL32K_IN,
        .XTAL32K_OUT,
        .PAD_AOXRSTn,
        .PF0, .PF1, .PF2, .PF3, .PF4, .PF5,

    // to Soc_Top

    /*    output logic      */  .clkxtl,
    /*    output logic [0:2]*/  .cmspad,
    /*    output logic      */  .padresetn,
    /*    input  logic      */  .dbgtxd,
    /*    output logic      */  .swdclk,
    /*    ioif.load         */  .swdio,
    /*    ioif.load         */  .iopad_A,//[0: 7],
    /*    ioif.load         */  .iopad_B,//[0:15],
    /*    ioif.load         */  .iopad_C,//[0:15],
    /*    ioif.load         */  .iopad_D,//[0:15],
    /*    ioif.load         */  .iopad_E,//[0:15],
                                .sddc_clk  (sddc_clk),
                                .sddc_cmd  (sddc_cmd),
                                .sddc_dat0  (sddc_dat0),
                                .sddc_dat1  (sddc_dat1),
                                .sddc_dat2  (sddc_dat2),
                                .sddc_dat3  (sddc_dat3),
    // to AO_Top
        
    /*    output logic      */  .clkxtl32k,
    /*    output logic      */  .ao_padresetn,
    /*    ioif.load         */  .ao_iopad_F//[0: 5]

    );



// ░▒▓████████████▓▒░ 
//     always on
// ░▒▓████████████▓▒░ 
    logic porresetn;

    ao_top ao(
        .clkao      (clkao),
        .apbs       (apbao),
        .clkxtl32k,
        .socpad     (iopad_F),
        .aopad      (ao_iopad_F),
        .porresetn  (porresetn),
        .padresetn  (ao_padresetn),
        .socresetn  (chipresetn),
        .pmupd      (socao_sleep),
        .pmusleep   (aopmu_sleep),
        .aowkupvld
    );

// ░▒▓████████████▓▒░ 
//     PMU
// ░▒▓████████████▓▒░ 

// ##aopmu_sleep

    pmu pmu(
        .porresetn(porresetn),
        .sleep(aopmu_sleep)
        );

endmodule
