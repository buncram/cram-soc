// SPDX-FileCopyrightText: 2023 Cramium Labs, Inc.
// SPDX-FileCopyrightText: 2022 Lawrie Griffiths
// SPDX-License-Identifier: BSD-2-Clause

// TODO:
//   - Write simple tests for instructions and modes that have not been covered by anything else (see list below)
//   - Test end-to-end IRQ handling with "actual handler"
//   - Test end-to-end DMA to the block with "actual controller"
//
// Additional unit tests required:
// EXEC-related:
//
// - OUT EXEC should be able to execute an OUT EXEC or OUT PC
// - OUT EXEC of a WAIT instruction should latch the EXEC'd instruction until its stall condition clears
// - EXEC write during a stalled imem-resident instruction will interrupt that instruction and then resume it after the EXEC'd instruction completes (e.g. a WAIT IRQ in imem can be interrupted by a IRQ instruction, which can set the same IRQ and cause the WAIT IRQ to fall through once resuming)
// - EXEC write of a jump during a stalled imem-resident instruction will break out of the stall and begin executing at the jump target
// - A stalled imem-resident WAIT should be replaced by a imem overwrite of that instruction (i.e. fetch is coherent with imem writes and repeatedly fetches during stall)
// - EXEC writes execute even when the SM is disabled via CTRL_SM_ENABLE
//
// FIFOs:
//
// - A program of the form out x, 32; in x, 32 (with autopull + autopush enabled) should permit exactly 10 words to be written to the TX FIFO before the TX FIFO becomes full -- 4 for each FIFO, plus 1 word in the X register and 1 in the OSR.
//
// IOs:
//
// - Side-set still takes place on cycles where the SM is stalled
// - Simultaneous side-set and OUT/SET of the same pin values gives precedence to side-set
// - pin indices > 32 wrap back through pin 0
//
// IRQs:
//
// - Multiple SMs can safely wait for and clear the same IRQ, as long as they have the same clock divisor and their
// dividers are sync'd
//
// Autopush/pull:
//
// - Autopull does not take place while the SM is disabled
// - Autopull will take place when an instruction is EXEC'd, even if the SM is not enabled via CTRL at that point. (The EXEC write forcibly enables the SM for the duration of the EXEC'd instruction).
// - OUT with empty OSR but nonempty TX FIFO should not set the TX stall flag
// - OUT with empty OSR and nonempty TX FIFO experiences a 1-cycle stall as there's no bypass of FIFO through OSR.
// - An EXEC write of any instruction (e.g. nop) to a disabled SM, with empty OSR and nonempty TX FIFO, should perform an autopull
// - An EXEC write of an OUT 32 to a disabled SM, with an empty OSR, and at least two words in the TX FIFO, consumes two words from the TX FIFO: one to fill the OSR so the OUT can execute, and the second to backfill the OSR when the OUT empties it. The latter is required to achieve full 1 OUT/clock throughput
//
// You'll also want to make sure you're covering all of the possible 16-bit opcodes, all combinations of shift direction/count, etc. I also remember the shift counter logic being quite fussy, so possibly a rich seam of bugs to mine there. Hopefully some of that is useful, let me know if you have any questions and I should be able to clarify
//
// Also: https://github.com/raspberrypi/pico-extras

// INTEGRATION:
//   - Ensure that irq0/irq1 are available to system DMA controller for chaining
//   - In general clock ratios bigger than 1:1 (bus clock faster than PIO clock) are not tested and likely invalid.
//   - Ensure that the "regular" GPIO block exists and can read input pins (otherwise there is no simple way to do this)
//   - If the PIO main clk is much slower than pclk, the CPU has to be careful about reading values
//     back that need to be side-effected (e.g. push to FIFO, then check to see level). This is because
//     it can take longer for the flag to update than a single loop of the CPU. However, as long as
//     clk is within 2x of pclk it should be OK. However, this is not the expected corner case, typically
//     we would expect that clk is faster or equal to pclk.
//
// DMA TIMING RESTRICTIONS:
//
// DMA cycles need to drop their request within 2 cycles of
// a write request on APB otherwise it will re-trigger the DMA request:
//
// T0  T1  T2  T3
// --------------
// W   I   I   S
//
// T0 is the last "write" on the APB of the DMA data.
//
// T1 and T2 are internal cycles for the DMAC as it cleans up
// the transfer.
//
// T3 is when it samples the level-sensitive request signal again.
//
// Thus, the APB would assert the write on T0, but the PIO block
// won't sample it until T1. It must drop the request by the end
// of T2, so that on the edge T3 it is not high again.
//
// This is a very tight timing when PCLK:PIO_CLK is equal to 1:1.
// In order to meet this timing, a CDC mode bit is provided in
// the supplemental configuration area, where each channel can have
// its push/pull pulse signals either run through a regular CDC,
// or it can just go through a simple edge-to-pulse converter.
// (see SFR_CDC_MODE)
//
// For ratios slower than 1:4 bus:pio clock, the regular CDC
// can meet timing, and thus any ratio less than 1:4 should
// be acceptable (1:5, 1:6, etc..)
//
// For ratios 1:2 and 1:1, the pulse converter should be used,
// and it is mandatory that the edges are synchronous and aligned
// between these two domains as there is effectively no CDC
// present.
//
// A ratio of 1:3 or any other fractional ratios are not compatible
// with DMA. However, any ratio can be used if DMA is not required.

// To PX: hopefully we can integrate this module with no manual fix-up for pin names, module names etc.
// if you need to make any changes let me know so I can pull them into the original source file!

`ifdef XVLOG // required for compatibility with xsim
`include "template.sv"
`include "apb_sfr_v0.1.sv"
`endif

module rp_pio #(
)(
    input logic         clk,
    input logic         pclk,
    input logic         resetn,
    input logic cmatpg, cmbist,

    ioif.drive          pio_gpio[0:31],
    output logic        irq0,
    output logic        irq1,

    apbif.slavein       apbs,
    apbif.slave         apbx
);
    localparam NUM_MACHINES = 4;

    logic [31:0] gpio_in;
    logic [31:0] gpio_out;
    logic [31:0] gpio_dir;

    // ---- apb -> peripheral wires ----
    wire             reset;
    wire [1:0]       mindex;
    wire [31:0]      din;
    wire [4:0]       index;
    wire [3:0]       action;
    wire [7:0]       irq_force_pulse;
    wire [7:0]       irq_force_level;
    wire             irq_force_action;
    wire [11:0]      irq0_inte;
    wire [11:0]      irq0_intf;
    wire [11:0]      irq1_inte;
    wire [11:0]      irq1_intf;
    wire [31:0]      sync_bypass;
    wire [31:0]      dout;
    wire  [11:0]     irq0_ints;
    wire  [11:0]     irq1_ints;
    wire  [3:0]      tx_empty;
    logic [3:0]      tx_full;
    logic [3:0]      tx_full_margin; // margined full signal for routing to INTs -> DMA; margin is to compensate for DMA sync latency
    logic [3:0]      rx_empty;
    logic [3:0]      rx_empty_margin; // margined empty signal for routing to INTs -> DMA; margin is to compensate for DMA sync latency
    wire  [3:0]      rx_full;

    // ----- peripheral module. Pulled into wrapper level so we can connect to state machine bits directly. ------
    // Shared instructions memory
    reg [15:0]  instr [0:31];

    // Configuration
    reg [NUM_MACHINES-1:0]   en;
    reg [NUM_MACHINES-1:0]   auto_pull;
    reg [NUM_MACHINES-1:0]   auto_push;
    reg [NUM_MACHINES-1:0]   sideset_enable_bit;
    reg [NUM_MACHINES-1:0]   in_shift_dir;
    reg [NUM_MACHINES-1:0]   out_shift_dir;
    reg [NUM_MACHINES-1:0]   status_sel;
    reg [NUM_MACHINES-1:0]   out_sticky;
    reg [NUM_MACHINES-1:0]   inline_out_en;
    reg [NUM_MACHINES-1:0]   side_pindir;
    reg [NUM_MACHINES-1:0]   exec_stalled;

    // Control
    reg [NUM_MACHINES-1:0]   push;
    reg [NUM_MACHINES-1:0]   pull;
    reg [NUM_MACHINES-1:0]   restart;
    reg [NUM_MACHINES-1:0]   clkdiv_restart;
    logic [NUM_MACHINES-1:0] imm;
    logic [NUM_MACHINES-1:0] imm_aligned; // this takes imm_sync and aligns it to the change of instruction state

    // Configuration
    reg [4:0]   pend            [0:NUM_MACHINES-1];
    reg [4:0]   wrap_target     [0:NUM_MACHINES-1];
    reg [15:0]  div_int         [0:NUM_MACHINES-1];
    reg [7:0]   div_frac        [0:NUM_MACHINES-1];
    reg [4:0]   pins_in_base    [0:NUM_MACHINES-1];
    reg [4:0]   pins_out_base   [0:NUM_MACHINES-1];
    reg [4:0]   pins_set_base   [0:NUM_MACHINES-1];
    reg [4:0]   pins_side_base  [0:NUM_MACHINES-1];
    reg [5:0]   pins_out_count  [0:NUM_MACHINES-1];
    reg [2:0]   pins_set_count  [0:NUM_MACHINES-1];
    reg [2:0]   pins_side_count [0:NUM_MACHINES-1];
    reg [4:0]   isr_threshold   [0:NUM_MACHINES-1];
    reg [4:0]   osr_threshold   [0:NUM_MACHINES-1];
    reg [3:0]   status_n        [0:NUM_MACHINES-1];
    reg [4:0]   out_en_sel      [0:NUM_MACHINES-1];
    reg [4:0]   jmp_pin         [0:NUM_MACHINES-1];

    reg [15:0]  curr_instr      [0:NUM_MACHINES-1];
    reg [31:0]  do_sticky       [0:NUM_MACHINES-1];
    logic [15:0]  imm_instr        [0:NUM_MACHINES-1];
    logic [15:0]  imm_instr_sync   [0:NUM_MACHINES-1];

    // Output from machines and fifos
    wire [31:0] output_pins         [0:NUM_MACHINES-1];
    wire [31:0] pin_directions      [0:NUM_MACHINES-1];
    wire [31:0] output_pins_stb     [0:NUM_MACHINES-1];
    wire [4:0]  pc                  [0:NUM_MACHINES-1];
    wire [31:0] mdin                [0:NUM_MACHINES-1];
    wire [31:0] mdout               [0:NUM_MACHINES-1];
    wire [31:0] pdout               [0:NUM_MACHINES-1];
    wire [7:0]  irq_flags_out       [0:NUM_MACHINES-1];
    wire [2:0]  rx_level            [0:NUM_MACHINES-1];
    wire [2:0]  tx_level            [0:NUM_MACHINES-1];
    wire [31:0] fdin                [0:NUM_MACHINES-1];

    logic [NUM_MACHINES-1:0]  mempty;
    logic [NUM_MACHINES-1:0]  mfull;
    wire  [NUM_MACHINES-1:0]  mpush;
    wire  [NUM_MACHINES-1:0]  mpull;
    wire  [NUM_MACHINES-1:0]  dbg_txstall;
    wire  [NUM_MACHINES-1:0]  dbg_rxstall;

    wire [7:0]      irq_flags_stb [0:NUM_MACHINES-1];
    reg [7:0]       irq_flags_stb_r [0:NUM_MACHINES-1];
    wire [7:0]      irq_flags_stb_edge [0:NUM_MACHINES-1];
    reg [7:0]       irq_flags_in;
    wire [7:0]      irq_flags_in_clear;
    wire            do_irq_flags_in_clear;

    wire [11:0]     irq_bundle;
    wire [11:0]     irq0_bank;
    wire [11:0]     irq1_bank;

    assign tx_empty = mempty;
    assign rx_full = mfull;

    wire [31:0] gpio_in_cleaned;
    reg  [31:0] gpio_in_sync0;
    reg  [31:0] gpio_in_sync1;

    // handle GPIO inversions, for better compatibility with existing code
    wire [31:0] oe_invert;
    wire [31:0] out_invert;
    wire [31:0] in_invert;

    // FIFO joining muxes
    logic [NUM_MACHINES-1:0] join_rx;
    logic [NUM_MACHINES-1:0] join_tx;
    logic [1:0] join_rx_tx [0:NUM_MACHINES-1];
    logic [1:0] join_rx_tx_r [0:NUM_MACHINES-1];
    logic [1:0] join_rx_tx_change [0:NUM_MACHINES-1];
    logic [31:0] tx_mux_din  [NUM_MACHINES-1:0];
    logic [31:0] rx_mux_din  [NUM_MACHINES-1:0];
    logic [NUM_MACHINES-1:0] tx_mux_push;
    logic [NUM_MACHINES-1:0] tx_mux_pull;
    logic [NUM_MACHINES-1:0] tx_fifo_empty;
    logic [NUM_MACHINES-1:0] tx_fifo_full;
    logic [NUM_MACHINES-1:0] tx_fifo_empty_margin;
    logic [NUM_MACHINES-1:0] tx_fifo_full_margin;
    logic [NUM_MACHINES-1:0] rx_mux_push;
    logic [NUM_MACHINES-1:0] rx_mux_pull;
    logic [NUM_MACHINES-1:0] rx_fifo_empty;
    logic [NUM_MACHINES-1:0] rx_fifo_full;
    logic [NUM_MACHINES-1:0] rx_fifo_empty_margin;
    logic [NUM_MACHINES-1:0] rx_fifo_full_margin;
    logic [1:0] fifo_tx_margin [0:NUM_MACHINES-1];
    logic [1:0] fifo_rx_margin [0:NUM_MACHINES-1];

    wire ctl_action;

    // debug
    logic [31:0] dbg_sr;
    wire         dbg_trig;
    wire [3:0] txstall;
    wire [3:0] txover;
    wire [3:0] rxunder;
    wire [3:0] rxstall;

    // synchronizers for .ar pulses
    logic ctl_action_sync;
    logic dbg_trig_sync;
    logic do_irq_flags_in_clear_sync;
    logic irq_force_action_sync;
    logic [3:0] push_sync;
    logic [3:0] pull_sync;
    logic [3:0] imm_sync;

    assign reset = !resetn;

    integer i;
    integer gpio_idx;

    generate
        for (genvar gp = 0; gp < 32; gp++) begin: gp_iface
            assign pio_gpio[gp].po = gpio_out[gp] ^ out_invert[gp];
            assign pio_gpio[gp].oe = gpio_dir[gp] ^ oe_invert[gp];
            assign gpio_in[gp] = pio_gpio[gp].pi ^ in_invert[gp];
        end
    endgenerate

    // resolve sticky & enable configurations
    logic [31:0] update_output [0:NUM_MACHINES-1];
    always @* begin
        for(i=0;i<NUM_MACHINES;i=i+1) begin
            for(gpio_idx=0;gpio_idx<32;gpio_idx=gpio_idx+1) begin
                case ({out_sticky[i], inline_out_en[i]})
                    2'b00: begin // only assert when out or set is run
                        update_output[i][gpio_idx] = output_pins_stb[i][gpio_idx];
                    end
                    2'b01: begin // inline out enable only
                        update_output[i][gpio_idx] = output_pins_stb[i][gpio_idx] && output_pins[i][out_en_sel[i]];
                    end
                    2'b10: begin // just sticky
                        update_output[i][gpio_idx] = // assert and continue to assert after the first out strobe
                                        (do_sticky[i][gpio_idx] || output_pins_stb[i][gpio_idx]);
                    end
                    2'b11: begin // sticky and inline enable
                        update_output[i][gpio_idx] = output_pins[i][out_en_sel[i]] // assert only if the designated output pin is set
                                        && (do_sticky[i][gpio_idx] || output_pins_stb[i][gpio_idx]);
                    end
                endcase
            end
        end
    end
    // Synchronous fetch of current instruction for each machine, and output priority resolution
    always @(posedge clk) begin
        for(i=0;i<NUM_MACHINES;i=i+1) begin
            curr_instr[i] <= instr[pc[i]];

            // Coalesce output pins, making sure the highest PIO wins
            for(gpio_idx=0;gpio_idx<32;gpio_idx=gpio_idx+1) begin
                // don't assert an out in sticky mode until an OUT strobe has happened
                if (reset || restart[i]) begin
                    do_sticky[i][gpio_idx] <= 0; // de-assert any sticky OUTs previously set by machine
                end else if (out_sticky[i] && output_pins_stb[i][gpio_idx]) begin
                    do_sticky[i][gpio_idx] <= 1;
                end

                if (reset) begin
                    gpio_out[gpio_idx] <= 0;
                    gpio_dir[gpio_idx] <= 0;
                end else if (update_output[i][gpio_idx]) begin
                    gpio_out[gpio_idx] <= output_pins[i][gpio_idx];
                    gpio_dir[gpio_idx] <= pin_directions[i][gpio_idx];
                end
            end
        end
    end

    // Generate the machines and associated TX and RX fifos
    generate
        genvar j;

        for(j=0;j<NUM_MACHINES;j=j+1) begin : mach
            // this aligns the immediate instruction to the actual "imm" pulse, so the two change
            // on exactly the same clock cycle. Otherwise, the instruction changes before the "imm" pulse
            // arrives, and this can cause a previously stalled "imm" instruction to double-execute the
            // imm command (because the unblocking instruction would present itself to the blocked/waiting
            // state machine before the next imm pulse arrives)
            always @(posedge clk) begin
                imm_aligned[j] <= imm_sync[j];
                if (imm_sync) begin
                    imm_instr_sync[j] <= imm_instr[j];
                end
            end
            pio_machine machine (
                .clk(clk),
                .reset(reset),
                .en(en[j]),
                .restart(restart[j] & ctl_action_sync),
                .clkdiv_restart(clkdiv_restart[j] & ctl_action_sync),
                .mindex(j[1:0]),
                .jmp_pin(jmp_pin[j]),
                .input_pins(gpio_in_cleaned),
                .output_pins(output_pins[j]),
                .output_pins_stb(output_pins_stb[j]),
                .pin_directions(pin_directions[j]),
                .sideset_enable_bit(pins_side_count[j] > 0 ? sideset_enable_bit[j] : 1'b0),
                .side_pindir(side_pindir[j]),
                .in_shift_dir(in_shift_dir[j]),
                .out_shift_dir(out_shift_dir[j]),
                .div_int(div_int[j]),
                .div_frac(div_frac[j]),
                .imm_instr(imm_instr_sync[j]),
                .curr_instr(curr_instr[j]),
                .imm(imm_aligned[j]),
                .pend(pend[j]),
                .exec_stalled(exec_stalled[j]),
                .wrap_target(wrap_target[j]),
                .pins_out_base(pins_out_base[j]),
                .pins_out_count(pins_out_count[j]),
                .pins_set_base(pins_set_base[j]),
                .pins_set_count(pins_set_count[j]),
                .pins_in_base(pins_in_base[j]),
                .pins_side_base(pins_side_base[j]),
                .pins_side_count(pins_side_count[j]),
                .auto_pull(auto_pull[j]),
                .auto_push(auto_push[j]),
                .isr_threshold(isr_threshold[j]),
                .osr_threshold(osr_threshold[j]),
                .irq_flags_in(irq_flags_in),
                .irq_flags_out(irq_flags_out[j]),
                .irq_flags_stb(irq_flags_stb[j]),
                .pc(pc[j]),
                .din(mdin[j]),
                .dout(mdout[j]),
                .pull(mpull[j]),
                .push(mpush[j]),
                .empty(mempty[j]),
                .full(mfull[j]),
                .status_sel(status_sel[j]),
                .status_n(status_n[j]),
                .tx_level(tx_level[j]),
                .rx_level(rx_level[j]),
                .dbg_txstall(dbg_txstall[j]),
                .dbg_rxstall(dbg_rxstall[j])
            );
            // join_tx/join_rx is the method for resetting FIFOs.
            assign join_rx_tx[j] = {join_rx[j], join_tx[j]};
            always @(posedge clk) begin
                join_rx_tx_r[j] <= join_rx_tx[j];
            end
            assign join_rx_tx_change[j] = join_rx_tx_r[j] ^ join_rx_tx[j];
            // FIFO join muxes
            always @* begin
                case ({join_rx[j], join_tx[j]})
                    2'b00: begin
                        tx_mux_din[j] = fdin[j];  // base case tx FIFO input
                        rx_mux_din[j] = mdout[j]; // base case rx FIFO input
                        tx_mux_push[j] = push_sync[j];
                        tx_mux_pull[j] = mpull[j];
                        mempty[j] = tx_fifo_empty[j];
                        tx_full[j] = tx_fifo_full[j];
                        tx_full_margin[j] = tx_fifo_full_margin[j];
                        rx_mux_push[j] = mpush[j];
                        rx_mux_pull[j] = pull_sync[j];
                        mfull[j] = rx_fifo_full[j];
                        rx_empty[j] = rx_fifo_empty[j];
                        rx_empty_margin[j] = rx_fifo_empty_margin[j];
                    end
                    2'b10: begin // join RX case
                        tx_mux_din[j] = mdout[j]; // wire incoming data to the tx fifo
                        rx_mux_din[j] = mdin[j];  // wire rx fifo data input to tx fifo output
                        tx_mux_push[j] = mpush[j];
                        tx_mux_pull[j] = !rx_fifo_full[j] && (tx_level[j] != 0);
                        mempty[j] = 1; // tx fifo is disabled
                        tx_full[j] = 1;
                        tx_full_margin[j] = 1;
                        rx_mux_push[j] = !rx_fifo_full[j] && (tx_level[j] != 0);
                        rx_mux_pull[j] = pull_sync[j];
                        mfull[j] = tx_fifo_full[j]; // only full if the outer fifo (TX fifo) is full
                        rx_empty[j] = rx_fifo_empty[j] && tx_fifo_empty[j]; // empty only when both are empty
                        rx_empty_margin[j] = rx_fifo_empty_margin[j] && tx_fifo_empty[j];
                    end
                    2'b01: begin // join TX case
                        tx_mux_din[j] = pdout[j]; // wire tx fifo data input to rx fifo output
                        rx_mux_din[j] = fdin[j];  // wire incoming data to the rx fifo
                        tx_mux_push[j] = !tx_fifo_full[j] && (rx_level[j] != 0);
                        tx_mux_pull[j] = mpull[j];
                        mempty[j] = rx_fifo_empty[j] && tx_fifo_empty[j]; // empty only when both are empty
                        tx_full[j] = rx_fifo_full[j]; // full only when outer FIFO (RX fifo) is full
                        tx_full_margin[j] = rx_fifo_full_margin[j]; // rx is the "exposed" fifo
                        rx_mux_push[j] = push_sync[j];
                        rx_mux_pull[j] = !tx_fifo_full[j] && (rx_level[j] != 0);
                        mfull[j] = 1;
                        rx_empty[j] = 1;
                        rx_empty_margin[j] = 1;
                    end
                    2'b11: begin // both joined, error condition: both FIFOs are disabled
                        tx_mux_din[j] = fdin[j];
                        rx_mux_din[j] = mdout[j];
                        tx_mux_push[j] = 0;
                        tx_mux_pull[j] = 0;
                        mempty[j] = 1;
                        tx_full[j] = 1;
                        rx_mux_push[j] = 0;
                        rx_mux_pull[j] = 0;
                        mfull[j] = 1;
                        rx_empty[j] = 1;
                        tx_full_margin[j] = 1;
                        rx_empty_margin[j] = 1;
                    end
                endcase
            end

            pio_fifo fifo_tx (
                .clk(clk),
                .reset(reset | (join_rx_tx_change[j] != 0)),
                .push(/*push[j]*/ tx_mux_push[j]),
                .pull(/*mpull[j]*/ tx_mux_pull[j]),
                .din(/*fdin[j]*/ tx_mux_din[j]),
                .dout(mdin[j]),
                .empty(/*mempty[j]*/ tx_fifo_empty[j]),
                .full(/*tx_full[j]*/ tx_fifo_full[j]),
                .margin(join_rx ? fifo_rx_margin[j]: fifo_tx_margin[j]),
                .margin_empty(tx_fifo_empty_margin[j]),
                .margin_full(tx_fifo_full_margin[j]),
                .level(tx_level[j])
            );

            pio_fifo fifo_rx (
                .clk(clk),
                .reset(reset | (join_rx_tx_change[j] != 0)),
                .push(/*mpush[j]*/ rx_mux_push[j]),
                .pull(/*pull[j]*/ rx_mux_pull[j]),
                .din(/*mdout[j]*/ rx_mux_din[j]),
                .dout(pdout[j]),
                .full(/*mfull[j]*/ rx_fifo_full[j]),
                .empty(/*rx_empty[j]*/ rx_fifo_empty[j]),
                .margin_empty(rx_fifo_empty_margin[j]),
                .margin_full(rx_fifo_full_margin[j]),
                .margin(join_tx ? fifo_tx_margin[j] : fifo_rx_margin[j]),
                .level(rx_level[j])
            );

            always @(posedge clk) begin
                if (reset) begin
                    irq_flags_stb_r[j] <= 0;
                end else begin
                    irq_flags_stb_r[j] <= irq_flags_stb[j];
                end
            end
            assign irq_flags_stb_edge[j] = ~irq_flags_stb_r[j] & irq_flags_stb[j];
        end
    endgenerate

    // IRQ state scoreboard
    generate
        genvar k;
        for (k=0; k<8; k=k+1) begin: irq_bits
            always @(posedge clk) begin
                if (reset) begin
                    irq_flags_in[k] <= 0;
                end else if (do_irq_flags_in_clear_sync & irq_flags_in_clear[k]) begin
                    irq_flags_in[k] <= 0;
                end else begin
                    // machine priority order is m0 < m1 < m2 < m3. Datasheet is vague on this
                    // but inferred from SMn_EXECCTRL docs stating this as a precedence order.
                    if (irq_force_pulse[k]) begin
                        irq_flags_in[k] <= 1;
                    end else if (irq_flags_stb_edge[3][k] != 0) begin
                        irq_flags_in[k] <= irq_flags_out[3][k];
                    end else if (irq_flags_stb_edge[2][k] != 0) begin
                        irq_flags_in[k] <= irq_flags_out[2][k];
                    end else if (irq_flags_stb_edge[1][k] != 0) begin
                        irq_flags_in[k] <= irq_flags_out[1][k];
                    end else if (irq_flags_stb_edge[0][k] != 0) begin
                        irq_flags_in[k] <= irq_flags_out[0][k];
                    end
                end
            end
        end
    endgenerate

    // reduce IRQ state to just two bits going to the CPU
    assign irq_bundle = {irq_flags_in[3:0], ~tx_full_margin, ~rx_empty_margin};
    assign irq0_ints = (irq_bundle | irq0_intf) & irq0_inte;
    assign irq1_ints = (irq_bundle | irq1_intf) & irq1_inte;
    assign irq0 = irq0_ints != 0;
    assign irq1 = irq1_ints != 0;

    // add metastability hardening, with optional bypass path
    always @(posedge clk) begin
        gpio_in_sync0 <= gpio_in;
        gpio_in_sync1 <= gpio_in_sync0;
    end
    generate
        genvar m;
        for(m = 0; m < 32; m = m + 1) begin: gen_bypass
            assign gpio_in_cleaned[m] = sync_bypass[m] ? gpio_in[m] : gpio_in_sync1[m];
        end
    endgenerate

    // add debug signals
    bit txstall0, txstall1, txstall2, txstall3;
    `theregfull(clk, resetn, txstall0, '0) <= ((dbg_txstall[0] | txstall0) & !(txstall[0] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, txstall1, '0) <= ((dbg_txstall[1] | txstall1) & !(txstall[1] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, txstall2, '0) <= ((dbg_txstall[2] | txstall2) & !(txstall[2] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, txstall3, '0) <= ((dbg_txstall[3] | txstall3) & !(txstall[3] & dbg_trig_sync)) ? 1 : 0;

    bit txover0, txover1, txover2, txover3;
    `theregfull(clk, resetn, txover0, '0) <= (((tx_full[0] & push_sync[0]) | txover0) & !(txover[0] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, txover1, '0) <= (((tx_full[1] & push_sync[1]) | txover1) & !(txover[1] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, txover2, '0) <= (((tx_full[2] & push_sync[2]) | txover2) & !(txover[2] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, txover3, '0) <= (((tx_full[3] & push_sync[3]) | txover3) & !(txover[3] & dbg_trig_sync)) ? 1 : 0;

    bit rxstall0, rxstall1, rxstall2, rxstall3;
    `theregfull(clk, resetn, rxstall0, '0) <= ((dbg_rxstall[0] | rxstall0) & !(rxstall[0] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, rxstall1, '0) <= ((dbg_rxstall[1] | rxstall1) & !(rxstall[1] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, rxstall2, '0) <= ((dbg_rxstall[2] | rxstall2) & !(rxstall[2] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, rxstall3, '0) <= ((dbg_rxstall[3] | rxstall3) & !(rxstall[3] & dbg_trig_sync)) ? 1 : 0;

    bit rxunder0, rxunder1, rxunder2, rxunder3;
    `theregfull(clk, resetn, rxunder0, '0) <= (((rx_empty[0] & pull_sync[0]) | rxunder0) & !(rxunder[0] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, rxunder1, '0) <= (((rx_empty[1] & pull_sync[1]) | rxunder1) & !(rxunder[1] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, rxunder2, '0) <= (((rx_empty[2] & pull_sync[2]) | rxunder2) & !(rxunder[2] & dbg_trig_sync)) ? 1 : 0;
    `theregfull(clk, resetn, rxunder3, '0) <= (((rx_empty[3] & pull_sync[3]) | rxunder3) & !(rxunder[3] & dbg_trig_sync)) ? 1 : 0;
    assign dbg_sr = {
        4'd0, txstall3, txstall2, txstall1, txstall0,
        4'd0, txover3, txover2, txover1, txover0,
        4'd0, rxunder3, rxunder2, rxunder1, rxunder0,
        4'd0, rxstall3, rxstall2, rxstall1, rxstall0
    };

    assign irq_force_pulse = irq_force_action_sync ? irq_force_level : 8'h0;
    // ---- SFR bank ----
    // logic pclk;
    // assign pclk = clk;
    logic apbrd, apbwr, sfrlock;
    assign sfrlock = '0;

    `apbs_common;
    assign  apbx.prdata = '0 |
            sfr_ctrl         .prdata32 |
            sfr_fstat        .prdata32 |
            sfr_fdebug       .prdata32 |
            sfr_flevel       .prdata32 |
            sfr_txf0         .prdata32 |
            sfr_txf1         .prdata32 |
            sfr_txf2         .prdata32 |
            sfr_txf3         .prdata32 |
            sfr_rxf0         .prdata32 |
            sfr_rxf1         .prdata32 |
            sfr_rxf2         .prdata32 |
            sfr_rxf3         .prdata32 |
            sfr_irq          .prdata32 |
            sfr_irq_force    .prdata32 |
            sfr_sync_bypass  .prdata32 |
            sfr_dbg_padout   .prdata32 |
            sfr_dbg_padoe    .prdata32 |
            sfr_dbg_cfginfo  .prdata32 |
            sfr_instr_mem0   .prdata32 |
            sfr_instr_mem1   .prdata32 |
            sfr_instr_mem2   .prdata32 |
            sfr_instr_mem3   .prdata32 |
            sfr_instr_mem4   .prdata32 |
            sfr_instr_mem5   .prdata32 |
            sfr_instr_mem6   .prdata32 |
            sfr_instr_mem7   .prdata32 |
            sfr_instr_mem8   .prdata32 |
            sfr_instr_mem9   .prdata32 |
            sfr_instr_mem10  .prdata32 |
            sfr_instr_mem11  .prdata32 |
            sfr_instr_mem12  .prdata32 |
            sfr_instr_mem13  .prdata32 |
            sfr_instr_mem14  .prdata32 |
            sfr_instr_mem15  .prdata32 |
            sfr_instr_mem16  .prdata32 |
            sfr_instr_mem17  .prdata32 |
            sfr_instr_mem18  .prdata32 |
            sfr_instr_mem19  .prdata32 |
            sfr_instr_mem20  .prdata32 |
            sfr_instr_mem21  .prdata32 |
            sfr_instr_mem22  .prdata32 |
            sfr_instr_mem23  .prdata32 |
            sfr_instr_mem24  .prdata32 |
            sfr_instr_mem25  .prdata32 |
            sfr_instr_mem26  .prdata32 |
            sfr_instr_mem27  .prdata32 |
            sfr_instr_mem28  .prdata32 |
            sfr_instr_mem29  .prdata32 |
            sfr_instr_mem30  .prdata32 |
            sfr_instr_mem31  .prdata32 |
            sfr_sm0_clkdiv   .prdata32 |
            sfr_sm0_execctrl .prdata32 |
            sfr_sm0_shiftctrl.prdata32 |
            sfr_sm0_addr     .prdata32 |
            sfr_sm0_instr    .prdata32 |
            sfr_sm0_pinctrl  .prdata32 |
            sfr_sm1_clkdiv   .prdata32 |
            sfr_sm1_execctrl .prdata32 |
            sfr_sm1_shiftctrl.prdata32 |
            sfr_sm1_addr     .prdata32 |
            sfr_sm1_instr    .prdata32 |
            sfr_sm1_pinctrl  .prdata32 |
            sfr_sm2_clkdiv   .prdata32 |
            sfr_sm2_execctrl .prdata32 |
            sfr_sm2_shiftctrl.prdata32 |
            sfr_sm2_addr     .prdata32 |
            sfr_sm2_instr    .prdata32 |
            sfr_sm2_pinctrl  .prdata32 |
            sfr_sm3_clkdiv   .prdata32 |
            sfr_sm3_execctrl .prdata32 |
            sfr_sm3_shiftctrl.prdata32 |
            sfr_sm3_addr     .prdata32 |
            sfr_sm3_instr    .prdata32 |
            sfr_sm3_pinctrl  .prdata32 |
            sfr_intr         .prdata32 |
            sfr_irq0_inte    .prdata32 |
            sfr_irq0_intf    .prdata32 |
            sfr_irq0_ints    .prdata32 |
            sfr_irq1_inte    .prdata32 |
            sfr_irq1_intf    .prdata32 |
            sfr_irq1_ints    .prdata32 |
            sfr_io_oe_inv    .prdata32 |
            sfr_io_o_inv     .prdata32 |
            sfr_io_i_inv     .prdata32 |
            sfr_fifo_margin  .prdata32 |
            sfr_zero0        .prdata32 |
            sfr_zero1        .prdata32 |
            sfr_zero2        .prdata32 |
            sfr_zero3        .prdata32
            ;

    bit do_action;

    // documentation clarity fields
    wire [7:0] unused_div [0:NUM_MACHINES-1];
    wire [1:0] resvd_exec [0:NUM_MACHINES-1];
    wire [15:0] resvd_shift [0:NUM_MACHINES-1];
    // docu debug register
    wire [3:0] nc_dbg0;
    wire [3:0] nc_dbg1;
    wire [3:0] nc_dbg2;
    wire [3:0] nc_dbg3;
    // docu interrupt register fields. Kind of a weird combine-then-split we're doing here but whatev...makes the documentation and header files better!
    wire [3:0] intr_sm;
    wire [3:0] intr_txnfull;
    wire [3:0] intr_rxnempty;
    assign intr_sm       = irq_bundle[11:8];
    assign intr_txnfull  = irq_bundle[7:4];
    assign intr_rxnempty = irq_bundle[3:0];
    wire [3:0] irq0_inte_sm;
    wire [3:0] irq0_inte_txnfull;
    wire [3:0] irq0_inte_rxnempty;
    assign irq0_inte[11:8] =  irq0_inte_sm      ;
    assign irq0_inte[7:4]  =  irq0_inte_txnfull ;
    assign irq0_inte[3:0]  =  irq0_inte_rxnempty;
    wire [3:0] irq0_intf_sm;
    wire [3:0] irq0_intf_txnfull;
    wire [3:0] irq0_intf_rxnempty;
    assign irq0_intf[11:8] = irq0_intf_sm      ;
    assign irq0_intf[7:4]  = irq0_intf_txnfull ;
    assign irq0_intf[3:0]  = irq0_intf_rxnempty;
    wire [3:0] irq0_ints_sm;
    wire [3:0] irq0_ints_txnfull;
    wire [3:0] irq0_ints_rxnempty;
    assign irq0_ints_sm       = irq0_ints[11:8];
    assign irq0_ints_txnfull  = irq0_ints[7:4];
    assign irq0_ints_rxnempty = irq0_ints[3:0];
    wire [3:0] irq1_inte_sm;
    wire [3:0] irq1_inte_txnfull;
    wire [3:0] irq1_inte_rxnempty;
    assign irq1_inte[11:8] = irq1_inte_sm      ;
    assign irq1_inte[7:4]  = irq1_inte_txnfull ;
    assign irq1_inte[3:0]  = irq1_inte_rxnempty;
    wire [3:0] irq1_intf_sm;
    wire [3:0] irq1_intf_txnfull;
    wire [3:0] irq1_intf_rxnempty;
    assign irq1_intf[11:8] = irq1_intf_sm      ;
    assign irq1_intf[7:4]  = irq1_intf_txnfull ;
    assign irq1_intf[3:0]  = irq1_intf_rxnempty;
    wire [3:0] irq1_ints_sm;
    wire [3:0] irq1_ints_txnfull;
    wire [3:0] irq1_ints_rxnempty;
    assign irq1_ints_sm       = irq1_ints[11:8];
    assign irq1_ints_txnfull  = irq1_ints[7:4];
    assign irq1_ints_rxnempty = irq1_ints[3:0];

    wire [1:0] fifo_tx_margin3;
    assign fifo_tx_margin[3] = fifo_tx_margin3;
    wire [1:0] fifo_tx_margin2;
    assign fifo_tx_margin[2] = fifo_tx_margin2;
    wire [1:0] fifo_tx_margin1;
    assign fifo_tx_margin[1] = fifo_tx_margin1;
    wire [1:0] fifo_tx_margin0;
    assign fifo_tx_margin[0] = fifo_tx_margin0;

    wire [1:0] fifo_rx_margin3;
    assign fifo_rx_margin[3] = fifo_rx_margin3;
    wire [1:0] fifo_rx_margin2;
    assign fifo_rx_margin[2] = fifo_rx_margin2;
    wire [1:0] fifo_rx_margin1;
    assign fifo_rx_margin[1] = fifo_rx_margin1;
    wire [1:0] fifo_rx_margin0;
    assign fifo_rx_margin[0] = fifo_rx_margin0;

    // nc fields
    wire exec_stalled_ro0;
    wire exec_stalled_ro1;
    wire exec_stalled_ro2;
    wire exec_stalled_ro3;
    wire [3:0] nc_exec_ar;

    apb_acr #(.A('h00), .DW(12))      sfr_ctrl             (.cr({clkdiv_restart, restart, en}), .ar(ctl_action), .prdata32(),.*);
    apb_sr  #(.A('h04), .DW(32))      sfr_fstat            (.sr({4'd0, tx_empty, 4'd0, tx_full, 4'd0, rx_empty, 4'd0, rx_full}), .prdata32(),.*);
    apb_ascr #(.A('h08), .DW(32))     sfr_fdebug           (.cr({nc_dbg0, txstall, nc_dbg1, txover, nc_dbg2, rxunder, nc_dbg3, rxstall}), .sr(dbg_sr), .ar(dbg_trig), .prdata32(),.*);
    apb_sr  #(.A('h0C), .DW(32))      sfr_flevel           (.sr({1'd0, rx_level[3], 1'd0, tx_level[3], 1'd0, rx_level[2], 1'd0, tx_level[2],
                                                            1'd0, rx_level[1], 1'd0, tx_level[1], 1'd0, rx_level[0], 1'd0, tx_level[0]}), .prdata32(),.*);
    apb_acr #(.A('h10), .DW(32))      sfr_txf0             (.cr(fdin[0]), .ar(push[0]), .prdata32(),.*);
    apb_acr #(.A('h14), .DW(32))      sfr_txf1             (.cr(fdin[1]), .ar(push[1]), .prdata32(),.*);
    apb_acr #(.A('h18), .DW(32))      sfr_txf2             (.cr(fdin[2]), .ar(push[2]), .prdata32(),.*);
    apb_acr #(.A('h1C), .DW(32))      sfr_txf3             (.cr(fdin[3]), .ar(push[3]), .prdata32(),.*);
    apb_asr #(.A('h20), .DW(32))      sfr_rxf0             (.sr(pdout[0]), .ar(pull[0]), .prdata32(),.*);
    apb_asr #(.A('h24), .DW(32))      sfr_rxf1             (.sr(pdout[1]), .ar(pull[1]), .prdata32(),.*);
    apb_asr #(.A('h28), .DW(32))      sfr_rxf2             (.sr(pdout[2]), .ar(pull[2]), .prdata32(),.*);
    apb_asr #(.A('h2C), .DW(32))      sfr_rxf3             (.sr(pdout[3]), .ar(pull[3]), .prdata32(),.*);
    apb_ascr #(.A('h30), .DW(8))      sfr_irq              (.cr(irq_flags_in_clear), .sr(irq_flags_in), .ar(do_irq_flags_in_clear), .prdata32(),.*);
    apb_acr #(.A('h34), .DW(8))       sfr_irq_force        (.cr(irq_force_level), .ar(irq_force_action), .prdata32(),.*);
    apb_cr  #(.A('h38), .DW(32))      sfr_sync_bypass      (.cr(sync_bypass), .prdata32(),.*);
    apb_sr  #(.A('h3C), .DW(32))      sfr_dbg_padout       (.sr(gpio_out), .prdata32(),.*);
    apb_sr  #(.A('h40), .DW(32))      sfr_dbg_padoe        (.sr(gpio_dir), .prdata32(),.*);
    apb_sr  #(.A('h44), .DW(32))      sfr_dbg_cfginfo      (.sr({16'd32, 8'd4, 8'd4}), .prdata32(),.*);
    apb_cr  #(.A('h48), .DW(16))      sfr_instr_mem0       (.cr(instr[0 ]), .prdata32(),.*);
    apb_cr  #(.A('h4C), .DW(16))      sfr_instr_mem1       (.cr(instr[1 ]), .prdata32(),.*);
    apb_cr  #(.A('h50), .DW(16))      sfr_instr_mem2       (.cr(instr[2 ]), .prdata32(),.*);
    apb_cr  #(.A('h54), .DW(16))      sfr_instr_mem3       (.cr(instr[3 ]), .prdata32(),.*);
    apb_cr  #(.A('h58), .DW(16))      sfr_instr_mem4       (.cr(instr[4 ]), .prdata32(),.*);
    apb_cr  #(.A('h5C), .DW(16))      sfr_instr_mem5       (.cr(instr[5 ]), .prdata32(),.*);
    apb_cr  #(.A('h60), .DW(16))      sfr_instr_mem6       (.cr(instr[6 ]), .prdata32(),.*);
    apb_cr  #(.A('h64), .DW(16))      sfr_instr_mem7       (.cr(instr[7 ]), .prdata32(),.*);
    apb_cr  #(.A('h68), .DW(16))      sfr_instr_mem8       (.cr(instr[8 ]), .prdata32(),.*);
    apb_cr  #(.A('h6C), .DW(16))      sfr_instr_mem9       (.cr(instr[9 ]), .prdata32(),.*);
    apb_cr  #(.A('h70), .DW(16))      sfr_instr_mem10      (.cr(instr[10]), .prdata32(),.*);
    apb_cr  #(.A('h74), .DW(16))      sfr_instr_mem11      (.cr(instr[11]), .prdata32(),.*);
    apb_cr  #(.A('h78), .DW(16))      sfr_instr_mem12      (.cr(instr[12]), .prdata32(),.*);
    apb_cr  #(.A('h7C), .DW(16))      sfr_instr_mem13      (.cr(instr[13]), .prdata32(),.*);
    apb_cr  #(.A('h80), .DW(16))      sfr_instr_mem14      (.cr(instr[14]), .prdata32(),.*);
    apb_cr  #(.A('h84), .DW(16))      sfr_instr_mem15      (.cr(instr[15]), .prdata32(),.*);
    apb_cr  #(.A('h88), .DW(16))      sfr_instr_mem16      (.cr(instr[16]), .prdata32(),.*);
    apb_cr  #(.A('h8C), .DW(16))      sfr_instr_mem17      (.cr(instr[17]), .prdata32(),.*);
    apb_cr  #(.A('h90), .DW(16))      sfr_instr_mem18      (.cr(instr[18]), .prdata32(),.*);
    apb_cr  #(.A('h94), .DW(16))      sfr_instr_mem19      (.cr(instr[19]), .prdata32(),.*);
    apb_cr  #(.A('h98), .DW(16))      sfr_instr_mem20      (.cr(instr[20]), .prdata32(),.*);
    apb_cr  #(.A('h9C), .DW(16))      sfr_instr_mem21      (.cr(instr[21]), .prdata32(),.*);
    apb_cr  #(.A('hA0), .DW(16))      sfr_instr_mem22      (.cr(instr[22]), .prdata32(),.*);
    apb_cr  #(.A('hA4), .DW(16))      sfr_instr_mem23      (.cr(instr[23]), .prdata32(),.*);
    apb_cr  #(.A('hA8), .DW(16))      sfr_instr_mem24      (.cr(instr[24]), .prdata32(),.*);
    apb_cr  #(.A('hAC), .DW(16))      sfr_instr_mem25      (.cr(instr[25]), .prdata32(),.*);
    apb_cr  #(.A('hB0), .DW(16))      sfr_instr_mem26      (.cr(instr[26]), .prdata32(),.*);
    apb_cr  #(.A('hB4), .DW(16))      sfr_instr_mem27      (.cr(instr[27]), .prdata32(),.*);
    apb_cr  #(.A('hB8), .DW(16))      sfr_instr_mem28      (.cr(instr[28]), .prdata32(),.*);
    apb_cr  #(.A('hBC), .DW(16))      sfr_instr_mem29      (.cr(instr[29]), .prdata32(),.*);
    apb_cr  #(.A('hC0), .DW(16))      sfr_instr_mem30      (.cr(instr[30]), .prdata32(),.*);
    apb_cr  #(.A('hC4), .DW(16))      sfr_instr_mem31      (.cr(instr[31]), .prdata32(),.*);

    apb_cr  #(.A('hC8), .DW(32),
              .IV(32'h00010000))      sfr_sm0_clkdiv       (.cr({div_int[0], div_frac[0], unused_div[0]}), .prdata32(),.*);
    apb_ascr #(.A('hCC), .DW(32),
               .IV(32'h1F000))        sfr_sm0_execctrl     (.cr({
                                                                exec_stalled_ro0, sideset_enable_bit[0],
                                                                side_pindir[0], jmp_pin[0], out_en_sel[0],
                                                                inline_out_en[0], out_sticky[0], pend[0],
                                                                wrap_target[0],
                                                                resvd_exec[0],
                                                                status_sel[0], status_n[0]
                                                                }),
                                                            .sr({
                                                                exec_stalled[0], sideset_enable_bit[0],
                                                                side_pindir[0], jmp_pin[0], out_en_sel[0],
                                                                inline_out_en[0], out_sticky[0], pend[0],
                                                                wrap_target[0],
                                                                resvd_exec[0],
                                                                status_sel[0], status_n[0]
                                                                }),
                                                            .ar(nc_exec_ar[0]), .prdata32(),.*);
    apb_cr  #(.A('hD0), .DW(32),
              .IV(32'hC0000))         sfr_sm0_shiftctrl    (.cr({
                                                                join_rx[0], join_tx[0], osr_threshold[0], isr_threshold[0],
                                                                out_shift_dir[0], in_shift_dir[0], auto_pull[0], auto_push[0],
                                                                resvd_shift[0]
                                                                }), .prdata32(),.*);
    apb_sr  #(.A('hD4), .DW(5))       sfr_sm0_addr         (.sr(pc[0]), .prdata32(),.*);
    apb_ascr #(.A('hD8), .DW(16))     sfr_sm0_instr        (.cr(imm_instr[0]), .sr(curr_instr[0]), .ar(imm[0]), .prdata32(),.*);
    apb_cr  #(.A('hDC), .DW(32),
             .IV(32'h14000000))       sfr_sm0_pinctrl      (.cr({
                                                                pins_side_count[0], pins_set_count[0], pins_out_count[0],
                                                                pins_in_base[0], pins_side_base[0], pins_set_base[0], pins_out_base[0]
                                                                }), .prdata32(),.*);

    apb_cr  #(.A('hE0), .DW(32),
               .IV(32'h00010000))     sfr_sm1_clkdiv       (.cr({div_int[1], div_frac[1], unused_div[1]}), .prdata32(),.*);
    apb_ascr #(.A('hE4), .DW(32),
               .IV(32'h1F000))        sfr_sm1_execctrl     (.cr({
                                                                exec_stalled_ro1, sideset_enable_bit[1],
                                                                side_pindir[1], jmp_pin[1], out_en_sel[1],
                                                                inline_out_en[1], out_sticky[1], pend[1],
                                                                wrap_target[1],
                                                                resvd_exec[1],
                                                                status_sel[1], status_n[1]
                                                                }),
                                                            .sr({
                                                                exec_stalled[1], sideset_enable_bit[1],
                                                                side_pindir[1], jmp_pin[1], out_en_sel[1],
                                                                inline_out_en[1], out_sticky[1], pend[1],
                                                                wrap_target[1],
                                                                resvd_exec[1],
                                                                status_sel[1], status_n[1]
                                                                }),
                                                            .ar(nc_exec_ar[1]), .prdata32(),.*);
    apb_cr  #(.A('hE8), .DW(32),
              .IV(32'hC0000))         sfr_sm1_shiftctrl    (.cr({
                                                                join_rx[1], join_tx[1], osr_threshold[1], isr_threshold[1],
                                                                out_shift_dir[1], in_shift_dir[1], auto_pull[1], auto_push[1],
                                                                resvd_shift[1]
                                                                }), .prdata32(),.*);
    apb_sr  #(.A('hEC), .DW(5))       sfr_sm1_addr         (.sr(pc[1]), .prdata32(),.*);
    apb_ascr #(.A('hF0), .DW(16))     sfr_sm1_instr        (.cr(imm_instr[1]), .sr(curr_instr[1]), .ar(imm[1]), .prdata32(),.*);
    apb_cr  #(.A('hF4), .DW(32),
              .IV(32'h14000000))      sfr_sm1_pinctrl      (.cr({
                                                                pins_side_count[1], pins_set_count[1], pins_out_count[1],
                                                                pins_in_base[1], pins_side_base[1], pins_set_base[1], pins_out_base[1]
                                                                }), .prdata32(),.*);

    apb_cr  #(.A('hF8), .DW(32),
              .IV(32'h00010000))      sfr_sm2_clkdiv       (.cr({div_int[2], div_frac[2], unused_div[2]}), .prdata32(),.*);
    apb_ascr #(.A('hFC), .DW(32),
               .IV(32'h1F000))        sfr_sm2_execctrl     (.cr({
                                                                exec_stalled_ro2, sideset_enable_bit[2],
                                                                side_pindir[2], jmp_pin[2], out_en_sel[2],
                                                                inline_out_en[2], out_sticky[2], pend[2],
                                                                wrap_target[2],
                                                                resvd_exec[2],
                                                                status_sel[2], status_n[2]
                                                                }),
                                                            .sr({
                                                                exec_stalled[2], sideset_enable_bit[2],
                                                                side_pindir[2], jmp_pin[2], out_en_sel[2],
                                                                inline_out_en[2], out_sticky[2], pend[2],
                                                                wrap_target[2],
                                                                resvd_exec[2],
                                                                status_sel[2], status_n[2]
                                                                }),
                                                            .ar(nc_exec_ar[2]), .prdata32(),.*);
    apb_cr  #(.A('h100), .DW(32),
              .IV(32'hC0000))         sfr_sm2_shiftctrl    (.cr({
                                                                join_rx[2], join_tx[2], osr_threshold[2], isr_threshold[2],
                                                                out_shift_dir[2], in_shift_dir[2], auto_pull[2], auto_push[2],
                                                                resvd_shift[2]
                                                                }), .prdata32(),.*);
    apb_sr  #(.A('h104), .DW(5))      sfr_sm2_addr         (.sr(pc[2]), .prdata32(),.*);
    apb_ascr #(.A('h108), .DW(16))    sfr_sm2_instr        (.cr(imm_instr[2]), .sr(curr_instr[2]), .ar(imm[2]), .prdata32(),.*);
    apb_cr  #(.A('h10C), .DW(32),
              .IV(32'h14000000))      sfr_sm2_pinctrl      (.cr({
                                                                pins_side_count[2], pins_set_count[2], pins_out_count[2],
                                                                pins_in_base[2], pins_side_base[2], pins_set_base[2], pins_out_base[2]
                                                                }), .prdata32(),.*);

    apb_cr  #(.A('h110), .DW(32),
               .IV(32'h00010000))     sfr_sm3_clkdiv       (.cr({div_int[3], div_frac[3], unused_div[3]}), .prdata32(),.*);
    apb_ascr #(.A('h114), .DW(32),
               .IV(32'h1F000))        sfr_sm3_execctrl     (.cr({
                                                                exec_stalled_ro3, sideset_enable_bit[3],
                                                                side_pindir[3], jmp_pin[3], out_en_sel[3],
                                                                inline_out_en[3], out_sticky[3], pend[3],
                                                                wrap_target[3],
                                                                resvd_exec[3],
                                                                status_sel[3], status_n[3]
                                                                }),
                                                            .sr({
                                                                exec_stalled[3], sideset_enable_bit[3],
                                                                side_pindir[3], jmp_pin[3], out_en_sel[3],
                                                                inline_out_en[3], out_sticky[3], pend[3],
                                                                wrap_target[3],
                                                                resvd_exec[3],
                                                                status_sel[3], status_n[3]
                                                                }),
                                                            .ar(nc_exec_ar[3]), .prdata32(),.*);
    apb_cr  #(.A('h118), .DW(32),
              .IV(32'hC0000))         sfr_sm3_shiftctrl    (.cr({
                                                                join_rx[3], join_tx[3], osr_threshold[3], isr_threshold[3],
                                                                out_shift_dir[3], in_shift_dir[3], auto_pull[3], auto_push[3],
                                                                resvd_shift[3]
                                                                }), .prdata32(),.*);
    apb_sr  #(.A('h11C), .DW(5))      sfr_sm3_addr         (.sr(pc[3]), .prdata32(),.*);
    apb_ascr #(.A('h120), .DW(16))    sfr_sm3_instr        (.cr(imm_instr[3]), .sr(curr_instr[3]), .ar(imm[3]), .prdata32(),.*);
    apb_cr  #(.A('h124), .DW(32),
              .IV(32'h14000000))      sfr_sm3_pinctrl      (.cr({
                                                                pins_side_count[3], pins_set_count[3], pins_out_count[3],
                                                                pins_in_base[3], pins_side_base[3], pins_set_base[3], pins_out_base[3]
                                                                }), .prdata32(),.*);

    apb_sr #(.A('h128), .DW(12))     sfr_intr             (.sr({intr_sm, intr_txnfull, intr_rxnempty}), .prdata32(),.*);
    apb_cr #(.A('h12C), .DW(12))     sfr_irq0_inte        (.cr({irq0_inte_sm, irq0_inte_txnfull, irq0_inte_rxnempty}), .prdata32(),.*);
    apb_cr #(.A('h130), .DW(12))     sfr_irq0_intf        (.cr({irq0_intf_sm, irq0_intf_txnfull, irq0_intf_rxnempty}), .prdata32(),.*);
    apb_sr #(.A('h134), .DW(12))     sfr_irq0_ints        (.sr({irq0_ints_sm, irq0_ints_txnfull, irq0_ints_rxnempty}), .prdata32(),.*);
    apb_cr #(.A('h138), .DW(12))     sfr_irq1_inte        (.cr({irq1_inte_sm, irq1_inte_txnfull, irq1_inte_rxnempty}), .prdata32(),.*);
    apb_cr #(.A('h13C), .DW(12))     sfr_irq1_intf        (.cr({irq1_intf_sm, irq1_intf_txnfull, irq1_intf_rxnempty}), .prdata32(),.*);
    apb_sr #(.A('h140), .DW(12))     sfr_irq1_ints        (.sr({irq1_ints_sm, irq1_ints_txnfull, irq1_ints_rxnempty}), .prdata32(),.*);

    // leave some registers unused as a "buffer" for forward compatibility

    // implement GPIO inversions within this block
    apb_cr #(.A('h180), .DW(32))     sfr_io_oe_inv        (.cr(oe_invert), .prdata32(),.*);
    apb_cr #(.A('h184), .DW(32))     sfr_io_o_inv         (.cr(out_invert), .prdata32(),.*);
    apb_cr #(.A('h188), .DW(32))     sfr_io_i_inv         (.cr(in_invert), .prdata32(),.*);
    apb_cr #(.A('h18C), .DW(16))     sfr_fifo_margin      (.cr({
                                                            fifo_rx_margin3, fifo_tx_margin3,
                                                            fifo_rx_margin2, fifo_tx_margin2,
                                                            fifo_rx_margin1, fifo_tx_margin1,
                                                            fifo_rx_margin0, fifo_tx_margin0}), .prdata32(),.*);
    apb_sr #(.A('h190), .DW(32))     sfr_zero0            (.sr(32'h0), .prdata32(),.*);  // bank of "zero reads" as DMA source target for initializing RAM
    apb_sr #(.A('h194), .DW(32))     sfr_zero1            (.sr(32'h0), .prdata32(),.*);
    apb_sr #(.A('h198), .DW(32))     sfr_zero2            (.sr(32'h0), .prdata32(),.*);
    apb_sr #(.A('h19C), .DW(32))     sfr_zero3            (.sr(32'h0), .prdata32(),.*);

    cdc_blinded       ctl_action_cdc   (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(ctl_action            ), .out_b(ctl_action_sync            ));
    cdc_blinded       dbg_trig_cdc     (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(dbg_trig              ), .out_b(dbg_trig_sync              ));
    cdc_blinded       irq_flags_cdc    (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(do_irq_flags_in_clear ), .out_b(do_irq_flags_in_clear_sync ));
    cdc_blinded       irq_force_cdc    (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(irq_force_action      ), .out_b(irq_force_action_sync      ));
    cdc_blinded       push_cdc[3:0]    (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(push                  ), .out_b(push_sync                  ));
    cdc_blinded       pull_cdc[3:0]    (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(pull                  ), .out_b(pull_sync                  ));
    cdc_blinded       imm_cdc[3:0]     (.reset(!resetn), .clk_a(pclk), .clk_b(clk), .in_a(imm                   ), .out_b(imm_sync                   ));
endmodule

// action + control register. Any write to this register will cause a pulse that
// can trigger an action, while also updating the value of the register
module apb_acr
#(
      parameter A=0,
      parameter AW=12,
      parameter DW=16,
      parameter IV=32'h0,
      parameter SFRCNT=1,
//      parameter SRMASK=32'h0,               // set write 1 to clr ( for status reg )
      parameter RMASK=32'hffff_ffff        // read mask to remove undefined bit
//      parameter REXTMASK=32'h0              // read ext mask
)(
        input  logic                          pclk        ,
        input  logic                          resetn      ,
        apbif.slavein                         apbs        ,
        input  bit                          sfrlock     ,
//        input  bit   [AW-1:0]               sfrpaddr    ,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrprdataext,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrsr       ,
        output logic [31:0]                 prdata32    ,
        output logic [0:SFRCNT-1][DW-1:0]   cr          ,
        output bit                          ar
);


    logic[DW-1:0] prdata;
    assign prdata32 = prdata | 32'h0;

    apb_sfr2 #(
            .AW          ( AW            ),
            .DW          ( DW            ),
            .IV          ( IV            ),
            .SFRCNT      ( SFRCNT        ),
            .RMASK       ( RMASK         ),      // read mask to remove undefined bit
            .FRMASK      ( 32'h0         ),      // set write 1 to clr ( for status reg )
            .SRMASK      ( 32'h0         )       // read ext mask
         )apb_sfr(
            .pclk        (pclk           ),
            .resetn      (resetn         ),
            .apbslave    (apbs           ),
            .sfrlock     (sfrlock        ),
            .sfrpaddr    (A[AW-1:0]      ),
            .sfrsr       ('0             ),
            .sfrfr       ('0             ),
            .sfrprdata   (prdata         ),
            .sfrdata     (cr             )
         );

    logic sfrapbwr;
    apb_sfrop2 #(
            .AW          ( AW            )
         )apb_sfrop(
            .apbslave    (apbs           ),
            .sfrlock     (sfrlock        ),
            .sfrpaddr    (A[AW-1:0]      ),
            .apbrd       (               ),
            .apbwr       (sfrapbwr       )
         );
    `theregfull(pclk, resetn, ar, '0) <= sfrapbwr;
endmodule

// action + status register. Any read to this register will cause a pulse that
// can trigger an action.
module apb_asr
#(
      parameter A=0,
      parameter AW=12,
      parameter DW=16,
//      parameter IV=32'h0,                   // useless
      parameter SFRCNT=1,
//      parameter SRMASK=32'h0,               // set write 1 to clr ( for status reg )
      parameter RMASK=32'hffff_ffff,        // read mask to remove undefined bit
      parameter SRMASK=32'hffff_ffff              // read ext mask
)(
        input  logic                          pclk        ,
        input  logic                          resetn      ,
        apbif.slavein                         apbs        ,
        input  bit                          sfrlock     ,
//        input  bit   [AW-1:0]               sfrpaddr    ,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrprdataext,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrsr       ,
        output logic [31:0]                 prdata32    ,
        input  logic [0:SFRCNT-1][DW-1:0]   sr          ,
        output bit                          ar
);


    logic[DW-1:0] prdata;
    assign prdata32 = prdata | 32'h0;

    apb_sfr2 #(
            .AW          ( AW            ),
            .DW          ( DW            ),
            .IV          ( '0            ),
            .SFRCNT      ( SFRCNT        ),
            .RMASK       ( RMASK         ),      // read mask to remove undefined bit
            .FRMASK      ( 32'h0         ),      // set write 1 to clr ( for status reg )
            .SRMASK      ( SRMASK        )       // read ext mask
         )apb_sfr(
            .pclk        (pclk           ),
            .resetn      (resetn         ),
            .apbslave    (apbs           ),
            .sfrlock     (sfrlock        ),
            .sfrpaddr    (A[AW-1:0]      ),
            .sfrsr       (sr             ),
            .sfrfr       ('0             ),
            .sfrprdata   (prdata         ),
            .sfrdata     (               )
         );

    logic sfrapbrd;
    apb_sfrop2 #(
            .AW          ( AW            )
         )apb_sfrop(
            .apbslave    (apbs           ),
            .sfrlock     (sfrlock        ),
            .sfrpaddr    (A[AW-1:0]      ),
            .apbrd       (sfrapbrd       ),
            .apbwr       (               )
         );
    `theregfull(pclk, resetn, ar, '0) <= sfrapbrd;
endmodule

// action + control register with status readback. Any write to this register
// will cause a pulse that can trigger an action, while also updating the cr value of
// the register. The status value returned is not related to the cr value.
module apb_ascr
#(
      parameter A=0,
      parameter AW=12,
      parameter DW=16,
      parameter IV=32'h0,
      parameter SFRCNT=1,
//      parameter SRMASK=32'h0,               // set write 1 to clr ( for status reg )
      parameter RMASK=32'hffff_ffff        // read mask to remove undefined bit
//      parameter REXTMASK=32'h0              // read ext mask
)(
        input  logic                          pclk        ,
        input  logic                          resetn      ,
        apbif.slavein                         apbs        ,
        input  bit                          sfrlock     ,
//        input  bit   [AW-1:0]               sfrpaddr    ,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrprdataext,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrsr       ,
        output logic [31:0]                 prdata32    ,
        output logic [0:SFRCNT-1][DW-1:0]   cr          ,
        input  logic [0:SFRCNT-1][DW-1:0]   sr          ,
        output bit                          ar
);


    logic[DW-1:0] prdata;
    assign prdata32 = prdata | 32'h0;

    apb_sfr2 #(
            .AW          ( AW            ),
            .DW          ( DW            ),
            .IV          ( IV            ),
            .SFRCNT      ( SFRCNT        ),
            .RMASK       ( RMASK         ),      // read mask to remove undefined bit
            .FRMASK      ( 32'h0         ),      // set write 1 to clr ( for status reg )
            .SRMASK      ( 32'hFFFF_FFFF )       // read ext mask
         )apb_sfr(
            .pclk        (pclk           ),
            .resetn      (resetn         ),
            .apbslave    (apbs           ),
            .sfrlock     (sfrlock        ),
            .sfrpaddr    (A[AW-1:0]      ),
            .sfrsr       (sr             ),
            .sfrfr       ('0             ),
            .sfrprdata   (prdata         ),
            .sfrdata     (cr             )
         );

    logic sfrapbwr;
    apb_sfrop2 #(
            .AW          ( AW            )
         )apb_sfrop(
            .apbslave    (apbs           ),
            .sfrlock     (sfrlock        ),
            .sfrpaddr    (A[AW-1:0]      ),
            .apbrd       (               ),
            .apbwr       (sfrapbwr       )
         );
    `theregfull(pclk, resetn, ar, '0) <= sfrapbwr;
endmodule

// synchronizer from common/synccell_v0.1.sv
/*
module syncpulse(
    input  bit clka,
    input  bit clkb,
    input  bit pin,
    output bit pout
    );

    bit ptoga, ptogb, ptogbreg;

    always@(posedge clka) ptoga <= ptoga ^ pin;

    syncbit u1(
        .clk(clkb),
        .sin(ptoga),
        .sout(ptogb)
        );

    always@(posedge clkb) ptogbreg <= ptogb;
    assign pout = ptogbreg ^ ptogb;

endmodule

module syncbit#(
    parameter RC = 2
)(
    input  bit clk,
    input  bit sin,
    output bit sout
    );

    bit [RC-1:0] sreg;

    always@(posedge clk) sreg <= { sreg, sin };
    assign sout = sreg[RC-1];

endmodule
*/