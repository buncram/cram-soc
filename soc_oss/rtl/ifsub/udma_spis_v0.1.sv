// Copyright 2018 ETH Zurich and University of Bologna.
// Copyright and related rights are licensed under the Solderpad Hardware
// License, Version 0.51 (the "License"); you may not use this file except in
// compliance with the License.  You may obtain a copy of the License at
// http://solderpad.org/licenses/SHL-0.51. Unless required by applicable law
// or agreed to in writing, software, hardware and materials distributed under
// this License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

///////////////////////////////////////////////////////////////////////////////
//
// Description: spis top level
//
///////////////////////////////////////////////////////////////////////////////
//
// Authors    : Antonio Pullini (pullinia@iis.ee.ethz.ch)
//
///////////////////////////////////////////////////////////////////////////////

module udma_spis #(
    parameter L2_AWIDTH_NOAL = 12,
    parameter TRANS_SIZE     = 16
) (
    input  logic                      sys_clk_i,
    input  logic                      periph_clk_i,
	input  logic   	                  rstn_i,

    input  logic                      spis_clk_i   ,
    input  logic                      spis_cs_i    ,
    input  logic                      spis_mosi_i  ,
    output logic                      spis_miso_o  ,
    output logic                      spis_miso_oe ,

//    output logic                      rx_char_event_o,
    output logic                      err_event_o,

	input  logic               [31:0] cfg_data_i,
	input  logic                [4:0] cfg_addr_i,
	input  logic                      cfg_valid_i,
	input  logic                      cfg_rwn_i,
	output logic                      cfg_ready_o,
    output logic               [31:0] cfg_data_o,

    output logic [L2_AWIDTH_NOAL-1:0] cfg_rx_startaddr_o,
    output logic     [TRANS_SIZE-1:0] cfg_rx_size_o,
    output logic                [1:0] cfg_rx_datasize_o,
    output logic                      cfg_rx_continuous_o,
    output logic                      cfg_rx_en_o,
    output logic                      cfg_rx_clr_o,
    input  logic                      cfg_rx_en_i,
    input  logic                      cfg_rx_pending_i,
    input  logic [L2_AWIDTH_NOAL-1:0] cfg_rx_curr_addr_i,
    input  logic     [TRANS_SIZE-1:0] cfg_rx_bytes_left_i,

    output logic [L2_AWIDTH_NOAL-1:0] cfg_tx_startaddr_o,
    output logic     [TRANS_SIZE-1:0] cfg_tx_size_o,
    output logic                [1:0] cfg_tx_datasize_o,
    output logic                      cfg_tx_continuous_o,
    output logic                      cfg_tx_en_o,
    output logic                      cfg_tx_clr_o,
    input  logic                      cfg_tx_en_i,
    input  logic                      cfg_tx_pending_i,
    input  logic [L2_AWIDTH_NOAL-1:0] cfg_tx_curr_addr_i,
    input  logic     [TRANS_SIZE-1:0] cfg_tx_bytes_left_i,

    output logic                      data_tx_req_o,
    input  logic                      data_tx_gnt_i,
    output logic                [1:0] data_tx_datasize_o,
    input  logic               [31:0] data_tx_i,
    input  logic                      data_tx_valid_i,
    output logic                      data_tx_ready_o,
             
    output logic                [1:0] data_rx_datasize_o,
    output logic               [31:0] data_rx_o,
    output logic                      data_rx_valid_o,
    input  logic                      data_rx_ready_i

);
// dummy
    logic                      spis_rx_i;
    logic                      spis_tx_o;
    logic                      rx_char_event_o;

    assign spis_rx_i = '1;
//    logic                      spis_clk_i   ;
//    logic                      spis_cs_i    ;
//    logic                      spis_mosi_i  ;
//    logic                      spis_miso_o  ;
//    logic                      spis_miso_oe ;

    assign spis_miso_o = '1;  
    assign spis_miso_oe = '0;  

// dummy end

    logic               [1:0]  s_spis_status;
    logic                      s_spis_stop_bits;
    logic                      s_spis_parity_en;
    logic              [15:0]  s_spis_div;
    logic               [1:0]  s_spis_bits;
    logic                      s_spis_rx_clean_fifo;
    logic                      s_spis_rx_polling_en;
    logic                      s_spis_rx_irq_en;
    logic                      s_spis_err_irq_en;
    logic                      s_spis_en_rx;
    logic                      s_spis_en_tx;
    logic                      s_data_rx_ready_mux;
    logic                      s_data_rx_ready;

    logic         s_data_tx_valid;
    logic         s_data_tx_ready;
    logic   [7:0] s_data_tx;
    logic         s_data_tx_dc_valid;
    logic         s_data_tx_dc_ready;
    logic   [7:0] s_data_tx_dc;
    logic         s_data_rx_dc_valid;
    logic         s_data_rx_dc_ready;
    logic   [7:0] s_data_rx_dc;

    logic         r_spis_stop_bits;
    logic         r_spis_parity_en;
    logic [15:0]  r_spis_div;
    logic  [1:0]  r_spis_bits;

    logic  [2:0]  r_spis_en_rx_sync;
    logic  [2:0]  r_spis_en_tx_sync;

    logic         s_spis_tx_sample;
    logic         s_spis_rx_sample;

    logic [1:0] [1:0] r_status_sync;

    logic         s_err_rx_overflow;
    logic         s_err_rx_overflow_sync;
    logic         s_err_rx_parity;
    logic         s_err_rx_parity_sync;
    logic         s_rx_char_event;
    logic         s_rx_char_event_sync;

    assign cfg_tx_datasize_o  = 2'b00;
    assign cfg_rx_datasize_o  = 2'b00;
    assign data_tx_datasize_o = 2'b00;
    assign data_rx_datasize_o = 2'b00;

    assign err_event_o = (s_err_rx_overflow_sync | s_err_rx_parity_sync) & s_spis_err_irq_en;
    assign rx_char_event_o = s_rx_char_event_sync & s_spis_rx_irq_en & ~s_spis_rx_polling_en;

    udma_spis_reg_if #(
        .L2_AWIDTH_NOAL(L2_AWIDTH_NOAL),
        .TRANS_SIZE(TRANS_SIZE)
    ) u_reg_if (
        .clk_i              ( sys_clk_i           ),
        .rstn_i             ( rstn_i              ),

        .cfg_data_i         ( cfg_data_i          ),
        .cfg_addr_i         ( cfg_addr_i          ),
        .cfg_valid_i        ( cfg_valid_i         ),
        .cfg_rwn_i          ( cfg_rwn_i           ),
        .cfg_ready_o        ( cfg_ready_o         ),
        .cfg_data_o         ( cfg_data_o          ),

        .cfg_rx_startaddr_o ( cfg_rx_startaddr_o  ),
        .cfg_rx_size_o      ( cfg_rx_size_o       ),
        .cfg_rx_continuous_o( cfg_rx_continuous_o ),
        .cfg_rx_en_o        ( cfg_rx_en_o         ),
        .cfg_rx_clr_o       ( cfg_rx_clr_o        ),
        .cfg_rx_en_i        ( cfg_rx_en_i         ),
        .cfg_rx_pending_i   ( cfg_rx_pending_i    ),
        .cfg_rx_curr_addr_i ( cfg_rx_curr_addr_i  ),
        .cfg_rx_bytes_left_i( cfg_rx_bytes_left_i ),

        .cfg_tx_startaddr_o ( cfg_tx_startaddr_o  ),
        .cfg_tx_size_o      ( cfg_tx_size_o       ),
        .cfg_tx_continuous_o( cfg_tx_continuous_o ),
        .cfg_tx_en_o        ( cfg_tx_en_o         ),
        .cfg_tx_clr_o       ( cfg_tx_clr_o        ),
        .cfg_tx_en_i        ( cfg_tx_en_i         ),
        .cfg_tx_pending_i   ( cfg_tx_pending_i    ),
        .cfg_tx_curr_addr_i ( cfg_tx_curr_addr_i  ),
        .cfg_tx_bytes_left_i( cfg_tx_bytes_left_i ),

        .rx_data_i          ( data_rx_o[7:0]      ),
        .rx_valid_i         ( data_rx_valid_o     ), // Pay attention to clock domain
        .rx_ready_o         ( s_data_rx_ready     ), // Pay attention to clock domain

        .status_i           ( r_status_sync[1]    ),
        .err_parity_i       ( s_err_rx_parity_sync   ),
        .err_overflow_i     ( s_err_rx_overflow_sync ),
        .stop_bits_o        ( s_spis_stop_bits    ),
        .parity_en_o        ( s_spis_parity_en    ),
        .divider_o          ( s_spis_div          ),
        .num_bits_o         ( s_spis_bits         ),
        .rx_clean_fifo_o    ( s_spis_rx_clean_fifo ),
        .rx_polling_en_o    ( s_spis_rx_polling_en ),
        .rx_irq_en_o        ( s_spis_rx_irq_en    ),
        .err_irq_en_o       ( s_spis_err_irq_en   ),
        .en_rx_o			( s_spis_en_rx        ),
        .en_tx_o			( s_spis_en_tx        )
    );


    io_tx_fifo #(
      .DATA_WIDTH(8),
      .BUFFER_DEPTH(2)
      ) u_fifo (
        .clk_i   ( sys_clk_i       ),
        .rstn_i  ( rstn_i          ),
        .clr_i   ( 1'b0            ),
        .data_o  ( s_data_tx       ),
        .valid_o ( s_data_tx_valid ),
        .ready_i ( s_data_tx_ready ),
        .req_o   ( data_tx_req_o   ),
        .gnt_i   ( data_tx_gnt_i   ),
        .valid_i ( data_tx_valid_i ),
        .data_i  ( data_tx_i[7:0]  ),
        .ready_o ( data_tx_ready_o )
    );

    udma_dc_fifo #(8,4) u_dc_fifo_tx
    (
        .src_clk_i    ( sys_clk_i          ),  
        .src_rstn_i   ( rstn_i             ),  
        .src_data_i   ( s_data_tx          ),
        .src_valid_i  ( s_data_tx_valid    ),
        .src_ready_o  ( s_data_tx_ready    ),
        .dst_clk_i    ( periph_clk_i       ),
        .dst_rstn_i   ( rstn_i             ),
        .dst_data_o   ( s_data_tx_dc       ),
        .dst_valid_o  ( s_data_tx_dc_valid ),
        .dst_ready_i  ( s_data_tx_dc_ready )
    );

    udma_uart_tx u_spis_tx(
        .clk_i           ( periph_clk_i       ),
        .rstn_i          ( rstn_i             ),
		.tx_o            ( spis_tx_o          ),
        .busy_o          ( s_spis_status[0]   ),
        .cfg_en_i        ( r_spis_en_tx_sync[2] ),
		.cfg_div_i       ( r_spis_div         ),
		.cfg_parity_en_i ( r_spis_parity_en   ),
		.cfg_bits_i      ( r_spis_bits        ),
		.cfg_stop_bits_i ( r_spis_stop_bits   ),
		.tx_data_i       ( s_data_tx_dc       ),
		.tx_valid_i      ( s_data_tx_dc_valid ),
		.tx_ready_o      ( s_data_tx_dc_ready )
    );


    udma_dc_fifo #(8,4) u_dc_fifo_rx
    (
        .src_clk_i    ( periph_clk_i       ),  
        .src_rstn_i   ( rstn_i & ~s_spis_rx_clean_fifo ),  
        .src_data_i   ( s_data_rx_dc       ),
        .src_valid_i  ( s_data_rx_dc_valid ),
        .src_ready_o  ( s_data_rx_dc_ready ),
        .dst_clk_i    ( sys_clk_i          ),
        .dst_rstn_i   ( rstn_i & ~s_spis_rx_clean_fifo ),
        .dst_data_o   ( data_rx_o[7:0]     ),
        .dst_valid_o  ( data_rx_valid_o    ),
        .dst_ready_i  ( s_data_rx_ready_mux    )
    );

   assign s_data_rx_ready_mux = (s_spis_rx_irq_en | s_spis_rx_polling_en) ? s_data_rx_ready : data_rx_ready_i;

    udma_uart_rx u_spis_rx(
        .clk_i           ( periph_clk_i       ),
        .rstn_i          ( rstn_i             ),
		.rx_i            ( spis_rx_i          ),
        .busy_o          ( s_spis_status[1]   ),
        .cfg_en_i        ( r_spis_en_rx_sync[2] ),
		.cfg_div_i       ( r_spis_div         ),
		.cfg_parity_en_i ( r_spis_parity_en   ),
		.cfg_bits_i      ( r_spis_bits        ),
		.cfg_stop_bits_i ( r_spis_stop_bits   ),
        .err_parity_o    ( s_err_rx_parity    ),
        .err_overflow_o  ( s_err_rx_overflow  ),
        .char_event_o    ( s_rx_char_event    ),
		.rx_data_o       ( s_data_rx_dc       ),
		.rx_valid_o      ( s_data_rx_dc_valid ),
		.rx_ready_i      ( s_data_rx_dc_ready )
    );

    edge_propagator i_ep_err_overflow (
        .clk_tx_i(periph_clk_i),
        .rstn_tx_i(rstn_i),
        .edge_i(s_err_rx_overflow),
        .clk_rx_i(sys_clk_i),
        .rstn_rx_i(rstn_i),
        .edge_o(s_err_rx_overflow_sync)
    );

    edge_propagator i_ep_err_parity (
        .clk_tx_i(periph_clk_i),
        .rstn_tx_i(rstn_i),
        .edge_i(s_err_rx_parity),
        .clk_rx_i(sys_clk_i),
        .rstn_rx_i(rstn_i),
        .edge_o(s_err_rx_parity_sync)
    );

    edge_propagator i_ep_event (
        .clk_tx_i(periph_clk_i),
        .rstn_tx_i(rstn_i),
        .edge_i(s_rx_char_event),
        .clk_rx_i(sys_clk_i),
        .rstn_rx_i(rstn_i),
        .edge_o(s_rx_char_event_sync)
    );


    assign s_spis_tx_sample = r_spis_en_tx_sync[1] & ! r_spis_en_tx_sync[2];
    assign s_spis_rx_sample = r_spis_en_rx_sync[1] & ! r_spis_en_rx_sync[2];

    always_ff @(posedge sys_clk_i or negedge rstn_i) 
    begin
        if(~rstn_i) 
        begin
            r_status_sync <= 0;
        end 
        else 
        begin
            r_status_sync[0] <= s_spis_status;
            r_status_sync[1] <= r_status_sync[0];
        end
    end

    always_ff @(posedge periph_clk_i or negedge rstn_i) 
    begin
        if(~rstn_i) begin
            r_spis_en_tx_sync <= 0;
            r_spis_en_rx_sync <= 0;
            r_spis_div        <= 0;
            r_spis_parity_en  <= 0;
            r_spis_bits       <= 0;
            r_spis_stop_bits  <= 0;
        end else begin
            r_spis_en_tx_sync <= {r_spis_en_tx_sync[1:0],s_spis_en_tx};
            r_spis_en_rx_sync <= {r_spis_en_rx_sync[1:0],s_spis_en_rx};
            if(s_spis_tx_sample || s_spis_rx_sample)
            begin
                r_spis_div        <= s_spis_div;
                r_spis_parity_en  <= s_spis_parity_en;
                r_spis_bits       <= s_spis_bits;
                r_spis_stop_bits  <= s_spis_stop_bits;
            end
        end
    end

    assign data_rx_o[31:8] = 'h0;

endmodule // udma_spis_top
