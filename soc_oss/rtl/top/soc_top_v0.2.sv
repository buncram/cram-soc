module soc_top (
        input   wire        ana_rng_0p1u,
        inout   wire [0:1]  ana_reramtest,
        input   wire        ana_rrpoc,
        input   wire [3:0]  ana_adcsrc,

        input logic         clkxtl,
        input logic [0:2]   cmspad,
        input logic         padresetn,
        input logic         chipresetn,
        output logic        clksys,
        output logic        coreresetn,
        output logic        cmsatpg,
        output logic        cmstest,
        output logic        cmsbist,
        output logic        cmsuser,
        output logic        cmsdone,

        output logic        dbgtxd,
        input logic         clkswd,
        ioif.drive          swdio,
        jtagif.slave        jtagvex,
        jtagif.slave        jtagrrc[0:1],
        jtagif.slave        jtagipt,
`ifdef FPGA
    input   logic   coresel_cm7,
`endif


        ioif.drive          iopad_A[0: 7],
        ioif.drive          iopad_B[0:15],
        ioif.drive          iopad_C[0:15],
        ioif.drive          iopad_D[0:15],
        ioif.drive          iopad_E[0:15],

        output padcfg_arm_t  iocfg_A[0: 7],
        output padcfg_arm_t  iocfg_B[0:15],
        output padcfg_arm_t  iocfg_C[0:15],
        output padcfg_arm_t  iocfg_D[0:15],
        output padcfg_arm_t  iocfg_E[0:15],
        output padcfg_arm_t  iocfg_F[0: 5],

    `UTMI_IF_DEF
//    ioif.drive                  sddc_clk,
//    ioif.drive                  sddc_cmd,
////    ioif.drive                  sddc_dat[3:0],
//    ioif.drive                  sddc_dat0,
//    ioif.drive                  sddc_dat1,
//    ioif.drive                  sddc_dat2,
//    ioif.drive                  sddc_dat3,

        output padcfg_arm_t  padcfg_qfc_sck,
        output padcfg_arm_t  padcfg_qfc_qds,
        output padcfg_arm_t  padcfg_qfc_ss,
        output padcfg_arm_t  padcfg_qfc_sio,
    ////    output padcfg_arm_t  padcfg_qfc_rwds,
        output padcfg_arm_t  padcfg_qfc_int,
        output padcfg_arm_t  padcfg_qfc_rst,

        ioif.drive qfc_sck,
        ioif.drive qfc_sckn,
        ioif.drive qfc_dqs,
        ioif.drive qfc_ss[1:0],
        ioif.drive qfc_sio[7:0],
    //    ioif.drive qfc_rwds,
        ioif.drive qfc_rstm[1:0],
        ioif.drive qfc_rsts[1:0],
        ioif.drive qfc_int,

        output  wire  VD09ENA             ,
        output  wire  VD25ENA             ,
        output  wire  VD33ENA             ,
        output  wire  VD09TL              ,
        output  wire  VD09TH              ,
        output  wire  VD25TL              ,
        output  wire  VD25TH              ,
        output  wire  VD33TL              ,
        output  wire  VD33TH              ,
        input   wire  VD09L               ,
        input   wire  VD09H               ,
        input   wire  VD25L               ,
        input   wire  VD25H               ,
        input   wire  VD33L               ,
        input   wire  VD33H               ,

        output wire pad_reton,
        output wire pad_retoff,

        // ao domain
        output logic        clksysao,
        output logic        clkao,
        apbif.master        apbao,
        apbif.master        apbudp,
        output logic        pclkudp,
        output logic        pmusleep,
        ioif.drive          iopad_F[0: 5],
        input logic         aowkupvld,
        output logic        aopmutrmset,
        output logic [63:0]       aopmutrmdata,
        input  logic        aowkupint,
        input  logic [6:0] osc_osc32m_cfg,
        input  logic [0:5] ao_iptpo,
        output  logic [0:12] iptpo13,
    // dft
        output logic jtagipt_tck,
        output logic jtagipt_resetn,
        output logic ipttap_shiftdr,
        output logic ipttap_updatedr,
        output logic ipttap_capturedr,
        output logic aotap_sel,
        output logic tap_tdo,
        input  logic aotap_tdi

);

//    apbif apbao();
//    dummyio_pu swdiopad(swdio);

    parameter ACKCNT = 8;
    parameter HCKCNT = 8;
    parameter ICKCNT = 8;
    parameter PCKCNT = 8;

    parameter BRC  = daric_cfg::BRC;
    parameter BRCW = daric_cfg::BRCW;
    parameter BRDW = daric_cfg::BRDW;
    parameter BRNUM_CMS = daric_cfg::BRNUM_CMS;
    parameter BRNUM_IPM = daric_cfg::BRNUM_IPM;
    parameter BRNUM_CFG = daric_cfg::BRNUM_CFG;
    parameter BRNUM_ACV = daric_cfg::BRNUM_ACV;
    parameter CW = 32;
    parameter SCFGWC = BRDW/CW;
    parameter IPMDC = BRNUM_IPM * BRDW / CW;
    parameter CFGDC = BRNUM_CFG * BRDW / CW;

//    logic               clksys;
    logic               clktop;
    logic               clkper;
    logic               fclk;
    logic               aclk;
    logic               hclk;
    logic               iclk;
    logic               pclk;
    logic               aoclk;
    logic               fclken;
    logic               aclken;
    logic               hclken;
    logic               iclken;
    logic               pclken;
    logic               aoclken;
    logic               fclken2;
    logic               aclken2;
    logic               hclken2;
    logic               iclken2;
    logic               pclken2;
    logic               aoclken2;
    logic               clkpkeen;
    logic [ACKCNT-1:0]  aclksub;
    logic [HCKCNT-1:0]  hclksub;
    logic [ICKCNT-1:0]  iclksub;
    logic [PCKCNT-1:0]  pclksub;
    logic               clk1M, clk32k;
//    logic               chipresetn;
    logic               secresetn;
    logic               wdtresetn;
    logic               vdresetn;
    logic               sysresetn;
//    logic               coreresetn;
    logic               sfrlock;
    logic [3:0]          brready;
    logic                brvld;
    logic [BRCW-1:0]     bridx;
    logic [BRDW-1:0]     brdat;
    logic               brdone;
    cms_pkg::cmsdata_e  cmsdata;
    logic               cmsdatavld;
    logic               cmsvrgn;
    logic               cmsscde;
//    logic               cmsdone;
    logic               cmserror;
    cms_pkg::cmscode_e  cmscode;
    logic apb_pclken;
    logic   cm7sleep;
    logic   cm7resetreq;
    logic aximclken;
    logic ahbpclken;
    logic ahbsclken;
    logic wdgintr, wdtreset, txd, duartintr, lclk;
    logic [1:0] tmintr;
    logic [IPMDC-1:0][31:0] iptrim32       ;
    logic                   iptrimdatavld  ;
    logic                   iptrimready    ;
    logic [CFGDC-1:0][31:0] syscfg32       ;
    logic wkupvld, ifsubwkupvld_async ;
    logic [6:0]             ipsleep;

    ahbif #(.AW(32),.DW(32),.IDW(4),.UW(4))
        ahbifsub(), ahbifsub0(), coreahb_sys(), coreahb_sec(), coreahb_ao();

    apbif #(.PAW(16),.DW(32)) apbsysbdg(), apbsecbdg(), apbaobdg();
    apbif #(.PAW(12),.DW(32)) apbsys[0:15]();//, apbsec[0:15]();
    nvrcfg_pkg::nvrcms_t     nvrcmsdata;
    nvrcfg_pkg::nvripm_t     nvripmdata;
    nvrcfg_pkg::nvrcfg_t     nvrcfgdata;

// ■■■■■■■■■■■
// system: sysctrl, cms, brc
// ■■■■■■■■■■■

    sysctrl #(
            .ACKCNT( ACKCNT ),
            .HCKCNT( HCKCNT ),
            .ICKCNT( ICKCNT ),
            .PCKCNT( PCKCNT ),
            .IPMDC ( IPMDC )
        )sysctrl(
    /*        input logic      */   .clkxtl      (clkxtl      ),
    /*        output logic     */   .clksys      (clksys      ),
    /*        output logic     */   .clksys1m    (clksysao    ),
    /*        output logic     */   .clktop      (clktop      ),
    /*        output logic     */   .clkper      (clkper      ),
    /*        output logic     */   .fclk        (fclk        ),
    /*        output logic     */   .aclk        (aclk        ),
    /*        output logic     */   .hclk        (hclk        ),
    /*        output logic     */   .iclk        (iclk        ),
    /*        output logic     */   .pclk        (pclk        ),
    /*        output logic     */   .aoclk       (aoclk       ),
    /*        output logic     */   .fclken      (fclken      ),
    /*        output logic     */   .aclken      (aclken      ),
    /*        output logic     */   .hclken      (hclken      ),
    /*        output logic     */   .iclken      (iclken      ),
    /*        output logic     */   .pclken      (pclken      ),
    /*        output logic     */   .aoclken     (aoclken     ),
    /*        output logic     */   .fclken2     (fclken2     ),
    /*        output logic     */   .aclken2     (aclken2     ),
    /*        output logic     */   .hclken2     (hclken2     ),
    /*        output logic     */   .iclken2     (iclken2     ),
    /*        output logic     */   .pclken2     (pclken2     ),
    /*        output logic     */   .aoclken2    (aoclken2    ),
    /*        output logic     */   .clkpkeen    (clkpkeen    ),
    /*        output logic     */   .aclksub     (aclksub     ),
    /*        output logic     */   .hclksub     (hclksub     ),
    /*        output logic     */   .iclksub     (iclksub     ),
    /*        output logic     */   .pclksub     (pclksub     ),
    /*        output logic     */   .clk1M       (clk1M       ),
    /*        output logic     */   .clk32k      (clk32k      ),
    /*        input   logic    */   .cmsatpg     (cmsatpg     ),
    /*        input   cms_pkg: */   .cmscode     (cmscode     ),
                                    .brdone      (brdone      ),
    /*        input logic      */   .chipresetn  (chipresetn  ),
    /*        input logic      */   .secresetn   (secresetn   ),
    /*        input logic      */   .padresetn   (padresetn   ),
    /*        input logic      */   .wdtresetn   (wdtresetn   ),
    /*        input logic      */   .vdresetn    (vdresetn    ),
    /*        output logic     */   .sysresetn   (sysresetn   ),
    /*        output logic     */   .coreresetn  (coreresetn  ),
                                    .wkupvld_async(aowkupvld | ifsubwkupvld_async),
    /*    output logic [IPMDC-1:0][31:0] */.iptrim32        ('0),
    /*    output logic                   */.iptrimdatavld   (iptrimdatavld),
    /*    input  logic                   */.iptrimready     (iptrimready),
                                    .coresleep   (cm7sleep    ),
                                    .ipsleep     (ipsleep     ),

                                    .pad_reton,
                                    .pad_retoff,
                                    .osc_osc32m_cfg(osc_osc32m_cfg),
    /*        input logic      */   .sfrlock     (sfrlock     ),
    /*        apbif.slave      */   .apbs         (apbsys[0]   ),
    /*        apbif.slave      */   .apbx         (apbsys[0]   )
        );

//        assign secresetn = 1'b1;
    //    assign wdtresetn = 1'b1;
        assign vdresetn  = 1'b1;
        assign pmusleep = ipsleep[3];
//        ##wkupvld
        assign pclkudp = pclk;

    cms cms(
        .clk    (clksys),
        .resetn (sysresetn),
        .chipresetn (chipresetn),
    /*    input logic [0:2] */  .cmspad      ( cmspad      ),
    /*    input cmsdata_e   */  .cmsdata     ( cmsdata     ),
    /*    input logic       */  .cmsdatavld  ( cmsdatavld  ),
    /*    output logic      */  .cmsatpg     ( cmsatpg     ),
    /*    output logic      */  .cmstest     ( cmstest     ),
    /*    output logic      */  .cmsuser     ( cmsuser     ),
    /*    output logic      */  .cmsvrgn     ( cmsvrgn     ),
    /*    output logic      */  .cmsscde     ( cmsscde     ),
    /*    output logic      */  .cmsdone     ( cmsdone     ),
    /*    output logic      */  .cmserror    ( cmserror    ),
    /*    output cmscode_e  */  .cmscode     ( cmscode     )
    );

    assign cmsbist = cmstest;

    brc#(
        .BRC  (BRC),
        .BRCW (BRCW),
        .BRDW (BRDW),
        .BRNUM_CMS (BRNUM_CMS),
        .BRNUM_IPM (BRNUM_IPM),
        .BRNUM_CFG (BRNUM_CFG),
        .CW (CW),
        .SCFGWC (SCFGWC),
        .IPMDC (IPMDC),
        .CFGDC (CFGDC),
        .CMSD_t (cms_pkg::cmsdata_e)
    )brc(
             .clk    ( clksys ),
             .resetn ( sysresetn ),
    /*    input  logic                   */.brvld           (brvld),
    /*    input  logic [BRCW-1:0]        */.bridx           (bridx),
    /*    input  logic [BRDW-1:0]        */.brdat           (brdat),
    /*    input  logic                   */.brdone          (brdone),
    /*    output logic [3:0]             */.brready         (brready),
    /*    output logic [BRDW-1:0]        */.nvrcmsdata      (nvrcmsdata),
    /*    output logic                   */.cmsdatavld      (cmsdatavld),
    /*    input  logic                   */.cmsdone         (cmsdone&&cmsuser),
    /*    output logic [IPMDC-1:0][31:0] */.nvripmdata,
    /*    output logic                   */.iptrimdatavld   (iptrimdatavld),
    /*    input  logic                   */.iptrimready     (iptrimready),
    /*    output logic [CFGDC-1:0][31:0] */.nvrcfgdata
    );
    //    assign cmsdatavld = bridx[0] & brvld;
    assign cmsdata = nvrcmsdata.cmsdata0;

    sync_pulse sync_pmutrmset ( .clka(clksys),    .resetn(sysresetn), .clkb(clksysao), .pulsea (iptrimdatavld), .pulseb( aopmutrmset ) );
    assign aopmutrmdata[63:0] = nvripmdata.ipm0[63:0];


// ░▒▓██▓▒░ ■■■■■■■■■■■
// ░▒▓██▓▒░  apbsys: evc, wdt, duart, tmr
// ░▒▓██▓▒░ ■■■■■■■■■■■

    parameter EVCNT  = daric_cfg::IRQCNT;
    parameter IRQCNT = daric_cfg::IRQCNT;
    parameter ERRCNT = daric_cfg::ERRCNT;

    bit [EVCNT-1:0]  ev;
    bit [ERRCNT-1:0] err;
    bit [IRQCNT-1:0] cm7irq;
    bit              cm7ev;
    bit              cm7nmi;
    bit              ifev_vld;
    bit [7:0]        ifev_dat;
    bit              ifev_rdy;
    bit [1:0]        tmr_ev;
    bit [3:0]        pwm_ev;
    bit              ifev_err;
    bit              coresuberr;
    bit              sceerr;
    bit              ifsuberr;
    bit              secsubrr;
    bit [7:0]        secirq;

    bit [31:0]       coresubev;
    bit [31:0]       sceev;
    bit [127:0]      ifsubev;

// valid event for M7 is 240b

    assign ev[31 :0  ] = {aowkupint,coresubev[30:0]};
    assign ev[63 :32 ] = sceev;
    assign ev[191:64 ] = ifsubev;
    assign ev[223:192] = err;
    assign ev[239:224] = '0 | secirq[7:0];
    assign ev[255:240] = '0;


    assign err[0] = |coresuberr;
    assign err[1] = |sceerr;
    assign err[2] = |ifsuberr;
    assign err[3] = |secirq;
    assign err[ERRCNT-1:4] = '0;

    evc#(
        .EVCNT  (EVCNT),
        .ERRCNT (ERRCNT),
        .IRQCNT (IRQCNT)
    )evc(
        .hclk       (hclk),
        .pclk       (pclk),
        .resetn     (coreresetn),
        .apbs       (apbsys[4]),
        .apbx       (apbsys[4]),
        .evin       (ev),
        .errin      (err),
        // m7
        .cm7irq,
        .cm7ev,
        .cm7nmi,
        // ifsub
        .ifev_vld,
        .ifev_dat,
        .ifev_rdy,
        .ifev_err,
        // timer
        .tmr_ev,
        .pwm_ev
    );

    wdg_intf wdt(
            .clk    (pclk),
            .resetn (coreresetn),
            .wdgclk (pclk),
            .apbs   (apbsys[1]),
            .wdgintr(wdgintr),
            .wdgrst (wdtreset)
        );
    assign wdtresetn = ~wdtreset;

    duart duart(
            .clk    (pclk),
            .sclk   (clksys),
            .resetn (coreresetn),
            .apbs   (apbsys[2]),
            .apbx   (apbsys[2]),
            .txd    (dbgtxd)
        );

    timer_intf tmr(
            .clk    (pclk),
            .resetn (coreresetn),
            .apbs   (apbsys[3]),
            .lclk   (clk32k),
            .evin   (tmr_ev),
            .tmintr (tmintr)
        );

    `ifdef SIM
    sim_mon mon(.clk (pclk),.apbs   (apbsys[15]),.apbx   (apbsys[15]));
    `else
    apbs_null as7(apbsys[15]);
    `endif

    apbs_nulls #(.SLVCNT(10)) as0(apbsys[5:14]);

// ░▒▓██▓▒░ ■■■■■■■■■■■
// ░▒▓██▓▒░  coresub
// ░▒▓██▓▒░ ■■■■■■■■■■■

    assign aximclken = aclken2;
    assign ahbpclken = aclken2 & hclken2;
    assign ahbsclken = aclken2 & hclken2;

    soc_coresub #(
        //parameter
    )soc_coresub(
                               .ana_rng_0p1u  (ana_rng_0p1u),
                               .ana_reramtest,
    /*    input   logic     */ .fclk        ( fclk        ),
    /*    input   logic     */ .aclk        ( aclk        ),
    /*    input   logic     */ .hclk        ( hclk        ),
    /*    input   logic     */ .iclk        ( iclk        ),
//                               .pclk        ( pclk        ),
                               .fclken      ( fclken      ),
//                               .pclken      ( apb_pclken  ),
    /*    input   logic     */ .clktop      ( clktop      ),
    /*    input   logic     */ .clktopen    ( fclken      ),
    /*    input   logic     */ .clksys      ( clksys      ),
    /*    input   logic     */ .clk1M       ( clk1M       ),
    /*    input   logic     */ .clkdma      ( hclksub[0]  ),
    /*    input   logic     */ .clksce      ( hclksub[1]  ),
    /*    input   logic     */ .clksceen    ( hclken      ),
    /*    input   logic     */ .clkpkeen    ( clkpkeen    ),
                               .clkvex      ( aclk        ),
                               .clkqfc      ( aclksub[1]  ),
    /*    input   logic     */ .aximclken   ( aximclken   ),
    /*    input   logic     */ .ahbpclken   ( ahbpclken   ),
    /*    input   logic     */ .ahbsclken   ( ahbsclken   ),
    /*    input   logic     */ .sysresetn   ( sysresetn   ),
    /*    input   logic     */ .coreresetn  ( coreresetn  ),
`ifdef FPGA
            .coresel_cm7,
`endif
    ///*    output cmsdata_e  */  .cmsdata     ( cmsdata     ),
    ///*    output logic      */  .cmsdatavld  ( cmsdatavld  ),
    /*   input  logic [3:0]      */.brready,
    /*   output logic            */.brvld,
    /*   output logic [BRCW-1:0] */.bridx,
    /*   output logic [BRDW-1:0] */.brdat,
    /*    output logic      */ .brdone      ( brdone      ),
    /*    output  logic     */ .cm7sleep    ( cm7sleep    ),
    /*    output  logic     */ .cm7resetreq ( cm7resetreq ),
    /*    input   cmscode_e */ .cmscode     ( cmscode     ),
                               .cmsatpg     (cmsatpg),
                               .cmsbist     (cmsbist),
                               .cm7_irq     ( cm7irq[IRQCNT-1-16:0] ),
                               .nvrcfgdata,
                               .cm7_nmi     ( cm7nmi      ),
                               .cm7_rxev    ( cm7ev       ),
                               .coresubevo  ( coresubev   ),
                               .coresuberro ( coresuberr  ),
                               .sceevo      ( sceev   ),
                               .sceerro     ( sceerr  ),
                               .rramsleep   ( ipsleep[2] ),
// qfc
                                .padcfg_qfc_sck,
                                .padcfg_qfc_qds,
                                .padcfg_qfc_ss,
                                .padcfg_qfc_sio,
//                                .padcfg_qfc_rwds,
                                .padcfg_qfc_int,
                                .padcfg_qfc_rst,

                                .qfc_sck,
                                .qfc_sckn,
                                .qfc_dqs,
                                .qfc_ss,
                                .qfc_sio,
//                                .qfc_rwds,
                                .qfc_rstm,
                                .qfc_rsts,
                                .qfc_int,


// vex
                                .jtagvex,
                                .jtagrrc,

    /*    ahbif.master      */ .bmxif_ahb32 ( ahbifsub0   ),
    /*    ahbif.master      */ .coreahb_sys ( coreahb_sys ),
    /*    ahbif.master      */ .coreahb_sec ( coreahb_sec ),
    /*    ahbif.master      */ .coreahb_ao  ( coreahb_ao  ),
    /*    input   logic     */ .clkswd      ( clkswd      ),
    /*    ioif.drive        */ .swdio       ( swdio       ),
                               .ana_rrpoc   ( ana_rrpoc   )
    );

    //apb_bdg


        assign apb_pclken = iclken2 & pclken2;
        apb_bdg uapbsysbdg(
    /*        input        */   .hclk     ( hclk            ),
    /*        input        */   .resetn   ( coreresetn      ),
    /*        input        */   .pclken   ( apb_pclken      ),
    /*        ahbif.slave  */   .ahbslave ( coreahb_sys     ),
    /*        apbif.master */   .apbmaster( apbsysbdg       )
        );

        apb_bdg uapbsecbdg(
    /*        input        */   .hclk     ( hclk            ),
    /*        input        */   .resetn   ( coreresetn      ),
    /*        input        */   .pclken   ( apb_pclken      ),
    /*        ahbif.slave  */   .ahbslave ( coreahb_sec     ),
    /*        apbif.master */   .apbmaster( apbsecbdg       )
        );



    apb_mux  #(.DECAW(4)) apbsysmux(.apbslave (apbsysbdg), .apbmaster(apbsys));
//    apb_mux  #(.DECAW(4)) apbsecmux(.apbslave (apbsecbdg), .apbmaster(apbsec));

//    apbs_nulls #(.SLVCNT(16)) asec0(apbsec);

// ░▒▓██▓▒░  ■■■■■■■■■■
// ░▒▓██▓▒░    secsub
// ░▒▓██▓▒░  ■■■■■■■■■■

    logic [daric_cfg::SENSORVDC-1:0] sensor_vd;
    logic [daric_cfg::SENSORVDC/2-1:0] sensor_vdena;
    logic [daric_cfg::SENSORVDC-1:0] sensor_vdtst;

    logic [daric_cfg::SENSORLDC-1:0] sensor_ld;
    logic [daric_cfg::SENSORLDC-1:0] sensor_ldtst;
    logic sensor_ldclk;

    assign  { VD09ENA,VD25ENA,VD33ENA } = sensor_vdena;
    assign  { VD09TL,VD09TH,VD25TL,VD25TH,VD33TL,VD33TH } = sensor_vdtst ;
    assign  sensor_vd = { VD09L,VD09H,VD25L,VD25H,VD33L,VD33H };

    secsub #(
        .MESHLC    ( daric_cfg::MESHLC ),
        .MESHPC    ( daric_cfg::MESHPC ),
        .SENSORVDC ( daric_cfg::SENSORVDC),
        .SENSORLDC ( daric_cfg::SENSORLDC),
        .GLCX      ( daric_cfg::GLCX ),
        .GLCY      ( daric_cfg::GLCY )
    )secsub(
/*        input logic  */  .clksys, // clksys
/*        input logic  */  .pclk,
/*        input logic  */  .porresetn   ( chipresetn ),
/*        input logic  */  .resetn      ( coreresetn ),
                           .cmsatpg,
/*        input logic  */  .vd          ( sensor_vd ),
/*        input logic  */  .ld          ( sensor_ld ),
                            .vdena      (sensor_vdena),
                            .vdtst      (sensor_vdtst),
                            .ldtst      (sensor_ldtst),
                            .ldclk      (sensor_ldclk),
/*        output logic  */ .vdresetn    ( secresetn ),
/*        output logic  */ .irq8        ( secirq ),
/*        apbif.slave  */  .apbs        ( apbsecbdg )
    );

generate
    for (genvar i = 0; i < daric_cfg::SENSORLDC; i++) begin : gLD
        ip_lightdet ldsensor(
                .analog_test_only(),
                .d2a_clk  ( sensor_ldclk ),
                .d2a_self_test_en ( sensor_ldtst[i] ),
                .light_out ( sensor_ld[i] )
        );
    end
endgenerate


// ░▒▓██▓▒░  ■■■■■■■■■■
// ░▒▓██▓▒░    ifsub
// ░▒▓██▓▒░  ■■■■■■■■■■

    ahb_sync#(
            .SYNCDOWN (1),
            .SYNCUP   (0)
        ) ahbifsub_syncdown (
            .hclk       (hclk        ),
            .resetn     (coreresetn  ),
            .hclken     (iclken2     ),
            .ahbslave   (ahbifsub0   ),
            .ahbmaster  (ahbifsub    )
        );

    ioif            iopad[0:16*6-1]();
    padcfg_arm_t    iocfg[0:16*6-1];

    soc_ifsub #(
    //    .IOC   (16*6),
    //    .EVCNT (32*4),
    //    .ERRCNT (1)
    )soc_ifsub(
    /*    input logic               */  .clk       ( iclk       ),
                                        .pclk         (pclk),
                                        .pclken       (pclken),

    /*    input logic               */  .clk32m    ( clksys     ),
                                        .clkao25m     (clksys),

    /*    input logic               */  .resetn    ( coreresetn ),
    /*    input logic               */  .perclk    ( clkper     ),
    /*    input logic               */  .cmsbist   ( cmsbist    ),
    /*    input logic               */  .cmsatpg   ( cmsatpg    ),
    /*    input logic               */  .clksys    ( clksys     ),
                                        .ioxlock   ( '0    ),
    /*    input  logic              */  .ifev_vld  ( ifev_vld   ),
    /*    input  logic [7:0]        */  .ifev_dat  ( ifev_dat   ),
    /*    output logic              */  .ifev_rdy  ( ifev_rdy   ),
    /*    output logic              */  .wkupvld   ( wkupvld    ),
    /*    output logic              */  .wkupvld_async   ( ifsubwkupvld_async    ),
    /*    ahbif.slave               */  .ahbs      ( ahbifsub   ),
    /*    output logic [EVCNT-1:0]  */  .ifsubevo  ( ifsubev    ),
    /*    output logic [ERRCNT-1:0] */  .ifsuberro ( ifsuberr   ),
//                                       .sddc_clk     (sddc_clk),
//                                       .sddc_cmd     (sddc_cmd),
//                                       .sddc_dat0     (sddc_dat0),
//                                       .sddc_dat1     (sddc_dat1),
//                                       .sddc_dat2     (sddc_dat2),
//                                       .sddc_dat3     (sddc_dat3),
                                        .ana_adcsrc,
                                        `UTMI_IF_INST
                                        .apbudp       (apbudp),
    /*    ioif.drive                */  .iopad     ( iopad      ),
                                        .iocfg
    );

    iothrus #( 8)uiopadA(.iodrv(iopad_A[0: 7]), .ioload(iopad[ 0: 7])); // [ 0:15]
    iothrus #(16)uiopadB(.iodrv(iopad_B[0:15]), .ioload(iopad[16:31])); // [16:31]
    iothrus #(16)uiopadC(.iodrv(iopad_C[0:15]), .ioload(iopad[32:47])); // [32:47]
    iothrus #(16)uiopadD(.iodrv(iopad_D[0:15]), .ioload(iopad[48:63])); // [48:63]
    iothrus #(16)uiopadE(.iodrv(iopad_E[0:15]), .ioload(iopad[64:79])); // [64:79]
    iothrus #( 6)uiopadF(.iodrv(iopad_F[0: 5]), .ioload(iopad[80:85])); // [80:95]

    assign iocfg_A[0: 7] = iocfg[ 0: 7];
    assign iocfg_B[0:15] = iocfg[16:31];
    assign iocfg_C[0:15] = iocfg[32:47];
    assign iocfg_D[0:15] = iocfg[48:63];
    assign iocfg_E[0:15] = iocfg[64:79];
    assign iocfg_F[0: 5] = iocfg[80:85];


    ioifld_nulls #( 8)uiopadAnulls(.ioifld(iopad[ 8:15]));
//    ioifld_nulls #( 2)uiopadCnulls(.ioifld(iopad[46:47]));
    ioifld_nulls #(10)uiopadFnulls(.ioifld(iopad[86:95]));

// ░▒▓██▓▒░  ■■■■■■■■■■
// ░▒▓██▓▒░    always on
// ░▒▓██▓▒░  ■■■■■■■■■■

    assign clkao = aoclk;

        apb_bdg uapbaobdg(
    /*        input        */   .hclk     ( hclk            ),
    /*        input        */   .resetn   ( coreresetn      ),
    /*        input        */   .pclken   ( aoclken2        ),
    /*        ahbif.slave  */   .ahbslave ( coreahb_ao      ),
    /*        apbif.master */   .apbmaster( apbaobdg        )
        );

    apb_thru uapbao(.apbslave (apbaobdg), .apbmaster(apbao));


// ░▒▓██▓▒░  ■■■■■■■■■■
// ░▒▓██▓▒░    always on
// ░▒▓██▓▒░  ■■■■■■■■■■

// jtag ipt
    assign jtagipt_tck = jtagipt.tck;
    assign jtagipt_resetn = sysresetn & jtagipt.trst;

  tap_top u_tap (
    // jtag
    .tms_i      ( jtagipt.tms ),
    .tck_i      ( jtagipt.tck ),
    .rst_ni     ( jtagipt_resetn ),
    .td_i       ( jtagipt.tdi ),
    .td_o       ( jtagipt.tdo ),
    // tap states
    .shift_dr_o     (ipttap_shiftdr),
    .update_dr_o    (ipttap_updatedr),
    .capture_dr_o   (ipttap_capturedr),
    // select signals for boundary scan or mbist
    .memory_sel_o   (),
    .fifo_sel_o     (),
    .confreg_sel_o  (aotap_sel),
    .clk_byp_sel_o  (),
    .observ_sel_o   (),
    // tdo signal connected to tdi of sub modules
    .scan_in_o      (tap_tdo),
    // tdi signals from sub modules
    .memory_out_i   ('0),
    .fifo_out_i     ('0),
    .confreg_out_i  (aotap_tdi),
    .clk_byp_out_i  ('0),
    .observ_out_i   ('0)
  );

  assign iptpo13[0:12] =  {
        VD09L               ,
        VD09H               ,
        VD25L               ,
        VD25H               ,
        VD33L               ,
        VD33H               ,
        clksysao,
        ao_iptpo[0:5]
        };


endmodule

/*
module dummyio_pu( ioif.load pad );
    assign pad.pi=1'b1;
endmodule
*/
