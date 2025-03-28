`include "template.sv"


module  ao_top2 #(
    parameter IOC=6,
    parameter IPTBW = 26+7+7 // pmu,osc32k,osc32m
 )(

//pbus
    input logic     clksys1m,
    input logic     pclk, //1M by default
    apbif.slavein   apbs,
    apbif.slave     apbx,

    input  logic    clkxtl32k,
    ioif.load       socpad[0:IOC-1],
    ioif.drive      aopad[0:IOC-1],

    input logic              pmutrmset_nvr,
    input logic [64-1:0]     pmutrm_nvr,

    input logic     ipsleep,
    output logic    aowkupvld,
    output logic    aowkupint,
    output logic [6:0] osc_osc32m_cfg,
    output logic [0:5] ao_iptpo,

// dft

    input  logic  jtag_tck,        //
    input  logic  jtag_resetn,     //
    input  logic  tap_sel,         //
    input  logic  tap_capturedr,   //
    input  logic  tap_shiftdr,     //
    input  logic  tap_updatedr,    //
    input  logic  tap_en,          //
    input  logic  tap_tdi,         //
    output logic  tap_tdo,         //

// pmu
    input   wire  pmu_POR                 ,    // pmu
    input   wire  pmu_BGRDY               ,    // pmu
    input   wire  pmu_VR25RDY             ,    // pmu
    input   wire  pmu_VR85ARDY            ,    // pmu
    input   wire  pmu_VR85DRDY            ,    // pmu
    output  wire  pmu_IOUTEN              ,    // pmu
    output  wire  pmu_IBIASEN             ,    // pmu
    output  wire  pmu_POCENA              ,    // pmu
    output  wire  pmu_VR25EN              ,    // pmu
    output  wire  pmu_VR85AEN             ,    // pmu
    output  wire  pmu_VR85DEN             ,    // pmu
    output  wire  pmu_VR85A95ENA          ,    // pmu
    output  wire  pmu_VR85D95ENA          ,    // pmu
    output  wire  pmu_VR85AOSENA          ,    // pmu
    output  wire  pmu_VR85DOSENA          ,    // pmu
    output  wire  pmu_TRM_LATCH_b         ,    // pmu
    output  wire [6-1:0] pmu_TRM_CUR      ,    // pmu
    output  wire [5-1:0] pmu_TRM_CTAT     ,    // pmu
    output  wire [5-1:0] pmu_TRM_PTAT     ,    // pmu
    output  wire [5-1:0] pmu_TRM_D1P2     ,    // pmu
    output  wire [5-1:0] pmu_TRM_DP60     ,    // pmu
    output  wire [3-1:0] pmu_PMU_TEST_EN  ,    // pmu
    output  wire [3-1:0] pmu_PMU_TEST_SEL ,    // pmu

// reset ctrl
    input  logic    padresetn,  // from aopad
    output logic    socresetn   // to   soc
);

    localparam TAPBW = IPTBW + 10 + 5 + 6; // osc,pmu_sr,pmu_cr,pmu_ctrl,pmu_tst
    localparam bit [6:0]    IV_OSC32KTRM =    7'b0100110;
    localparam bit [6:0]    IV_OSC32MTRM =    7'b0100110;
    localparam bit [9:0]    IV_PMUCR =       10'h3f3;
    localparam bit [IPTBW-1:0] IV_PMUTRM =     {IV_OSC32MTRM[6:0], IV_OSC32KTRM[6:0], 6'h0,5'h10,5'h10,5'h10,5'h10};
    localparam bit [5:0]    IV_PMUDFT =     {3'h0, 3'h2};

    localparam bit [TAPBW-1:0] IV_TAP = { 5'h0, IV_PMUCR[9:0], IV_PMUTRM[IPTBW-1:0], IV_PMUDFT[5:0] };

// vddao pin

    logic [TAPBW-1:0] tap_regout, tap_regin;

    logic     s_BGRDY               ;
    logic     s_VR25RDY             ;
    logic     s_VR85ARDY            ;
    logic     s_VR85DRDY            ;
    logic     s_POR                 ;
    logic     s_IOUTEN              ;
    logic     s_IBIASEN             ;
    logic     s_POCENA              ;
    logic     s_VR25EN              ;
    logic     s_VR85AEN             ;
    logic     s_VR85DEN             ;
    logic     s_VR85A95ENA          ;
    logic     s_VR85D95ENA          ;
    logic     s_VR85AOSENA          ;
    logic     s_VR85DOSENA          ;
    logic [6-1:0]    s_TRM_CUR      , pmutrmlp_CUR      , pmutrmreg_CUR      ;
    logic [5-1:0]    s_TRM_CTAT     , pmutrmlp_CTAT     , pmutrmreg_CTAT     ;
    logic [5-1:0]    s_TRM_PTAT     , pmutrmlp_PTAT     , pmutrmreg_PTAT     ;
    logic [5-1:0]    s_TRM_D1P2     , pmutrmlp_D1P2     , pmutrmreg_D1P2     ;
    logic [5-1:0]    s_TRM_DP60     , pmutrmlp_DP60     , pmutrmreg_DP60     ;
    logic [6:0]      s_osc32k_cfg,    osc32kcfglp,        osc32kcfgreg,           osc_osc32k_cfg ;
    logic [6:0]      s_osc32m_cfg,    osc32mcfglp,        osc32mcfgreg           ;//osc_osc32m_cfg ;
    logic [3-1:0]    s_PMU_TEST_SEL , pmudft_testsel;
    logic [3-1:0]    s_PMU_TEST_EN  , pmudft_testen ;

    bit [IOC+4-1:0] wkupmask;
    logic cmsatpg;
    bit pmupdresetn;
    bit clkosc32k, clk32k;

    jtagreg #(
        .JTAGREGSIZE(TAPBW),
        .IV(IV_TAP),
        .SYNC(0)
    ) i_jtagreg1 (
        .clk_i           (jtag_tck),
        .rst_ni          (jtag_resetn),
        .enable_i        (tap_sel),
        .capture_dr_i    (tap_capturedr),
        .shift_dr_i      (tap_shiftdr),
        .update_dr_i     (tap_updatedr),
        .jtagreg_in_i    (tap_regin),
        .mode_i          (tap_en),
        .scan_in_i       (tap_tdi),
        .scan_out_o      (tap_tdo),
        .jtagreg_out_o   (tap_regout)
    );

    assign tap_regin =
    {
        pmu_BGRDY               ,
        pmu_VR25RDY             ,
        pmu_VR85ARDY            ,
        pmu_VR85DRDY            ,
        pmu_POR                 ,
        s_IOUTEN              ,
        s_IBIASEN             ,
        s_POCENA              ,
        s_VR25EN              ,
        s_VR85AEN             ,
        s_VR85DEN             ,
        s_VR85A95ENA          ,
        s_VR85D95ENA          ,
        s_VR85AOSENA          ,
        s_VR85DOSENA          ,
//        s_TRM_LATCH_b         ,
        s_osc32m_cfg[6:0]     ,
        s_osc32k_cfg[6:0]     ,
        s_TRM_CUR [6-1:0]     ,
        s_TRM_CTAT[5-1:0]     ,
        s_TRM_PTAT[5-1:0]     ,
        s_TRM_D1P2[5-1:0]     ,
        s_TRM_DP60[5-1:0]     ,
        s_PMU_TEST_SEL[3-1:0] ,
        s_PMU_TEST_EN [3-1:0]
    };

    assign {
        s_BGRDY               ,
        s_VR25RDY             ,
        s_VR85ARDY            ,
        s_VR85DRDY            ,
        s_POR                 ,
        pmu_IOUTEN              ,
        pmu_IBIASEN             ,
        pmu_POCENA              ,
        pmu_VR25EN              ,
        pmu_VR85AEN             ,
        pmu_VR85DEN             ,
        pmu_VR85A95ENA          ,
        pmu_VR85D95ENA          ,
        pmu_VR85AOSENA          ,
        pmu_VR85DOSENA          ,
//        pmu_TRM_LATCH_b         ,
        osc_osc32m_cfg[6:0]     ,
        osc_osc32k_cfg[6:0]     ,
        pmu_TRM_CUR [6-1:0]     ,
        pmu_TRM_CTAT[5-1:0]     ,
        pmu_TRM_PTAT[5-1:0]     ,
        pmu_TRM_D1P2[5-1:0]     ,
        pmu_TRM_DP60[5-1:0]     ,
        pmu_PMU_TEST_SEL[3-1:0] ,
        pmu_PMU_TEST_EN [3-1:0]
    } = tap_regout;

    assign ao_iptpo =     {
        clkosc32k ,
        pmu_BGRDY               ,
        pmu_VR25RDY             ,
        pmu_VR85ARDY            ,
        pmu_VR85DRDY            ,
        pmu_POR
        };


//  pd / wakeup
// ■■■■■■■■■■■■■■■

    logic porresetn, resetn, aopdreg;// clk;
    logic pmucr_vr25en, pmucr_vr85aen, pmucr_vr85den, pmucr_iouten, pmucr_ibiasen, pmucr_pocen, pmucr_vr85a95en, pmucr_vr85d95en, pmucr_vr85aosen, pmucr_vr85dosen;
    logic pmucrlp_vr25en, pmucrlp_vr85aen, pmucrlp_vr85den, pmucrlp_iouten, pmucrlp_ibiasen, pmucrlp_pocen, pmucrlp_vr85a95en, pmucrlp_vr85d95en, pmucrlp_vr85aosen, pmucrlp_vr85dosen;
    logic [9:0] sfrpmucr, sfrpmucrlp;
    logic pmucr_vr25pd, pmucr_vr85apd, pmucr_vr85dpd;
    logic [3:0] sfrpmucrpd;
    logic [IPTBW-1:0] sfrpmutrmcr, sfrpmutrmcrlp, pmutrmreg;
    logic [4:0] sfrpmusr;
    logic [5:0] sfrpmudftcr;
    logic pmutrmset_sfr, sfrpmutrmar;
    logic oscpd;

    assign pmu_TRM_LATCH_b       = porresetn;
    assign s_VR25EN              = pmucr_vr25en  & ~( aopdreg & pmucr_vr25pd );
    assign s_VR85AEN             = pmucr_vr85aen & ~( aopdreg & pmucr_vr85apd );
    assign s_VR85DEN             = pmucr_vr85den & ~( aopdreg & pmucr_vr85dpd );
    assign s_IOUTEN              = ipsleep ? pmucrlp_iouten :    pmucr_iouten;
    assign s_IBIASEN             = ipsleep ? pmucrlp_ibiasen :   pmucr_ibiasen;
    assign s_POCENA              = ipsleep ? pmucrlp_pocen :     pmucr_pocen;
    assign s_VR85A95ENA          = ipsleep ? pmucrlp_vr85a95en : pmucr_vr85a95en;
    assign s_VR85D95ENA          = ipsleep ? pmucrlp_vr85d95en : pmucr_vr85d95en;
    assign s_VR85AOSENA          = ipsleep ? pmucrlp_vr85aosen : pmucr_vr85aosen;
    assign s_VR85DOSENA          = ipsleep ? pmucrlp_vr85dosen : pmucr_vr85dosen;
    assign s_osc32m_cfg[6:0]     = ipsleep ? osc32mcfglp : osc32mcfgreg;
    assign s_osc32k_cfg[6:0]     = ipsleep ? osc32kcfglp : osc32kcfgreg;
    assign s_TRM_CUR [6-1:0]     = ipsleep ? pmutrmlp_CUR :  pmutrmreg_CUR ;
    assign s_TRM_CTAT[5-1:0]     = ipsleep ? pmutrmlp_CTAT : pmutrmreg_CTAT;
    assign s_TRM_PTAT[5-1:0]     = ipsleep ? pmutrmlp_PTAT : pmutrmreg_PTAT;
    assign s_TRM_D1P2[5-1:0]     = ipsleep ? pmutrmlp_D1P2 : pmutrmreg_D1P2;
    assign s_TRM_DP60[5-1:0]     = ipsleep ? pmutrmlp_DP60 : pmutrmreg_DP60;
    assign s_PMU_TEST_SEL[3-1:0] = pmudft_testsel[2:0];
    assign s_PMU_TEST_EN [3-1:0] = pmudft_testen[2:0];

    assign {
            pmucr_vr25en, pmucr_vr85aen, pmucr_vr85den,
            pmucr_iouten, pmucr_ibiasen, pmucr_pocen,
            pmucr_vr85a95en, pmucr_vr85d95en,
            pmucr_vr85aosen, pmucr_vr85dosen
        } = sfrpmucr;
    assign {
            pmucrlp_vr25en, pmucrlp_vr85aen, pmucrlp_vr85den, // no use
            pmucrlp_iouten, pmucrlp_ibiasen, pmucrlp_pocen,
            pmucrlp_vr85a95en, pmucrlp_vr85d95en,
            pmucrlp_vr85aosen, pmucrlp_vr85dosen
        } = sfrpmucrlp;
    assign {
            oscpd, pmucr_vr25pd, pmucr_vr85apd, pmucr_vr85dpd
        } = sfrpmucrpd;

    assign {
            osc32mcfglp, osc32kcfglp, pmutrmlp_CUR , pmutrmlp_CTAT, pmutrmlp_PTAT, pmutrmlp_D1P2, pmutrmlp_DP60
        } = sfrpmutrmcrlp;
    assign {
            osc32mcfgreg, osc32kcfgreg, pmutrmreg_CUR , pmutrmreg_CTAT, pmutrmreg_PTAT, pmutrmreg_D1P2, pmutrmreg_DP60
        } = pmutrmreg;
    assign {
            pmudft_testsel, pmudft_testen
        } = sfrpmudftcr;

    assign sfrpmusr = { pmu_BGRDY, pmu_VR25RDY, pmu_VR85ARDY, pmu_VR85DRDY, ~pmu_POR };

    `theregfull( clksys1m, porresetn, pmutrmreg, IV_PMUTRM    ) <= pmutrmset_sfr ? sfrpmutrmcr : ( pmutrmset_nvr & pmutrm_nvr[63] ) ? { pmutrm_nvr[45:32], pmutrm_nvr[25:0]} : pmutrmreg;

    sync_pulse sync_pmutrmset_nvr ( .clka(pclk),    .resetn, .clkb(clksys1m), .pulsea (sfrpmutrmar), .pulseb( pmutrmset_sfr ) );

// apb sfr
// ■■■■■■■■■■■■■■■

    logic [13:0] apbao2_paddr;
    logic [31:0] apbao2_pwdata, apbao2_prdata;
    logic apbao2_pwrite, apbao2_pread, aoperi_clrint;
    logic        sfrpmupdar;
    logic        clk32kselreg;
    logic [13:0] clk1hzfd;
    logic [4:0]  rstcrmask;
    logic [9:0]  aofr;

    logic apbrd, apbwr;
    logic sfrlock;

    assign sfrlock = '0;

    `apbs_common;
    assign apbx.prdata = '0
                        | cr_apbao2_paddr.prdata32 | sr_apbao2_prdata.prdata32 | fr_aofr.prdata32
                        | sfr_pmucr.prdata32 | sfr_pmucrlp.prdata32 | sfr_pmucrpd.prdata32
                        | cr_clk32ksel.prdata32 | cr_clk1hzfd.prdata32 | cr_wkupmask .prdata32 | cr_rstcrmask.prdata32
                        | sr_pmusr.prdata32 | fr_pmufr.prdata32
                        | sfr_pmutrm.prdata32 | sfr_pmutrmlp.prdata32 | sfr_pmutrmlp.prdata32 | sfr_osctrm.prdata32
                        | sfr_pmudft.prdata32
                        ;

    apb_cr #(.A('h00), .DW(14))                 cr_apbao2_paddr  (.cr( apbao2_paddr       ), .prdata32(),.*);
    apb_cr #(.A('h04), .DW(32))                 cr_apbao2_pwdata (.cr( apbao2_pwdata      ), .prdata32(),.*);
    apb_sr #(.A('h04), .DW(32))                 sr_apbao2_prdata (.sr( apbao2_prdata      ), .prdata32(),.*);
    apb_ar #(.A('h08), .AR(32'h5a))             ar_apbao2_pwrite (.ar( apbao2_pwrite      ),.*);
    apb_ar #(.A('h08), .AR(32'ha5))             ar_apbao2_pread  (.ar( apbao2_pread       ),.*);
    apb_ar #(.A('h08), .AR(32'haa))             ar_aoperi_clrint (.ar( aoperi_clrint      ),.*);
    apb_fr #(.A('h0C), .DW(10))                 fr_aofr          (.fr( aofr               ), .prdata32(),.*);

    apb_cr #(.A('h10), .DW(10), .IV(IV_PMUCR))   sfr_pmucr     (.cr(sfrpmucr      ), .prdata32(),.*);
    apb_cr #(.A('h14), .DW(10), .IV(IV_PMUCR))   sfr_pmucrlp   (.cr(sfrpmucrlp    ), .prdata32(),.*);
    apb_cr #(.A('h18), .DW(4) )                  sfr_pmucrpd   (.cr(sfrpmucrpd    ), .prdata32(),.*);
    apb_ar #(.A('h1c), .AR(32'h5a))              sfr_pmupdar   (.ar(sfrpmupdar    ),             .*);

    apb_cr #(.A('h20), .DW(1))                   cr_clk32ksel     (.cr( clk32kselreg       ), .prdata32(),.*);
    apb_cr #(.A('h24), .DW(14), .IV(14'h3fff))   cr_clk1hzfd      (.cr( clk1hzfd           ), .prdata32(),.*);
    apb_cr #(.A('h28), .DW(10))                  cr_wkupmask      (.cr( wkupmask           ), .prdata32(),.*);
    apb_cr #(.A('h2c), .DW(5),  .IV(5'h1f))      cr_rstcrmask     (.cr( rstcrmask          ), .prdata32(),.*);

    apb_sr #(.A('h30), .DW(5))                   sr_pmusr     (.sr(sfrpmusr      ), .prdata32(),.*);
    apb_fr #(.A('h34), .DW(5))                   fr_pmufr     (.fr(~sfrpmusr      ), .prdata32(),.*);

    apb_cr #(.A('h40), .DW(26), .IV(IV_PMUTRM[25:0]))   sfr_pmutrm    (.cr(sfrpmutrmcr[25:0]   ), .prdata32(),.*);
    apb_cr #(.A('h44), .DW(26), .IV(IV_PMUTRM[25:0]))   sfr_pmutrmlp  (.cr(sfrpmutrmcrlp[25:0] ), .prdata32(),.*);
    apb_ar #(.A('h48), .AR(32'h5a))                     sfr_pmutrmar  (.ar(sfrpmutrmar   ),             .*);
    apb_cr #(.A('h4C), .DW(7*4), .IV({IV_PMUTRM[39:33],IV_PMUTRM[39:33],IV_PMUTRM[32:26],IV_PMUTRM[32:26]}))
                                                        sfr_osctrm    (.cr({sfrpmutrmcrlp[39:33],sfrpmutrmcr[39:33],sfrpmutrmcrlp[32:26],sfrpmutrmcr[32:26]} ), .prdata32(),.*);

    apb_cr #(.A('h50), .DW(6),  .IV(IV_PMUDFT))  sfr_pmudft    (.cr(sfrpmudftcr   ), .prdata32(),.*);

//  rst
// ■■■■■■■■■■■■■■■

    logic [4:0] rstsrc;

    assign rstsrc = { pmu_BGRDY, pmu_VR25RDY, pmu_VR85ARDY, pmu_VR85DRDY, ~pmu_POR } | rstcrmask;
    assign porresetn = &rstsrc & padresetn ;
    assign resetn = porresetn;

    `ifdef SIM
    parameter RSTEXTCNT = 10;
    `else
    parameter RSTEXTCNT = 312; //@32khz, 31.25 for 1ms.
    `endif

    aoresetgen #(.ICNT(1),.EXTCNT(RSTEXTCNT))gensocreset(
        .clk         ( clk32k ),
        .cmsatpg     ( cmsatpg ),
        .resetn      ( porresetn  ),
        .resetnin    ( padresetn & ~aopdreg ),
        .resetnout   ( socresetn )
    );


    `ifdef SIM
        initial begin
//            cmsatpg = '1;
            #1; cmsatpg = '0;
        end
    `else
        assign cmsatpg = 0;
    `endif

//  clk
// ■■■■■■■■■■■■■■■


//    assign clk = clksys1m;

`ifdef FPGA
    assign clkosc32k = clkxtl32k;
    assign clk32k = clkosc32k;
`else
    logic clk32k_unbuf;
    logic clk32k0, clk32k1;
    logic clkosc32ken, clkxtl32ken;

    OSC_32K osc32K ( .EN(~oscpd), .CFG(osc_osc32k_cfg),      .CKO( clkosc32k  ) );
    cgudyncswt uclk32ksel(
        .clk0   (clkosc32k),
        .clk1   (clkxtl32k),
        .resetn (porresetn),
        .clksel (clk32kselreg),
        .clk0en (clkosc32ken),
        .clk1en (clkxtl32ken)
    );
    ICG uclk32k0 ( .CK (clkosc32k), .EN ( clkosc32ken ), .SE(cmsatpg), .CKG ( clk32k0 ));
    ICG uclk32k1 ( .CK (clkxtl32k), .EN ( clkxtl32ken ), .SE(cmsatpg), .CKG ( clk32k1 ));

    CLKCELL_BUF buf_clk32k(.A(clk32k_unbuf),.Z(clk32k));
    assign clk32k_unbuf = clk32k0 | clk32k1 ;
`endif
    logic clk1hzcnthit, clk1hz, clk1hz_unbuf;
    logic [13:0] clk1hzcnt;

    `theregfull( clk32k, resetn, clk1hz_unbuf, 0 ) <= clk1hzcnthit ^ clk1hz_unbuf ;
    `theregfull( clk32k, resetn, clk1hzcnt, 0 ) <= clk1hzcnthit ? '0 : clk1hzcnt + 1 ;
    assign clk1hzcnthit = ( clk1hzcnt == clk1hzfd );

    CLKCELL_BUF buf_clk1hz(.A(clk1hz_unbuf),.Z(clk1hz));

//  for syn
// ■■■■■■■■■■■■■■■
    logic ao_iso_enable_temp;
    wire ao_iso_enable;

    assign ao_iso_enable_temp = aopdreg;

`ifdef FPGA
    assign ao_iso_enable = ao_iso_enable_temp;
`else
    BUFFD8BWP35P140 u_iso_en_buf ( .I(ao_iso_enable_temp), .Z(ao_iso_enable) );
`endif

//  apb peri
// ■■■■■■■■■■■■■■■
    logic apbao2_pwrite_clk32k;
    logic wdtintr, tmrintr, rtcintr, wdtreset;

    logic apbao2_pread_clk32k;  //eco

    sync_pulse sync_apbao2_pwrite ( .clka(pclk),    .resetn, .clkb(clk32k), .pulsea (apbao2_pwrite), .pulseb( apbao2_pwrite_clk32k ) );
    sync_pulse sync_apbao2_pread ( .clka(pclk),    .resetn, .clkb(clk32k), .pulsea (apbao2_pread), .pulseb( apbao2_pread_clk32k ) ); //eco

    ao_peri aoperi(
            .pclk            (clk32k),
            .presetn         (porresetn),
            .pwrite          (apbao2_pwrite_clk32k),
            .penable         (~apbao2_pread_clk32k),    //eco
            .psel            (apbao2_paddr[13:12]),
            .paddr           (apbao2_paddr[11:2]),
            .pwdata          (apbao2_pwdata),
            .prdata          (apbao2_prdata),

            .clk32k (clk32k),
            .clk1hz (clk1hz),

            .wdtintr(wdtintr),
            .tmrintr(tmrintr),
            .rtcintr(rtcintr),
            .wdtrst (wdtreset)
    );

// io ctrl
// ■■■■■■■■■■■■■■■

    generate
        for (genvar i = 0; i < IOC; i++) begin:gg
            assign aopad[i].po = socpad[i].po;
            assign aopad[i].pu = '1;
            assign aopad[i].oe = socpad[i].oe;
            assign socpad[i].pi = aopad[i].pi;
        end
    endgenerate

//  pd / wakeup
// ■■■■■■■■■■■■■■■

    logic [9:0] pmupdresetsrc;
    logic aoperi_clrint_clk32k;
    logic [1:0] aowkupintregs;
    logic wkupint;

    `theregfull( pclk, pmupdresetn, aopdreg, '0 ) <= sfrpmupdar | aopdreg;

    assign pmupdresetn = ( ~|pmupdresetsrc ) & porresetn;

    assign aofr = {
                ~aopad[0].pi, ~aopad[1].pi, ~aopad[2].pi, ~aopad[3].pi, ~aopad[4].pi, ~aopad[5].pi,
                wdtreset, wdtintr, tmrintr, rtcintr
                };
    assign pmupdresetsrc =
                {
                ~aopad[0].pi, ~aopad[1].pi, ~aopad[2].pi, ~aopad[3].pi, ~aopad[4].pi, ~aopad[5].pi,
                wdtreset, wdtintr, tmrintr, rtcintr
                }& ~wkupmask ;

    assign aowkupvld = ~pmupdresetn; //wakeup for socpd

    sync_pulse sync_aoperi_clrint ( .clka(pclk),    .resetn, .clkb(clk32k), .pulsea (aoperi_clrint), .pulseb( aoperi_clrint_clk32k ) );

    `theregfull( clk32k, resetn, wkupint, '0 ) <= (|pmupdresetsrc) ? '1 : aoperi_clrint_clk32k ? '0 : wkupint;
    `theregfull( pclk, resetn, aowkupintregs, '0 ) <= { aowkupintregs, wkupint };
    assign aowkupint = aowkupintregs[1];

endmodule



module aoresetgen #(
        parameter ICNT = 4,
        parameter EXTCNT = 4096,
        parameter ECW = $clog2(EXTCNT)
    )(
        input   logic               clk,
        input   logic               cmsatpg,
        input   logic               resetn,
        input   logic [0:ICNT-1]    resetnin,
        output  logic               resetnout
    );

    bit [ECW-1:0] resetextcnt;
    logic resetextcnthit;
    logic resetext;
    logic resetninx;

    assign resetninx = &resetnin & resetn;

    `theregfull(clk, resetninx, resetextcnt, '0) <= resetextcnthit ? resetextcnt : resetextcnt + 1;
    `theregfull(clk, resetninx, resetext,    '0) <= resetextcnthit ;

    assign resetextcnthit = resetextcnt == EXTCNT-1;

`ifdef FPGA
    BUFG u0 (.I(resetext), .O(resetnout));
`else
    assign resetnout = cmsatpg ? 1'b1 : resetext;
`endif
endmodule

//`ifdef SIM
//module BUFFD8BWP35P140 (
//     input  wire I,
//     output wire Z
// );
//
//    assign Z = I;
//
// endmodule
//`endif

module dummytb_ao_top (
);
    parameter IOC=6;
    parameter IPTBW = 26+7;
    apbif     apbs(),apbx();
    ioif       socpad[0:IOC-1]();
    ioif      aopad[0:IOC-1]();
    logic     clksys1m;
    logic     pclk;
    logic    clkxtl32k;
    logic                 pmutrmset_nvr;
    logic [64-1:0]     pmutrm_nvr;
    logic     ipsleep;
    logic    aowkupvld;
    logic  jtag_tck;
    logic  jtag_resetn;
    logic  tap_sel;
    logic  tap_capturedr;
    logic  tap_shiftdr;
    logic  tap_updatedr;
    logic  tap_en;
    logic  tap_tdi;
    logic  tap_tdo;
    wire  pmu_VR25EN;
    wire  pmu_VR85AEN;
    wire  pmu_VR85DEN;
    wire  pmu_BGRDY;
    wire  pmu_VR25RDY;
    wire  pmu_VR85ARDY;
    wire  pmu_VR85DRDY;
    wire  pmu_VR85A95ENA;
    wire  pmu_VR85D95ENA;
    wire  pmu_VR85AOSENA;
    wire  pmu_VR85DOSENA;
    wire  pmu_TRM_LATCH_b;
    wire [6-1:0] pmu_TRM_CUR;
    wire [5-1:0] pmu_TRM_CTAT;
    wire [5-1:0] pmu_TRM_PTAT;
    wire [5-1:0] pmu_TRM_D1P2;
    wire [5-1:0] pmu_TRM_DP60;
    wire  pmu_IOUTEN;
    wire  pmu_IBIASEN;
    wire  pmu_POR;
    wire [3-1:0] pmu_PMU_TEST_EN;
    wire [3-1:0] pmu_PMU_TEST_SEL;
    logic    padresetn;
    logic    socresetn;
    logic   pmu_POCENA;
    logic    aowkupint;
    logic [6:0] osc_osc32m_cfg;
    logic [0:5] ao_iptpo;

    ao_top2  u(.*);

endmodule : dummytb_ao_top


`ifdef SIM
module BUFFD8BWP35P140 (
     input  wire I,
     output wire Z
 );

    assign Z = I;

 endmodule
`endif
