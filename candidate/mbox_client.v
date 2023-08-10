// -----------------------------------------------------------------------------
// Auto-Generated by:        __   _ __      _  __
//                          / /  (_) /____ | |/_/
//                         / /__/ / __/ -_)>  <
//                        /____/_/\__/\__/_/|_|
//                     Build your hardware, easily!
//                   https://github.com/enjoy-digital/litex
//
// Filename   : mbox_client.v
// Device     : generic
// LiteX sha1 : e08384a2
// Date       : 2023-08-10 18:05:38
//------------------------------------------------------------------------------

`timescale 1ns / 1ps

//------------------------------------------------------------------------------
// Module
//------------------------------------------------------------------------------

module mbox_client (
    input  wire          aclk_reset_n,
    input  wire          pclk_reset_n,
    input  wire          aclk,
    input  wire          pclk,
    output wire   [31:0] mbox_w_dat,
    output wire          mbox_w_valid,
    input  wire          mbox_w_ready,
    output wire          mbox_w_done,
    input  wire   [31:0] mbox_r_dat,
    input  wire          mbox_r_valid,
    output wire          mbox_r_ready,
    input  wire          mbox_r_done,
    output reg           mbox_w_abort,
    input  wire          mbox_r_abort,
    input  wire   [31:0] sfr_cr_wdata,
    input  wire          sfr_cr_wdata_written,
    output wire   [31:0] sfr_sr_rdata,
    input  wire          sfr_sr_rdata_read,
    output wire          sfr_int_available,
    output wire          sfr_int_abort_init,
    output wire          sfr_int_abort_done,
    output wire          sfr_int_error,
    input  wire          sfr_sr_read,
    output wire          sfr_sr_rx_avail,
    output wire          sfr_sr_tx_free,
    output wire          sfr_sr_abort_in_progress,
    output wire          sfr_sr_abort_ack,
    output reg           sfr_sr_rx_err,
    output reg           sfr_sr_tx_err,
    input  wire          sfr_ar_abort,
    input  wire          sfr_ar_done
);


//------------------------------------------------------------------------------
// Signals
//------------------------------------------------------------------------------

wire          sys_clk;
wire          sys_rst;
wire          pclk_clk;
wire          pclk_rst;
wire          cr_wdata_written_aclk;
wire          sr_rdata_read_aclk;
wire          sr_read_aclk;
wire          ar_abort_aclk;
wire          ar_done_aclk;
wire          int_available_aclk;
reg           int_abort_init_aclk;
reg           int_abort_done_aclk;
wire          wdata_sync_i;
wire          wdata_sync_o;
wire          wdata_sync_ps_i;
wire          wdata_sync_ps_o;
reg           wdata_sync_ps_toggle_i;
wire          wdata_sync_ps_toggle_o;
reg           wdata_sync_ps_toggle_o_r;
wire          wdata_sync_ps_ack_i;
wire          wdata_sync_ps_ack_o;
reg           wdata_sync_ps_ack_toggle_i;
wire          wdata_sync_ps_ack_toggle_o;
reg           wdata_sync_ps_ack_toggle_o_r;
reg           wdata_sync_blind;
wire          rdata_sync_i;
wire          rdata_sync_o;
wire          rdata_sync_ps_i;
wire          rdata_sync_ps_o;
reg           rdata_sync_ps_toggle_i;
wire          rdata_sync_ps_toggle_o;
reg           rdata_sync_ps_toggle_o_r;
wire          rdata_sync_ps_ack_i;
wire          rdata_sync_ps_ack_o;
reg           rdata_sync_ps_ack_toggle_i;
wire          rdata_sync_ps_ack_toggle_o;
reg           rdata_sync_ps_ack_toggle_o_r;
reg           rdata_sync_blind;
wire          read_sync_i;
wire          read_sync_o;
wire          read_sync_ps_i;
wire          read_sync_ps_o;
reg           read_sync_ps_toggle_i;
wire          read_sync_ps_toggle_o;
reg           read_sync_ps_toggle_o_r;
wire          read_sync_ps_ack_i;
wire          read_sync_ps_ack_o;
reg           read_sync_ps_ack_toggle_i;
wire          read_sync_ps_ack_toggle_o;
reg           read_sync_ps_ack_toggle_o_r;
reg           read_sync_blind;
wire          abort_sync_i;
wire          abort_sync_o;
wire          abort_sync_ps_i;
wire          abort_sync_ps_o;
reg           abort_sync_ps_toggle_i;
wire          abort_sync_ps_toggle_o;
reg           abort_sync_ps_toggle_o_r;
wire          abort_sync_ps_ack_i;
wire          abort_sync_ps_ack_o;
reg           abort_sync_ps_ack_toggle_i;
wire          abort_sync_ps_ack_toggle_o;
reg           abort_sync_ps_ack_toggle_o_r;
reg           abort_sync_blind;
wire          done_sync_i;
wire          done_sync_o;
wire          done_sync_ps_i;
wire          done_sync_ps_o;
reg           done_sync_ps_toggle_i;
wire          done_sync_ps_toggle_o;
reg           done_sync_ps_toggle_o_r;
wire          done_sync_ps_ack_i;
wire          done_sync_ps_ack_o;
reg           done_sync_ps_ack_toggle_i;
wire          done_sync_ps_ack_toggle_o;
reg           done_sync_ps_ack_toggle_o_r;
reg           done_sync_blind;
wire          int_available_sync_i;
wire          int_available_sync_o;
wire          int_available_sync_ps_i;
wire          int_available_sync_ps_o;
reg           int_available_sync_ps_toggle_i;
wire          int_available_sync_ps_toggle_o;
reg           int_available_sync_ps_toggle_o_r;
wire          int_available_sync_ps_ack_i;
wire          int_available_sync_ps_ack_o;
reg           int_available_sync_ps_ack_toggle_i;
wire          int_available_sync_ps_ack_toggle_o;
reg           int_available_sync_ps_ack_toggle_o_r;
reg           int_available_sync_blind;
wire          int_abort_init_sync_i;
wire          int_abort_init_sync_o;
wire          int_abort_init_sync_ps_i;
wire          int_abort_init_sync_ps_o;
reg           int_abort_init_sync_ps_toggle_i;
wire          int_abort_init_sync_ps_toggle_o;
reg           int_abort_init_sync_ps_toggle_o_r;
wire          int_abort_init_sync_ps_ack_i;
wire          int_abort_init_sync_ps_ack_o;
reg           int_abort_init_sync_ps_ack_toggle_i;
wire          int_abort_init_sync_ps_ack_toggle_o;
reg           int_abort_init_sync_ps_ack_toggle_o_r;
reg           int_abort_init_sync_blind;
wire          int_abort_done_sync_i;
wire          int_abort_done_sync_o;
wire          int_abort_done_sync_ps_i;
wire          int_abort_done_sync_ps_o;
reg           int_abort_done_sync_ps_toggle_i;
wire          int_abort_done_sync_ps_toggle_o;
reg           int_abort_done_sync_ps_toggle_o_r;
wire          int_abort_done_sync_ps_ack_i;
wire          int_abort_done_sync_ps_ack_o;
reg           int_abort_done_sync_ps_ack_toggle_i;
wire          int_abort_done_sync_ps_ack_toggle_o;
reg           int_abort_done_sync_ps_ack_toggle_o_r;
reg           int_abort_done_sync_blind;
reg           abort_in_progress;
reg           abort_ack;
reg           w_pending;
reg           w_valid_r;
reg           ar_done_r;
reg           ar_abort_r;
reg           sr_read_r;
reg           rdata_read_r;
reg     [1:0] state;
reg     [1:0] next_state;
reg           abort_ack_next_value0;
reg           abort_ack_next_value_ce0;
reg           abort_in_progress_next_value1;
reg           abort_in_progress_next_value_ce1;
reg           multiregimpl0_regs0;
reg           multiregimpl0_regs1;
reg           multiregimpl1_regs0;
reg           multiregimpl1_regs1;
reg           multiregimpl2_regs0;
reg           multiregimpl2_regs1;
reg           multiregimpl3_regs0;
reg           multiregimpl3_regs1;
reg           multiregimpl4_regs0;
reg           multiregimpl4_regs1;
reg           multiregimpl5_regs0;
reg           multiregimpl5_regs1;
reg           multiregimpl6_regs0;
reg           multiregimpl6_regs1;
reg           multiregimpl7_regs0;
reg           multiregimpl7_regs1;
reg           multiregimpl8_regs0;
reg           multiregimpl8_regs1;
reg           multiregimpl9_regs0;
reg           multiregimpl9_regs1;
reg           multiregimpl10_regs0;
reg           multiregimpl10_regs1;
reg           multiregimpl11_regs0;
reg           multiregimpl11_regs1;
reg           multiregimpl12_regs0;
reg           multiregimpl12_regs1;
reg           multiregimpl13_regs0;
reg           multiregimpl13_regs1;
reg           multiregimpl14_regs0;
reg           multiregimpl14_regs1;
reg           multiregimpl15_regs0;
reg           multiregimpl15_regs1;

//------------------------------------------------------------------------------
// Combinatorial Logic
//------------------------------------------------------------------------------

assign sys_clk = aclk;
assign sys_rst = (~aclk_reset_n);
assign pclk_clk = pclk;
assign pclk_rst = (~pclk_reset_n);
assign wdata_sync_i = sfr_cr_wdata_written;
assign cr_wdata_written_aclk = wdata_sync_o;
assign rdata_sync_i = sfr_sr_rdata_read;
assign sr_rdata_read_aclk = rdata_sync_o;
assign read_sync_i = sfr_sr_read;
assign sr_read_aclk = read_sync_o;
assign abort_sync_i = sfr_ar_abort;
assign ar_abort_aclk = abort_sync_o;
assign done_sync_i = sfr_ar_done;
assign ar_done_aclk = done_sync_o;
assign sfr_int_available = int_available_sync_o;
assign int_available_sync_i = int_available_aclk;
assign sfr_int_abort_init = int_abort_init_sync_o;
assign int_abort_init_sync_i = int_abort_init_aclk;
assign sfr_int_abort_done = int_abort_done_sync_o;
assign int_abort_done_sync_i = int_abort_done_aclk;
assign sfr_int_error = (sfr_sr_tx_err | sfr_sr_rx_err);
assign mbox_w_dat = sfr_cr_wdata;
assign mbox_w_valid = ((cr_wdata_written_aclk & (~w_valid_r)) | w_pending);
assign mbox_w_done = (ar_done_aclk & (~ar_done_r));
assign sfr_sr_tx_free = (~(mbox_w_valid | w_pending));
assign sfr_sr_rdata = mbox_r_dat;
assign mbox_r_ready = (sr_rdata_read_aclk & (~rdata_read_r));
assign int_available_aclk = mbox_r_done;
assign sfr_sr_rx_avail = mbox_r_valid;
assign sfr_sr_abort_in_progress = abort_in_progress;
assign sfr_sr_abort_ack = abort_ack;
assign wdata_sync_ps_i = (wdata_sync_i & (~wdata_sync_blind));
assign wdata_sync_ps_ack_i = wdata_sync_ps_o;
assign wdata_sync_o = wdata_sync_ps_o;
assign wdata_sync_ps_o = (wdata_sync_ps_toggle_o ^ wdata_sync_ps_toggle_o_r);
assign wdata_sync_ps_ack_o = (wdata_sync_ps_ack_toggle_o ^ wdata_sync_ps_ack_toggle_o_r);
assign rdata_sync_ps_i = (rdata_sync_i & (~rdata_sync_blind));
assign rdata_sync_ps_ack_i = rdata_sync_ps_o;
assign rdata_sync_o = rdata_sync_ps_o;
assign rdata_sync_ps_o = (rdata_sync_ps_toggle_o ^ rdata_sync_ps_toggle_o_r);
assign rdata_sync_ps_ack_o = (rdata_sync_ps_ack_toggle_o ^ rdata_sync_ps_ack_toggle_o_r);
assign read_sync_ps_i = (read_sync_i & (~read_sync_blind));
assign read_sync_ps_ack_i = read_sync_ps_o;
assign read_sync_o = read_sync_ps_o;
assign read_sync_ps_o = (read_sync_ps_toggle_o ^ read_sync_ps_toggle_o_r);
assign read_sync_ps_ack_o = (read_sync_ps_ack_toggle_o ^ read_sync_ps_ack_toggle_o_r);
assign abort_sync_ps_i = (abort_sync_i & (~abort_sync_blind));
assign abort_sync_ps_ack_i = abort_sync_ps_o;
assign abort_sync_o = abort_sync_ps_o;
assign abort_sync_ps_o = (abort_sync_ps_toggle_o ^ abort_sync_ps_toggle_o_r);
assign abort_sync_ps_ack_o = (abort_sync_ps_ack_toggle_o ^ abort_sync_ps_ack_toggle_o_r);
assign done_sync_ps_i = (done_sync_i & (~done_sync_blind));
assign done_sync_ps_ack_i = done_sync_ps_o;
assign done_sync_o = done_sync_ps_o;
assign done_sync_ps_o = (done_sync_ps_toggle_o ^ done_sync_ps_toggle_o_r);
assign done_sync_ps_ack_o = (done_sync_ps_ack_toggle_o ^ done_sync_ps_ack_toggle_o_r);
assign int_available_sync_ps_i = (int_available_sync_i & (~int_available_sync_blind));
assign int_available_sync_ps_ack_i = int_available_sync_ps_o;
assign int_available_sync_o = int_available_sync_ps_o;
assign int_available_sync_ps_o = (int_available_sync_ps_toggle_o ^ int_available_sync_ps_toggle_o_r);
assign int_available_sync_ps_ack_o = (int_available_sync_ps_ack_toggle_o ^ int_available_sync_ps_ack_toggle_o_r);
assign int_abort_init_sync_ps_i = (int_abort_init_sync_i & (~int_abort_init_sync_blind));
assign int_abort_init_sync_ps_ack_i = int_abort_init_sync_ps_o;
assign int_abort_init_sync_o = int_abort_init_sync_ps_o;
assign int_abort_init_sync_ps_o = (int_abort_init_sync_ps_toggle_o ^ int_abort_init_sync_ps_toggle_o_r);
assign int_abort_init_sync_ps_ack_o = (int_abort_init_sync_ps_ack_toggle_o ^ int_abort_init_sync_ps_ack_toggle_o_r);
assign int_abort_done_sync_ps_i = (int_abort_done_sync_i & (~int_abort_done_sync_blind));
assign int_abort_done_sync_ps_ack_i = int_abort_done_sync_ps_o;
assign int_abort_done_sync_o = int_abort_done_sync_ps_o;
assign int_abort_done_sync_ps_o = (int_abort_done_sync_ps_toggle_o ^ int_abort_done_sync_ps_toggle_o_r);
assign int_abort_done_sync_ps_ack_o = (int_abort_done_sync_ps_ack_toggle_o ^ int_abort_done_sync_ps_ack_toggle_o_r);
always @(*) begin
    next_state <= 2'd0;
    mbox_w_abort <= 1'd0;
    abort_ack_next_value0 <= 1'd0;
    abort_ack_next_value_ce0 <= 1'd0;
    abort_in_progress_next_value1 <= 1'd0;
    abort_in_progress_next_value_ce1 <= 1'd0;
    int_abort_init_aclk <= 1'd0;
    int_abort_done_aclk <= 1'd0;
    next_state <= state;
    case (state)
        1'd1: begin
            if (mbox_r_abort) begin
                next_state <= 1'd0;
                abort_in_progress_next_value1 <= 1'd0;
                abort_in_progress_next_value_ce1 <= 1'd1;
                int_abort_done_aclk <= 1'd1;
            end
            mbox_w_abort <= 1'd1;
        end
        2'd2: begin
            if ((ar_abort_aclk & (~ar_abort_r))) begin
                next_state <= 1'd0;
                abort_in_progress_next_value1 <= 1'd0;
                abort_in_progress_next_value_ce1 <= 1'd1;
                abort_ack_next_value0 <= 1'd1;
                abort_ack_next_value_ce0 <= 1'd1;
                mbox_w_abort <= 1'd1;
            end else begin
                mbox_w_abort <= 1'd0;
            end
        end
        default: begin
            if (((ar_abort_aclk & (~ar_abort_r)) & (~mbox_r_abort))) begin
                next_state <= 1'd1;
                abort_ack_next_value0 <= 1'd0;
                abort_ack_next_value_ce0 <= 1'd1;
                abort_in_progress_next_value1 <= 1'd1;
                abort_in_progress_next_value_ce1 <= 1'd1;
                mbox_w_abort <= 1'd1;
            end else begin
                if (((ar_abort_aclk & (~ar_abort_r)) & mbox_r_abort)) begin
                    next_state <= 1'd0;
                    abort_ack_next_value0 <= 1'd1;
                    abort_ack_next_value_ce0 <= 1'd1;
                    mbox_w_abort <= 1'd1;
                end else begin
                    if (((~(ar_abort_aclk & (~ar_abort_r))) & mbox_r_abort)) begin
                        next_state <= 2'd2;
                        abort_in_progress_next_value1 <= 1'd1;
                        abort_in_progress_next_value_ce1 <= 1'd1;
                        int_abort_init_aclk <= 1'd1;
                        mbox_w_abort <= 1'd0;
                    end else begin
                        mbox_w_abort <= 1'd0;
                    end
                end
            end
        end
    endcase
end
assign wdata_sync_ps_toggle_o = multiregimpl0_regs1;
assign wdata_sync_ps_ack_toggle_o = multiregimpl1_regs1;
assign rdata_sync_ps_toggle_o = multiregimpl2_regs1;
assign rdata_sync_ps_ack_toggle_o = multiregimpl3_regs1;
assign read_sync_ps_toggle_o = multiregimpl4_regs1;
assign read_sync_ps_ack_toggle_o = multiregimpl5_regs1;
assign abort_sync_ps_toggle_o = multiregimpl6_regs1;
assign abort_sync_ps_ack_toggle_o = multiregimpl7_regs1;
assign done_sync_ps_toggle_o = multiregimpl8_regs1;
assign done_sync_ps_ack_toggle_o = multiregimpl9_regs1;
assign int_available_sync_ps_toggle_o = multiregimpl10_regs1;
assign int_available_sync_ps_ack_toggle_o = multiregimpl11_regs1;
assign int_abort_init_sync_ps_toggle_o = multiregimpl12_regs1;
assign int_abort_init_sync_ps_ack_toggle_o = multiregimpl13_regs1;
assign int_abort_done_sync_ps_toggle_o = multiregimpl14_regs1;
assign int_abort_done_sync_ps_ack_toggle_o = multiregimpl15_regs1;


//------------------------------------------------------------------------------
// Synchronous Logic
//------------------------------------------------------------------------------

always @(posedge pclk_clk) begin
    if (wdata_sync_i) begin
        wdata_sync_blind <= 1'd1;
    end
    if (wdata_sync_ps_ack_o) begin
        wdata_sync_blind <= 1'd0;
    end
    if (wdata_sync_ps_i) begin
        wdata_sync_ps_toggle_i <= (~wdata_sync_ps_toggle_i);
    end
    wdata_sync_ps_ack_toggle_o_r <= wdata_sync_ps_ack_toggle_o;
    if (rdata_sync_i) begin
        rdata_sync_blind <= 1'd1;
    end
    if (rdata_sync_ps_ack_o) begin
        rdata_sync_blind <= 1'd0;
    end
    if (rdata_sync_ps_i) begin
        rdata_sync_ps_toggle_i <= (~rdata_sync_ps_toggle_i);
    end
    rdata_sync_ps_ack_toggle_o_r <= rdata_sync_ps_ack_toggle_o;
    if (read_sync_i) begin
        read_sync_blind <= 1'd1;
    end
    if (read_sync_ps_ack_o) begin
        read_sync_blind <= 1'd0;
    end
    if (read_sync_ps_i) begin
        read_sync_ps_toggle_i <= (~read_sync_ps_toggle_i);
    end
    read_sync_ps_ack_toggle_o_r <= read_sync_ps_ack_toggle_o;
    if (abort_sync_i) begin
        abort_sync_blind <= 1'd1;
    end
    if (abort_sync_ps_ack_o) begin
        abort_sync_blind <= 1'd0;
    end
    if (abort_sync_ps_i) begin
        abort_sync_ps_toggle_i <= (~abort_sync_ps_toggle_i);
    end
    abort_sync_ps_ack_toggle_o_r <= abort_sync_ps_ack_toggle_o;
    if (done_sync_i) begin
        done_sync_blind <= 1'd1;
    end
    if (done_sync_ps_ack_o) begin
        done_sync_blind <= 1'd0;
    end
    if (done_sync_ps_i) begin
        done_sync_ps_toggle_i <= (~done_sync_ps_toggle_i);
    end
    done_sync_ps_ack_toggle_o_r <= done_sync_ps_ack_toggle_o;
    int_available_sync_ps_toggle_o_r <= int_available_sync_ps_toggle_o;
    if (int_available_sync_ps_ack_i) begin
        int_available_sync_ps_ack_toggle_i <= (~int_available_sync_ps_ack_toggle_i);
    end
    int_abort_init_sync_ps_toggle_o_r <= int_abort_init_sync_ps_toggle_o;
    if (int_abort_init_sync_ps_ack_i) begin
        int_abort_init_sync_ps_ack_toggle_i <= (~int_abort_init_sync_ps_ack_toggle_i);
    end
    int_abort_done_sync_ps_toggle_o_r <= int_abort_done_sync_ps_toggle_o;
    if (int_abort_done_sync_ps_ack_i) begin
        int_abort_done_sync_ps_ack_toggle_i <= (~int_abort_done_sync_ps_ack_toggle_i);
    end
    if (pclk_rst) begin
        wdata_sync_ps_toggle_i <= 1'd0;
        wdata_sync_blind <= 1'd0;
        rdata_sync_ps_toggle_i <= 1'd0;
        rdata_sync_blind <= 1'd0;
        read_sync_ps_toggle_i <= 1'd0;
        read_sync_blind <= 1'd0;
        abort_sync_ps_toggle_i <= 1'd0;
        abort_sync_blind <= 1'd0;
        done_sync_ps_toggle_i <= 1'd0;
        done_sync_blind <= 1'd0;
        int_available_sync_ps_ack_toggle_i <= 1'd0;
        int_abort_init_sync_ps_ack_toggle_i <= 1'd0;
        int_abort_done_sync_ps_ack_toggle_i <= 1'd0;
    end
    multiregimpl1_regs0 <= wdata_sync_ps_ack_toggle_i;
    multiregimpl1_regs1 <= multiregimpl1_regs0;
    multiregimpl3_regs0 <= rdata_sync_ps_ack_toggle_i;
    multiregimpl3_regs1 <= multiregimpl3_regs0;
    multiregimpl5_regs0 <= read_sync_ps_ack_toggle_i;
    multiregimpl5_regs1 <= multiregimpl5_regs0;
    multiregimpl7_regs0 <= abort_sync_ps_ack_toggle_i;
    multiregimpl7_regs1 <= multiregimpl7_regs0;
    multiregimpl9_regs0 <= done_sync_ps_ack_toggle_i;
    multiregimpl9_regs1 <= multiregimpl9_regs0;
    multiregimpl10_regs0 <= int_available_sync_ps_toggle_i;
    multiregimpl10_regs1 <= multiregimpl10_regs0;
    multiregimpl12_regs0 <= int_abort_init_sync_ps_toggle_i;
    multiregimpl12_regs1 <= multiregimpl12_regs0;
    multiregimpl14_regs0 <= int_abort_done_sync_ps_toggle_i;
    multiregimpl14_regs1 <= multiregimpl14_regs0;
end

always @(posedge sys_clk) begin
    w_valid_r <= cr_wdata_written_aclk;
    ar_done_r <= ar_done_aclk;
    ar_abort_r <= ar_abort_aclk;
    if (((cr_wdata_written_aclk & (~w_valid_r)) & (~mbox_w_ready))) begin
        w_pending <= 1'd1;
    end else begin
        if ((mbox_w_ready | ((sfr_sr_tx_err & sr_read_aclk) & (~sr_read_r)))) begin
            w_pending <= 1'd0;
        end else begin
            w_pending <= w_pending;
        end
    end
    sr_read_r <= sr_read_aclk;
    if ((sr_read_aclk & (~sr_read_r))) begin
        sfr_sr_tx_err <= 1'd0;
    end else begin
        if (((mbox_w_valid & (~mbox_w_ready)) & w_pending)) begin
            sfr_sr_tx_err <= 1'd1;
        end else begin
            sfr_sr_tx_err <= sfr_sr_tx_err;
        end
    end
    rdata_read_r <= sr_rdata_read_aclk;
    if ((sr_read_aclk & (~sr_read_r))) begin
        sfr_sr_rx_err <= 1'd0;
    end else begin
        if ((mbox_r_ready & (~mbox_r_valid))) begin
            sfr_sr_rx_err <= 1'd1;
        end else begin
            sfr_sr_rx_err <= sfr_sr_rx_err;
        end
    end
    wdata_sync_ps_toggle_o_r <= wdata_sync_ps_toggle_o;
    if (wdata_sync_ps_ack_i) begin
        wdata_sync_ps_ack_toggle_i <= (~wdata_sync_ps_ack_toggle_i);
    end
    rdata_sync_ps_toggle_o_r <= rdata_sync_ps_toggle_o;
    if (rdata_sync_ps_ack_i) begin
        rdata_sync_ps_ack_toggle_i <= (~rdata_sync_ps_ack_toggle_i);
    end
    read_sync_ps_toggle_o_r <= read_sync_ps_toggle_o;
    if (read_sync_ps_ack_i) begin
        read_sync_ps_ack_toggle_i <= (~read_sync_ps_ack_toggle_i);
    end
    abort_sync_ps_toggle_o_r <= abort_sync_ps_toggle_o;
    if (abort_sync_ps_ack_i) begin
        abort_sync_ps_ack_toggle_i <= (~abort_sync_ps_ack_toggle_i);
    end
    done_sync_ps_toggle_o_r <= done_sync_ps_toggle_o;
    if (done_sync_ps_ack_i) begin
        done_sync_ps_ack_toggle_i <= (~done_sync_ps_ack_toggle_i);
    end
    if (int_available_sync_i) begin
        int_available_sync_blind <= 1'd1;
    end
    if (int_available_sync_ps_ack_o) begin
        int_available_sync_blind <= 1'd0;
    end
    if (int_available_sync_ps_i) begin
        int_available_sync_ps_toggle_i <= (~int_available_sync_ps_toggle_i);
    end
    int_available_sync_ps_ack_toggle_o_r <= int_available_sync_ps_ack_toggle_o;
    if (int_abort_init_sync_i) begin
        int_abort_init_sync_blind <= 1'd1;
    end
    if (int_abort_init_sync_ps_ack_o) begin
        int_abort_init_sync_blind <= 1'd0;
    end
    if (int_abort_init_sync_ps_i) begin
        int_abort_init_sync_ps_toggle_i <= (~int_abort_init_sync_ps_toggle_i);
    end
    int_abort_init_sync_ps_ack_toggle_o_r <= int_abort_init_sync_ps_ack_toggle_o;
    if (int_abort_done_sync_i) begin
        int_abort_done_sync_blind <= 1'd1;
    end
    if (int_abort_done_sync_ps_ack_o) begin
        int_abort_done_sync_blind <= 1'd0;
    end
    if (int_abort_done_sync_ps_i) begin
        int_abort_done_sync_ps_toggle_i <= (~int_abort_done_sync_ps_toggle_i);
    end
    int_abort_done_sync_ps_ack_toggle_o_r <= int_abort_done_sync_ps_ack_toggle_o;
    state <= next_state;
    if (abort_ack_next_value_ce0) begin
        abort_ack <= abort_ack_next_value0;
    end
    if (abort_in_progress_next_value_ce1) begin
        abort_in_progress <= abort_in_progress_next_value1;
    end
    if (sys_rst) begin
        sfr_sr_rx_err <= 1'd0;
        sfr_sr_tx_err <= 1'd0;
        wdata_sync_ps_ack_toggle_i <= 1'd0;
        rdata_sync_ps_ack_toggle_i <= 1'd0;
        read_sync_ps_ack_toggle_i <= 1'd0;
        abort_sync_ps_ack_toggle_i <= 1'd0;
        done_sync_ps_ack_toggle_i <= 1'd0;
        int_available_sync_ps_toggle_i <= 1'd0;
        int_available_sync_blind <= 1'd0;
        int_abort_init_sync_ps_toggle_i <= 1'd0;
        int_abort_init_sync_blind <= 1'd0;
        int_abort_done_sync_ps_toggle_i <= 1'd0;
        int_abort_done_sync_blind <= 1'd0;
        abort_in_progress <= 1'd0;
        abort_ack <= 1'd0;
        w_pending <= 1'd0;
        w_valid_r <= 1'd0;
        ar_done_r <= 1'd0;
        ar_abort_r <= 1'd0;
        sr_read_r <= 1'd0;
        rdata_read_r <= 1'd0;
        state <= 2'd0;
    end
    multiregimpl0_regs0 <= wdata_sync_ps_toggle_i;
    multiregimpl0_regs1 <= multiregimpl0_regs0;
    multiregimpl2_regs0 <= rdata_sync_ps_toggle_i;
    multiregimpl2_regs1 <= multiregimpl2_regs0;
    multiregimpl4_regs0 <= read_sync_ps_toggle_i;
    multiregimpl4_regs1 <= multiregimpl4_regs0;
    multiregimpl6_regs0 <= abort_sync_ps_toggle_i;
    multiregimpl6_regs1 <= multiregimpl6_regs0;
    multiregimpl8_regs0 <= done_sync_ps_toggle_i;
    multiregimpl8_regs1 <= multiregimpl8_regs0;
    multiregimpl11_regs0 <= int_available_sync_ps_ack_toggle_i;
    multiregimpl11_regs1 <= multiregimpl11_regs0;
    multiregimpl13_regs0 <= int_abort_init_sync_ps_ack_toggle_i;
    multiregimpl13_regs1 <= multiregimpl13_regs0;
    multiregimpl15_regs0 <= int_abort_done_sync_ps_ack_toggle_i;
    multiregimpl15_regs1 <= multiregimpl15_regs0;
end


//------------------------------------------------------------------------------
// Specialized Logic
//------------------------------------------------------------------------------

endmodule

// -----------------------------------------------------------------------------
//  Auto-Generated by LiteX on 2023-08-10 18:05:38.
//------------------------------------------------------------------------------
