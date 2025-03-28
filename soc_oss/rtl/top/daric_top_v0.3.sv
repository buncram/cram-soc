`include "rtl/ifsub/utmi_def_v0.1.sv"
`include "rtl/top/pad_frame_v0.3_arm.sv"
module daric_top (

// workmode pads
`ifndef FPGA
    input wire PAD_WMS0,
    input wire PAD_WMS1,
    input wire PAD_WMS2,
    inout  wire XTAL_OUT,
//    input  wire XTAL24M_IN,
//    inout  wire XTAL24M_OUT,
    input  wire XTAL48M_IN,
    inout  wire XTAL48M_OUT,
`else
    input  wire coresel_cm7,
`endif
// external resetn
    input wire PAD_XRSTn,
    input  wire XTAL_IN,

// qspiflash

    inout wire QFC_SCK   ,
    inout wire QFC_SCKN  ,
    inout wire QFC_QDS   ,//RWDS
    inout wire QFC_SS0   ,
    inout wire QFC_SS1   ,
    inout wire QFC_SIO0  ,
    inout wire QFC_SIO1  ,
    inout wire QFC_SIO2  ,
    inout wire QFC_SIO3  ,
    inout wire QFC_SIO4  ,
    inout wire QFC_SIO5  ,
    inout wire QFC_SIO6  ,
    inout wire QFC_SIO7  ,
//    inout wire QFC_RWDS  ,
    inout wire QFC_INT   ,
    inout wire QFC_RSTM0 ,
    inout wire QFC_RSTS0 ,
////    inout wire QFC_RSTM1 ,
////    inout wire QFC_RSTS1 ,

// USB
// ADC

// swd, d-uart
    output wire PAD_DUART,
    input  wire PAD_SWDCK,
    inout  wire PAD_SWDIO,
    inout  wire PAD_JTCK  ,
    inout  wire PAD_JTMS  ,
    inout  wire PAD_JTDI  ,
    inout  wire PAD_JTDO  ,
    inout  wire PAD_JTRST ,

    inout wire PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7,
    inout wire PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15,
    inout wire PC0, PC1, PC2, PC3, PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15,
    inout wire PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15,
`ifdef MPW
`else
    inout wire PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7, PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15,
`endif
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
    input  wire UTMIPAD_txready,
    output wire UTMIPAD_txvalid,
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
`ifdef MPW
    inout  wire PF0
`else
    inout  wire PF0, PF1, PF2, PF3, PF4, PF5
`endif
);

    `ifdef MPW
    wire PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7, PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15;
    wire PF1, PF2, PF3, PF4, PF5;
    `endif


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
    logic        cmsatpg;
    logic        cmstest;
    logic        cmsbist;
    logic        cmsuser;
    logic        cmsvld;

/*
    ioif          sddc_clk();
    ioif          sddc_cmd();
//    ioif          sddc_dat[3:0]();

    ioif sddc_dat0();
    ioif sddc_dat1();
    ioif sddc_dat2();
    ioif sddc_dat3();
*/

    apbif #(.PAW(12))       apbudp();
    logic         socao_sleep, aopmu_sleep, aowkupvld, aowkupint;

    logic            coreresetn;

`ifdef FPGA
    logic PAD_WMS0; assign PAD_WMS0 = '0;
    logic PAD_WMS1; assign PAD_WMS1 = '0;
    logic PAD_WMS2; assign PAD_WMS2 = '0;
    wire XTAL_OUT;
    wire XTAL32K_IN, XTAL32K_OUT;
//    wire XTAL24M_IN, XTAL24M_OUT;
    wire XTAL48M_IN, XTAL48M_OUT;

    logic sysresetn, clktop;
    logic [4:0] corecnt, syscnt;

    assign sysresetn =  soc.sysresetn;
    assign clktop = soc.clktop;

//    assign COREACT = corecnt[4];
//    assign SYSACT = syscnt[4];

    `theregfull(clktop, coreresetn, corecnt, '0) <= corecnt + 'h1;
    `theregfull(clktop, sysresetn,  syscnt, '0) <= syscnt + 'h1;

    logic [11:0] clk32kcnt = 0;
    logic clk32kreg = 0;

    always@(posedge XTAL_IN) clk32kcnt <= ( clk32kcnt == 'd750 - 1 ) ? 0 : clk32kcnt + 1;
    always@(posedge XTAL_IN) clk32kreg <= clk32kreg ^ ( clk32kcnt == 'd750 - 1 );
    assign XTAL32K_IN = clk32kreg;

    PULLUP upucoresel(coresel_cm7);

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
    logic            clk48m, pclkudp;
    logic            clkxtl48m;
//    logic porresetn;

`ifdef FPGA
//    `theregfull( clkxtl, porresetn, clk24m, '0 ) <= ~clk24m;
    assign clk48m = clkxtl;
`else
//    assign clk24m = '0;//clkxtl24m;
    assign clk48m = clkxtl48m;
`endif

//ioif sddc_clk();
//ioif sddc_cmd();
//ioif sddc_dat0();
//ioif sddc_dat1();
//ioif sddc_dat2();
//ioif sddc_dat3();

    wire [3:0]  ANA_ADC_SI;
    wire [0:1]  ANA_RR_TEST;
    wire        ANA_PMU_POCRR;
    wire        ANA_PMU_IN0P1U;
    wire        ANA_PMU_ANA_TEST;

    jtagif          jtagvex();
    jtagif          jtagipt();
    jtagif          jtagrrc[0:1]();
    logic [0:12]    iptpo13;
    ioif            qfc_sck();
    ioif            qfc_sckn();
    ioif            qfc_dqs();
    ioif            qfc_ss[1:0]();
    ioif            qfc_sio[7:0]();
//    ioif            qfc_rwds();
    ioif            qfc_rstm[1:0]();
    ioif            qfc_rsts[1:0]();
    ioif            qfc_int();

    padcfg_arm_t  padcfg_qfc_sck;
    padcfg_arm_t  padcfg_qfc_qds;
    padcfg_arm_t  padcfg_qfc_ss;
    padcfg_arm_t  padcfg_qfc_sio;
//    padcfg_arm_t  padcfg_qfc_rwds;
    padcfg_arm_t  padcfg_qfc_int;
    padcfg_arm_t  padcfg_qfc_rst;

    padcfg_arm_t  iocfg_A[0: 7];
    padcfg_arm_t  iocfg_B[0:15];
    padcfg_arm_t  iocfg_C[0:15];
    padcfg_arm_t  iocfg_D[0:15];
    padcfg_arm_t  iocfg_E[0:15];
    padcfg_arm_t  iocfg_F[0: 5];

    logic jtagipt_tck;
    logic jtagipt_resetn;
    logic ipttap_shiftdr;
    logic ipttap_updatedr;
    logic ipttap_capturedr;
    logic aotap_sel;
    logic tap_tdm;
    logic aotap_tds;
    logic clksysao;
    logic        aopmutrmset;
    logic [63:0]       aopmutrmdata;
    logic vd_VD09ENA;
    logic vd_VD09TL;
    logic vd_VD09TH;
    logic vd_VD09L;
    logic vd_VD09H;
    logic vd_VD25ENA;
    logic vd_VD25TL;
    logic vd_VD25TH;
    logic vd_VD25L;
    logic vd_VD25H;
    logic vd_VD33ENA;
    logic vd_VD33TL;
    logic vd_VD33TH;
    logic vd_VD33L;
    logic vd_VD33H;
    logic pad_rto_px;
    logic pad_sns_px;
    logic pad_rto_ao;
    logic pad_sns_ao;
    logic pad_rto_qfc;
    logic pad_sns_qfc;
    logic pad_rto_rr1;
    logic pad_sns_rr1;
    logic pad_rto_rr0;
    logic pad_sns_rr0;
    logic pad_rto_pmu;
    logic pad_sns_pmu;

    logic pvsense_reton;
    logic pvsense_retoff;
    logic [6:0] osc_osc32m_cfg;
    logic [0:5] ao_iptpo;

// ░▒▓████████████▓▒░
//     soc
// ░▒▓████████████▓▒░

    soc_top soc(
                                    .ana_rng_0p1u  (ANA_PMU_IN0P1U),
                                    .ana_reramtest (ANA_RR_TEST),
                                    .ana_rrpoc     (ANA_PMU_POCRR),
                                    .ana_adcsrc    (ANA_ADC_SI),
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
//                                    .sddc_clk  (sddc_clk),
//                                    .sddc_cmd  (sddc_cmd),
//                                    .sddc_dat0  (sddc_dat0),
//                                    .sddc_dat1  (sddc_dat1),
//                                    .sddc_dat2  (sddc_dat2),
//                                    .sddc_dat3  (sddc_dat3),
                                    `UTMI_IF_INST
    // qfc
                                    .padcfg_qfc_sck,
                                    .padcfg_qfc_qds,
                                    .padcfg_qfc_ss,
                                    .padcfg_qfc_sio,
////                                    .padcfg_qfc_rwds,
                                    .padcfg_qfc_int,
                                    .padcfg_qfc_rst,

                                    .qfc_sck,
                                    .qfc_sckn,
                                    .qfc_dqs,
                                    .qfc_ss,
                                    .qfc_sio,
//                                    .qfc_rwds,
                                    .qfc_rstm,
                                    .qfc_rsts,
                                    .qfc_int,

    // vex
                                    .jtagvex,
                                    .jtagrrc,
                                    .jtagipt,
                                    .apbudp,
                                    .pclkudp,
                                    .cmsatpg,
                                    .cmstest,
                                    .cmsbist,
                                    .cmsuser,
                                    .cmsdone(cmsvld),

                                    .VD09ENA      ( vd_VD09ENA ),
                                    .VD09TL       ( vd_VD09TL ),
                                    .VD09TH       ( vd_VD09TH ),
                                    .VD09L        ( vd_VD09L ),
                                    .VD09H        ( vd_VD09H ),
                                    .VD25ENA      ( vd_VD25ENA ),
                                    .VD25TL       ( vd_VD25TL ),
                                    .VD25TH       ( vd_VD25TH ),
                                    .VD25L        ( vd_VD25L ),
                                    .VD25H        ( vd_VD25H ),
                                    .VD33ENA      ( vd_VD33ENA ),
                                    .VD33TL       ( vd_VD33TL ),
                                    .VD33TH       ( vd_VD33TH ),
                                    .VD33L        ( vd_VD33L ),
                                    .VD33H        ( vd_VD33H ),

                                    .pad_reton(pvsense_reton),
                                    .pad_retoff(pvsense_retoff),
            // ao domain
                                    .clksysao,
    /*        input logic       */  .chipresetn,
                                    .coreresetn,
                                    .clkao,
                                    .apbao,
                                    .pmusleep(socao_sleep),
    /*        ioif.drive        */  .iopad_F,
                                    .aowkupvld,
                                    .aopmutrmset,
                                    .aopmutrmdata,
                                    .osc_osc32m_cfg,
                                    .ao_iptpo,
                                    .iptpo13           (iptpo13[0:12]),
                                    .jtagipt_tck       (jtagipt_tck),
                                    .jtagipt_resetn    (jtagipt_resetn),
                                    .ipttap_shiftdr    (ipttap_shiftdr),
                                    .ipttap_updatedr   (ipttap_updatedr),
                                    .ipttap_capturedr  (ipttap_capturedr),
                                    .aotap_sel         (aotap_sel),
                                    .tap_tdo           (tap_tdm),
                                    .aotap_tdi         (aotap_tds),

                                    .*
    );

// ░▒▓████████████▓▒░
//     usbphy
// ░▒▓████████████▓▒░


`ifdef FPGA
apbs_null unull(apbudp);

/*    output wire*/ assign UTMIPAD_clk0            = '0;//clk24m;
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
//  .VCCA3P3 (),
//  .VCCCORE (),
//  .VDD (),
//  .VSS (),
//  .VSSA (),
//  .VSSD (),
/*input               */ .utmi_refclk         ( clk48m ),
/*input               */ .POR_reset           ( ~chipresetn ),
/*input               */ .fss_serialmode_0    ( '0 ),
/*input               */ .fss_txenablez_0     ( '0 ),
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
                         .pready              ( apbudp.pready ),
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
//        assign apbudp.pready       = 1        ;
        assign apbudp.pslverr      = 0        ;

`ifdef SIM
inno_usb_phy_mon udp_mon(
//  .VCCA3P3 (),
//  .VCCCORE (),
//  .VDD (),
//  .VSS (),
//  .VSSA (),
//  .VSSD (),
/*input               */ .utmi_refclk         ( udp.utmi_refclk         ),
/*input               */ .POR_reset           ( udp.POR_reset           ),
/*input               */ .fss_serialmode_0    ( udp.fss_serialmode_0    ),
/*input               */ .fss_txenablez_0     ( udp.fss_txenablez_0     ),
/*input               */ .fss_txdata_0        ( udp.fss_txdata_0        ),
/*input               */ .fss_txsezero_0      ( udp.fss_txsezero_0      ),
/*output              */ .fss_rxdp_0          ( udp.fss_rxdp_0          ),
/*output              */ .fss_rxdm_0          ( udp.fss_rxdm_0          ),
/*output              */ .fss_rxrcv_0         ( udp.fss_rxrcv_0         ),
/*input               */ .utmi_reset_0        ( udp.utmi_reset_0        ),
/*output              */ .utmi_clk_0          ( udp.utmi_clk_0          ),
/*output  [1:0]       */ .utmi_linestat_0     ( udp.utmi_linestat_0     ),
/*output              */ .utmi_rxactive_0     ( udp.utmi_rxactive_0     ),
/*output              */ .utmi_rxvalid_l_0    ( udp.utmi_rxvalid_l_0    ),
/*output              */ .utmi_rxerror_0      ( udp.utmi_rxerror_0      ),
/*output  [7:0]       */ .utmi_rxdata_0       ( udp.utmi_rxdata_0       ),
/*output              */ .utmi_txready_0      ( udp.utmi_txready_0      ),
/*output              */ .utmi_hostdisc_0     ( udp.utmi_hostdisc_0     ),
/*input               */ .utmi_txvalid_l_0    ( udp.utmi_txvalid_l_0    ),
/*input   [7:0]       */ .utmi_txdata_0       ( udp.utmi_txdata_0       ),
/*input   [1:0]       */ .utmi_xcvrselect_0   ( udp.utmi_xcvrselect_0   ),
/*input               */ .utmi_termselect_0   ( udp.utmi_termselect_0   ),
/*input   [1:0]       */ .utmi_opmode_0       ( udp.utmi_opmode_0       ),
/*input               */ .utmi_suspendm_0     ( udp.utmi_suspendm_0     ),
/*input               */ .utmi_dppulldown_0   ( udp.utmi_dppulldown_0   ),
/*input               */ .utmi_dmpulldown_0   ( udp.utmi_dmpulldown_0   ),
/*input               */ .utmi_biston_0       ( udp.utmi_biston_0       ),
/*input   [1:0]       */ .utmi_testcontrol_0  ( udp.utmi_testcontrol_0  ),
/*output              */ .utmi_bistdone_0     ( udp.utmi_bistdone_0     ),
/*output  [1:0]       */ .utmi_status_0       ( udp.utmi_status_0       ),
/*input               */ .pclk                ( udp.pclk                ),
/*input               */ .penable             ( udp.penable             ),
/*input               */ .psel                ( udp.psel                ),
/*input               */ .pwrite              ( udp.pwrite              ),
/*input               */ .presetn             ( udp.presetn             ),
/*input        [31:0] */ .paddr               ( udp.paddr               ),
/*input        [31:0] */ .pwdata              ( udp.pwdata              ),
/*output       [31:0] */ .prdata              ( udp.prdata              ),
/*output              */ .clk48m              ( udp.clk48m              ),
/*output              */ .clk60m              ( udp.clk60m              ),
/*output              */ .clk12m              ( udp.clk12m              ),
/*input               */ .dft_clk             ( udp.dft_clk             ),
/*input               */ .dft_reset           ( udp.dft_reset           ),
/*input               */ .dft_mode            ( udp.dft_mode            ),
/*input               */ .dft_se              ( udp.dft_se              ),
/*input       [12:0]  */ .dft_si_0            ( udp.dft_si_0            ),
/*output      [12:0]  */ .dft_so_0            ( udp.dft_so_0            ),
/*inout               */ .USB0PN              ( udp.USB0PN              ),
/*inout               */ .USB0PP              ( udp.USB0PP              )
);
`endif


`endif
// ░▒▓████████████▓▒░
//     pads
// ░▒▓████████████▓▒░

`ifndef FPGA
powerpad powerpad(
    .rto_px(pad_rto_px),
    .sns_px(pad_sns_px),
    .rto_ao(pad_rto_ao),
    .sns_ao(pad_sns_ao),
    .rto_qfc(pad_rto_qfc),
    .sns_qfc(pad_sns_qfc),
    .rto_rr1(pad_rto_rr1),
    .sns_rr1(pad_sns_rr1),
    .rto_rr0(pad_rto_rr0),
    .sns_rr0(pad_sns_rr0),
    .rto_pmu(pad_rto_pmu),
    .sns_pmu(pad_sns_pmu),

    .pvsense_reton,
    .pvsense_retoff
);
`endif


    pad_frame pad(

    // workmode pads
        .PAD_WMS0,
        .PAD_WMS1,
        .PAD_WMS2,

    // external resetn
        .PAD_XRSTn,
        .XTAL_IN,
        .XTAL_OUT   ,
        .XTAL48M_IN ,
        .XTAL48M_OUT,

    // qspiflash
        .QFC_SCK   ,
        .QFC_SCKN  ,
        .QFC_QDS   ,
        .QFC_SS0   ,
        .QFC_SS1   ,
        .QFC_SIO0  ,
        .QFC_SIO1  ,
        .QFC_SIO2  ,
        .QFC_SIO3  ,
        .QFC_SIO4  ,
        .QFC_SIO5  ,
        .QFC_SIO6  ,
        .QFC_SIO7  ,
////        .QFC_RWDS  ,
        .QFC_INT   ,
        .QFC_RSTM0 ,
        .QFC_RSTS0 ,
//        .QFC_RSTM1 ,
//        .QFC_RSTS1 ,

    // SDDC
/*
        .PAD_SDCLK,
        .PAD_SDCMD,
        .PAD_SDDAT0,
        .PAD_SDDAT1,
        .PAD_SDDAT2,
        .PAD_SDDAT3,
*/
    // ADC

    // swd, d-uart
        .PAD_DUART,
        .PAD_SWDCK,
        .PAD_SWDIO,
        .PAD_JTCK ,
        .PAD_JTMS ,
        .PAD_JTDI ,
        .PAD_JTDO ,
        .PAD_JTRST,

        .PA0, .PA1, .PA2, .PA3, .PA4, .PA5, .PA6, .PA7,
        .PB0, .PB1, .PB2, .PB3, .PB4, .PB5, .PB6, .PB7, .PB8, .PB9, .PB10, .PB11, .PB12, .PB13, .PB14, .PB15,
        .PC0, .PC1, .PC2, .PC3, .PC4, .PC5, .PC6, .PC7, .PC8, .PC9, .PC10, .PC11, .PC12, .PC13, .PC14, .PC15,
        .PD0, .PD1, .PD2, .PD3, .PD4, .PD5, .PD6, .PD7, .PD8, .PD9, .PD10, .PD11, .PD12, .PD13, .PD14, .PD15,
        .PE0, .PE1, .PE2, .PE3, .PE4, .PE5, .PE6, .PE7, .PE8, .PE9, .PE10, .PE11, .PE12, .PE13, .PE14, .PE15,

    // always on
//        .XTAL32K_IN,
//        .XTAL32K_OUT,
//        .PAD_AOXRSTn,
//        .PF0, .PF1, .PF2, .PF3, .PF4, .PF5,

    // to Soc_Top

    /*    output logic      */  .clkxtl,
    /*    output logic [0:2]*/  .cmspad,
    /*    output logic      */  .padresetn,
                                .clkxtl48m,
                                .cmstest,
                                .cmsvld,
                                .ipto13(iptpo13[0:12]),

    /*    output logic      */  .swdclk,
    /*    ioif.load         */  .swdio,
    /*    input  logic      */  .dbgtxd,
                                .jtagvex,
                                .jtagrrc,
                                .jtagipt,

    /*    ioif.load         */  .iopad_A,//[0: 7],
    /*    ioif.load         */  .iopad_B,//[0:15],
    /*    ioif.load         */  .iopad_C,//[0:15],
    /*    ioif.load         */  .iopad_D,//[0:15],
    /*    ioif.load         */  .iopad_E,//[0:15],

        .iocfg_A,
        .iocfg_B,
        .iocfg_C,
        .iocfg_D,
        .iocfg_E,

        .qfc_sck,
        .qfc_sckn,
        .qfc_dqs,
        .qfc_ss,
        .qfc_sio,
//        .qfc_rwds,
        .qfc_rstm,
        .qfc_rsts,
        .qfc_int,

        .padcfg_qfc_sck,
        .padcfg_qfc_qds,
        .padcfg_qfc_ss,
        .padcfg_qfc_sio,
//        .padcfg_qfc_rwds,
        .padcfg_qfc_int,
        .padcfg_qfc_rst,

        .rtosnspx  ({ pad_rto_px  , pad_sns_px  })  ,
        .rtosnsqfc ({ pad_rto_qfc , pad_sns_qfc })   ,
        .rtosnsrr1 ({ pad_rto_rr1 , pad_sns_rr1 })   ,
        .rtosnsrr0 ({ pad_rto_rr0 , pad_sns_rr0 })   ,
        .rtosnspmu ({ pad_rto_pmu , pad_sns_pmu })   ,



// analog
        .pmu_ana_test   (ANA_PMU_ANA_TEST),
        .ana_reramtest  (ANA_RR_TEST),
        .adc_si         (ANA_ADC_SI)

    );


// ░▒▓████████████▓▒░
//     always on
// ░▒▓████████████▓▒░


// pmu
    wire  pmu_POR                 ;
    wire  pmu_BGRDY               ;
    wire  pmu_VR25RDY             ;
    wire  pmu_VR85ARDY            ;
    wire  pmu_VR85DRDY            ;
    wire  pmu_IOUTEN              ;
    wire  pmu_IBIASEN             ;
    wire  pmu_POCENA              ;
    wire  pmu_VR25EN              ;
    wire  pmu_VR85AEN             ;
    wire  pmu_VR85DEN             ;
    wire  pmu_VR85A95ENA          ;
    wire  pmu_VR85D95ENA          ;
    wire  pmu_VR85AOSENA          ;
    wire  pmu_VR85DOSENA          ;
    wire  pmu_TRM_LATCH_b         ;
    wire [6-1:0] pmu_TRM_CUR      ;
    wire [5-1:0] pmu_TRM_CTAT     ;
    wire [5-1:0] pmu_TRM_PTAT     ;
    wire [5-1:0] pmu_TRM_D1P2     ;
    wire [5-1:0] pmu_TRM_DP60     ;
    wire [3-1:0] pmu_PMU_TEST_EN  ;
    wire [3-1:0] pmu_PMU_TEST_SEL ;

ao_top2 ao(
//pbus
    .clksys1m       (clksysao),
    .pclk           (clkao),
    .apbs           (apbao),
    .apbx           (apbao),

    .clkxtl32k,
    .socpad         (iopad_F),
    .aopad          (ao_iopad_F),

    .pmutrmset_nvr  (aopmutrmset),
    .pmutrm_nvr     (aopmutrmdata),

    .ipsleep        (socao_sleep),
    .aowkupvld      (aowkupvld),
    .aowkupint      (aowkupint),
    .osc_osc32m_cfg,
    .ao_iptpo,

// dft
    .jtag_tck         (jtagipt_tck),
    .jtag_resetn      (jtagipt_resetn),
    .tap_sel          (aotap_sel),
    .tap_capturedr    (ipttap_capturedr),
    .tap_shiftdr      (ipttap_shiftdr),
    .tap_updatedr     (ipttap_updatedr),
    .tap_en           (cmstest),
    .tap_tdi          (tap_tdm),
    .tap_tdo          (aotap_tds),

// pmu
    .pmu_POR          ,
    .pmu_BGRDY        ,
    .pmu_VR25RDY      ,
    .pmu_VR85ARDY     ,
    .pmu_VR85DRDY     ,
    .pmu_IOUTEN       ,
    .pmu_IBIASEN      ,
    .pmu_POCENA       ,
    .pmu_VR25EN       ,
    .pmu_VR85AEN      ,
    .pmu_VR85DEN      ,
    .pmu_VR85A95ENA   ,
    .pmu_VR85D95ENA   ,
    .pmu_VR85AOSENA   ,
    .pmu_VR85DOSENA   ,
    .pmu_TRM_LATCH_b  ,
    .pmu_TRM_CUR      ,
    .pmu_TRM_CTAT     ,
    .pmu_TRM_PTAT     ,
    .pmu_TRM_D1P2     ,
    .pmu_TRM_DP60     ,
    .pmu_PMU_TEST_EN  ,
    .pmu_PMU_TEST_SEL ,

// reset ctrl
    .padresetn  (ao_padresetn),
    .socresetn  (chipresetn)
);

    // always on pad
    padao_frame padao(

        .XTAL32K_IN,
        .XTAL32K_OUT,
        .PAD_AOXRSTn,
        .PF0, .PF1, .PF2, .PF3, .PF4, .PF5,
    // to AO_Top

                                .rtosnsao  ({ pad_rto_ao  , pad_sns_ao  })  ,
    /*    output logic      */  .clkxtl32k,
    /*    output logic      */  .ao_padresetn,
    /*    ioif.load         */  .ao_iopad_F//[0: 5]
    );

// ░▒▓████████████▓▒░
//     PMU
// ░▒▓████████████▓▒░


pmu pmu(
    .VR25ENA      ( pmu_VR25EN ),
    .VR85AENA     ( pmu_VR85AEN ),
    .VR85DENA     ( pmu_VR85DEN ),
    .BGRDY        ( pmu_BGRDY ),
    .VR25RDY      ( pmu_VR25RDY ),
    .VR85ARDY     ( pmu_VR85ARDY ),
    .VR85DRDY     ( pmu_VR85DRDY ),
    .VR85A95ENA   ( pmu_VR85A95ENA ),
    .VR85D95ENA   ( pmu_VR85D95ENA ),
    .VR85AOSENA   ( pmu_VR85AOSENA ),
    .VR85DOSENA   ( pmu_VR85DOSENA ),
    .TRM_LATCH_b  ( pmu_TRM_LATCH_b ),
    .TRM_CUR      ( pmu_TRM_CUR ),
    .TRM_CTAT     ( pmu_TRM_CTAT ),
    .TRM_PTAT     ( pmu_TRM_PTAT ),
    .TRM_D1P2     ( pmu_TRM_D1P2 ),
    .TRM_DP60     ( pmu_TRM_DP60 ),
    .IOUTENA      ( pmu_IOUTEN ),
    .IBIASENA     ( pmu_IBIASEN ),
    .POCENA       ( pmu_POCENA ),
    .POR          ( pmu_POR ),
    .PMU_TEST_EN  ( pmu_PMU_TEST_EN ),
    .PMU_TEST_SEL ( pmu_PMU_TEST_SEL ),
    .PMU_ANA_TEST ( ANA_PMU_ANA_TEST ),
    .ANA_IN0P1U   ( ANA_PMU_IN0P1U ),
    .POC_IO       ( ANA_PMU_POCRR ),
    .VD09ENA      ( vd_VD09ENA ),
    .VD09TL       ( vd_VD09TL ),
    .VD09TH       ( vd_VD09TH ),
    .VD09L        ( vd_VD09L ),
    .VD09H        ( vd_VD09H ),
    .VD25ENA      ( vd_VD25ENA ),
    .VD25TL       ( vd_VD25TL ),
    .VD25TH       ( vd_VD25TH ),
    .VD25L        ( vd_VD25L ),
    .VD25H        ( vd_VD25H ),
    .VD33ENA      ( vd_VD33ENA ),
    .VD33TL       ( vd_VD33TL ),
    .VD33TH       ( vd_VD33TH ),
    .VD33L        ( vd_VD33L ),
    .VD33H        ( vd_VD33H )
);

endmodule
