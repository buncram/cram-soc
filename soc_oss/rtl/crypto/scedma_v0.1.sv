module scedma #
(
    parameter AXID = 'h5,
    parameter AW = scedma_pkg::AW,
    parameter DW = scedma_pkg::DW,
    parameter FFCNT = scedma_pkg::FFCNT,
    parameter adr_t BA = 0,
    parameter SEGCNT  = scedma_pkg::SEGCNT,
    parameter segcfg_t [0:SEGCNT-1] SEGCFGS = scedma_pkg::SEGCFGS,
    parameter TRANSCNTW = scedma_pkg::TRANSCNTW,
    parameter TSW = TRANSCNTW
)(
    input logic clk,   
    input logic resetn,

    apbif.slavein apbs,
    apbif.slave   apbx,
    axiif.master  axim[1:0],

    output chnlreq_t       xchrpreq,
    input  chnlres_t       xchrpres,
    output chnlreq_t       xchwpreq,
    input  chnlres_t       xchwpres,

    output chnlreq_t       schrpreq,
    input  chnlres_t       schrpres,
    output chnlreq_t       schwpreq,
    input  chnlres_t       schwpres,

    output chnlreq_t       ichrpreq,
    input  chnlres_t       ichrpres,
    output chnlreq_t       ichwpreq,
    input  chnlres_t       ichwpres,

    input   bit [0:FFCNT-1] segfifoen,
    output bit[2:0]           sr_sdma,
    output bit[2:0]           fr_sdma,

    output bit [7:0]       intr,
    output bit [7:0]       err
);
    bit [15:0]  ichcr_segid;
    bit [7:0]   ichcr_rpsegid;
    bit [7:0]   ichcr_wpsegid;
    adr_t       ichcr_rpstart;
    adr_t       ichcr_wpstart;
    adr_t       ichcr_transize;
    bit [3:0]   ichcr_opt;
    bit [2:0]   ichcr_opt_ltx;
    bit         ichcr_opt_xor;

    bit                xchcr_func,          schcr_func;
    bit [7:0]          xchcr_opt,           schcr_opt;
    bit [31:0]         xchcr_axstart,       schcr_axstart;
    bit [7:0]          xchcr_segid,         schcr_segid;
    adr_t              xchcr_segstart,      schcr_segstart;
    bit[TRANSCNTW-1:0] xchcr_transize,      schcr_transize;

    logic ichstart, xchstart, schstart;
    logic ichbusy, xchbusy, schbusy;
    logic ichdone, xchdone, schdone;

    logic [7:0]  schcr_intr, xchcr_intr, ichcr_intr, schcr_err, xchcr_err;
    logic apbrd, apbwr, sfrlock;
    logic pclk;

// sfr
// ■■■■■■■■■■■■■■■ 

    assign pclk = clk;
    assign sfrlock = 0;

//    apb_sr #(.A('h18 ), .DW(1)    ) sfr_sdma_sr      (.sr( sr_sdma        ), .prdata32(),.*);
    apb_ar #(.A('h00 ), .AR('h5a) ) sfr_ichstart_ar  (.ar( ichstart       ), .*);
    apb_ar #(.A('h00 ), .AR('ha5) ) sfr_xchstart_ar  (.ar( xchstart       ), .*);
    apb_ar #(.A('h00 ), .AR('haa) ) sfr_schstart_ar  (.ar( schstart       ), .*);

    apb_cr #(.A('h10 ), .DW(1)    ) sfr_xch_func     (.cr( xchcr_func     ), .prdata32(),.*);
    apb_cr #(.A('h14 ), .DW(8)    ) sfr_xch_opt      (.cr( xchcr_opt      ), .prdata32(),.*);
    apb_cr #(.A('h18 ), .DW(32)   ) sfr_xch_axstart  (.cr( xchcr_axstart  ), .prdata32(),.*);
    apb_cr #(.A('h1c ), .DW(8)    ) sfr_xch_segid    (.cr( xchcr_segid    ), .prdata32(),.*);
    apb_cr #(.A('h20 ), .DW(AW)   ) sfr_xch_segstart (.cr( xchcr_segstart ), .prdata32(),.*);
    apb_cr #(.A('h24 ), .DW(TSW)  ) sfr_xch_transize (.cr( xchcr_transize ), .prdata32(),.*);

    apb_cr #(.A('h30 ), .DW(1)    ) sfr_sch_func     (.cr( schcr_func     ), .prdata32(),.*);
    apb_cr #(.A('h34 ), .DW(8)    ) sfr_sch_opt      (.cr( schcr_opt      ), .prdata32(),.*);
    apb_cr #(.A('h38 ), .DW(32)   ) sfr_sch_axstart  (.cr( schcr_axstart  ), .prdata32(),.*);
    apb_cr #(.A('h3c ), .DW(8)    ) sfr_sch_segid    (.cr( schcr_segid    ), .prdata32(),.*);
    apb_cr #(.A('h40 ), .DW(AW)   ) sfr_sch_segstart (.cr( schcr_segstart ), .prdata32(),.*);
    apb_cr #(.A('h44 ), .DW(TSW)  ) sfr_sch_transize (.cr( schcr_transize ), .prdata32(),.*);

    apb_cr #(.A('h50 ), .DW(4)    ) sfr_ich_opt      (.cr( ichcr_opt      ), .prdata32(),.*);
    apb_cr #(.A('h54 ), .DW(16)   ) sfr_ich_segid    (.cr( ichcr_segid    ), .prdata32(),.*);
    apb_cr #(.A('h58 ), .DW(AW)   ) sfr_ich_rpstart  (.cr( ichcr_rpstart  ), .prdata32(),.*);
    apb_cr #(.A('h5c ), .DW(AW)   ) sfr_ich_wpstart  (.cr( ichcr_wpstart  ), .prdata32(),.*);
    apb_cr #(.A('h60 ), .DW(AW)   ) sfr_ich_transize (.cr( ichcr_transize ), .prdata32(),.*);

    assign sr_sdma = { xchbusy, schbusy, ichbusy };
    assign fr_sdma = { xchdone, schdone, ichdone };
    assign { ichcr_rpsegid, ichcr_wpsegid } = ichcr_segid;
    assign { ichcr_opt_ltx, ichcr_opt_xor } = ichcr_opt;

    `apbs_common;
    assign apbx.prdata = '0
        | sfr_xch_func.prdata32 | sfr_xch_opt.prdata32 | sfr_xch_axstart.prdata32 | sfr_xch_segid.prdata32 | sfr_xch_segstart.prdata32 | sfr_xch_transize.prdata32
        | sfr_sch_func.prdata32 | sfr_sch_opt.prdata32 | sfr_sch_axstart.prdata32 | sfr_sch_segid.prdata32 | sfr_sch_segstart.prdata32 | sfr_sch_transize.prdata32
        | sfr_ich_opt.prdata32 | sfr_ich_segid.prdata32 | sfr_ich_rpstart.prdata32 | sfr_ich_wpstart.prdata32 | sfr_ich_transize.prdata32;

// axi mux
// ■■■■■■■■■■■■■■■ 
//
//    parameter XAW  = 32;
//    parameter XDW  = 32;
//    parameter XIDW  = 8;
//    parameter XUDW  = 8;
//    parameter XLENW = 3;
//
//     axiif #(
//        .AW     ( XAW    ),
//        .DW     ( XDW    ),
//        .LENW   ( XLENW  ),
//        .IDW    ( XIDW   ),
//        .UW     ( XUDW   )
//      ) xchaxi(), schaxi();
//
//     AXI_BUS #(
//        .AXI_ADDR_WIDTH     ( XAW    ),
//        .AXI_DATA_WIDTH     ( XDW    ),
//        .AXI_ID_WIDTH       ( XIDW   ),
//        .AXI_USER_WIDTH     ( XUDW   )
//      ) axim_pulp[1:0](), axis_pulp();
//
//    axi_mux_intf #(
//    /*  parameter int unsigned*/ .SLV_AXI_ID_WIDTH ( XIDW   ), // Synopsys DC requires default value for params
//    /*  parameter int unsigned*/ .MST_AXI_ID_WIDTH ( XIDW+1 ),
//    /*  parameter int unsigned*/ .AXI_ADDR_WIDTH   ( XAW    ),
//    /*  parameter int unsigned*/ .AXI_DATA_WIDTH   ( XDW    ),
//    /*  parameter int unsigned*/ .AXI_USER_WIDTH   ( XUDW   ),
//    /*  parameter int unsigned*/ .NO_SLV_PORTS     ( 2      ), // Number of slave ports
//    /*  parameter int unsigned*/ .MAX_W_TRANS      ( 1      ),
//    /*  parameter bit         */ .FALL_THROUGH     ( 1'b0  ),
//    /*  parameter bit         */ .SPILL_AW         ( 1'b0  ),
//    /*  parameter bit         */ .SPILL_W          ( 1'b0  ),
//    /*  parameter bit         */ .SPILL_B          ( 1'b0  ),
//    /*  parameter bit         */ .SPILL_AR         ( 1'b0  ),
//    /*  parameter bit         */ .SPILL_R          ( 1'b0  )
//    ) axi_mux (
//          .clk_i                  ( clk     ),
//          .rst_ni                 ( resetn  ),
//          .test_i                 ( 1'b0    ),
//          .slv                    ( axim_pulp ),
//          .mst                    ( axis_pulp )
//    );
//
//    axitrans_axi2pulp at1( .axis(xchaxi), .axim(axim_pulp[0]) );
//    axitrans_axi2pulp at2( .axis(schaxi), .axim(axim_pulp[1]) );
//    axitrans_pulp2axi at3( .axis(axis_pulp), .axim(axim ));
//
// axi chnl
    // ■■■■■■■■■■■■■■■ 

    scedmachnl_axim  #(
        .PM_AXID   (AXID      ),          
        .AW        (AW        ),     
        .DW        (DW        ),     
        .FFCNT     (FFCNT     ),        
        .BA        (BA        ),     
        .SEGCNT    (SEGCNT    ),         
        .SEGCFGS   (SEGCFGS   ),          
        .TRANSCNTW (TRANSCNTW )            
    )axim_sec(
    /*    input logic                */ .clk            (clk               ),
    /*    input logic                */ .resetn         (resetn            ),
    /*    axiif.master               */ .axim           (axim[1]           ),
    /*    output chnlreq_t           */ .rpreq          (schrpreq          ),
    /*    input  chnlres_t           */ .rpres          (schrpres          ),
    /*    output chnlreq_t           */ .wpreq          (schwpreq          ),
    /*    input  chnlres_t           */ .wpres          (schwpres          ),
    /*    input   bit                */ .start          (schstart          ),
    /*    output  bit                */ .busy           (schbusy           ),
    /*    output  bit                */ .done           (schdone           ),
    /*    input   bit [0:FFCNT-1]    */ .segfifoen      (segfifoen         ),
    /*    input   bit                */ .cr_func        (schcr_func        ),
    /*    input   bit [15:0]         */ .cr_opt         (schcr_opt         ),
    /*    input   bit [31:0]         */ .cr_axaddrstart (schcr_axstart     ),
    /*    input   adr_t              */ .cr_segid       (schcr_segid       ),
    /*    input   adr_t              */ .cr_segptrstart (schcr_segstart    ),
    /*    input   bit[TRANSCNTW-1:0] */ .cr_transize    (schcr_transize    ),
    /*    output bit [7:0]           */ .intr           (schcr_intr        ),
    /*    output bit [7:0]           */ .err            (schcr_err         )
    );

    scedmachnl_axim  #(
        .PM_AXID   (AXID+1    ),          
        .AW        (AW        ),     
        .DW        (DW        ),     
        .FFCNT     (FFCNT     ),        
        .BA        (BA        ),     
        .SEGCNT    (SEGCNT    ),         
        .SEGCFGS   (SEGCFGS   ),          
        .TRANSCNTW (TRANSCNTW )            
    )axim_gnl(
    /*    input logic                */ .clk            (clk               ),
    /*    input logic                */ .resetn         (resetn            ),
    /*    axiif.master               */ .axim           (axim[0]           ),
    /*    output chnlreq_t           */ .rpreq          (xchrpreq          ),
    /*    input  chnlres_t           */ .rpres          (xchrpres          ),
    /*    output chnlreq_t           */ .wpreq          (xchwpreq          ),
    /*    input  chnlres_t           */ .wpres          (xchwpres          ),
    /*    input   bit                */ .start          (xchstart          ),
    /*    output  bit                */ .busy           (xchbusy           ),
    /*    output  bit                */ .done           (xchdone           ),
    /*    input   bit [0:FFCNT-1]    */ .segfifoen      (segfifoen         ),
    /*    input   bit                */ .cr_func        (xchcr_func        ),
    /*    input   bit [15:0]         */ .cr_opt         (xchcr_opt         ),
    /*    input   bit [31:0]         */ .cr_axaddrstart (xchcr_axstart     ),
    /*    input   adr_t              */ .cr_segid       (xchcr_segid       ),
    /*    input   adr_t              */ .cr_segptrstart (xchcr_segstart    ),
    /*    input   bit[TRANSCNTW-1:0] */ .cr_transize    (xchcr_transize    ),
    /*    output bit [7:0]           */ .intr           (xchcr_intr        ),
    /*    output bit [7:0]           */ .err            (xchcr_err         )
    );

    logic [7:0] ich_wpffid;
    segcfg_t ichrpsegcfg, ichrpsegcfgreg;
    segcfg_t ichwpsegcfg, ichwpsegcfgreg;
    chnlcfg_t ichthecfg;

    assign ichrpsegcfg = SEGCFGS[ichcr_rpsegid];
    assign ichwpsegcfg = SEGCFGS[ichcr_wpsegid];
    `theregrn( ichrpsegcfgreg ) <= ichrpsegcfg;
    `theregrn( ichwpsegcfgreg ) <= ichwpsegcfg;
    assign ich_wpffid = ichwpsegcfgreg.fifoid;

    assign ichthecfg.chnlid = 'd1;
    assign ichthecfg.rpsegcfg = ichrpsegcfgreg;
    assign ichthecfg.wpsegcfg = ichwpsegcfgreg;
    assign ichthecfg.rpptr_start = ichcr_rpstart; 
    assign ichthecfg.wpptr_start = ichcr_wpstart;
    assign ichthecfg.wpffen = segfifoen[ich_wpffid];// & ichwpsegcfgreg.isfifo;
    assign ichthecfg.transsize = ichcr_transize;
    assign ichthecfg.opt_ltx = ichcr_opt_ltx  | '0;
    assign ichthecfg.opt_xor = ichcr_opt_xor;
    assign ichthecfg.opt_cmpp = '0;
    assign ichthecfg.opt_prm = '0;

    scedma_chnl  #(.TCW(TRANSCNTW),.DW(DW))chix(
        .clk,
        .resetn,
        .thecfg   (ichthecfg),
        .start    (ichstart),
        .busy     (ichbusy),
        .done     (ichdone),
        .rpreq    (ichrpreq ),
        .rpres    (ichrpres ),
        .wpreq    (ichwpreq ),
        .wpres    (ichwpres ),
        .intr     (ichcr_intr )
    );

endmodule

module dummytb_sce_dma ();
    parameter AXID = 'h5;
    parameter AW = scedma_pkg::AW;
    parameter DW = scedma_pkg::DW;
    parameter FFCNT = scedma_pkg::FFCNT;
    parameter adr_t BA = 0;
    parameter SEGCNT  = scedma_pkg::SEGCNT;
    parameter segcfg_t [0:SEGCNT-1] SEGCFGS = scedma_pkg::SEGCFGS;
    parameter TRANSCNTW = 16;

    logic clk;
    logic resetn;
    chnlreq_t       xchrpreq;
    chnlres_t       xchrpres;
    chnlreq_t       xchwpreq;
    chnlres_t       xchwpres;
    chnlreq_t       schrpreq;
    chnlres_t       schrpres;
    chnlreq_t       schwpreq;
    chnlres_t       schwpres;
    chnlreq_t       ichrpreq;
    chnlres_t       ichrpres;
    chnlreq_t       ichwpreq;
    chnlres_t       ichwpres;
    bit [0:FFCNT-1] segfifoen;
    bit[2:0]           sr_sdma;
    bit[2:0]           fr_sdma;
    bit [7:0]       intr;
    bit [7:0]       err;

    apbif #(.PAW(12),.DW(32)) apbs();
    apbif apbx();
    axiif axim[0:1]();

    scedma u0(.*);

endmodule
