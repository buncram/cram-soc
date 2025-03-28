`include "template.sv"
//`include "icg.v"

//import cms_pkg::*;

module sysctrl #(

    parameter ACKCNT = 8,
    parameter HCKCNT = 8,
    parameter ICKCNT = 8,
    parameter PCKCNT = 8,
    parameter IPMDC = 32

    )(

        input logic clkxtl,

        output logic clksys,
        output logic clktop,
        output logic clkper,
        output logic fclk,
        output logic aclk,
        output logic hclk,
        output logic iclk,
        output logic pclk,

        output logic fclken,
        output logic aclken,
        output logic hclken,
        output logic iclken,
        output logic pclken,

        output logic fclken2,
        output logic aclken2,
        output logic hclken2,
        output logic iclken2,
        output logic pclken2,

//        output logic fclksub,
        output logic [ACKCNT-1:0] aclksub,
        output logic [HCKCNT-1:0] hclksub,
        output logic [ICKCNT-1:0] iclksub,
        output logic [PCKCNT-1:0] pclksub,

        output logic clk1M, 
        output logic clk32k,

        input   logic       cmsatpg,
        input   cms_pkg::cmscode_e   cmscode,
        input   logic       brdone,

        input logic chipresetn,
        input logic secresetn, //##
        input logic padresetn,
        input logic wdtresetn,
        input logic vdresetn,

        output logic sysresetn,
        output logic coreresetn,
        input  logic wkupvld_async,

        input  logic [IPMDC-1:0][31:0]  iptrim32,
        input  logic                    iptrimdatavld,
        output logic                    iptrimready,

        input  logic        coresleep,
        output logic [6:0]  ipsleep,

        input logic sfrlock,
        apbif.slavein apbs,
        apbif.slave   apbx

    );

    logic [15:0]    ipc_en;
    logic [15:0]    ipc_lpen;
    wire           resetn;
    logic           ipc_oscen;
    logic [6:0]     ipc_osc;
    logic           ipc_pllen;
    logic   clktopenin;

    logic clkosc, clkpll0, clkpll1;
    logic               cfgseltop, cfgselsys, cfgset, cfgsetar;
    bit   [0:4][7:0]    cfgfd, cfgfdlp, cgufd;
    bit   [0:4][15:0]   cfgfdcr;
    bit [7:0]           clk1Mcnt;
    bit [9:0]           clk32kcnt;
    bit [3:0]           fsvld;
    bit [3:0][15:0]     fsfreq;
    bit [15:0]          fsintv;
    logic cmsresetn;
    logic [15:0]    rcufr;
    logic apbrd, apbwr;
    bit [15:0]  cgusec, cgulp;
    bit         cfgsel0, cfgsel0lp, cfgsel1;
    bit [1:0]   cfgsel0cr;
    bit [7:0]   aclksubgate, hclksubgate, iclksubgate, pclksubgate;
    logic       sysreset_sw, corereset_sw;
    bit [15:0] ipccr;
    logic [16:0]  ipc_pllmn;
    logic [24:0]  ipc_pllf;
    logic [14:0]  ipc_pllq;
    bit pdreg;
    logic lp_deepen;
    logic pdresetn;
    bit [1:0] coresleepregs_clksys;
    logic clk32m;

    localparam RCUEXTCNT = 256;

// cgu source
// ■■■■■■■■■■■■■■■
        // depends on the IP

            parameter PLL_PREDIV_W  = 5  ;
            parameter PLL_FBDIV_W   = 12  ;
            parameter PLL_FRAC_W    = 24  ;
            parameter PLL_POSTDIV_W0 = 3  ;
            parameter PLL_POSTDIV_W1 = 3 ;

            logic ipc_setcfgpll;
            logic [ PLL_PREDIV_W-1   : 0 ] pll_m   ;
            logic [ PLL_FBDIV_W -1   : 0 ] pll_n   ;
            logic [ PLL_FRAC_W-1     : 0 ] pll_f   ;
            logic                          pll_fen ;
            logic [ PLL_POSTDIV_W0-1 : 0 ] pll_q00 ;
            logic [ PLL_POSTDIV_W1-1 : 0 ] pll_q10 ;
            logic [ PLL_POSTDIV_W0-1 : 0 ] pll_q01 ;
            logic [ PLL_POSTDIV_W1-1 : 0 ] pll_q11 ;

            assign pll_m = ipc_pllmn[16:12];
            assign pll_n = ipc_pllmn[11: 0];
            assign pll_f = ipc_pllf[23: 0];
            assign pll_fen = ipc_pllf[24];
            assign pll_q00 = ipc_pllq[ 2: 0];
            assign pll_q10 = ipc_pllq[ 6: 4];
            assign pll_q01 = ipc_pllq[10: 8];
            assign pll_q11 = ipc_pllq[14:12];


    `ifdef FPGA

//        localparam PLLMNIV = 16'h4080; // 200
        localparam PLLMNIV = 16'h4040; // 100

            logic [7:0] clkpll1cfg, clkpll0cfg;

            assign { clkpll1cfg, clkpll0cfg } = ipc_pllmn[15:0];
            assign clkosc = clk32m;


            dyna_clk_dual #(
                .START_F(16) 
            ) udrp (
                .osc_clk        (clkxtl),
                .clk_sel_pins0  (clkpll0cfg),
                .clk_sel_pins1  (clkpll1cfg),
                .clkdrp0        (clkpll0),
                .clkdrp1        (clkpll1),
                .clk_32M        (clk32m),
                .led            (),
                .o_lock         ()
            );

    `else

        localparam PLLMNIV = 16'h0; 
//        assign clk32m = clkxtl;
        CLKCELL_BUF buf_clkxtl(.A(clkxtl),.Z(clk32m));

        OSC_32M osc32M ( .EN(ipc_oscen), .CFG(ipc_osc),      .CKO( clkosc  ) );
//        OSC_SIM #(1.30)     pll0   ( .EN(ipc_pllen), .CFG(ipc_pllcfg[6:0]), .CKO( clkpll0 ) );
//        OSC_SIM #(10)       pll1   ( .EN(ipc_pllen), .CFG(ipc_pllcfg[6:0]), .CKO( clkpll1 ) );

        cgupll #(
            .PREDIV_W   ( PLL_PREDIV_W   ),
            .FBDIV_W    ( PLL_FBDIV_W    ),
            .FRAC_W     ( PLL_FRAC_W     ),
            .POSTDIV_W0 ( PLL_POSTDIV_W0 ),
            .POSTDIV_W1 ( PLL_POSTDIV_W1 )
        )upll(
            .clk      ( clksys ),
            .resetn   ( sysresetn ),
            .cmsatpg  ( cmsatpg ),
            .setcfg   ( ipc_setcfgpll ),
            .refclk   ( clksys  ),
            .pllen    ( ipc_pllen   ),
            .pll_m    ( pll_m   ),
            .pll_n    ( pll_n   ),
            .pll_f    ( pll_f   ),
            .pll_fen  ( pll_fen ),
            .pll_q00  ( pll_q00 ),
            .pll_q10  ( pll_q10 ),
            .pll_q01  ( pll_q01 ),
            .pll_q11  ( pll_q11 ),
            .clkpll0, 
            .clkpll1
        );

    `endif

// cgu path sel, cgu fd
// ■■■■■■■■■■■■■■■

    bit [2:0] cfgsetinitregs;
    bit       cfgsetinit;
    logic cfgsetslp, cfgsetwkup;
    logic cfgsetdpslp, cfgsetdpwkup;
    bit [0:1]   clksysselen, clktopselen;

    cgucore
    #(
//        .FDW = 8,
//        .GEARLMT = 2**FDW
    )ucgucore(
        /*input   logic [0:ICNT-1]    */.clksrc             ({clkosc,clk32m,clkpll0,clkpll1}),   
                                        .cmsatpg            (cmsatpg),
        /*input   logic               */.porresetn          (sysresetn),   
        /*input   logic               */.resetn             (sysresetn),//coreresetn),   
        /*input   logic               */.clksyssel          (cfgselsys),       
        /*input   logic               */.clktopselupdate    (cfgset|cfgsetinit|cfgsetslp|cfgsetwkup|cfgsetdpslp|cfgsetdpwkup),
        /*input   logic               */.clktopsel          (cfgseltop),       
        /*output  logic               */.clksys             (clksys),   
        /*output  logic               */.clktop             (clktop),   
        /*output  logic               */.clkper             (clkper),   
        /*input   logic               */.clktopenin         (clktopenin),       
        /*input   bit   [0:OCNT-1][FDW*/.fd                 (cgufd),
        /*input   bit                 */.fdload             (cfgset|cfgsetinit|cfgsetslp|cfgsetwkup|cfgsetdpslp|cfgsetdpwkup),
        /*output  logic [0:OCNT-1]    */.clkout             ({fclk,aclk,hclk,iclk,pclk}),   
        /*output  logic [0:OCNT-1]    */.clkouten           ({fclken,aclken,hclken,iclken,pclken}),   
        /*output  logic [0:OCNT-1]    */.clkouten_atparent  ({fclken2,aclken2,hclken2,iclken2,pclken2}),
                                        .clksysselen        (clksysselen),
                                        .clktopselen        (clktopselen)
    );

    assign cfgsel0   = cfgsel0cr[0];
    assign cfgsel0lp = cfgsel0cr[1];
    assign cfgseltop = cfgsetdpslp ? '0 : cfgsetslp ? cfgsel0lp : cfgsel0;

    assign { cfgfdlp[0], cfgfd[0] } = cfgfdcr[0];
    assign { cfgfdlp[1], cfgfd[1] } = cfgfdcr[1];
    assign { cfgfdlp[2], cfgfd[2] } = cfgfdcr[2];
    assign { cfgfdlp[3], cfgfd[3] } = cfgfdcr[3];
    assign { cfgfdlp[4], cfgfd[4] } = cfgfdcr[4];

    assign cgufd = cfgsetdpslp ? '0 : cfgsetslp ? cfgfdlp : cfgfd;

    assign cfgselsys = cfgsel1;
    assign clktopenin = 1'b1;

//    `theregfull(clktop, coreresetn, cfgsetinitregs, 3'h1 ) <= cfgsetinitregs * 2;
    assign cfgsetinit = '0;//cfgsetinitregs[1];

    assign cfgset = cfgsetar & pclken;


// clk subgating

genvar gvi;

generate
    for (gvi = 0; gvi < ACKCNT; gvi++) begin: genaclksub
            ICG uaclksub ( .CK (clktop   ), .EN ( aclken & aclksubgate[gvi] ), .SE(cmsatpg), .CKG ( aclksub[gvi] ));
    end
    for (gvi = 0; gvi < HCKCNT; gvi++) begin: genhclksub
            ICG uhclksub ( .CK (clktop   ), .EN ( hclken & hclksubgate[gvi] ), .SE(cmsatpg), .CKG ( hclksub[gvi] ));
    end
    for (gvi = 0; gvi < ICKCNT; gvi++) begin: geniclksub
            ICG uiclksub ( .CK (clktop   ), .EN ( iclken & iclksubgate[gvi] ), .SE(cmsatpg), .CKG ( iclksub[gvi] ));
    end
    for (gvi = 0; gvi < PCKCNT; gvi++) begin: genpclksub
            ICG upclksub ( .CK (clktop   ), .EN ( pclken & pclksubgate[gvi] ), .SE(cmsatpg), .CKG ( pclksub[gvi] ));
    end
endgenerate

// cgu: freq meter, fixed clk
// ■■■■■■■■■■■■■■■

    freqmeter #(
            .FSCNT(4)
        )cgufs(
            .clk        (clksys),
            .cmsatpg    (cmsatpg),
            .resetn     (resetn),
            .interval   (fsintv),
            .clkin      ({clkosc,clk32m,clkpll0,clkpll1}),
            .fsvld      (fsvld),
            .fsfreq     (fsfreq)
        );

    `theregfull( clksys, sysresetn, clk1Mcnt, 0 ) <= ( clk1Mcnt == fsintv -1 ) ? 0 : clk1Mcnt + 1;
    `theregfull( clksys, sysresetn, clk1M, 0 )    <= ( clk1Mcnt == fsintv/2 ) ? 1'b1 : ( clk1Mcnt == fsintv -1 ) ? 1'b0 : clk1M;

    `theregfull( clksys, sysresetn, clk32kcnt, 0 ) <= ( clk32kcnt == 1000 -1 ) ? 0 : clk32kcnt + 1;
    `theregfull( clksys, sysresetn, clk32k, 0 )    <= ( clk32kcnt == 1000/2 ) ? 1'b1 : ( clk32kcnt == 1000 -1 ) ? 1'b0 : clk32k;

// ip ctrl
// ■■■■■■■■■■■■■■■

// static ip control

    assign ipc_oscen = ( ipc_en[0] | cfgsel0 == 0 ) & ~ipsleep[0] | clksysselen[0] | ~clktopselen[1];
    assign ipc_pllen = ( ipc_en[1] | cfgsel1 == 1 ) & ~ipsleep[1] | clktopselen[1] | ~clktopselen[0];


// ip fsm flow

    localparam IPFLOWFSM_FDOFF = 8'd16;
    localparam IPFLOWFSM_PD = 8'd32;
    localparam IPFLOWFSM_FDON = 8'd243;
    localparam IPFLOWFSM_DONE = 8'd255;

    logic [7:0] ipflowfsm;
    logic       ipflowstart;
    logic       ipflow_settrim, ipflow_setar, ipflow_ipsleep, ipflow_settrim_corereset;
    logic [2:0] coreresetnregs;

`ifdef FPGA
    assign ipflowstart = '0;
    assign ipflowfsm = '0;
`else
    assign ipflowstart = ipflow_settrim | ipflow_setar | ipflow_ipsleep | ipflow_settrim_corereset; // @clksys
    `theregfull( clksys, sysresetn, ipflowfsm, '1 ) <= ( ipflowfsm != IPFLOWFSM_DONE ) | ipflowstart ? 
                                                                (( ipflowfsm == IPFLOWFSM_PD ) & coresleep & pdreg  ? ipflowfsm : ipflowfsm + 1 ) : ipflowfsm;
`endif

    logic ipflowfsm_pd, ipflowfsm_fdoff, ipflowfsm_fdon;
    `theregfull( clksys, sysresetn, ipflowfsm_pd,    '0 ) <= ( ipflowfsm == IPFLOWFSM_PD );
    `theregfull( clksys, sysresetn, ipflowfsm_fdoff, '0 ) <= ( ipflowfsm == IPFLOWFSM_FDOFF );
    `theregfull( clksys, sysresetn, ipflowfsm_fdon,  '0 ) <= ( ipflowfsm == IPFLOWFSM_FDON );

    sync_pulse ipflowfoff(
        .clka       (clksys),
        .resetn     (sysresetn),
        .pulsea     (ipflowfsm_fdoff),
        .clkb       (clktop),
        .pulseb     (cfgsetdpslp)
    );    

    sync_pulse ipflowfon(
        .clka       (clksys),
        .resetn     (sysresetn),
        .pulsea     (ipflowfsm_fdon),
        .clkb       (clktop),
        .pulseb     (cfgsetdpwkup)
    );    
    
    `theregfull( clksys, sysresetn, coreresetnregs, '0 ) <= { coreresetnregs, coreresetn };
    assign ipflow_settrim_corereset = ~coreresetnregs[1] & coreresetnregs[2];

// iptrim handshake

    logic iptrimdatavldreg, iptrimbusy;

    `theregfull( clksys, sysresetn, iptrimbusy, '0) <= ipflow_settrim ? '1 : ( ipflowfsm == IPFLOWFSM_DONE ) ? '0 : iptrimbusy;
    `theregfull( clksys, sysresetn, iptrimready, '0 ) <= iptrimready | (( ipflowfsm == IPFLOWFSM_DONE ) & iptrimbusy );
    `theregfull( clksys, sysresetn, iptrimdatavldreg, '0 ) <= iptrimdatavld;
    assign ipflow_settrim = iptrimdatavldreg;


// lp
// ■■■■■■■■■■■■■■■

    logic lp_fdlpen, lp_iplpen, lp_clkstplpen;

    assign lp_fdlpen = cgulp[0] & ~lp_deepen;
    assign lp_iplpen = cgulp[1];
    assign lp_deepen = cgulp[2];

// cgu lp

    logic coresleepreg, coresleeprise, coresleepfall;

    assign cfgsetwkup = coresleepfall & lp_fdlpen;
    assign cfgsetslp  = coresleeprise & lp_fdlpen;
    `theregfull( clktop, coreresetn, coresleepreg, '0 ) <= coresleep;
    assign coresleeprise =  coresleep & ~coresleepreg;
    assign coresleepfall = ~coresleep &  coresleepreg;

    `theregfull( clksys, pdresetn, coresleepregs_clksys, '0 ) <= { coresleepregs_clksys, 1'b1};
    `theregfull( clksys, pdresetn, pdreg, '0 ) <= ( coresleepregs_clksys[0] & ~coresleepregs_clksys[1] ) & lp_deepen | pdreg;
    assign pdresetn = coreresetn & ~wkupvld_async & coresleep;

// ip lp

    assign ipsleep = ipflow_ipsleep ? ipc_lpen : '0;

    assign ipflow_ipsleep = coresleep & lp_iplpen;

// reset ctrl
// ■■■■■■■■■■■■■■■

    assign cmsresetn = ( cmscode == cms_pkg::CMS_USER ) & brdone;
    assign rcufr = '0 | ~{ sysresetgen.resetnin, coreresetgen.resetnin };

    resetgen #(.ICNT(4),.EXTCNT(RCUEXTCNT))sysresetgen(
        .clk         ( clksys ),
        .cmsatpg     ( cmsatpg ),
        .resetn      ( chipresetn ),
        .resetnin    ( { chipresetn, vdresetn, secresetn, ~sysreset_sw } ),
        .resetnout   ( sysresetn )
    );

    resetgen #(.ICNT(5),.EXTCNT(RCUEXTCNT))coreresetgen(
        .clk         ( clksys ),
        .cmsatpg     ( cmsatpg ),
        .resetn      ( sysresetn ),
        .resetnin    ( { padresetn, sysresetn, cmsresetn, wdtresetn, ~corereset_sw } ),
        .resetnout   ( coreresetn )
    );

    assign resetn = coreresetn;

// sfr
// ■■■■■■■■■■■■■■■

    `apbs_common;
    assign apbx.prdata = '0
                | sfr_cgusec.prdata32  | sfr_cgulp.prdata32 
                | sfr_cgusel0.prdata32 | sfr_cgufd.prdata32 | sfr_cgusel1.prdata32 
                | sfr_cgufssr.prdata32 | sfr_cgufscr.prdata32 | sfr_cgufsvld.prdata32 
                | sfr_aclkgr.prdata32  | sfr_hclkgr.prdata32 | sfr_iclkgr.prdata32 | sfr_pclkgr.prdata32  
                | sfr_rcusrcfr.prdata32 
                | sfr_ipccr.prdata32 | sfr_ipcen.prdata32 | sfr_ipclpen.prdata32 | sfr_ipcosc.prdata32
                | sfr_ipcpllmn.prdata32 | sfr_ipcpllf.prdata32 | sfr_ipcpllq.prdata32
                ;

// cgu cfg cr
    apb_cr #(.A('h00), .DW(16))     sfr_cgusec  (.cr(cgusec),   .prdata32(),.*);
    apb_cr #(.A('h04), .DW(16))     sfr_cgulp   (.cr(cgulp ),   .prdata32(),.*);

    apb_cr #(.A('h10), .DW(1*2))      sfr_cgusel0 (.cr(cfgsel0cr),  .prdata32(),.*);
    apb_cr #(.A('h14), .DW(8*2), 
             .IV('h7f7f), .SFRCNT(5))   
                                    sfr_cgufd   (.cr(cfgfdcr),    .prdata32(),.*);
    apb_ar #(.A('h2c), .AR('h32))   sfr_cguset  (.ar(cfgsetar),               .*);

    apb_cr #(.A('h30), .DW(1))      sfr_cgusel1 (.cr(cfgsel1),  .prdata32(),  .resetn(sysresetn),.*);

// cgu freq meter 
    apb_sr #(.A('h40), .DW(16), .SFRCNT(4)   )  sfr_cgufssr   (.sr(fsfreq),     .prdata32(),.*);
    apb_sr #(.A('h50), .DW(4)            )      sfr_cgufsvld  (.sr(fsvld),      .prdata32(),.*);
    apb_cr #(.A('h54), .DW(16), .IV('d32))      sfr_cgufscr   (.cr(fsintv),     .prdata32(),.*);

// clkgate
    apb_cr #(.A('h60), .DW(ACKCNT),   .IV('hff)) sfr_aclkgr     (.cr(aclksubgate),.prdata32(),.*);
    apb_cr #(.A('h64), .DW(HCKCNT),   .IV('hff)) sfr_hclkgr     (.cr(hclksubgate),.prdata32(),.*);
    apb_cr #(.A('h68), .DW(ICKCNT),   .IV('hff)) sfr_iclkgr     (.cr(iclksubgate),.prdata32(),.*);
    apb_cr #(.A('h6c), .DW(PCKCNT),   .IV('hff)) sfr_pclkgr     (.cr(pclksubgate),.prdata32(),.*);

// rcu
    apb_ar #(.A('h80), .AR('h55aa))        sfr_rcurst0    (.ar(sysreset_sw),   .*);
    apb_ar #(.A('h84), .AR('h55aa))        sfr_rcurst1    (.ar(corereset_sw),  .*);
    apb_fr #(.A('h88), .DW(16)    )        sfr_rcusrcfr   (.fr(rcufr),  .prdata32(),.*);

// ipc
    apb_ar #(.A('h90), .AR('h32))              sfr_ipcarpll       (.ar(ipc_setcfgpll),                .*);
    apb_ar #(.A('h90), .AR('h57))              sfr_ipcaripflow    (.ar(ipflow_setar),                .*);
    apb_cr #(.A('h94), .DW(16), .IV('h01) )    sfr_ipcen    (.cr(ipc_en),    .prdata32(),.*);
    apb_cr #(.A('h98), .DW(16), .IV('h01) )    sfr_ipclpen  (.cr(ipc_lpen),  .prdata32(),.*);
    apb_cr #(.A('h9c), .DW(7),  .IV('h26) )    sfr_ipcosc   (.cr(ipc_osc),   .prdata32(),.*);
    apb_cr #(.A('ha0), .DW(17), .IV(PLLMNIV))  sfr_ipcpllmn (.cr(ipc_pllmn), .prdata32(),.*);
    apb_cr #(.A('ha4), .DW(25), .IV('hff) )    sfr_ipcpllf  (.cr(ipc_pllf),  .prdata32(),.*);
    apb_cr #(.A('ha8), .DW(15), .IV('0)   )    sfr_ipcpllq  (.cr(ipc_pllq),  .prdata32(),.*);
    apb_cr #(.A('hac), .DW(16), .IV('0)   )    sfr_ipccr    (.cr(ipccr),     .prdata32(),.*);

endmodule : sysctrl

module resetgen #(
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

/*
module ipctrl (                         
    input clk,    // Clock                      
    input resetn, // Clock Enable                       
    input rst_n,  // Asynchronous reset active low                      
                            
);                          
                            
                            
                            
                            
                            
    ## ipc_nvrld <= bistrd_ipcldreg | socresetrise                      
                            
                            
                            
                            
ipc_nvr, ipc_nvrld                          
ipc_sfr, ipc_sfrld                          
ipc_lp, lpc_lpmd                            
                            
ipc_out                         
                            
                            
ipc_nvrld                           
                            
                            
    `theregrn( ipc_sfr ) <=  ipc_nvrld ? ipc_nvr : ipc_sfr;                     
                            
    `theregrn( ipc_sfrreg ) <=  ipc_nvrld ? ipc_nvr :
                                ipc_sfrld ? ipc_sfr : ipc_sfrreg;   
                            
                            
ipc_out = socresetn ? ipc_sfrmux : ipc_nvr;                         
ipc_sfrmux = lpc_lpmd ? ipc_lp : ipc_sfrreg;                            
                            
                            
                            
                            
endmodule
*/

module dummytb_sysctrl();
    parameter IPMDC = 32;

         bit clkxtl;
         bit clksys;
         bit clktop;
         bit clkper;
         bit fclk;
         bit aclk;
         bit hclk;
         bit iclk;
         bit pclk;
         bit fclken;
         bit aclken;
         bit hclken;
         bit iclken;
         bit pclken;
         bit fclken2;
         bit aclken2;
         bit hclken2;
         bit iclken2;
         bit pclken2;
         bit [7:0] aclksub;
         bit [7:0] hclksub;
         bit [7:0] iclksub;
         bit [7:0] pclksub;
         bit clk1M;
         bit clk32k;
           bit       cmsatpg;
           cms_pkg::cmscode_e   cmscode;
         bit chipresetn;
         bit secresetn;
         bit padresetn;
         bit wdtresetn;
         bit vdresetn;
         bit sysresetn;
         bit coreresetn;
         bit sfrlock;
         bit brdone;
         bit wkupvld_async, coresleep;
         bit [6:0]  ipsleep;
    logic [IPMDC-1:0][31:0] iptrim32        ;
    logic                   iptrimdatavld   ;
    logic                   iptrimready     ;


apbif apbs();
apbif apbx();
sysctrl u1(.*);

endmodule

