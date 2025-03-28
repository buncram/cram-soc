
`include "template.sv"
import trc_pkg::*;
//import rrc_pkg::*;


module rrc #(
    parameter BRC  = 128,
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
    axiif.slave             axis,               // 64
    ahbif.slavein           ahbs,               // 32 **apb =32 reigsters.
    ahbif.slave             ahbx,               // 32 **apb =32 reigsters.
    input  wire [3:0]       brready,
    output wire             brvld,
    output reg  [BRCW-1:0]  bridx,
    output wire [BRDW-1:0]  brdat,
    output reg              brdone,
    output wire             rrcint,


    //  rram macro
    //  ==============================

    output rrc_pkg::rri_t [1:0] rri,
    input  rrc_pkg::rro_t [1:0] rro,        


    //  trbx control
    //  ==============================

    input wire              cmsatpg,
    input wire              scan_resetn,
    input wire              scan_test,
    input wire              scan_en,

    input wire              test_resetn,
    input wire              test_en,

    jtagif.slave            jtag[0:1]   

);


  //  ahb slave
  //  ==============================

    assign rrcint = 0;

  //  bist initial read
  //  ==============================

  // bist-stage
  // 0  trc_cmd = READ      virgin state, or pattern (1*256-bit pattern)     mode selection ready (virgin, test, user)
  // 1  trc_cmd = READ      for system analog   (3*256-bit)
  // 7  trc_cmd = RECALL    for rram IP itself  (recall DIN[3:0]=4'b0101,  [3]= 1, then shour recall of 6us)
  // 2  trc_cmd = READ      system control  (4*256-bit)
  // 3  trc_cmd = READ      access control ram  (128*256-bit)

    bit [3:0] brfsm;
    bit counten_brfsm;
    bit recallvld;

    bit trc_busy;
    bit trc_err;
    bit trc_info_lock_err;
    bit [145:0] trc_dout_s0;
    bit [145:0] trc_dout_s1;
    bit [3:0] fd0;
    bit [2:0] fd0half;

    bit [BRCW:0]  bridx_org;
    bit [BRCW-1:0][BRDW-1:0] brdatreg;
    `theregfull(clksys, sysresetn, bridx_org, '0) <= ( bridx_org != BRC ) & brvld ? bridx_org + 1 : bridx_org;
//    assign brdat = brdatreg[bridx];
    assign brdat = {trc_dout_s1[127:0],trc_dout_s0[127:0]};//faye

    `theregfull(clksys, sysresetn, brfsm, '0) <=
            ( brfsm == 0 ) & ( bridx_org == 0 )  & brvld ? 1 :                  // rram recall auto start
            ( brfsm == 1 ) & ( bridx_org == 1 )  & brvld ? 2 :                  // cms pattern
            ( brfsm == 2 ) & ( bridx_org == 4 )  & brvld ? 3 :                  // ip trimming
 //         ( brfsm == 7 ) & ( bridx_org == 3 )  & brvld ? 6 :                  // rram recall start
 //         ( brfsm == 6 ) & ( bridx_org == 3 )  & brvld & recallvld ? 2 :      // rram recall wait
            ( brfsm == 3 ) & ( bridx_org == 8 ) & brvld ? 4 :                   // system cfg
            ( brfsm == 4 ) & ( bridx_org == BRC) & brvld ? 5 :                  // acv
                                                        brfsm;

//  assign counten_brfsm = ( brfsm[3:2] == 2'b00 ) & recallvld;                 // stop index when recall

    `theregfull(clksys, sysresetn, brdone, '0 ) <= brdone | ( brfsm == 5 );

    bit [3:0] bronefsm;
    logic [4:0] brready0;
    assign brready0 = { brready, 1'b1 };
//    `theregfull( clksys, sysresetn, bronefsm, '0 ) <= ( brfsm == 7 ) ? 0 :
//                                                      ( bronefsm == 0 ) ? ( brready0[brfsm[1:0]] ? 1 : bronefsm ):
//                                                      ( bronefsm == 7 ) ? 0 : bronefsm + 1;

    `theregfull(clksys, sysresetn, brdatreg[bridx], '0) <= brvld ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : brdatreg[bridx];

    bit brvldreg;
    assign brvld = !trc_busy;
//  assign brvld = ( bronefsm == fd0 );           // rramclk@25MHz, one read
    `theregfull(clksys, sysresetn, brvldreg, '0 ) <= brvld;

    bit [3:0]   brfsm_cmd;
    bit         brfsm_info;
    bit [BRCW-1:0]  brfsm_xadr;
    bit [4:0]   brfsm_yadr;
    bit [3:0]   brfsm_udin;
    bit         brfsm_rramclk;
    bit         brfsm_rramclk_inv;
    bit         brfsm_desel;

    assign brfsm_rramclk = clksys;// ( bronefsm > fd0half );
    assign brfsm_rramclk_inv = ~clksys;//( bronefsm <= fd0half );

    assign brfsm_cmd = brfsm_desel ? TRC_IDLE : TRC_READ;

/* 
    always@(*)
    casex(brfsm)
        4'h1: brfsm_cmd = TRC_READ;
        4'h2: brfsm_cmd = TRC_READ;
        4'h3: brfsm_cmd = TRC_READ;
        4'h4: brfsm_cmd = TRC_READ;
        4'h7: brfsm_cmd = TRC_RECALL;
        default: brfsm_cmd = TRC_IDLE;
    endcase
*/


    assign bridx = bridx_org - 'h1;

    assign brfsm_desel = (brfsm == 0) | (brfsm == 5);

    assign brfsm_udin = brfsm_desel ? 'h0 : 4'b0101;        // din[3:0]=4'b0101, recall all CFG settings
    assign brfsm_info = brfsm_desel ? 1'b0 : 1'b1;
    assign brfsm_xadr = brfsm_desel ? 'h0 : bridx[BRCW-1:5];
    assign brfsm_yadr = brfsm_desel ? 'h0 : bridx[4:0];


  //  ahb-rram register access
  //  ==============================


//  `ahbs_common
    assign ahbx.hready = 'b1;
    assign ahbx.hresp = 'h0;
    assign ahbx.hrdata = '0
                | sfr_rrccr.hrdata32
                | sfr_rrcfd.hrdata32
                | sfr_rrcsr.hrdata32 ;


  // [3:0]  TRC_CMD 
  // [0]    TRC_Busy
  // [1]    TRC_Error
  // [5]    Nap 
  // [6]    Power Down  

    bit [15:0] rrccr;
    bit [15:0] rrcfd;
    bit [0:8][15:0] rrcsr;
    bit ip_user_nap_i;
    bit ip_user_pd_i;
    bit [3:0] ip_user_cmd_i;
    bit [63:0] trc_regif_dout_s0;
    bit [63:0] trc_regif_dout_s1;
// faye i change the address
    ahb_cr #(.A('h0C), .DW(16))                 sfr_rrccr  (.cr(rrccr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_cr #(.A('h04), .DW(16), .IV('h0007))    sfr_rrcfd  (.cr(rrcfd), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);
    ahb_sr #(.A('h08), .DW(16), .SFRCNT(9))     sfr_rrcsr  (.sr(rrcsr), .hrdata32(), .resetn(coreresetn), .sfrlock(1'b0), .*);

    assign rrcsr[0][5:0] = {ip_user_cmd_i, trc_err, trc_busy};
    assign ip_user_nap_i = rrccr[0];
    assign ip_user_pd_i = rrccr[1];
    assign fd0 = rrcfd[3:0];
    assign fd0half = rrcfd[3:1];
    assign rrcsr[1] = trc_regif_dout_s0[15:0];
    assign rrcsr[2] = trc_regif_dout_s0[31:16];
    assign rrcsr[3] = trc_regif_dout_s0[47:32];
    assign rrcsr[4] = trc_regif_dout_s0[63:48];
    assign rrcsr[5] = trc_regif_dout_s1[15:0];
    assign rrcsr[6] = trc_regif_dout_s1[31:16];
    assign rrcsr[7] = trc_regif_dout_s1[47:32];
    assign rrcsr[8] = trc_regif_dout_s1[63:48];

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

    bit secvld;
    assign secvld = 1'b1;


  //  secure access sram instantiate (TBD)
  //  ==============================
  //  bist read initial only currently

    bit secure_access_sram_cs;
    bit secure_access_sram_wr;
    bit [6:0] secure_access_sram_addr;
    bit [255:0] secure_access_sram_wdata;
    bit [255:0] secure_access_sram_rdata;
    ramif #(.RAW(8), .DW(256)) acramport();

    assign secure_access_sram_cs = ( brfsm == 3 ) & brvldreg;
    assign secure_access_sram_wr = ( brfsm == 3 ) & brvldreg;
    assign secure_access_sram_addr = ( bridx - 8 );
    assign secure_access_sram_wdata = brdatreg[bridx];

    assign acramport.ramen      = 1'b1                          ;
    assign acramport.ramcs      = secure_access_sram_cs         ;
    assign acramport.ramwr      = {32{secure_access_sram_wr}}   ;
    assign acramport.ramaddr    = secure_access_sram_addr       ;
    assign acramport.ramwdata   = secure_access_sram_wdata      ;
    assign acramport.ramrdata   = secure_access_sram_rdata      ;

    sram #(.AW(8),.DW(256)) u_secure_access_sram (
            .clk        ( clk                           ),
            .ramport    ( acramport                     )
    );

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

    bit axi_rramclk;
    bit rramclken;
    bit [3:0] rramclkcnt;

    assign ahb_array_trans = clken & ahbarray.hsel & ahbarray.htrans[1] & secvld & brdone & ahbarray.hready;
    assign ahb_array_read  = ahb_array_trans & !ahbarray.hwrite & !haddr_match;
    assign ahb_array_write = ahb_array_trans & ahbarray.hwrite;
     
     `theregfull(clk, coreresetn, haddr_reg, '0) <= ahb_array_trans ? ahbarray.haddr : haddr_reg;
    assign haddr_match = ( ahbarray.haddr[31:5] == haddr_reg[31:5] );

    bit [BRDW-1:0] ahb_rd_buf;
    bit [BRDW-1:0] ahb_wr_buf;

    bit trc_busy_reg;
    bit trc_dout_ready_s1, trc_dout_ready_s0, trc_dout_ready_reg;
    bit trc_dout_ready;

//    `theregfull(clktop, sysresetn, trc_busy_reg, '1) <= trc_busy;
//    `theregfull(clktop, sysresetn, trc_dout_ready_reg, '1) <= trc_dout_ready;   
    `theregfull(clktop, sysresetn, trc_busy_reg, '1) <= clken ? trc_busy : trc_busy_reg; // faye
    `theregfull(clktop, sysresetn, trc_dout_ready_reg, '1) <= clken ? trc_dout_ready : trc_dout_ready_reg;   //faye


    bit trc_busy_done, trc_dout_ready_done;
    assign trc_busy_done = !trc_busy & trc_busy_reg;
    assign trc_dout_ready_done = trc_dout_ready & !trc_dout_ready_reg;


    always@(*)
    casex(haddr_reg[4:3])
        3'h1: ahbarray.hrdata = ahb_rd_buf[127 : 64];
        3'h2: ahbarray.hrdata = ahb_rd_buf[191 : 128];
        3'h3: ahbarray.hrdata = ahb_rd_buf[255 : 192];
        default: ahbarray.hrdata = ahb_rd_buf[63:0];
    endcase

    `theregfull(clktop, coreresetn, ahb_rd_buf, '0) <= trc_dout_ready_done ? {trc_dout_s1[127:0],trc_dout_s0[127:0]} : ahb_rd_buf;
    `theregfull(clktop, coreresetn, ahb_wr_buf, '0) <= ahb_array_write ? {ahbarray.hwdata,ahb_wr_buf[255:32]} : ahb_wr_buf;

    assign ahbarray.hready = ( rrcfsm == 0 );
    assign ahbarray.hresp = 2'h0;

  //  rram rd/write control
  //  ==============================

    `theregfull(clktop, coreresetn, rrcfsm, '0) <=
            ( rrcfsm == 0 ) & ahb_array_read ? 1 :                              // rram read operation
            ( rrcfsm == 1 ) & trc_dout_ready_done & clken ? 0 :                 // rram read done
            ( rrcfsm == 0 ) & ahb_array_write ? 2 :                             // rram load operation
            ( rrcfsm == 2 ) & trc_busy_done ? 3 :                               // rram write operation
            ( rrcfsm == 3 ) & trc_busy_done & clken ? 0 :                       // rran write done, back to idle
                                                        rrcfsm;

 // `theregfull(clksys, sysresetn, rrcvld, '1) <= trc_busy;
 //  assign rrcvld = trc_busy_done;

    `theregfull( clktop, coreresetn, rramclkcnt, '0 ) <= (trc_dout_ready_done | ( rrcfsm == 0 )) ? 0 :
                                                         ( rramclkcnt == fd0 ) ? 0 : rramclkcnt + 1;
//  assign axi_rramclk = ( rramclkcnt < fd0half );
//  assign axi_rramclk_inv = ( rramclkcnt <= fd0half );
    assign rramclken = (( rrcfsm == 0 ) && (ahb_array_read | ahb_array_write)) | (( rramclkcnt == fd0 ) && ( rrcfsm !== 0 ));
//  assign fdload = (bridx_org == BRC-1);

/*
    cgufdsync rrc_fdu(
                .clk            (clktop),
                .resetn         (sysresetn),
                .clk0en         (1),
                .clk1en         (clken),
                .fd2            (fd0),
                .fdload         (fdload),
                .clk2en         (rramclken),
                .clk2en_atclk1  (rramclken_atclk)
            );
*/

  ICG rramicg ( .CK (clktop   ), .EN ( rramclken ), .SE(cmsatpg), .CKG ( axi_rramclk ));

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
        4'h2: axi_cmd = trc_busy ? TRC_IDLE : TRC_LOAD;
        4'h3: axi_cmd = trc_busy ? TRC_IDLE : TRC_WRITE;
        default: axi_cmd = TRC_IDLE;
    endcase


 //   `theregfull( clktop, coreresetn, axi_cmd, '0 ) <= ahb_array_read ? TRC_READ :
//                                                      ahb_array_write ? TRC_LOAD :
 //                                                        ( rrcfsm == 2 ) & trc_busy_done ? TRC_WRITE : TRC_IDLE;

//    assign axi_cmd = ahb_array_read ? TRC_READ :
//                        ahb_array_write ? TRC_LOAD :
//                            ( rrcfsm == 2 ) & trc_busy_done ? TRC_WRITE : TRC_IDLE;

    assign axi_din = ahb_wr_buf;
    assign axi_info = 1'b0;
    assign axi_xadr = haddr_reg[21:10];
    assign axi_yadr = haddr_reg[9:5];

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

 // bit rramclk_inv;

    bit ip_user_ifren1_i;
    bit ip_user_reden_i;
    bit trc_write_suspend_i;
    bit trc_write_resume_i;
    bit trc_write_abort_i;


    assign trc_busy = trc_busy_s0 | trc_busy_s1;
    assign trc_dout_ready = trc_dout_ready_s0 & trc_dout_ready_s1;
    assign trc_err = trc_err_s0 | trc_err_s1;
    assign trc_info_lock_err = trc_info_lock_err_s0 | trc_info_lock_err_s1;

    assign rramclk = axi_rramclk | (brfsm_rramclk & !coreresetn);
 // assign rramclk_inv = brfsm_rramclk_inv | axi_rramclk_inv;
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

    bit rramtck;
    bit rramtck_inv;
    assign rramtck = 'h0;
    assign rramtck_inv = 'h0;


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


