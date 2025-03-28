module sddc #(
    parameter NUM_OF_IO_FUNC             = 3'h7 ,    
    parameter CFGBAAW = 32
)(
    input bit   clk,    
    input bit   pclk,    
    input bit   resetn,
    input bit   cmsatpg,
    input bit   cmsbist,
    ahbif.slave     ahbs,//aw12/dw32
    ahbif.master    ahbm,//aw32/dw32
    apbif.slavein   apbs,//aw12/dw32
    apbif.slave     apbx,

    ioif.drive sddc_clk,
    ioif.drive sddc_cmd,
//    ioif.drive sddc_dat[3:0],
    ioif.drive sddc_dat0,
    ioif.drive sddc_dat1,
    ioif.drive sddc_dat2,
    ioif.drive sddc_dat3,

    output logic irq

);

//  io interface
//  ====
//

    logic clksdio              ;
    logic sddc_cmd_w           ;
    logic o_sddc_cmd           ;
    logic o_sddc_cmd_en        ;
    logic o_sddc_cmd_pullup    ;
    logic [3:0] sddc_data_4w         ;
    logic [3:0] o_sddc_data          ;
    logic [3:0] o_sddc_data_en       ;
    logic [3:0] o_sddc_data_pullup   ;
    logic sddcio_pu_clk, sddcio_enable;
    logic [1:0] cr_io;
    assign sddc_clk.oe = '0;
    assign sddc_clk.pu = sddcio_pu_clk;
    assign sddc_clk.po = '0;
    assign clksdio = sddc_clk.pi;

    assign sddc_cmd.oe = sddcio_enable & o_sddc_cmd_en;
    assign sddc_cmd.pu = sddcio_enable ? o_sddc_cmd_pullup : 1'b1;
    assign sddc_cmd.po = o_sddc_cmd;
    assign sddc_cmd_w = sddc_cmd.pi;

    assign sddc_dat0.oe = sddcio_enable & o_sddc_data_en[0];
    assign sddc_dat1.oe = sddcio_enable & o_sddc_data_en[1];
    assign sddc_dat2.oe = sddcio_enable & o_sddc_data_en[2];
    assign sddc_dat3.oe = sddcio_enable & o_sddc_data_en[3];
    assign sddc_dat0.pu = sddcio_enable ? o_sddc_data_pullup : 1'b1;
    assign sddc_dat1.pu = sddcio_enable ? o_sddc_data_pullup : 1'b1;
    assign sddc_dat2.pu = sddcio_enable ? o_sddc_data_pullup : 1'b1;
    assign sddc_dat3.pu = sddcio_enable ? o_sddc_data_pullup : 1'b1;
    assign sddc_dat0.po = o_sddc_data[0]; assign sddc_data_4w[0] = sddc_dat0.pi;
    assign sddc_dat1.po = o_sddc_data[1]; assign sddc_data_4w[1] = sddc_dat1.pi;
    assign sddc_dat2.po = o_sddc_data[2]; assign sddc_data_4w[2] = sddc_dat2.pi;
    assign sddc_dat3.po = o_sddc_data[3]; assign sddc_data_4w[3] = sddc_dat3.pi;

//  sfr
//  ====
//
    bit [23:0] cfg_reg_ocr;
    bit [7:0]  cfg_rd_fifo_threshold;
    bit [7:0] cfg_reg_cccr_sdio_revision, cfg_reg_sd_spec_revision;

    bit [CFGBAAW-1:0] cfg_base_addr_csa;
    bit [0:7][CFGBAAW-1:0] cfg_base_addr_io_func;

    bit [0:7][16:0] cfg_reg_func_cis_ptr;
    bit [0:7][7:0] cfg_reg_func_ext_std_code;

    logic apbrd, apbwr;
    logic sfrlock;
    logic sdioresetn;
    logic arreset;
    logic [1:0] ahbs_hresp2;

    `theregrn( sfrlock ) <= '0;

    `apbs_common;
    assign apbx.prdata = '0
                        | sfr_io.prdata32 
                        | cr_ocr.prdata32 | cr_rdffthres.prdata32 | cr_rev.prdata32
                        | cr_bacsa.prdata32 | cr_baiofn.prdata32 | cr_fncisptr.prdata32 | cr_fnextstdcode.prdata32
                        ;

    apb_cr #(.A('h00), .DW(2), .IV(2'b10))          sfr_io          (.cr(cr_io),.prdata32(),.*);
    apb_ar #(.A('h04), .AR(32'h5a))     sfr_ar          (.ar(arreset),.*);

    apb_cr #(.A('h10), .DW(24))         cr_ocr          (.cr(cfg_reg_ocr), .prdata32(),.*);
    apb_cr #(.A('h14), .DW( 8))         cr_rdffthres    (.cr(cfg_rd_fifo_threshold), .prdata32(),.*);
    apb_cr #(.A('h18), .DW(16))         cr_rev          (.cr({cfg_reg_cccr_sdio_revision,cfg_reg_sd_spec_revision}), .prdata32(),.*);

    apb_cr #(.A('h1C), .DW(CFGBAAW))                cr_bacsa    (.cr(cfg_base_addr_csa), .prdata32(),.*);
    apb_cr #(.A('h20), .DW(CFGBAAW), .SFRCNT(8))    cr_baiofn   (.cr(cfg_base_addr_io_func), .prdata32(),.*);

    apb_cr #(.A('h40), .DW(17), .SFRCNT(8))    cr_fncisptr        (.cr(cfg_reg_func_cis_ptr), .prdata32(),.*);
    apb_cr #(.A('h60), .DW( 8), .SFRCNT(8))    cr_fnextstdcode    (.cr(cfg_reg_func_ext_std_code), .prdata32(),.*);

assign { sddcio_pu_clk, sddcio_enable } = cr_io;

// reset

    scresetgen #(.ICNT(1),.EXTCNT(16))sdresetgen(
        .clk         ( clk ),
        .cmsatpg     ( cmsatpg ),
        .resetn      ( resetn ),
        .resetnin    ( ~arreset ),
        .resetnout   ( sdioresetn )
    );

//  inst
//  ====
//

sdvt_sdio_device_core  #(.NUM_OF_IO_FUNC(NUM_OF_IO_FUNC))u (
    .cmsatpg, .cmsbist,
    .i_clk                         (clk                           ),
    .i_rst_n                       (sdioresetn                    ),

    .i_sdio_clk                    (clksdio                       ),
    .i_sdio_cmd                    (sddc_cmd_w                    ),
    .i_sdio_data                   (sddc_data_4w                  ),
    .o_sdio_cmd                    (o_sddc_cmd                    ),
    .o_sdio_cmd_en                 (o_sddc_cmd_en                 ),
    .o_sdio_data                   (o_sddc_data                   ),
    .o_sdio_data_en                (o_sddc_data_en                ),
    .o_sdio_cmd_pullup             (o_sddc_cmd_pullup             ),
    .o_sdio_data_pullup            (o_sddc_data_pullup            ),

    .i_reg_ocr                     ( cfg_reg_ocr                  ),
    .i_rd_fifo_threshold           ( cfg_rd_fifo_threshold        ),

    .i_base_addr_io_func0          ( cfg_base_addr_io_func[0]  | 64'h0 ),
    .i_base_addr_io_func1          ( cfg_base_addr_io_func[1]  | 64'h0 ),
    .i_base_addr_io_func2          ( cfg_base_addr_io_func[2]  | 64'h0 ),
    .i_base_addr_io_func3          ( cfg_base_addr_io_func[3]  | 64'h0 ),
    .i_base_addr_io_func4          ( cfg_base_addr_io_func[4]  | 64'h0 ),
    .i_base_addr_io_func5          ( cfg_base_addr_io_func[5]  | 64'h0 ),
    .i_base_addr_io_func6          ( cfg_base_addr_io_func[6]  | 64'h0 ),
    .i_base_addr_io_func7          ( cfg_base_addr_io_func[7]  | 64'h0 ),
    .i_base_addr_csa               ( cfg_base_addr_csa         | 64'h0 ),

    .i_reg_cccr_sdio_revision      ( cfg_reg_cccr_sdio_revision      ),
    .i_reg_sd_spec_revision        ( cfg_reg_sd_spec_revision        ),

    .i_reg_func0_cis_ptr           ( cfg_reg_func_cis_ptr[0]           ),
    .i_reg_func1_ext_std_code      ( cfg_reg_func_ext_std_code[1]      ),
    .i_reg_func1_cis_ptr           ( cfg_reg_func_cis_ptr[1]           ),
    .i_reg_func2_ext_std_code      ( cfg_reg_func_ext_std_code[2]      ),
    .i_reg_func2_cis_ptr           ( cfg_reg_func_cis_ptr[2]           ),
    .i_reg_func3_ext_std_code      ( cfg_reg_func_ext_std_code[3]      ),
    .i_reg_func3_cis_ptr           ( cfg_reg_func_cis_ptr[3]           ),
    .i_reg_func4_ext_std_code      ( cfg_reg_func_ext_std_code[4]      ),
    .i_reg_func4_cis_ptr           ( cfg_reg_func_cis_ptr[4]           ),
    .i_reg_func5_ext_std_code      ( cfg_reg_func_ext_std_code[5]      ),
    .i_reg_func5_cis_ptr           ( cfg_reg_func_cis_ptr[5]           ),
    .i_reg_func6_ext_std_code      ( cfg_reg_func_ext_std_code[6]      ),
    .i_reg_func6_cis_ptr           ( cfg_reg_func_cis_ptr[6]           ),
    .i_reg_func7_ext_std_code      ( cfg_reg_func_ext_std_code[7]      ),
    .i_reg_func7_cis_ptr           ( cfg_reg_func_cis_ptr[7]           ),

//    .i_irq                         ( '0                    ),

    .i_ahb_slv_addr                ( ahbs.haddr[11:0]      ),
    .i_ahb_slv_trans               ( ahbs.htrans           ),
    .i_ahb_slv_write               ( ahbs.hwrite           ),
    .i_ahb_slv_burst               ( ahbs.hburst           ),
    .i_ahb_slv_size                ( ahbs.hsize            ),
    .i_ahb_slv_wdata               ( ahbs.hwdata           ),
    .i_ahb_slv_sel                 ( ahbs.hsel             ),
    .i_ahb_slv_ready               ( ahbs.hreadym          ),
    .o_ahb_slv_rdata               ( ahbs.hrdata           ),
    .o_ahb_slv_ready               ( ahbs.hready           ),
    .o_ahb_slv_resp                ( ahbs_hresp2           ),

    .o_irq                         ( irq                   ),

    .o_ahb_mst_addr                ( ahbm.haddr[31:0]      ),
    .o_ahb_mst_trans               ( ahbm.htrans           ),
    .o_ahb_mst_write               ( ahbm.hwrite           ),
    .o_ahb_mst_size                ( ahbm.hsize            ),
    .o_ahb_mst_burst               ( ahbm.hburst           ),
    .o_ahb_mst_prot                ( ahbm.hprot            ),
    .o_ahb_mst_wdata               ( ahbm.hwdata           ),
    .o_ahb_mst_busreq              (                       ),
    .o_ahb_mst_lock                ( ahbm.hmasterlock      ),
    .i_ahb_mst_rdata               ( ahbm.hrdata           ),
    .i_ahb_mst_ready               ( ahbm.hready           ),
    .i_ahb_mst_resp                ( ahbm.hresp | 2'h0      ),
    .i_ahb_mst_grant               ( 1'b1                  ),
    .*
  );

    assign ahbs.hresp = ahbs_hresp2;

endmodule

module dummytb_sddc ();
    parameter CFGBAAW = 32;

    bit   clk;
    bit   pclk;
    bit   resetn;
    bit   cmsatpg;
    bit   cmsbist;
    ahbif  #(.AW(12))  ahbs();
    ahbif    ahbm();
    apbif  #(.PAW(12))  apbs();

    ioif sddc_clk();
    ioif sddc_cmd();
    ioif sddc_dat0();
    ioif sddc_dat1();
    ioif sddc_dat2();
    ioif sddc_dat3();

    wire irq;

    sddc u(.apbs(apbs),.apbx(apbs),.*);

endmodule

