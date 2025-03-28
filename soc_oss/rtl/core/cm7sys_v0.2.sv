`include "template.sv"

module cm7sys
  #(

    // ref doc-iim 4.17

    parameter PM_CFGITCMSZ = 4'h7,  // 64KB
    parameter PM_CFGDTCMSZ = 4'h7,  // 64KB
    parameter PM_CFGAHBPSZ = 3'h2,  // 128MB
    parameter PM_CFGSTCALIB_10MS = 24'd_9_999, // @1MHz
    parameter PM_INITRETRYEN = 2'h0,    // [0] itcm, [1] dtcm
    parameter PM_INITRMWEN = 2'h0,      // [0] itcm, [1] dtcm
    parameter PM_COREUSERCNT = 4,

    parameter PM_AXIM_IDW = 8,
    parameter PM_AXIM_LENW = 8,
    parameter PM_AXIM_UW = 8,
    parameter PM_AHBP_IDW = 8,
    parameter PM_AHBP_UW = 8,
    parameter PM_AHBS_IDW = 8,
    parameter PM_AHBS_UW = 8,

    // ------------------------------------------------------------------------
    // Cortex-M7 Processor Parameterization
    // ------------------------------------------------------------------------
    `include "cm7sys_cfg_v0.1.svh"
    ,
    // ------------------------------------------------------------------------
    parameter SWMD      = 0,     // Serial Wire Multi Drop support
                                 //   0 = no support
                                 //   1 = spported
    // ------------------------------------------------------------------------
    // The following parameters are for the DAP
    // ------------------------------------------------------------------------
    parameter BASEADDR =  32'hE00FD003,
                                // Allows configuration of the ROM table
                                // base address which is read from the
                                // AP during debug sessions. This is pointing
                                // to the system level (MCU) ROM table by
                                // default (address 0xE00FD000).
                                // If additional level of ROM table are added,
                                // this should be overridden to point to the
                                // highest level ROM table.
                                // If the system level (MCU) ROM table address
                                // is changed (specific by SYSROMTABLEBASE, a
                                // localparam in this file), this value also
                                // need to be updated.
    // ------------------------------------------------------------------------
    parameter TARGETID   = 32'h00000000,
                                // 31:28=TREVISION 27:12=TPARTNO
                                // 11:1=TDESIGNER 0=1
    // ------------------------------------------------------------------------
    // The following parameters are for the main ROM table
    // ------------------------------------------------------------------------
    parameter JEPID      = 7'h00,  // JEP106 identification code
    // ------------------------------------------------------------------------
    parameter JEPCONT    = 4'h0,   // JEP106 continuation code
    // ------------------------------------------------------------------------
    parameter PARTNUM    = 12'h000 // Part number (for MCU)
                                   // Reflected in PIDR0 and PIDR1
    // ------------------------------------------------------------------------

)(

// system ctrl
    input   logic               clk,            // Free running clock
    input   logic               resetn,
    input   logic               coreresetn,

    output  logic               cm7_resetreq,
    output  logic               cm7_sleep,
    input   logic               clkcm7sten_1M,

// cfg
    input   logic               cm7cfg_dev,
    input   logic [31:0]        cm7cfg_iv,// = 32'h6000_0000;

// test mode
    input   logic               cmsatpg,
    input   logic               cmsbist,
//    mbist.master                mbistif,

// interrupt, nmi, events
    input   logic [IRQNUM-1:0]  cm7_irq,
    input   logic               cm7_nmi,
    input   logic               cm7_rxev,

// amba
    input   logic               aximclken,       // axi clk enable
    input   logic               ahbpclken,
    input   logic               ahbsclken,
    axiif.master                axim,
    ahbif.master                ahbp,
    ahbif.slave                 ahbs,

// coreuser 
    input   axi_pkg::xbar_rule_32_t [0:PM_COREUSERCNT-1]       coreusermap,
    output  logic                   [0:PM_COREUSERCNT-1]       coreuser,

// debug
    input   logic               swclk,
    ioif.drive                  swdio

);

// axim cfg:
//
//  .AW     (32),
//  .DW     (64),
//  .IDW    (8),
//  .LENW   (8),
//  .UW     (8)

// ahbp/ahbs cfg:
//
//  .AW     (32),
//  .DW     (32),
//  .IDW    (8),
//  .UW     (8)


// bit

  bit [63:0]  TSVALUEB;
  bit                      sys_itcmcs;
  bit [23:3]               sys_itcmaddr;
  bit [ 7:0]               sys_itcmbytewr;
  bit [63:0]               sys_itcmwdata;
  bit [63:0]               sys_itcmrdata;
  bit [63:0]               sys_itcmrdata0;
  bit                      sys_itwait;
  bit                      sys_iterr;
  bit                      sys_itretry;
  bit                      sys_d0tcmcs;
  bit [23:3]               sys_d0tcmaddr;
  bit [ 3:0]               sys_d0tcmbytewr;
  bit [31:0]               sys_d0tcmwdata;
  bit [31:0]               sys_d0tcmrdata;
  bit                      sys_d0wait;
  bit                      sys_d0err;
  bit                      sys_d0retry;
  bit                      sys_d1tcmcs;
  bit [23:3]               sys_d1tcmaddr;
  bit [ 3:0]               sys_d1tcmbytewr;
  bit [31:0]               sys_d1tcmwdata;
  bit [31:0]               sys_d1tcmrdata;
  bit                      sys_d1wait;
  bit                      sys_d1err;
  bit                      sys_d1retry;


  bit          ahbd_hready;
  bit          ahbd_hresp;
  bit [31:0]   ahbd_hrdata;
  bit [ 1:0]   ahbd_htrans;
  bit          ahbd_hwrite;
  bit [ 1:0]   ahbd_hsize;
  bit [ 2:0]   ahbd_hburst;
  bit [ 3:0]   ahbd_hprot;
  bit [31:0]   ahbd_haddr;
  bit [31:0]   ahbd_hwdata;


    ahbif ahbp0();
    ahbif ahbs0();

    logic                   dbghalt;

    logic clkcm7in, clkcm7en, clkcm7fen, clkcm7hen;
    logic cm7_sleeping, cm7_sleepdeep, cm7_gatehclk;


    assign clkcm7in = clk;

//    assign clkcm7en = ~cm7_sleep;
    assign clkcm7en = ~cm7_sleeping;
    assign clkcm7fen = 1'b1;
    assign clkcm7hen = clkcm7en | ~cm7_gatehclk;

    assign cm7_sleep = cm7_sleeping & cm7_sleepdeep;
    assign nPORESET = resetn;
    assign nSYSRESET = coreresetn;

  CORTEXM7INTEGRATIONCS
    #(
      .FPU                           (FPU),
      .ICACHE                        (ICACHE),
      .DCACHE                        (DCACHE),
      .CACHEECC                      (CACHEECC),
      .MPU                           (MPU),
      .IRQNUM                        (IRQNUM),
      .IRQLVL                        (IRQLVL),
      .DBGLVL                        (DBGLVL),
      .TRC                           (TRC),
      .LOCKSTEP                      (LOCKSTEP),
      .RAR                           (RAR),
      .DW                            (DW),
      .ETM                           (ETM),
      .CTI                           (CTI),
      .WIC                           (WIC),
      .WICLINES                      (WICLINES),
      .ICACHESIZE                    (ICACHESIZE),
      .DCACHESIZE                    (DCACHESIZE)
     )
  cm7
    (

      // ---------------------------------------------------------------------
      // ATB-D interface: unused
      // ---------------------bbb------------------------------------------------

      .ATREADYMD                      (1'b1),
      .AFVALIDMD                      (1'b0),
      .ATVALIDMD                      (),
      .AFREADYMD                      (),
      .ATDATAMD                       (),
      .ATBYTESMD                      (),
      .ATIDMD                         (),


      .CLKIN                          (clkcm7in),
      .CLKEN                          (clkcm7en),
      .FCLKEN                         (clkcm7fen),
      .HCLKEN                         (clkcm7hen),
      .CLK1EN                         (1'b0),
      .FCLK1EN                        (1'b0),
      .HCLK1EN                        (1'b0),
      .ETMCLKEN                       (1'b0),       // ref from IK's sim
      .STCLKEN                        (clkcm7sten_1M),
      .nSYSRESET                      (coreresetn),
      .SYSRESETREQ                    (cm7_resetreq),
      .nPORESET                       (resetn),
      .nDBGETMRESET                   (1'b0),
      .CPUWAIT                        (1'b0),

      .CFGBIGEND                      (1'b0),
      .CFGITCMSZ                      (PM_CFGITCMSZ),
      .CFGDTCMSZ                      (PM_CFGDTCMSZ),
      .CFGAHBPSZ                      (PM_CFGAHBPSZ),
      .CFGSTCALIB                     (PM_CFGSTCALIB_10MS|26'h0),

      .INITTCMEN                      (2'b11),
      .INITRMWEN                      (2'b00),
      .INITRETRYEN                    (2'b00),
      .INITAHBPEN                     (1'b1),
      .INITVTOR                       (cm7cfg_iv[31:7]),

      .ITCMCS                         (sys_itcmcs),
      .ITCMADDR                       (sys_itcmaddr[23:3]),
      .ITCMWR                         (),
      .ITCMBYTEWR                     (sys_itcmbytewr[7:0]),
      .ITCMPRIV                       (),
      .ITCMMASTER                     (),
      .ITCMWDATA                      (sys_itcmwdata[63:0]),
      .ITCMMBISTIN                    (),
      .ITCMWAIT                       (sys_itwait),
      .ITCMERR                        (sys_iterr),
      .ITCMRDATA                      (sys_itcmrdata[63:0]),
      .ITCMRETRY                      (sys_itretry),
      .ITCMMBISTOUT                   ({8{1'b0}}),

      .D0TCMCS                        (sys_d0tcmcs),
      .D0TCMADDR                      (sys_d0tcmaddr[23:3]),
      .D0TCMWR                        (),
      .D0TCMBYTEWR                    (sys_d0tcmbytewr[3:0]),
      .D0TCMPRIV                      (),
      .D0TCMMASTER                    (),
      .D0TCMWDATA                     (sys_d0tcmwdata[31:0]),
      .D0TCMMBISTIN                   (),
      .D0TCMWAIT                      (sys_d0wait),
      .D0TCMERR                       (sys_d0err),
      .D0TCMRDATA                     (sys_d0tcmrdata[31:0]),
      .D0TCMRETRY                     (sys_d0retry),
      .D0TCMMBISTOUT                  ({7{1'b0}}),
      .D1TCMCS                        (sys_d1tcmcs),
      .D1TCMADDR                      (sys_d1tcmaddr[23:3]),
      .D1TCMWR                        (),
      .D1TCMBYTEWR                    (sys_d1tcmbytewr[3:0]),
      .D1TCMPRIV                      (),
      .D1TCMMASTER                    (),
      .D1TCMWDATA                     (sys_d1tcmwdata[31:0]),
      .D1TCMMBISTIN                   (),
      .D1TCMWAIT                      (sys_d1wait),
      .D1TCMERR                       (sys_d1err),
      .D1TCMRDATA                     (sys_d1tcmrdata[31:0]),
      .D1TCMRETRY                     (sys_d1retry),
      .D1TCMMBISTOUT                  ({7{1'b0}}),

      .ACLKEN                         (aximclken),

      .ARVALID                        (axim.arvalid),
      .ARADDR                         (axim.araddr),
      .ARID                           (axim.arid[2:0]),
      .ARBURST                        (axim.arburst),
      .ARLEN                          (axim.arlen[2:0]),
      .ARSIZE                         (axim.arsize[1:0]),
      .ARLOCK                         (axim.arlock),
      .ARCACHE                        (axim.arcache),
      .ARPROT                         (axim.arprot),
      .ARMASTER                       (),
      .ARINNER                        (),
      .ARSHARE                        (),
      .ARREADY                        (axim.arready),

      .AWVALID                        (axim.awvalid),
      .AWADDR                         (axim.awaddr),
      .AWID                           (axim.awid[1:0]),
      .AWBURST                        (axim.awburst),
      .AWLEN                          (),
      .AWSIZE                         (axim.awsize[1:0]),
      .AWLOCK                         (axim.awlock),
      .AWCACHE                        (axim.awcache),
      .AWPROT                         (axim.awprot),
      .AWMASTER                       (),
      .AWINNER                        (),
      .AWSHARE                        (),
      .AWSPARSE                       (),
      .AWREADY                        (axim.awready),
      .RREADY                         (axim.rready),
      .RVALID                         (axim.rvalid),
      .RID                            (axim.rid[2:0]),
      .RLAST                          (axim.rlast),
      .RDATA                          (axim.rdata),
      .RRESP                          (axim.rresp),
      .WVALID                         (axim.wvalid),
      .WID                            (),
      .WDATA                          (axim.wdata),
      .WSTRB                          (axim.wstrb),
      .WLAST                          (axim.wlast),
      .WREADY                         (axim.wready),
      .BREADY                         (axim.bready),
      .BVALID                         (axim.bvalid),
      .BID                            (axim.bid[1:0]),
      .BRESP                          (axim.bresp),

      .HTRANSP                        (ahbp0.htrans),
      .HWRITEP                        (ahbp0.hwrite),
      .HSIZEP                         (ahbp0.hsize),
      .HBURSTP                        (ahbp0.hburst),
      .HPROTP                         (ahbp0.hprot),
      .HMASTERP                       (ahbp0.hmaster[0]),
      .HADDRP                         (ahbp0.haddr),
      .HWDATAP                        (ahbp0.hwdata),
      .HREADYP                        (ahbp0.hready),
      .HRESPP                         (ahbp0.hresp),
      .HRDATAP                        (ahbp0.hrdata),
      .EXREQP                         (),
      .EXRESPP                        (1'b0),

      .HREADYOUTS                     (ahbs0.hready),
      .HRESPS                         (ahbs0.hresp),
      .HRDATAS                        (ahbs0.hrdata),
      .AHBSRDY                        (),
      .AHBSPRI                        (1'b0),
      .WABORTS                        (),
      .HREADYS                        (ahbs0.hreadym),
      .HSELS                          (ahbs0.hsel),
      .HTRANSS                        (ahbs0.htrans),
      .HWRITES                        (ahbs0.hwrite),
      .HSIZES                         (ahbs0.hsize),
      .HBURSTS                        (ahbs0.hburst),
      .HPROTS                         (ahbs0.hprot),
      .HADDRS                         (ahbs0.haddr),
      .HWDATAS                        (ahbs0.hwdata),

      .HREADYD                        (ahbd_hready),
      .HRESPD                         (ahbd_hresp),
      .HRDATAD                        (ahbd_hrdata),
      .HTRANSD                        (ahbd_htrans),
      .HWRITED                        (ahbd_hwrite),
      .HSIZED                         (ahbd_hsize|3'h0),
      .HBURSTD                        (ahbd_hburst),
      .HPROTD                         (ahbd_hprot),
      .HADDRD                         (ahbd_haddr),
      .HWDATAD                        (ahbd_hwdata),

      .PENABLE                        (),
      .PSEL                           (),
      .PADDR                          (),
      .PADDR31                        (),
      .PWRITE                         (),
      .PWDATA                         (),
      .PREADY                         (1'b1),
      .PSLVERR                        (1'b0),
      .PRDATA                         (32'h0),

      .TRCENA                         (),
      .ATVALID                        (),
      .ATID                           (),
      .ATDATA                         (),
      .AFREADY                        (),     /* flush not supported by TPIU */
      .ATREADY                        (1'b1),
      .AFVALID                        (1'b0), /* flush not supported by TPIU */

      .ATREADYMI                      (1'b1),
      .AFVALIDMI                      (1'b0),
      .ATVALIDMI                      (),
      .AFREADYMI                      (),
      .ATDATAMI                       (),
      .ATIDMI                         (),

      .SYNCREQI                       (1'b0), /* not supported by TPIU */
      .SYNCREQD                       (1'b0), /* not supported by TPIU */

      .TRIGGER                        (),
      .DSYNC                          (),

      .TSVALUEB                       (TSVALUEB),
      .TSCLKCHANGE                    (1'b0),

      .HALTED                         (dbghalt),
      .DBGRESTARTED                   (),
      .DBGEN                          (cm7cfg_dev),
      .NIDEN                          (cm7cfg_dev),
      .EDBGRQ                         (1'b0),
      .DBGRESTART                     (1'b0),
      .IADDR                          (),  // this is not real pc
      .IADBGPROT                      (1'b0),

      .IRQ                            (cm7_irq|240'h0),
      .NMI                            (cm7_nmi),

      .SLEEPING                       (cm7_sleeping),
      .SLEEPDEEP                      (cm7_sleepdeep),
      .GATEHCLK                       (cm7_gatehclk),
      .SLEEPHOLDACKn                  (),
      .SLEEPHOLDREQn                  (1'b1),
      .WAKEUP                         (),
      .WICSENSE                       (),
      .WICENREQ                       (1'b0),
      .WICENACK                       (),
      .ETMPWRUPREQ                    (),

      .LOCKUP                         (), 
      .TXEV                           (), 
      .ICERR                          (),
      .DCERR                          (),
      .ICDET                          (),
      .DCDET                          (),
      .RXEV                           (cm7_rxev), 

      .FPIXC                          (),
      .FPIDC                          (),
      .FPOFC                          (),
      .FPUFC                          (),
      .FPDZC                          (),
      .FPIOC                          (),

      .CTICHIN                        ({4{1'b0}}),
      .CTICHOUT                       (),
      .CTIIRQ                         (),

      .MBISTACK                       (),
      .MBISTOUTDATA                   (),
      .MBISTIMPERR                    (),
      .nMBISTRESET                    (1'b1),
      .MBISTREQ                       (1'b0),
      .MBISTADDR                      ({21{1'b0}}),
      .MBISTINDATA                    ({78{1'b0}}),
      .MBISTWRITEEN                   (1'b0),
      .MBISTREADEN                    (1'b0),
      .MBISTARRAY                     ({5{1'b0}}),
      .MBISTBE                        ({10{1'b0}}),
      .MBISTCFG                       ({4{1'b0}}),

      .ECOREVNUM                      (36'h0),

      .DCCMINP                        (8'h00),
      .DCCMOUT                        (),
      .DCCMINP2                       (8'h00),
      .DCCMOUT2                       (),

      .DFTSE                          (cmsatpg),
      .DFTRSTDISABLE                  (cmsatpg),
      .DFTRAMHOLD                     (cmsatpg),
      .CTLPPBLOCK                     (4'h0)
    );

  `theregrn(TSVALUEB) <= ~coreresetn ? 0 : ( TSVALUEB + 1 );

    assign axim.arid[PM_AXIM_IDW-1:3] = '0;
    assign axim.awid[PM_AXIM_IDW-1:2] = '0;
    assign axim.aruser = '0;
    assign axim.awuser = '0;

    assign axim.awlen = '0 | cm7.AWLEN;
    assign axim.arlen = '0 | cm7.ARLEN;

    assign ahbp.hsel = 1'b1;
    assign ahbp.hreadym = 1'b1;
    assign ahbp.hmasterlock = 1'b0;
    assign ahbp.hauser = '0;
    assign ahbp.hwuser = '0;
    assign ahbs.hruser = '0;
//    assign ahbs.hwuser = '0;

  // --------------------------------------------------------------------------
  // Cortex-M7 core user
  // --------------------------------------------------------------------------

  assign ahbp0.hreadym = 1'b1;
  assign ahbp0.hmasterlock = 1'b0;
  assign ahbp0.hsel = 1'b1;
  assign ahbp0.hmaster[3:1] = 'h3;

    ahb_sync#(
            .SYNCDOWN (1),
            .SYNCUP   (0)
        ) ahbp_syncdown (
            .hclk       (clk        ),
            .resetn     (resetn     ),
            .hclken     (ahbpclken  ),
            .ahbslave   (ahbp0      ),
            .ahbmaster  (ahbp       )
        );

    ahb_sync#(
            .SYNCDOWN (0),
            .SYNCUP   (1)
        )ahbs_syncup (
            .hclk       (clk        ),
            .resetn     (resetn     ),
            .hclken     (ahbsclken  ),
            .ahbslave   (ahbs       ),
            .ahbmaster  (ahbs0      )
        );

  // --------------------------------------------------------------------------
  // Cortex-M7 core user
  // --------------------------------------------------------------------------

    bit [31:0] corecm7pc;

    assign corecm7pc = cm7.u_cortexm7.u_top_sys.u_core.u_cm7_dpu.u_dpu_prog_flow.pc_r_ex1[31:1] * 2;

    genvar gvi;
    generate
        for( gvi = 0; gvi < PM_COREUSERCNT; gvi++ ) begin: GENCOREUSER
            `theregrn( coreuser[gvi] ) <= ( corecm7pc >= coreusermap[gvi].start_addr) & ( corecm7pc < coreusermap[gvi].end_addr);
        end
    endgenerate

  // --------------------------------------------------------------------------
  // Cortex-M7 Debug Access Port
  // --------------------------------------------------------------------------

  //DAP Tie-offs
  wire [31:0] dap_baseaddr    = BASEADDR;
  wire [31:0] dap_targetid    = TARGETID;
  wire [ 7:0] dap_ecorevnum   = 0;//ECOREVNUM[43:36];

  //Synchronise the DAP resets

  wire apreset_n;
  wire dpreset_n;
  wire pre_mux_apreset_n;
  wire pre_mux_dpreset_n;

  cm7cell_sync
  u_cm7cell_sync_apreset
    (
      .clk_i                          (clkcm7in),
      .inp_i                          (nPORESET),
      .resetn_i                       (nPORESET),
      .out_o                          (pre_mux_apreset_n)
    );

  cm7cell_sync
  u_cm7cell_sync_dpreset
    (
      .clk_i                          (swclk),
      .inp_i                          (nPORESET),
      .resetn_i                       (nPORESET),
      .out_o                          (pre_mux_dpreset_n)
    );

  assign apreset_n = cmsatpg | pre_mux_apreset_n;
  assign dpreset_n = cmsatpg | pre_mux_dpreset_n;

  logic swdio_oen;

  CM7DAP
    #(
      .SWMD                           (SWMD),
      .RAR                            (RAR)
    )
  u_cm7dap
    (
      .SWCLKTCK                       (swclk),
      .DPRESETn                       (dpreset_n),
      .DCLK                           (clkcm7in),
      .APRESETn                       (apreset_n),

      .nTRST                          (1'b0),
      .TDI                            (1'b0),
      .TDO                            (),
      .nTDOEN                         (),

      .SWDITMS                        (swdio.pi),
      .SWDO                           (swdio.po),
      .SWDOEN                         (swdio_oen),
      .SWDETECT                       (), /*deliberately unconnected*/

      .HALTED                         (dbghalt),

      .CDBGPWRUPREQ                   (),
      .CDBGPWRUPACK                   (1'b0),

      .DEVICEEN                       (cm7cfg_dev),

      .SLVADDR                        (ahbd_haddr),
      .SLVWDATA                       (ahbd_hwdata),
      .SLVTRANS                       (ahbd_htrans),
      .SLVPROT                        (ahbd_hprot),
      .SLVWRITE                       (ahbd_hwrite),
      .SLVSIZE                        (ahbd_hsize),
      .SLVRDATA                       (ahbd_hrdata),
      .SLVREADY                       (ahbd_hready),
      .SLVRESP                        (ahbd_hresp),

      .CFGJTAGnSW                     (1'b0),
      .BASEADDR                       (dap_baseaddr),
      .TARGETID                       (dap_targetid),
      .INSTANCEID                     (4'h0),
      .ECOREVNUM                      (dap_ecorevnum),

      .DFTSE                          (cmsatpg)
    );

  //Cortex-M7 DAP does not generate bursts
  assign ahbd_hburst = 3'b000;
  assign swdio.oe = ~swdio_oen;

//-----------------------------------------------------------------------------
// TCMs
//-----------------------------------------------------------------------------

// ITCM

/*
// fake long jump
    bit         fakelongen;
    bit [63:0]  fakelonginstr;
    assign fakelonginstr = ###;cfg_initvtor
    `theregsn( fakelongen ) <= ~( axim.ar_valid & axim.ar_ready );
     assign sys_itcmrdata = fakelongen ? fakelonginstr : sys_itcmrdata0;
*/
     assign sys_itcmrdata = sys_itcmrdata0;

`ifdef SIM

//`define ARM_CM7IK_CFGITCMSZ 


// DAP ROM Table Base Address
//   32'hE00FD003 = System Level ROM Table (default)
//   32'hE00FE003 = PIL ROM Table
//   32'hE00FF003 = Cortex-M7 ROM Table
`define  ARM_CM7IK_BASEADDR  32'hE00FD003

// Endianness of processor in MCU and Debug Driver system - default value: 0 (LE)
`define  ARM_CM7IK_CFGBIGEND  1'b0

//
// Implementation level pin configuration defines
//

// ITCM size: 1MB (default)
`define ARM_CM7IK_CFGITCMSZ 4'b1011

// DTCM size: 1MB (default)
`define ARM_CM7IK_CFGDTCMSZ 4'b1011

// AHBP region size: 512MB (default)
`define ARM_CM7IK_CFGAHBPSZ 3'b100

// STCALIB value for 100Mhz SCLK: ((100MHz x 10ms)/3 - 1) = 333333 (with skew) = 0x1051615
`define ARM_CM7IK_CFGSTCALIB 26'h1051615

// TCM enable out of reset
`define ARM_CM7IK_INITTCMEN 2'b11

// TCM RMW enables out of reset
`define ARM_CM7IK_INITRMWEN 2'b00

// TCM Retry enables out of reset
`define ARM_CM7IK_INITRETRYEN 2'b00

// ITCM Wait state
`define ARM_CM7IK_ITCMWAIT 7'h0

// DTCM Wait state
`define ARM_CM7IK_DTCMWAIT 7'h0

// AHBP enable out of reset
`define ARM_CM7IK_INITAHBPEN 1'b1

// DAP - JTAG or SW
`define ARM_CM7IK_CFGJTAGnSW 1'b0

// Vector Table initialisation value
`define ARM_CM7IK_INITVTOR 25'h0000000

// DAP INSTANCEID
`define ARM_CM7IK_INSTANCEID 4'h0

//
// Testbench defines
//

// Primary clock cycle (in ns) 100 MHz
`define  ARM_CM7IK_CLK_PERIOD 10.0

// Input delay for netlist simulation
`define  ARM_INPUT_DELAY 1

// Power-On-Reset assertion factor - 3 cycles
`define  ARM_CM7IK_POR_CYCLES 3

// Simulation Runaway Time Out factor - 6 million cycles
`define  ARM_CM7IK_TIMEOUT_CYCLES 6000000

// Power-On-Reset assertion time
`define  ARM_CM7IK_POR_LENGTH  (`ARM_CM7IK_POR_CYCLES*`ARM_CM7IK_CLK_PERIOD)

// Timeout
`define  ARM_CM7IK_SIM_TIMEOUT (`ARM_CM7IK_TIMEOUT_CYCLES*`ARM_CM7IK_CLK_PERIOD)




  cm7_ik_tcm_ram 
   #(.DATA_BYTES (8),
     .MEMSIZE    (`ARM_CM7IK_CFGITCMSZ),
     .MEMNAME    ("ITCM"),
     .MEMFILE    ("tests/image.bin"),
     .INIT       (0),
     .RESET      (1),
     .WAIT_CYC   (`ARM_CM7IK_ITCMWAIT),
     .ERRADDR    (11'h3ff),
     .RTRYADDR   (11'h038)
     )
  u_itcm_ram
    (.clk        (clkcm7in),
     .addr_i     (sys_itcmaddr[`ARM_CM7IK_CFGITCMSZ+8:3]),
     .wd_i       (sys_itcmwdata[63:0]),
     .cs_i       (sys_itcmcs),
     .we_i       (sys_itcmbytewr[7:0]),
     .rd_o       (sys_itcmrdata0[63:0]),
     .wait_o     (sys_itwait),
     .err_o      (sys_iterr),
     .retry_o    (sys_itretry)
     );

// D0TCM

  cm7_ik_tcm_ram 
   #(.DATA_BYTES (4),
     .MEMSIZE    (`ARM_CM7IK_CFGDTCMSZ),
     .MEMNAME    ("D0TCM"),
     .RESET      (1),
     .WAIT_CYC   (`ARM_CM7IK_DTCMWAIT),
     .ERRADDR    (16'hffff),
     .RTRYADDR   (17'h1fffd)
     )
  u_d0tcm_ram
    (.clk        (clkcm7in),
     .addr_i     (sys_d0tcmaddr[`ARM_CM7IK_CFGDTCMSZ+8:3]),
     .wd_i       (sys_d0tcmwdata[31:0]),
     .cs_i       (sys_d0tcmcs),
     .we_i       (sys_d0tcmbytewr[3:0]),
     .rd_o       (sys_d0tcmrdata[31:0]),
     .wait_o     (sys_d0wait),
     .err_o      (sys_d0err),
     .retry_o    (sys_d0retry)
     );

// D1TCM

  cm7_ik_tcm_ram 
   #(.DATA_BYTES (4),
     .MEMSIZE    (`ARM_CM7IK_CFGDTCMSZ),
     .MEMNAME    ("D1TCM"), 
     .RESET      (1),
     .WAIT_CYC   (`ARM_CM7IK_DTCMWAIT),
     .ERRADDR    (16'hffff),
     .RTRYADDR   (17'h1ffff)
     )
  u_d1tcm_ram
    (.clk        (clkcm7in),
     .addr_i     (sys_d1tcmaddr[`ARM_CM7IK_CFGDTCMSZ+8:3]),
     .wd_i       (sys_d1tcmwdata[31:0]),
     .cs_i       (sys_d1tcmcs),
     .we_i       (sys_d1tcmbytewr[3:0]),
     .rd_o       (sys_d1tcmrdata[31:0]),
     .wait_o     (sys_d1wait),
     .err_o      (sys_d1err),
     .retry_o    (sys_d1retry)
     );
`endif

endmodule : cm7sys




module __dummy_tb_cm7sys_ ();

ioif swdio();
axiif axim();
ahbif ahbp(), ahbs();

cm7sys u1
(

// system ctrl
    .clk('0),            // Free running clock
    .resetn('0),
    .coreresetn('0),

    .cm7_resetreq(),
    .cm7_sleep(),
    .clkcm7sten_1M('0),

// cfg
    .cm7cfg_dev('0),
    .cm7cfg_iv('0),// = 32'h6000_0000;

// test mode
    .cmsatpg('0),
    .cmsbist('0),
//    mbist.master                mbistif,

// interrupt, nmi, events
    .cm7_irq('0),
    .cm7_nmi('0),
    .cm7_rxev('0),

// amba
    .aximclken('0),       // axi clk enable
    .ahbpclken('0),
    .ahbsclken('0),
    .axim(axim),
    .ahbp(ahbp),
    .ahbs(ahbs),

// coreuser 
    .coreusermap('0),
    .coreuser(),

// debug
    .swclk('0),
    .swdio(swdio)

);

axis_null u0(axim);
ahbs_null u2(ahbp);
ahbm_null u3(ahbs);


endmodule


