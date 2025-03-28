module sdvt_sdio_device_sync_dpram #(
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

endmodule;
