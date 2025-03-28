
`include "template.sv"
import trc_pkg::*;
//import rrc_pkg::*;


module rrc #(
    parameter BRC  = 16,
    parameter BRCW = $clog2(BRC),
    parameter BRDW = 256
)(
    
    //  system
    //  ==============================

    input wire              clk,                // axi clk, 400MHz
    input wire              clktop,             // 800MHz
    input wire              clksys,             // 32MHz for draft sim
    input wire              clken,
    input wire              hclk,               // ahb clk 200MHz
    input wire              sysresetn,
    input wire              coreresetn,
    input wire              rramsleep,
    output wire             rrcint,

    //  security control
    //  ==============================

    input wire  [15:0]      trustkey,
    input wire  [1:0]       sceuser,
    input wire  [1:0]       coreuser_cm7,
    input wire  [1:0]       coreuser_vex,

    //  amba bus
    //  ==============================

    axiif.slave             axis,               // 64
    ahbif.slavein           ahbs,               // 32 **apb =32 reigsters.
    ahbif.slave             ahbx,               // 32 **apb =32 reigsters.

    //  bist read
    //  ==============================

    input  wire [3:0]       brready,
    output wire             brvld,
    output reg  [BRCW-1:0]  bridx,
    output wire [BRDW-1:0]  brdat,
    output reg              brdone,

    //  rram macro
    //  ==============================

    output rrc_pkg::rri_t [1:0] rri,
    input  rrc_pkg::rro_t [1:0] rro,        


    //  test control
    //  ==============================

    input wire              cmsatpg,
    input wire              scan_resetn,
    input wire              scan_test,
    input wire              scan_en,
    input wire              test_resetn,
    input wire              test_en,

    jtagif.slave            jtag[0:1]   

);


  //  interrupt source
  //  ==============================

    assign rrcint = 0;

  //  bist initial read
  //  ==============================

  // bist-stage
  // 0  trc_cmd = auto RECALL      
  // 1  trc_cmd = READ      virgin state, or pattern (1*256-bit pattern)     mode selection ready (virgin, test, user)
  // 1  trc_cmd = READ      for system analog   (3*256-bit)
  // 2  trc_cmd = READ      system control  (4*256-bit)
  // 3  trc_cmd = READ      access control ram  (128*256-bit)

    bit [3:0] brfsm;
    bit counten_brfsm;
    bit recallvld;

    bit trc_busy;
    bit trc_err;
    bit trc_info_lock_err;
    bit trc_dout_ready;
    bit [145:0] trc_dout_s0;
    bit [145:0] trc_dout_s1;
    bit [3:0] fd0;
    bit [2:0] fd0half;
    bit brvldreg;
    bit [BRCW:0]  bridx_org;
    bit [8:0] bridx_acv;
    bit [BRCW-1:0][BRDW-1:0] brdatreg;
    bit bridx_org_vld,bridx_acv_vld;
    bit acram_wrbusy, acram_wrbusy_reg, acram_wrdone;

    `theregfull(clksys, sysresetn, bridx_org, '0) <= bridx_org_vld & brvld ? bridx_org + 1 : bridx_org;
    `theregfull(clksys, sysresetn, bridx_acv, '0) <= bridx_acv_vld & acram_wrdone ? bridx_acv + 1 : bridx_acv;

    `theregfull(clksys, sysresetn, brfsm, '0) <=
            ( brfsm == 0 ) & ( bridx_org == '0 ) & !trc_busy ?  1 :                     // rram recall auto start
            ( brfsm == 1 ) & ( bridx_org == '0 ) & brvld & brready[0] ? 2 :             // cms pattern
            ( brfsm == 2 ) & ( bridx_org == 'd3 ) & brvld & brready[1] ? 3 :            // ip trimming
            ( brfsm == 3 ) & ( bridx_org == BRC-1 ) & brvld & brready[2] ? 4 :          // system cfg
            ( brfsm == 4 ) & ( bridx_acv == 'd511 ) & brvld ? 5 :                       // acv
                                                                    brfsm;                                          
                                                                

    `theregfull(clksys, sysresetn, brdone, '0 ) <= brdone | ( brfsm == 5 );

    assign bridx_org_vld = (( brfsm == 2 ) & ( bridx_org < 'd3 )) |
                                (( brfsm == 3 ) & ( bridx_org < BRC-1 ));

    assign bridx_acv_vld = ( brfsm == 4 ) & ( bridx_acv < 'd511 ); 

    assign brvld = trc_dout_ready;
    `theregfull(clksys, sysresetn, brvldreg, '0 ) <= brvld;
    `theregfull(clksys, sysresetn, brdatreg[bridx], '0) <= brvld ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : brdatreg[bridx];
    assign brdat = brdatreg[bridx];
    assign bridx = bridx_org;

    bit [3:0]   brfsm_cmd;
    bit         brfsm_info;
    bit [11:0]  brfsm_xadr;
    bit [4:0]   brfsm_yadr;
    bit [3:0]   brfsm_udin;
    bit         brfsm_desel,brfsm_acv_desel;

    assign brfsm_desel = (brfsm == 0) | (brfsm == 5);
    assign brfsm_acv_desel = (brfsm == 4);

    assign brfsm_cmd = brfsm_desel ? TRC_IDLE : TRC_READ;
    assign brfsm_udin = brfsm_desel ? 'h0 : 4'b0101;            // din[3:0]=4'b0101, recall all CFG settings
    assign brfsm_info = brfsm_desel | brfsm_acv_desel ? 1'b0 : 1'b1;
    assign brfsm_xadr = brfsm_desel ? 'h0 : 
                            brfsm_acv_desel ? {8'b11_1101_11,bridx_acv[8:5]} : 'h0;
    assign brfsm_yadr = brfsm_desel ? 'h0 : 
                            brfsm_acv_desel ? bridx_acv[4:0] : {1'b0,bridx[3:0]};


  //  secure access sram instantiate 
  //  ==============================
  //  bist read initial 
  //  write rrc cr bit

    bit acram_cs;
    bit acram_wr;
    bit [10:0] acram_addr;
    bit [63:0] acram_rdata;
    bit [63:0] acram_wdata;
    bit [255:0] acram_wrbuf;
    bit [1:0] acram_idx;

    `theregfull(clksys, sysresetn, acram_wrbuf, '0) <= brvld & ( brfsm == 4 ) ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : acram_wrbuf;
    `theregfull(clksys, sysresetn, acram_wrbusy, '0) <= ( acram_idx == 2'b11 ) ? 1'b0 :
                                                            brvld & ( brfsm == 4 ) ? 1'b1 : acram_wrbusy;
    `theregfull(clksys, sysresetn, acram_idx, '0) <= acram_wrbusy ? acram_idx + 1 : acram_idx;
    `theregfull(clksys, sysresetn, acram_wrbusy_reg, '0) <= acram_wrbusy;
    assign acram_wrdone = acram_wrbusy_reg & !acram_wrbusy;

    always@(*)
    casex(acram_idx)
        2'h1: acram_wdata = acram_wrbuf[127 : 64];
        2'h2: acram_wdata = acram_wrbuf[191 : 128];
        2'h3: acram_wdata = acram_wrbuf[255 : 192];
        default: acram_wdata = acram_wrbuf[63:0];
    endcase

    assign acram_cs = !acram_wrbusy;
    assign acram_wr = !acram_wrbusy;
    assign acram_addr = {bridx_acv,acram_idx};

    acram2kx64 acram (
         .q(acram_rdata), 
         .clk(clktop),
         .cen(acram_cs),
         .gwen(acram_wr),
         .a(acram_addr),
         .d(acram_wdata),
        `sram_sp_uhde_inst
         );


    bit secvld;
    assign secvld = 1'b1;


  //  ahb-rram register access
  //  ==============================

//  `ahbs_common
    assign ahbx.hready = 'b1;
    assign ahbx.hresp = 'h0;
    assign ahbx.hrdata = '0
                | sfr_rrccr.hrdata32
                | sfr_rrcfd.hrdata32
                | sfr_rrcsr.hrdata32 ;

    bit [15:0] rrccr;
    bit [15:0] rrcar;
    bit [15:0] rrcfd;
    bit [0:8][15:0] rrcsr;
    bit ip_user_nap_i;
    bit ip_user_pd_i;
    bit rrcar_load_run, rrcar_write_run;
    bit [3:0] ip_user_cmd_i;
    bit [63:0] trc_regif_dout_s0;
    bit [63:0] trc_regif_dout_s1;

// faye i change the address
    ahb_cr #(.A('h00), .DW(16))                 sfr_rrccr  (.cr(rrccr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_cr #(.A('h04), .DW(16), .IV('h0007))    sfr_rrcfd  (.cr(rrcfd), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_cr #(.A('h08), .DW(16))                 sfr_rrcar  (.cr(rrcar), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_sr #(.A('h0C), .DW(16), .SFRCNT(9))     sfr_rrcsr  (.sr(rrcsr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);

    assign ip_user_nap_i = '0; //rrccr[0];
    assign ip_user_pd_i = '0; //rrccr[1];

    assign rrcar_load_run = (rrcar == 16'h5200 );
    assign rrcar_write_run = (rrcar == 16'h9528 );

    assign fd0 = rrcfd[3:0];
    assign fd0half = rrcfd[3:1];

    assign rrcsr[0][5:0] = {ip_user_cmd_i, trc_err, trc_busy};
    assign rrcsr[1] = trc_regif_dout_s0[15:0];
    assign rrcsr[2] = trc_regif_dout_s0[31:16];
    assign rrcsr[3] = trc_regif_dout_s0[47:32];
    assign rrcsr[4] = trc_regif_dout_s0[63:48];
    assign rrcsr[5] = trc_regif_dout_s1[15:0];
    assign rrcsr[6] = trc_regif_dout_s1[31:16];
    assign rrcsr[7] = trc_regif_dout_s1[47:32];
    assign rrcsr[8] = trc_regif_dout_s1[63:48];

  //  axi-ahb-rram read/write buffer
  //  ==============================

    ahbif #(.AW(32),.DW(64),.IDW(),.UW()) ahbarray();

    axi_ahb_bdg #(.AW(32), .DW(64)) u_rrcbdg (
        .clk            ( clk                   ),
        .resetn         ( coreresetn            ),
        .axislave       ( axis                  ),
        .ahbmaster      ( ahbarray              )
    );

    bit rramclk;
    bit ahb_array_trans;
    bit ahb_array_read;
    bit ahb_array_write;
    bit haddr_match;
    bit [3:0] rrcfsm;
    bit rrcvld;
    bit [31:0] haddr_reg;
    bit [31:0] hwdata_reg;
    bit [3:0]  hauser_reg;

    bit axi_rramclk;
    bit rramclken;
    bit [3:0] rramclkcnt;
    bit ahb_write_flag;

    assign ahb_array_trans = clken & ahbarray.hsel & ahbarray.htrans[1] & secvld & brdone & ahbarray.hready;
    assign ahb_array_read  = ahb_array_trans & !ahbarray.hwrite & !haddr_match;
    assign ahb_array_write = ahb_array_trans & ahbarray.hwrite;
     
     `theregfull(clk, coreresetn, haddr_reg, '0) <= ahb_array_trans ? ahbarray.haddr : haddr_reg;
     `theregfull(clk, coreresetn, hwdata_reg, '0) <= ahb_array_trans ? ahbarray.hwdata : hwdata_reg; 
     `theregfull(clk, coreresetn, hauser_reg, '0) <= ahb_array_trans & !ahbarray.hwrite ? ahbarray.hauser : hauser_reg;         
     `theregfull(clk, coreresetn, ahb_write_flag, '0) <= ahb_array_trans ? ahbarray.hwrite : ahb_write_flag;

    assign haddr_match = ( ahbarray.haddr[31:5] == haddr_reg[31:5] );

    bit [BRDW-1:0] ahb_rd_buf;
    bit [BRDW-1:0] ahb_wr_buf;

    bit trc_busy_reg;
    bit trc_dout_ready_s1, trc_dout_ready_s0, trc_dout_ready_reg;
    bit trc_busy_done, trc_dout_ready_done;
    bit trc_dout_ready_done_reg;
    bit [255:0] ahb_wr_data;

    `theregfull(clktop, sysresetn, trc_busy_reg, '1) <= clken ? trc_busy : trc_busy_reg; // faye
    `theregfull(clktop, sysresetn, trc_dout_ready_reg, '1) <= clken ? trc_dout_ready : trc_dout_ready_reg;   //faye

    assign trc_busy_done = !trc_busy & trc_busy_reg;
    assign trc_dout_ready_done = trc_dout_ready & !trc_dout_ready_reg;

    always@(*)
    casex(haddr_reg[4:3])
        2'h1: ahbarray.hrdata = ahb_rd_buf[127 : 64];
        2'h2: ahbarray.hrdata = ahb_rd_buf[191 : 128];
        2'h3: ahbarray.hrdata = ahb_rd_buf[255 : 192];
        default: ahbarray.hrdata = ahb_rd_buf[63:0];
    endcase

    `theregfull(clktop, coreresetn, ahb_rd_buf, '0) <= trc_dout_ready_done ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : ahb_rd_buf;

    `theregfull(clktop, coreresetn, ahb_wr_buf, '0) <= trc_dout_ready_done & ahb_write_flag ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : 
                                                            trc_dout_ready_done_reg ? ahb_wr_data : ahb_wr_buf;

    assign ahb_wr_data[31:0]    = (haddr_reg[4:2]==0) ? hwdata_reg: ahb_wr_buf[31:0];
    assign ahb_wr_data[63:32]   = (haddr_reg[4:2]==1) ? hwdata_reg : ahb_wr_buf[63:32];
    assign ahb_wr_data[95:64]   = (haddr_reg[4:2]==2) ? hwdata_reg : ahb_wr_buf[95:64];
    assign ahb_wr_data[127:96]  = (haddr_reg[4:2]==3) ? hwdata_reg : ahb_wr_buf[127:96];
    assign ahb_wr_data[159:128] = (haddr_reg[4:2]==4) ? hwdata_reg : ahb_wr_buf[159:128];
    assign ahb_wr_data[191:160] = (haddr_reg[4:2]==5) ? hwdata_reg : ahb_wr_buf[191:160];
    assign ahb_wr_data[223:192] = (haddr_reg[4:2]==6) ? hwdata_reg : ahb_wr_buf[223:192];
    assign ahb_wr_data[255:224] = (haddr_reg[4:2]==7) ? hwdata_reg : ahb_wr_buf[255:224];

    assign ahbarray.hready = ( rrcfsm == 0 );
    assign ahbarray.hresp = 2'h0;
    assign ahbarray.hruser = hauser_reg;


  //  secure memory access control
  //  ==============================

  //  Access comparison Logic
  //    slot type   master      mode(AxPROT)        slot owner  lock    access
  //    keyslot     x           xN                  O_OWNER     x       allow
  //                SCE.R/W     exclusive           =sceuser    0       allow
  //                SCE.R/W     secure              =sceuser    1       allow
  //                M7.R/W      privilege           =coreuser   0       allow
  //                M7          x                   x           1       deny
  //    dataslot    x           x                   NO_OWNER    x       allow
  //                M7.R        x                   =coreuser   x       allow
  //                M7.W        privilege           =coreuser   1       allow
  //                SCE.R       exclusive/secure    =sceuser    0       allow
  //                SCE.W       exclusive/secure    =sceuser    1       allow

/*
    assign cm7sel = ( ahbarray.hauser == AMBAID4_CM7A );
    assign vexsel = ( ahbarray.hauser == AMBAID4_VEXI ) |  ahbarray.hauser == AMBAID4_VEXD );
    assign scesel = ( ahbarray.hauser == AMBAID4_SCEA ) |  ahbarray.hauser == AMBAID4_SCES );
    assign coreuser = scesel ? sceuser :
                        vexsel ? coreuser_vex : coreuser_cm7;
    assign keytype_vld = ( segid == )




    
keycfg(32-bit)  reserved
    akey_id[3:0]
    master_user[1:0]
    keytype[7:0]
    slot_owner_op[3:0]
    lock[4:0]
    
    
datacfg(16-bit) reserved
    master_user[1:0]
    sce_user[3:0]
    slot_owner_op[3:0]
    lock[3:0]

*/



  //  rram rd/write control
  //  ==============================

    `theregfull(clktop, sysresetn, trc_dout_ready_done_reg, '1) <= trc_dout_ready_done;

    `theregfull(clktop, coreresetn, rrcfsm, '0) <=
            ( rrcfsm == 0 ) & (ahb_array_read | ahb_array_write) ? 1 :          // rram read operation, wr_buf load.
            ( rrcfsm == 1 ) & trc_dout_ready_done ? 0 :                         // rram read done, wr_buf load done.
            ( rrcfsm == 0 ) & rrcar_load_run ? 2 :                              // rram load start
            ( rrcfsm == 2 ) & rramclken ? 0 :                                   // rram load done  (**** liza clock domian)
            ( rrcfsm == 0 ) & rrcar_write_run ? 3 :                             // rram write start
            ( rrcfsm == 3 ) & trc_busy_done ? 0 :                               // rran write done
                                                        rrcfsm;

    bit [3:0]   axi_cmd;
    bit         axi_info;
    bit [11:0]  axi_xadr;
    bit [4:0]   axi_yadr;
    bit [255:0] axi_din;
    bit         trc_busy_delay;

    `theregfull(rramclk, coreresetn, trc_busy_delay, '0) <= trc_busy;

    always@(*)
    casex(rrcfsm)
        4'h1: axi_cmd = trc_busy | trc_busy_delay | trc_dout_ready ? TRC_IDLE : TRC_READ;
        4'h2: axi_cmd = TRC_LOAD;
        4'h3: axi_cmd = trc_busy ? TRC_IDLE : TRC_WRITE;
        default: axi_cmd = TRC_IDLE;
    endcase

    assign axi_din = ahb_wr_buf;
    assign axi_info = haddr_reg[22];            //0x6040_0000 mapping to INF0
    assign axi_xadr = haddr_reg[21:10];
    assign axi_yadr = haddr_reg[9:5];


  //  rram clock generater 
  //  ==============================

    `theregfull( clktop, coreresetn, rramclkcnt, '0 ) <= (trc_dout_ready_done | ( rrcfsm == 0 )) ? 0 :
                                                         ( rramclkcnt == fd0 ) ? 0 : rramclkcnt + 1;

    assign rramclken = (( rrcfsm == 0 ) && (ahb_array_read | ahb_array_write)) | (( rramclkcnt == fd0 ) && ( rrcfsm !== 0 ));

    ICG rramicg ( .CK (clktop   ), .EN ( rramclken ), .SE(cmsatpg), .CKG ( axi_rramclk ));
    assign rramclk = axi_rramclk | (clksys & !coreresetn);


  //  rram intf handler
  //  ==============================

    bit [255:0] ip_user_udin_i;
    bit [35:0] ip_user_tdin_i;

    bit trc_busy_s0;
    bit trc_busy_s1;
    bit trc_err_s0;
    bit trc_err_s1;
    bit trc_info_lock_err_s0;
    bit trc_info_lock_err_s1;

    bit ip_user_info_i;
    bit [11:0] ip_user_xadr_i;
    bit [4:0] ip_user_yadr_i;
    bit ip_user_write_abort_i;
    bit [63:0] ip_user_trc_data_i;

    bit ip_user_ifren1_i;
    bit ip_user_reden_i;
    bit trc_write_suspend_i;
    bit trc_write_resume_i;
    bit trc_write_abort_i;

    assign trc_busy = trc_busy_s0 | trc_busy_s1;
    assign trc_dout_ready = trc_dout_ready_s0 & trc_dout_ready_s1;
    assign trc_err = trc_err_s0 | trc_err_s1;
    assign trc_info_lock_err = trc_info_lock_err_s0 | trc_info_lock_err_s1;

    assign ip_user_cmd_i         = axi_cmd | brfsm_cmd ;
    assign ip_user_info_i        = axi_info | brfsm_info ;
    assign ip_user_xadr_i        = axi_xadr | brfsm_xadr ;
    assign ip_user_yadr_i        = axi_yadr | brfsm_yadr ;
    assign ip_user_udin_i        = axi_din | {252'h0,brfsm_udin} ;
    assign ip_user_tdin_i        = 'h0;
    assign ip_user_trc_data_i    = 'h0;
    assign ip_user_write_abort_i = 'h0;
    assign ip_user_ifren1_i      = 'h0;
    assign ip_user_reden_i       = 'h0;
    assign trc_write_suspend_i   = 'h0;
    assign trc_write_resume_i    = 'h0;
    assign trc_write_abort_i     = 'h0;

  //  rram trbcx ip_s1 instantiate
  //  ==============================

    trbcx1r32_daric_wrapper u_trbcx_s0(

        //test mode
        .bist_enable                        ( test_en                   ),
        .bist_rst_n                         ( test_resetn               ),
        .jtag_trst_n                        ( ~jtag[0].trst             ),
        .clk                                ( rramclk                   ),
//      .inv_clk                            ( rramclk_inv               ),
        .bist_clk                           ( '0                        ),
        .tck                                ( jtag[0].tck               ),
        .inv_tck                            ( ~jtag[0].tck              ),
        .tms                                ( jtag[0].tms               ),
        .tdi                                ( jtag[0].tdi               ),
        .tdo                                ( jtag[0].tdo               ),
    
        .scan_test                          ( scan_test                 ),
//      .scan_en                            ( scan_en                   ),
//      .scan_rst_n                         ( scan_resetn               ),
        
        //user mode     
        .rst_n                              ( sysresetn                 ),
        .ip_user_cmd_i                      ( ip_user_cmd_i             ),
        .ip_user_info_i                     ( ip_user_info_i            ),
        .ip_user_ifren1_i                   ( ip_user_ifren1_i          ),
        .ip_user_reden_i                    ( ip_user_reden_i           ),

        .ip_user_xadr_i                     ( ip_user_xadr_i            ),
        .ip_user_yadr_i                     ( ip_user_yadr_i            ),
        .ip_user_udin_i                     ( ip_user_udin_i[127:0]     ),
        .ip_user_tdin_i                     ( ip_user_tdin_i[17:0]      ),
        .ip_user_nap_i                      ( ip_user_nap_i             ),
        .ip_user_pd_i                       ( ip_user_pd_i              ),
        .ip_user_trc_data_i                 ( ip_user_trc_data_i        ),
//      .ip_user_write_abort_i              ( ip_user_write_abort_i     ),
        .trc_dout_o                         ( trc_dout_s0               ),
        .trc_dout_ready_o                   ( trc_dout_ready_s0         ),
        .ecc_err_o                          (                           ),
        .trc_regif_dout_o                   ( trc_regif_dout_s0         ),
        .trc_busy_o                         ( trc_busy_s0               ),
        .trc_err_o                          ( trc_err_s0                ),
        `ifdef OPT_IFR1_LOCK
        .trc_info_lock_err_o                (                           ),
        `endif

     `ifdef OPT_ASYNC_READ
        .async_access_i                     (                           ),     
        .async_ifren_i                      (                           ),      
        .async_ifren1_i                     (                           ),      
        .async_reden_i                      (                           ),      
        .async_read_i                       (                           ),      
        .async_pch_ext_i                    (                           ),      
        .async_xadr_i                       (                           ),                    
        .async_yadr_i                       (                           ),                     
        .async_rram_rdone_o                 (                           ),              
        `endif   

        .trc_write_suspend_i                (trc_write_suspend_i        ),                                                             
        .trc_write_resume_i                 (trc_write_resume_i         ),                                          
        .trc_write_abort_i                  (trc_write_abort_i          ),                                          
        .trc_write_suspend_o                (                           ),                                              
        .trc_set_failure_status             (                           ),                                              
        .trc_reset_failure_status           (                           ),                                                  
        .trc_fourth_read_failure_status     (                           ),                                                      
        .sw_r_cfg_status                    (                           ),               

        .rri                                (rri[0]                     ),
        .rro                                (rro[0]                     )

        );


  //  rram trbcx ip_s2 instantiate
  //  ==============================

   trbcx1r32_daric_wrapper u_trbcx_s1(

        //test mode
        .bist_enable                        ( test_en                   ),
        .bist_rst_n                         ( test_resetn               ),
        .jtag_trst_n                        ( ~jtag[1].trst             ),
        .clk                                ( rramclk                   ),
//      .inv_clk                            ( rramclk_inv               ),
        .bist_clk                           ( '0                        ),
        .tck                                ( jtag[1].tck               ),
        .inv_tck                            ( ~jtag[1].tck              ),
        .tms                                ( jtag[1].tms               ),
        .tdi                                ( jtag[1].tdi               ),
        .tdo                                ( jtag[1].tdo               ),
    
        .scan_test                          ( scan_test                 ),
//      .scan_en                            ( scan_en                   ),
//      .scan_rst_n                         ( scan_resetn               ),
        
        //user mode     
        .rst_n                              ( sysresetn                 ),
        .ip_user_cmd_i                      ( ip_user_cmd_i             ),
        .ip_user_info_i                     ( ip_user_info_i            ),
        .ip_user_ifren1_i                   ( ip_user_ifren1_i          ),
        .ip_user_reden_i                    ( ip_user_reden_i           ),

        .ip_user_xadr_i                     ( ip_user_xadr_i            ),
        .ip_user_yadr_i                     ( ip_user_yadr_i            ),
        .ip_user_udin_i                     ( ip_user_udin_i[255:128]   ),
        .ip_user_tdin_i                     ( ip_user_tdin_i[17:0]      ),
        .ip_user_nap_i                      ( ip_user_nap_i             ),
        .ip_user_pd_i                       ( ip_user_pd_i              ),
        .ip_user_trc_data_i                 ( ip_user_trc_data_i        ),
//      .ip_user_write_abort_i              ( ip_user_write_abort_i     ),
        .trc_dout_o                         ( trc_dout_s1               ),
        .trc_dout_ready_o                   ( trc_dout_ready_s1         ),
        .ecc_err_o                          (                           ),
        .trc_regif_dout_o                   ( trc_regif_dout_s1         ),
        .trc_busy_o                         ( trc_busy_s1               ),
        .trc_err_o                          ( trc_err_s1                ),
        `ifdef OPT_IFR1_LOCK
        .trc_info_lock_err_o                (                           ),
        `endif

     `ifdef OPT_ASYNC_READ
        .async_access_i                     (                           ),     
        .async_ifren_i                      (                           ),      
        .async_ifren1_i                     (                           ),      
        .async_reden_i                      (                           ),      
        .async_read_i                       (                           ),      
        .async_pch_ext_i                    (                           ),      
        .async_xadr_i                       (                           ),                    
        .async_yadr_i                       (                           ),                     
        .async_rram_rdone_o                 (                           ),              
        `endif   

        .trc_write_suspend_i                (trc_write_suspend_i        ),                                                             
        .trc_write_resume_i                 (trc_write_resume_i         ),                                          
        .trc_write_abort_i                  (trc_write_abort_i          ),                                          
        .trc_write_suspend_o                (                           ),                                              
        .trc_set_failure_status             (                           ),                                              
        .trc_reset_failure_status           (                           ),                                                  
        .trc_fourth_read_failure_status     (                           ),                                                      
        .sw_r_cfg_status                    (                           ),               


        .rri                                (rri[1]                     ),
        .rro                                (rro[1]                     )

        );


endmodule : rrc

/*
module dummytb_rramctrl();

    ahbif #(.AW(16))ahb1();
    ramif #(.RAW(14))srambus();
    bit hclk,resetn,ramclken;

    ahbm_null u1(ahb1);
    rram_ctrl u2(.hclk(hclk),.resetn(resetn),.ahbslave(ahb1),.rammaster(srambus),.ramclken(ramclken));
    sram u3(.clk(hclk),.ramport(srambus));

    assign srambus.ramrdata = 0;
    
    

endmodule

*/

  //  scan/bist test control (optional)
  //  ==============================


//  //  bist initial read
//  //  ==============================
//
//  // bist-stage
//  // 1    trc_cmd = READ      virgin state, or pattern (1*256-bit pattern)
//  //     mode selection ready (virgin, test, user)
//  // 2    trc_cmd = READ      for system analog   (3*256-bit)
//  // 3    trc_cmd = RECALL    for rram IP itself  (recall DIN[3:0]=4'b0101,  [3]= 1, then shour recall of 6us)
//  // 4    trc_cmd = READ      system control  (4*256-bit)
//  // 5    trc_cmd = READ      access control ram  (128*256-bit)
//
//    bit [2:0] bist_state;
//    bit [7:0] bist_done;
//    bit [3:0] bist_cmd;
//
//    `theregfull( rramclk, resetn, bist_state, 3'h0 ) <= bist_stage_start ? (bist_state + 3'h1) : bist_state;
//    `theregfull( rramclk, resetn, bist_index, 8'h0 ) <= (|bist_state) & !bist_rdy ? (bist_index + 8'h1) : bist_index;
//    `theregfull( rramclk, resetn, bist_rdy, 1'b0 ) <= bist_stage_start ? 1'b0 : (bist_index == bist_done) ? 1'b1 : bist_rdy;
//
//    always@(*)
//    casex(bist_state)
//        3'h1: bist_done = 8'h01;
//        3'h2: bist_done = 8'h04;
//        3'h3: bist_done = 8'h04;
//        3'h4: bist_done = 8'h08;
//        3'h5: bist_done = 8'h88;
//        default: bist_done = 8'h00;
//    endcase
//
//    always@(*)
//    casex(bist_state)
//        3'h1: bist_cmd = TRC_READ;
//        3'h2: bist_cmd = TRC_READ;
//        3'h3: bist_cmd = TRC_RECALL;
//        3'h4: bist_cmd = TRC_READ;
//        3'h5: bist_cmd = TRC_READ;
//        default: bist_cmd = TRC_IDLE;
//    endcase


