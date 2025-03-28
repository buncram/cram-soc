module qfc #(
    parameter QFC_SSCNT = 4,
    parameter QFC_SIOCNT = 4,
    parameter TX_FIFO_DEPTH  = 128,
    parameter RX_FIFO_DEPTH  = 128,
    parameter CMD_FIFO_DEPTH = 32
    )(
    input bit   clk,    
    input bit   hclk,##
    input bit   pclk,    
    input bit   resetn,
    input bit   cmsatpg,
    input bit   cmsbist,
    ahbif.slave     axim,
    ahbif.master    ahbm,
    apbif.slavein   apbs,
    apbif.slave     apbx,

    ioif.drive qfc_sck,
    ioif.drive qfc_ss[QFC_SSCNT-1:0],
    ioif.drive qfc_sio[QFC_SSCNT-1:0],
    ioif.drive qfc_dqs,

    output logic irq

);

//  io interface
//  ====
//

    logic o_spi_sck, i_spi_dqs;
    logic [7:0] o_spi_mosi_en, i_spi_miso, o_spi_mosi, o_spi_ss;
    logic qfcio_pu_sck, qfcio_pu_dqs, qfcio_pu_ss, qfcio_pu_sio, qfcio_enable;


    assign qfc_sck.pu = qfcio_pu_sck;
    assign qfc_dqs.pu = qfcio_pu_dqs;

    assign qfc_sck.oe = qfcio_enable;
    assign qfc_sck.po = o_spi_sck;

    assign qfc_dqs.oe = '0;
    assign qfc_dqs.po = '0;
    assign i_spi_dqs = qfc_dqs.pi;

    generate
        for (genvar i = 0; i < 8; i++) begin:
            if( i < QFC_SIOCNT )begin:
                assign qfc_sio[i].pu = qfcio_pu_sio;
                assign qfc_sio[i].oe = qfcio_enable & o_spi_mosi_en[i];
                assign qfc_sio[i].po = o_spi_mosi[i];
                assign i_spi_miso[i] = qfc_sio[i].pi;
            end
            else begin:
                assign i_spi_miso[i] = '1;
            end
            if( i < QFC_SSCNT )begin:
                assign qfc_ss[i].oe = qfcio_enable;
                assign qfc_ss[i].pu = qfcio_pu_ss;
                assign qfc_ss[i].po = o_spi_ss[i];
            end
        end
    endgenerate

//  sfr
//  ====
//

    logic apbrd, apbwr;
    logic sfrlock;
    logic sdioresetn;
    logic arreset;
    logic [1:0] ahbs_hresp2;
    bit [4:0] cr_io;
    bit [31:0] cfg_xip_opcode;
    bit [7:0]  cfg_xip_rd_opcode, cfg_xip_rd_opcode_ext, cfg_xip_wr_opcode, cfg_xip_wr_opcode_ext;
    bit [1:0]  cfg_xip_addr_mode;
    bit [5:0]  cfg_xip_width;
    bit [6:0]  cfg_xip_ssel;
    bit [7:0]  cfg_xip_rd_dummy_cycs, cfg_xip_wr_dummy_cycs;
    bit [15:0] cfg_xip_dumcyc;
    bit cfg_xip_clk_phase, cfg_xip_lsb_first, cfg_xip_side_band;
    bit [2:0] cfg_xip_ddr_mode;
    bit [7:0] cfg_xip_prescaler;
    bit [13:0] cfg_xip_cfg;

    `theregrn( sfrlock ) <= '0;

    `apbs_common;
    assign apbx.prdata = '0
                        | sfr_io.prdata32 
                        | cr_xip_addrmode.prdata32 | cr_xip_opcode.prdata32
                        | cr_xip_width.prdata32 | cr_xip_ssel.prdata32 | cr_xip_dumcyc.prdata32 | cr_xip_cfg.prdata32
                        ;

    apb_cr #(.A('h00), .DW(5), .IV(5'h1E))          sfr_io          (.cr(cr_io), .prdata32(),.*);
    apb_ar #(.A('h04), .AR(32'h5a))     sfr_ar          (.ar(arreset),.*);

    apb_cr #(.A('h10), .DW( 2))         cr_xip_addrmode (.cr(cfg_xip_addr_mode), .prdata32(),.*);
    apb_cr #(.A('h14), .DW(32))         cr_xip_opcode   (.cr(cfg_xip_opcode), .prdata32(),.*);
    apb_cr #(.A('h18), .DW( 6))         cr_xip_width    (.cr(cfg_xip_width), .prdata32(),.*);
    apb_cr #(.A('h1C), .DW( 7))         cr_xip_ssel     (.cr(cfg_xip_ssel), .prdata32(),.*);
    apb_cr #(.A('h20), .DW(16))         cr_xip_dumcyc   (.cr(cfg_xip_dumcyc) .prdata32(),.*);
    apb_cr #(.A('h24), .DW(14))         cr_xip_cfg      (.cr(cfg_xip_cfg), .prdata32(),.*);

assign { qfcio_pu_sck, qfcio_pu_dqs, qfcio_pu_ss, qfcio_pu_sio, qfcio_enable } = cr_io;
assign { cfg_xip_rd_opcode, cfg_xip_rd_opcode_ext, cfg_xip_wr_opcode, cfg_xip_wr_opcode_ext } = cfg_xip_opcode;
assign { cfg_xip_rd_dummy_cycs, cfg_xip_wr_dummy_cycs } = cfg_xip_dumcyc;
assign { cfg_xip_clk_phase, cfg_xip_lsb_first, cfg_xip_side_band, cfg_xip_ddr_mode, cfg_xip_prescaler } = cfg_xip_cfg;

    bit cfg_boot_enable = '0;
    bit [7:0] cfg_boot_dummy_cycs = '0;
    bit [2:0] cfg_boot_width = '0;
    bit [6:0] cfg_boot_ssel = '0;
    bit [7:0] cfg_boot_prescaler = '0;
    bit cfg_boot_clk_phase = '0;
    bit cfg_boot_lsb_first = '0;
    bit [31:0] cfg_boot_size = '0;
    bit cfg_boot_ddr_mode = '0;
    bit cfg_boot_side_band = '0;
    bit [31:0] cfg_boot_dst_addr = '0;

// reset

    sdresetgen #(.ICNT(1),.EXTCNT(16))sdresetgen(
        .clk         ( clk ),
        .cmsatpg     ( cmsatpg ),
        .resetn      ( resetn ),
        .resetnin    ( ~arreset ),
        .resetnout   ( qfcresetn )
    );


//  axi
//  ====
//

//  aes
//  ====
//


//  inst
//  ====
//

  sdvt_spi_master_core  #(.TX_FIFO_DEPTH(TX_FIFO_DEPTH),.RX_FIFO_DEPTH(RX_FIFO_DEPTH),.CMD_FIFO_DEPTH(CMD_FIFO_DEPTH))u (

    .cmsatpg                       (cmsatpg                     ),
    .cmsbist                       (cmsbist                     ),
    .i_clk                         (clk                         ),
    .i_dma_clk                     (hclk                        ),

    .i_rst_n                       (qfcresetn                   ),
    .i_dma_rst_n                   (qfcresetn                   ),

    .o_spi_sck                     (o_spi_sck                     ),
    .o_spi_mosi                    (o_spi_mosi                    ),
    .o_spi_mosi_en                 (o_spi_mosi_en                 ),
    .i_spi_miso                    (i_spi_miso                    ),
    .o_spi_ss                      (o_spi_ss                      ),
    .i_spi_dqs                     (i_spi_dqs                     ),

    .i_cfg_xip_addr_mode           (cfg_xip_addr_mode            ),
    .i_cfg_xip_rd_opcode           (cfg_xip_rd_opcode            ),
    .i_cfg_xip_rd_opcode_ext       (cfg_xip_rd_opcode_ext        ),
    .i_cfg_xip_wr_opcode           (cfg_xip_wr_opcode            ),
    .i_cfg_xip_wr_opcode_ext       (cfg_xip_wr_opcode_ext        ),
    .i_cfg_xip_width               (cfg_xip_width                ),
    .i_cfg_xip_ssel                (cfg_xip_ssel                 ),
    .i_cfg_xip_wr_dummy_cycs       (cfg_xip_wr_dummy_cycs        ),
    .i_cfg_xip_rd_dummy_cycs       (cfg_xip_rd_dummy_cycs        ),
    .i_cfg_xip_prescaler           (cfg_xip_prescaler            ),
    .i_cfg_xip_clk_phase           (cfg_xip_clk_phase            ),
    .i_cfg_xip_lsb_first           (cfg_xip_lsb_first            ),
    .i_cfg_xip_side_band           (cfg_xip_side_band            ),
    .i_cfg_xip_ddr_mode            (cfg_xip_ddr_mode             ),
    .i_cfg_boot_enable             (cfg_boot_enable              ),
    .i_cfg_boot_dummy_cycs         (cfg_boot_dummy_cycs          ),
    .i_cfg_boot_width              (cfg_boot_width               ),
    .i_cfg_boot_ssel               (cfg_boot_ssel                ),
    .i_cfg_boot_prescaler          (cfg_boot_prescaler           ),
    .i_cfg_boot_clk_phase          (cfg_boot_clk_phase           ),
    .i_cfg_boot_lsb_first          (cfg_boot_lsb_first           ),
    .i_cfg_boot_size               (cfg_boot_size                ),
    .i_cfg_boot_ddr_mode           (cfg_boot_ddr_mode            ),
    .i_cfg_boot_side_band          (cfg_boot_side_band           ),
    .i_cfg_boot_dst_addr           (cfg_boot_dst_addr            ),

    .i_scan_rst                    (i_scan_rst                    ),
    .i_scan_clk                    (i_scan_clk                    ),
    .i_scan_mode                   (i_scan_mode                   ),
    .i_scan_in                     (i_scan_in                     ),
    .o_scan_out                    (o_scan_out                    ),

    .i_apb_slv_addr                ( apbx.paddr[8:0]              ),
    .i_apb_slv_wdata               ( apbx.pwdata                  ),
    .o_apb_slv_rdata               ( apbx.prdata                  ),
    .i_apb_slv_sel                 ( apbx.psel & apbx.paddr[9]    ),
    .i_apb_slv_enable              ( apbx.penable                 ),
    .i_apb_slv_write               ( apbx.pwrite                  ),
    .o_apb_slv_ready               ( apbx.pready                  ),
    .o_apb_slv_err                 ( apbx.pslverr                 ),

    .o_irq                         ( irq                          ),

    .i_ahb_slv_addr                ( ahbs.haddr                   ),##
    .i_ahb_slv_trans               ( ahbs.htrans                  ),
    .i_ahb_slv_write               ( ahbs.hwrite                  ),
    .i_ahb_slv_burst               ( ahbs.hburst                  ),
    .i_ahb_slv_size                ( ahbs.hsize                   ),
    .i_ahb_slv_wdata               ( ahbs.hwdata                  ),
    .i_ahb_slv_sel                 ( ahbs.hsel                    ),
    .i_ahb_slv_ready               ( ahbs.hready                  ),
    .o_ahb_slv_rdata               ( ahbs.hrdata                  ),
    .o_ahb_slv_ready               ( ahbs.hready                  ),
    .o_ahb_slv_resp                ( ahbs_hresp2                  ),

    .o_ahb_mst_addr                ( ahbm.haddr                   ),
    .o_ahb_mst_trans               ( ahbm.htrans                  ),
    .o_ahb_mst_write               ( ahbm.hwrite                  ),
    .o_ahb_mst_size                ( ahbm.hsize                   ),
    .o_ahb_mst_burst               ( ahbm.hburst                  ),
    .o_ahb_mst_prot                ( ahbm.hprot                   ),
    .o_ahb_mst_wdata               ( ahbm.hwdata                  ),
    .o_ahb_mst_busreq              (                              ),
    .o_ahb_mst_lock                ( ahbm.hmasterlock             ),
    .i_ahb_mst_rdata               ( ahbm.hrdata                  ),
    .i_ahb_mst_ready               ( ahbm.hready                  ),
    .i_ahb_mst_resp                ( ahbm.hresp | 2'h0            ),
    .i_ahb_mst_grant               ( '1                           )
  );

    assign ahbs.hresp = ahbs_hresp2;

endmodule


module sdvt_spi_device_sync_dpram #(
  parameter SYNC_DPRAM_ADDR_WIDTH      = 4                ,  // Address bus width
  parameter SYNC_DPRAM_DEPTH           = 16               ,  // FIFO Depth
  parameter SYNC_DPRAM_DATA_WIDTH      = 32                  // Data port width
 ) (
  input     wire cmsatpg,
  input     wire cmsbist,
  input     wire                                   i_wr_clk                       , // SYNC DP RAM - Write Clock input
  input     wire                                   i_rd_clk                       , // SYNC DP RAM - Read Clock input
  input     wire                                   i_sync_dpram_write             , // SYNC DP RAM - Write control
  input     wire   [SYNC_DPRAM_ADDR_WIDTH - 1:0]   i_sync_dpram_waddr             , // SYNC DP RAM - Write pointer input
  input     wire   [SYNC_DPRAM_DATA_WIDTH - 1:0]   i_sync_dpram_data              , // SYNC DP RAM - Write data input 
  input     wire                                   i_sync_dpram_read              , // SYNC DP RAM - Read control
  input     wire   [SYNC_DPRAM_ADDR_WIDTH - 1:0]   i_sync_dpram_raddr             , // SYNC DP RAM - Read Pointer input
  output    reg    [SYNC_DPRAM_DATA_WIDTH - 1:0]   o_sync_dpram_data                // SYNC DP RAM - Read data
);

`ifdef FPGA
    bramdp #(.AW(SYNC_DPRAM_ADDR_WIDTH),.DW(SYNC_DPRAM_DATA_WIDTH))u(
        .rclk(i_rd_clk),
        .wclk(i_wr_clk),
        .rramaddr(i_sync_dpram_raddr),
        .wramaddr(i_sync_dpram_waddr),
        .rramrd(i_sync_dpram_read),
        .wramwr(i_sync_dpram_write),
        .rramrdata(o_sync_dpram_data),
        .wramwdata(i_sync_dpram_data)
        );
`else
  //1-------------------------------------------------------------------------------------------------
  // Define local variables 
  //1-------------------------------------------------------------------------------------------------
  reg [SYNC_DPRAM_DATA_WIDTH-1:0]            reg_storage_2d_bv [0:(SYNC_DPRAM_DEPTH - 1)]             ; // Register storage memory

  //1-------------------------------------------------------------------------------------------------
  // Write clock 
  //1-------------------------------------------------------------------------------------------------
  always @ (posedge i_wr_clk)
    begin
      //3---------------------------------------------------------------------------------------------
      // write output data 
      //3---------------------------------------------------------------------------------------------
      if (i_sync_dpram_write == 1) begin
        reg_storage_2d_bv[i_sync_dpram_waddr] <= i_sync_dpram_data; // spyglass disable ResetFlop-ML
      end
    end

  //1-------------------------------------------------------------------------------------------------
  // Read clock 
  //1-------------------------------------------------------------------------------------------------
  always @ (posedge i_rd_clk)
    begin
      //3---------------------------------------------------------------------------------------------
      // Read output data 
      //3---------------------------------------------------------------------------------------------
      if (i_sync_dpram_read) begin
        o_sync_dpram_data <= reg_storage_2d_bv[i_sync_dpram_raddr]; // spyglass disable ResetFlop-ML
      end
    end
`endif

endmodule



