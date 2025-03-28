
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
    input wire              rramsleep,          // rram sleep enter and exit
    output wire             rrcint,

    //  security control
    //  ==============================

    input wire  [15:0]      trustkey,
    input wire  [7:0]       sceuser,
    input wire  [7:0]       coreuser_cm7,
    input wire  [7:0]       coreuser_vex,

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
    input cms_pkg::cmscode_e   cmscode,
 // input wire              cmsatpg,
 // input wire              scan_resetn,
 // input wire              scan_test,
 // input wire              scan_en,
 // input wire              test_resetn,
 // input wire              test_en,

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

    bit bist_enable, rramclk0, rramclk1;
    bit scan_test;
    bit rramclk_org, rramclk_tck0, rramclk_tck1;

    bit [3:0] brfsm;
    bit counten_brfsm;
    bit recallvld;
    bit trc_err;
    bit trc_info_lock_err;
    bit [145:0] trc_dout_s0;
    bit [145:0] trc_dout_s1;
    bit [3:0] fd0;
    bit [2:0] fd0half;
    bit [BRCW:0]  bridx_org;
    bit [8:0] bridx_acv;
    bit [BRC-1:0][BRDW-1:0] brdatreg;
    bit bridx_org_vld,bridx_acv_vld;
    bit acram_wrbusy, acram_wrbusy_reg, acram_wrdone;
    bit [3:0] brfsm_reg;
    bit brfsm_read,acram_wrdone_reg;
    bit trc_busy,trc_busy_sreg;
    bit trc_dout_ready,trc_dout_ready_sreg;
    bit trc_busy_sdone, trc_dout_ready_sdone;
    bit trc_busy_sdone_reg, trc_dout_ready_sdone_reg;

    `theregfull(clksys, sysresetn, trc_busy_sreg, '0) <= ( brfsm == 'h5 ) ? 'b0 : trc_busy;
    `theregfull(clksys, sysresetn, trc_dout_ready_sreg, '0) <= ( brfsm == 'h5 ) ? 'b0 : trc_dout_ready;
    `theregfull(clksys, sysresetn, trc_busy_sdone_reg, '0) <= trc_busy_sdone;
    `theregfull(clksys, sysresetn, trc_dout_ready_sdone_reg, '0) <= trc_dout_ready_sdone;

    assign trc_busy_sdone = !trc_busy & trc_busy_sreg;
    assign trc_dout_ready_sdone = trc_dout_ready & !trc_dout_ready_sreg;

    `theregfull(clksys, sysresetn, acram_wrdone_reg, '1) <= acram_wrdone;

    `theregfull(clksys, sysresetn, bridx_org, '0) <= bridx_org_vld & trc_dout_ready_sdone ? bridx_org + 1 : bridx_org;
    `theregfull(clksys, sysresetn, bridx_acv, '0) <= bridx_acv_vld & acram_wrdone ? bridx_acv + 1 : bridx_acv;

    `theregfull(clksys, sysresetn, brfsm, '0) <=
            ( brfsm == 0 ) & ( bridx_org == '0 ) & trc_busy_sdone ?  1 :                                // rram recall auto start
            ( brfsm == 1 ) & ( bridx_org == '0 ) & trc_dout_ready_sdone & !brready[0] ? 7 :             // wait
            ( brfsm == 7 ) & ( bridx_org == 'd1 ) & brready[0] ? 2 :                                    // cms pattern
            ( brfsm == 2 ) & ( bridx_org == 'd3 ) & trc_dout_ready_sdone & !brready[1] ? 7 :            // wait
            ( brfsm == 7 ) & ( bridx_org == 'd4 ) & brready[1] ? 3 :                                    // ip trimming
            ( brfsm == 3 ) & ( bridx_org == BRC-1 ) & trc_dout_ready_sdone & brready[2] ? 4 :           // system cfg
            ( brfsm == 4 ) & ( bridx_acv == 'd511 ) & acram_wrdone ? 5 :                            // acv
                                                                    brfsm;

    `theregfull(clksys, sysresetn, brdone, '0 ) <= brdone | ( brfsm == 'h5 );
    `theregfull(clksys, sysresetn, brfsm_reg, '0) <= brfsm;

    assign bridx_org_vld = (( brfsm == 1 ) & ( bridx_org == 'd0 )) |
                            (( brfsm == 2 ) & ( bridx_org <= 'd3 )) |
                                (( brfsm == 3 ) & ( bridx_org <= BRC-1 ));

    assign bridx_acv_vld = ( brfsm == 4 ) & ( bridx_acv <= 'd511 );

    assign brvld = trc_dout_ready_sdone_reg & (brfsm != 'h5);
    `theregfull(clksys, sysresetn, brdatreg[bridx_org], '0) <= trc_dout_ready_sdone ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : brdatreg[bridx_org];
    assign brdat = brdatreg[bridx];
    assign bridx = bridx_org-1;

    bit [3:0]   brfsm_cmd;
    bit         brfsm_info;
    bit [11:0]  brfsm_xadr;
    bit [4:0]   brfsm_yadr;
    bit [3:0]   brfsm_udin;
    bit         brfsm_desel,brfsm_acv_desel;


    assign brfsm_desel = (brfsm == 0) | (brfsm == 5);
    assign brfsm_acv_desel = (brfsm == 4);
    assign brfsm_read = (brfsm_reg == 0) & (brfsm == 1) |
                            (brfsm_reg == 7) & (brfsm == 2) |
                            (brfsm_reg == 7) & (brfsm == 3) |
                            (brfsm_reg == 3) & (brfsm == 4) |
                            (bridx_org > 0 ) & (brfsm != 7) & (brfsm[2] != 1) & trc_dout_ready_sdone_reg |
                            bridx_acv_vld & acram_wrdone_reg;

    assign brfsm_cmd = brfsm_read ? TRC_READ : TRC_IDLE;
    assign brfsm_udin = 'h0;                                                // din[3:0]=4'b0101, recall all CFG settings
    assign brfsm_info = brfsm_desel | brfsm_acv_desel ? 1'b0 : 1'b1;
    assign brfsm_xadr = brfsm_desel ? 'h0 :
                            brfsm_acv_desel ? {8'b11_1101_11,bridx_acv[8:5]} : 'h0;
    assign brfsm_yadr = brfsm_desel ? 'h0 :
                            brfsm_acv_desel ? bridx_acv[4:0] : {1'b0,bridx_org[3:0]};


  //  secure access sram instantiate
  //  ==============================
  //  bist read initial
  //  write rrc cr bit

    bit [31:0] haddr_reg;
    bit ahb_write_flag, ahb_read_flag;
    bit keysel, datasel;
    bit [BRDW-1:0] ahb_rd_buf;
    bit [BRDW-1:0] ahb_wr_buf;
    bit [31:0] hwaddr_reg;
    bit rram_load_run, rram_write_run;
    bit acram_cs;
    bit acram_wr;
    bit [10:0] acram_addr;
    bit [63:0] acram_rdata;
    bit [63:0] acram_wdata;
    bit [255:0] acram_wrbuf;
    bit [1:0] acram_idx;
    bit rramcfg_vld, acram_rdbusy;

    assign rramcfg_vld = (hwaddr_reg[31:14] == {16'h603D, 2'b11});

    `theregfull(clktop, sysresetn, acram_wrbuf, '0) <= trc_dout_ready_sdone & ( brfsm == 4 ) ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} :        //initial from bistread
                                                        rram_write_run & rramcfg_vld ? ahb_wr_buf : acram_wrbuf;                                //ahb write configuration rram-acram region
    `theregfull(clktop, sysresetn, acram_wrbusy, '0) <= ( acram_idx == 2'b11 ) ? 1'b0 :
                                                            trc_dout_ready_sdone & ( brfsm == 4 ) | (rram_write_run & rramcfg_vld) ? 1'b1 : acram_wrbusy;
    `theregfull(clktop, sysresetn, acram_rdbusy, '0) <= trc_dout_ready_sdone ? 1'b0 :
                                                            ahb_read_flag & (keysel | datasel) ? 1'b1 : acram_rdbusy;
    `theregfull(clktop, sysresetn, acram_wrbusy_reg, '0) <= acram_wrbusy;
    assign acram_wrdone = acram_wrbusy_reg & !acram_wrbusy;

    `theregfull(clktop, sysresetn, acram_idx, '0) <= acram_wrbusy ? acram_idx + 1 : acram_idx;

    always@(*)
    casex(acram_idx)
        2'h1: acram_wdata = acram_wrbuf[127 : 64];
        2'h2: acram_wdata = acram_wrbuf[191 : 128];
        2'h3: acram_wdata = acram_wrbuf[255 : 192];
        default: acram_wdata = acram_wrbuf[63:0];
    endcase

    assign acram_cs = (!acram_wrbusy) & (!acram_rdbusy);
    assign acram_wr = !acram_wrbusy;
    assign acram_addr = ( brfsm == 4 ) ? {bridx_acv,acram_idx} :
                            acram_wrbusy ? {haddr_reg[13:5],acram_idx} :
                            acram_rdbusy & keysel ? {1'b0, haddr_reg[15:6]} :
                            acram_rdbusy & datasel ? {2'b10, haddr_reg[15:7]} : 11'h0;

    acram2kx64 acram (
         .q(acram_rdata),
         .clk(clktop),
         .cen(acram_cs),
         .gwen(acram_wr),
         .a(acram_addr),
         .d(acram_wdata),
        `sram_sp_uhde_inst
         );


  //  ahb-rram register access
  //  ==============================

//  `ahbs_common
    assign ahbx.hready = 'b1;
    assign ahbx.hresp = 'h0;
    assign ahbx.hrdata = '0
                | sfr_rrccr.hrdata32
                | sfr_rrcfd.hrdata32
                | sfr_rrcsr.hrdata32
		        | sfr_rrcfr.hrdata32;

    bit cfg_access_error_athlck, data_access_error_athclk, key_access_error_athclk;
    bit [1:0] rrccr;
    bit [2:0] rrcfr;
    bit [3:0] rrcfd;
    bit [5:0] rrcsr;
    bit ip_user_nap_i;
    bit ip_user_pd_i;
    bit [3:0] ip_user_cmd_i;
    bit [63:0] trc_regif_dout_s0;
    bit [63:0] trc_regif_dout_s1;

    ahb_cr #(.A('h00), .DW(2))                 sfr_rrccr  (.cr(rrccr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_cr #(.A('h04), .DW(4), .IV('h7))      sfr_rrcfd  (.cr(rrcfd), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
//  ahb_ar #(.A('h08), .AR(16'h5200 ))          sfr_rrcar0  (.ar(rrcar_load_run), .resetn(coreresetn), .sfrlock(1'b0), .*);
//  ahb_ar #(.A('h0C), .AR(16'h9528 ))          sfr_rrcar1  (.ar(rrcar_write_run), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_sr #(.A('h08), .DW(6))                 sfr_rrcsr  (.sr(rrcsr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_fr #(.A('h0C), .DW(3))                 sfr_rrcfr  (.fr(rrcfr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);

    assign fd0 = rrcfd[3:0];
    assign fd0half = rrcfd[3:1];
    assign rrcsr[5:0] = {ip_user_cmd_i, trc_err, trc_busy};
    assign rrcfr = {cfg_access_error_athlck, data_access_error_athclk,key_access_error_athclk};

  //  axi-ahb-rram read/write buffer
  //  ==============================

    bit [7:0] axid_reg;
    bit [2:0] axprot_reg;
    bit secvld;
    bit [15:0] datacfg;
    bit [31:0] keycfg;
    bit cm7sel, vexsel, scesel;
    bit [7:0] coreuser_mux;
    bit [1:0] coreuser_in, userid_k, userid_d;
    bit [7:0] keytype_in, keytype_k, keytype_d;
    bit pri_op, sec_op, inst_op, data_op;

    bit core_rd_lock_k, core_wr_lock_k, sce_wr_lock_k, sce_rd_lock_k;
    bit [3:0] akeyid;
    bit core_rd_lock_d, core_wr_lock_d, sce_wr_lock_d, sce_rd_lock_d;
    bit key_access_error, data_access_error,cfg_access_error;
    bit keycfg_lock, datacfg_lock;


    ahbif #(.AW(32),.DW(64),.IDW(),.UW()) ahbarray();

    axi_ahb_bdg #(.AW(32), .DW(64)) u_rrcbdg (
        .clk            ( clk                   ),
        .resetn         ( coreresetn            ),
        .axislave       ( axis                  ),
        .ahbmaster      ( ahbarray              )
    );

    bit ahb_array_trans;
    bit ahb_array_read;
    bit ahb_array_write;
    bit haddr_match;
    bit rrcvld;

    bit [3:0]  hauser_reg;
    bit [2:0] rrcfsm;
//  bit rramclk;
    bit rramclken;
    bit [3:0] rramclkcnt;

    bit trc_busy_reg;
    bit trc_dout_ready_s1, trc_dout_ready_s0, trc_dout_ready_reg;
    bit trc_busy_done, trc_dout_ready_done;
    bit trc_dout_ready_done_reg, ahb_array_write_reg;

    `theregfull(clktop, sysresetn, trc_busy_reg, '1) <= clken ? trc_busy : trc_busy_reg; // faye
    `theregfull(clktop, sysresetn, trc_dout_ready_reg, '1) <= clken ? trc_dout_ready : trc_dout_ready_reg;   //faye

    assign trc_busy_done = !trc_busy & trc_busy_reg;
    assign trc_dout_ready_done = trc_dout_ready & !trc_dout_ready_reg;


    assign ahb_array_trans = clken & ahbarray.htrans[1] & brdone & ahbarray.hready;
    assign ahb_array_read  = ahb_array_trans & !ahbarray.hwrite & !haddr_match & ahbarray.hsel;
    assign ahb_array_write = ahb_array_trans & ahbarray.hwrite & ahbarray.hsel;

     `theregfull(clktop, coreresetn, haddr_reg, '0) <= ahb_array_trans & ahbarray.hsel ? ahbarray.haddr : haddr_reg;
     `theregfull(clktop, coreresetn, hwaddr_reg, '0) <= ahb_array_trans & ahbarray.hsel & ahbarray.hwrite ? ahbarray.haddr : hwaddr_reg;
     `theregfull(clktop, coreresetn, hauser_reg, '0) <= ahb_array_trans & !ahbarray.hwrite & ahbarray.hsel ? ahbarray.hauser : hauser_reg;
     `theregfull(clktop, coreresetn, ahb_write_flag, '0) <= ahb_array_trans ? ahbarray.hsel & ahbarray.hwrite : ahb_write_flag;
     `theregfull(clktop, coreresetn, ahb_read_flag, '0) <= ahb_array_trans ? ahbarray.hsel & !ahbarray.hwrite : ahb_read_flag;
     `theregfull(clktop, coreresetn, ahb_array_write_reg, '0) <= ahb_array_write;

    assign haddr_match = ( ahbarray.haddr[31:5] == haddr_reg[31:5] );

    always@(*)
    casex(haddr_reg[4:3])
        2'h1: ahbarray.hrdata = ahb_rd_buf[127 : 64];
        2'h2: ahbarray.hrdata = ahb_rd_buf[191 : 128];
        2'h3: ahbarray.hrdata = ahb_rd_buf[255 : 192];
        default: ahbarray.hrdata = ahb_rd_buf[63:0];
    endcase

    `theregfull(clktop, coreresetn, ahb_rd_buf, '0) <= trc_dout_ready_done ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : ahb_rd_buf;

    `theregfull(clktop, coreresetn, ahb_wr_buf[63:0],   '0) <= ahb_array_write_reg & !rrccr[1] & (hwaddr_reg[4:3]=='h0) ? ahbarray.hwdata : ahb_wr_buf[63:0];
    `theregfull(clktop, coreresetn, ahb_wr_buf[127:64], '0) <= ahb_array_write_reg & !rrccr[1] & (hwaddr_reg[4:3]=='h1) ? ahbarray.hwdata : ahb_wr_buf[127:64];
    `theregfull(clktop, coreresetn, ahb_wr_buf[191:128],'0) <= ahb_array_write_reg & !rrccr[1] & (hwaddr_reg[4:3]=='h2) ? ahbarray.hwdata : ahb_wr_buf[191:128];
    `theregfull(clktop, coreresetn, ahb_wr_buf[255:192],'0) <= ahb_array_write_reg & !rrccr[1] & (hwaddr_reg[4:3]=='h3) ? ahbarray.hwdata : ahb_wr_buf[255:192];

    assign rram_load_run = ahb_array_write_reg & rrccr[1] & (ahbarray.hwdata == 'h5200);
    assign rram_write_run = ahb_array_write_reg & rrccr[1] & (ahbarray.hwdata == 'h9528);

    assign ahbarray.hready = ( rrcfsm == 0 );
    assign ahbarray.hresp = 2'h0;
    assign ahbarray.hruser = hauser_reg;


  //  rram rd/write control
  //  ==============================

    bit trc_busyreg1;

    `theregfull(rramclk_org, coreresetn, trc_busyreg1, '0 ) <= trc_busy;
    `theregfull(clktop, sysresetn, trc_dout_ready_done_reg, '1) <= trc_dout_ready_done;

    `theregfull(clktop, coreresetn, rrcfsm, '0) <=
            ( rrcfsm == 0 ) & rramsleep ? 5 :                                   // rram enter sleep mode
            ( rrcfsm == 5 ) & !trc_busy & trc_busyreg1 ? 0 :                    // rram exit sleep mode, no clock for trc_busy_done sample
            ( rrcfsm == 0 ) & ahb_array_read ? 1 :                              // rram read operation
            ( rrcfsm == 1 ) & trc_dout_ready_done ? 0 :                         // rram read done
            ( rrcfsm == 0 ) & ahb_array_write  & !rrccr[1] ? 2 :                // rram wr_buf load
            ( rrcfsm == 2 ) & ahb_array_write_reg ? 0 :                         // rram wr_buf load done
            ( rrcfsm == 0 ) & rram_load_run ? 3 :                               // rram load start
            ( rrcfsm == 3 ) & rramclken ? 0 :                                   // rram load done  (**** liza clock domian)
            ( rrcfsm == 0 ) & rram_write_run ? 4 :                              // rram write start
            ( rrcfsm == 4 ) & trc_busy_done ? 0 :                               // rran write done
                                                        rrcfsm;

    bit [3:0]   axi_cmd;
    bit         axi_info;
    bit [11:0]  axi_xadr;
    bit [4:0]   axi_yadr;
    bit [255:0] axi_din;
    bit         trc_busy_delay;

    `theregfull(rramclk_org, coreresetn, trc_busy_delay, '0) <= trc_busy;

    always@(*)
    casex(rrcfsm)
        3'h1: axi_cmd = trc_busy | trc_busy_delay | trc_dout_ready ? TRC_IDLE : TRC_READ;
        3'h3: axi_cmd = TRC_LOAD;
        3'h4: axi_cmd = trc_busy ? TRC_IDLE : TRC_WRITE;
        default: axi_cmd = TRC_IDLE;
    endcase

    assign axi_din = ahb_wr_buf;
    assign axi_info = haddr_reg[22];            //0x6040_0000 mapping to INF0
    assign axi_xadr = haddr_reg[21:10];
    assign axi_yadr = haddr_reg[9:5];

    assign ip_user_nap_i = (rrcfsm == 5) & !rrccr[0] & rramsleep;  // =0, nap
    assign ip_user_pd_i = (rrcfsm == 5) & rrccr[0] & rramsleep;    // =1, power down


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

    `theregfull(clktop, coreresetn, axid_reg, '0) <= axis.arvalid & clken ? axis.arid :
                                                        axis.awvalid & clken ? axis.awid : axid_reg;
    `theregfull(clktop, coreresetn, axprot_reg, '0) <= axis.arvalid & clken ? axis.arprot :
                                                        axis.awvalid & clken ? axis.awprot : axprot_reg;

    assign cm7sel = ( ahbarray.hauser == AMBAID4_CM7A );
    assign vexsel = ( ahbarray.hauser == AMBAID4_VEXI ) | ( ahbarray.hauser == AMBAID4_VEXD );
    assign scesel = ( ahbarray.hauser == AMBAID4_SCEA ) | ( ahbarray.hauser == AMBAID4_SCES );

    assign coreuser_mux = scesel ? sceuser :
                            vexsel ? coreuser_vex : coreuser_cm7;
    assign coreuser_in = coreuser_mux[7] ? 2'b11 :
                            coreuser_mux[6] ? 2'b10 :
                            coreuser_mux[5] ? 2'b01 : 2'b00;

    assign keytype_in = axid_reg;
    assign pri_op = axprot_reg[0];
    assign sec_op = !axprot_reg[1];
    assign inst_op = axprot_reg[2];
    assign data_op = !axprot_reg[2];

    assign keysel = ( haddr_reg[31:16] == 16'h603F );
    assign datasel =  ( haddr_reg[31:16] == 16'h603E );

  //assign acram_addr_op = datasel ? { haddr_reg[15:6],2'b00 } :
  //                            keysel ? { haddr_reg[15:5],1'b0 } : 11'h0;
  //assign acram_cs_op = ahbarray.hsel & ahbarray.htrans[1];

    assign datacfg = (haddr_reg[6:5] == 2'b01) ? acram_rdata[31:16] :
                        (haddr_reg[6:5] == 2'b10) ? acram_rdata[47:32] :
                        (haddr_reg[6:5] == 2'b11) ? acram_rdata[63:48] : acram_rdata[15:0];
    assign keycfg = haddr_reg[5] ? acram_rdata[63:32] : acram_rdata[31:0];

    assign userid_k = keycfg[1:0];
    assign core_rd_lock_k = keycfg[3];
    assign core_wr_lock_k = keycfg[4];
    assign sce_rd_lock_k = keycfg[5];
    assign sce_wr_lock_k = keycfg[6];
    assign keycfg_lock = keycfg[7];
    assign keytype_k = keycfg[15:8];
    assign akeyid = keycfg[19:16];

    assign userid_d = datacfg[1:0];
    assign core_rd_lock_d = datacfg[3];
    assign core_wr_lock_d = datacfg[4];
    assign sce_rd_lock_d = datacfg[5];
    assign sce_wr_lock_d = datacfg[6];
    assign datacfg_lock = datacfg[7];
    assign keytype_d = datacfg[15:8];

    assign key_access_error = ((coreuser_in != userid_k) & (ahb_write_flag | ahb_read_flag) |
                                core_rd_lock_k & ahb_read_flag & (cm7sel|vexsel) & pri_op |
                                core_wr_lock_k & ahb_write_flag & (cm7sel|vexsel) & pri_op |
                                sce_rd_lock_k & ahb_read_flag & scesel & sec_op & (keytype_in != keytype_k) |
                                sce_wr_lock_k & ahb_write_flag & scesel & sec_op |
                                !trustkey[akeyid] ) & data_op & keysel;

    assign data_access_error = ((coreuser_in != userid_k) & (ahb_write_flag | ahb_read_flag) |
                                core_rd_lock_d & ahb_read_flag & (cm7sel|vexsel) |
                                core_wr_lock_d & ahb_write_flag & (cm7sel|vexsel) & pri_op |
                                sce_rd_lock_d & ahb_read_flag & scesel & sec_op & (keytype_in != keytype_d) |
                                sce_wr_lock_d & ahb_write_flag & scesel & sec_op ) & data_op & datasel;

    assign cfg_access_error = keycfg_lock & (hwaddr_reg[31:13] == {16'h603D,3'b110}) & ahb_write_flag |
                                datacfg_lock & (hwaddr_reg[31:12] == {16'h603D,4'b1110}) & ahb_write_flag;

    sync_pulse sync_key_error ( .clka(clktop),    .resetn(coreresetn), .clkb(hclk), .pulsea (key_access_error), .pulseb( key_access_error_athclk ) );
    sync_pulse sync_data_error ( .clka(clktop),    .resetn(coreresetn), .clkb(hclk), .pulsea (data_access_error), .pulseb( data_access_error_athclk ) );
    sync_pulse sync_cfg_error ( .clka(clktop),    .resetn(coreresetn), .clkb(hclk), .pulsea (cfg_access_error), .pulseb( cfg_access_error_athlck ) );

    assign secvld = 1'b1;

  //  rram clock generater
  //  ==============================

  bit rramclk0_unbuf, rramclk1_unbuf;
  bit rramsleepreg0, rramsleepreg1;

    assign bist_enable = (cmscode == CMS_VRGN) | (cmscode == CMS_TEST);
    assign scan_test = (cmscode == CMS_ATPG);

    `theregfull( rramclk_org, coreresetn, rramsleepreg0, '0 ) <= rramsleep;
    `theregfull( rramclk_org, coreresetn, rramsleepreg1, '0 ) <= rramsleepreg0;

    `theregfull( clktop, coreresetn, rramclkcnt, '0 ) <= ( trc_dout_ready_done | ( rrcfsm == 0 ) | rramsleepreg1 & rramsleep) ? 0 :
                                                         ( rramclkcnt == fd0 ) ? 0 : rramclkcnt + 1;

    assign rramclken = (( rrcfsm == 0 ) & (ahb_array_read | ahb_array_write)) |
                        (( rramclkcnt == fd0 ) && ( rrcfsm !== 0 )) |
                        (brfsm != 3'h5) & (cmscode == CMS_USER) |
                        (brfsm <= 3'h1) & (cmscode != CMS_USER);

    ICG rramicg_org ( .CK (clktop   ),      .EN ( rramclken ), .SE(scan_test), .CKG ( rramclk_org ));
    ICG rramicg_tck0 ( .CK (jtag[0].tck ),   .EN ( bist_enable ), .SE(scan_test), .CKG ( rramclk_tck0 ));
    ICG rramicg_tck1 ( .CK (jtag[1].tck ),   .EN ( bist_enable ), .SE(scan_test), .CKG ( rramclk_tck1 ));

    CLKCELL_BUF buf_rramclk0(.A(rramclk0_unbuf),.Z(rramclk0));
    CLKCELL_BUF buf_rramclk1(.A(rramclk1_unbuf),.Z(rramclk1));


    assign rramclk0_unbuf = rramclk_org | rramclk_tck0;
    assign rramclk1_unbuf = rramclk_org | rramclk_tck1;

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
        .bist_enable                        ( bist_enable               ),
        .bist_rst_n                         ( sysresetn                 ),
        .jtag_trst_n                        ( jtag[0].trst              ),
        .clk                                ( rramclk0                  ),
//      .inv_clk                            ( rramclk_inv               ),
        .bist_clk                           ( jtag[0].tck               ),
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
        .bist_enable                        ( bist_enable               ),
        .bist_rst_n                         ( sysresetn                 ),
        .jtag_trst_n                        ( jtag[1].trst              ),
        .clk                                ( rramclk1                  ),
//      .inv_clk                            ( rramclk_inv               ),
        .bist_clk                           ( jtag[0].tck               ),
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


