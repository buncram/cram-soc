`ifdef XVLOG // required for compatibility with xsim
`include "template.sv"
`include "apb_sfr_v0.1.sv"
`endif


module bio #(
)(
    input logic         aclk,
    input logic         pclk,
    input logic         resetn,
    input logic         cmatpg, cmbist,

    ioif.drive          bio_gpio[0:31],
    output logic        irq[3:0],

    apbif.slavein       apbs,
    apbif.slave         apbx
);
    localparam NUM_MACH = 4;
    localparam MEM_SIZE_BYTES = 0x1000;
    localparam MEM_SIZE_WORDS = 0x1000 / 4;
    localparam MEM_ADDR_BITS = $clog2(MEM_SIZE_WORDS);

    /////////////////////// module glue
    logic reset;

    /////////////////////// GPIO hookup
    logic [31:0] gpio_in;
    logic [31:0] gpio_out;
    logic [31:0] gpio_dir;
    integer i;
    integer gpio_idx;
    logic [31:0] oe_invert;
    logic [31:0] out_invert;
    logic [31:0] in_invert;
    logic [31:0] gpio_in_cleaned;
    logic [31:0] gpio_in_sync0;
    logic [31:0] gpio_in_sync1;

    generate
        for (genvar gp = 0; gp < 32; gp++) begin: gp_iface
            assign bio_gpio[gp].po = gpio_out[gp] ^ out_invert[gp];
            assign bio_gpio[gp].oe = gpio_dir[gp] ^ oe_invert[gp];
            assign gpio_in[gp] = bio_gpio[gp].pi ^ in_invert[gp];
        end
    endgenerate
    // add metastability hardening, with optional bypass path
    always @(posedge aclk) begin
        gpio_in_sync0 <= gpio_in;
        gpio_in_sync1 <= gpio_in_sync0;
    end
    generate
        genvar m;
        for(m = 0; m < 32; m = m + 1) begin: gen_bypass
            assign gpio_in_cleaned[m] = sync_bypass[m] ? gpio_in[m] : gpio_in_sync1[m];
        end
    endgenerate

    /////////////////////// machine hookup
    logic [15:0]  div_int      [NUM_MACH];
    logic [7:0]   div_frac     [NUM_MACH];
    logic [7:0]  unused_div    [NUM_MACH];
    logic [NUM_MACH-1:0]       clkdiv_restart;
    logic [NUM_MACH-1:0]       restart;
    logic [NUM_MACH-1:0]       penable;
    logic [NUM_MACH-1:0]       core_ena;

    logic [NUM_MACH-1:0]       core_clk;
    logic [NUM_MACH-1:0]       stall;
    logic [NUM_MACH-1:0]       trap;

    // Memory interfaces
	// logic [NUM_MACH-1:0]       mem_valid;
	// logic [NUM_MACH-1:0]       mem_instr;
	logic [NUM_MACH-1:0]       mem_ready;

	// logic [31:0] mem_addr   [NUM_MACH];
	// logic [31:0] mem_wdata  [NUM_MACH];
	// logic [ 3:0] mem_wstrb  [NUM_MACH];
	logic [31:0] mem_rdata      [NUM_MACH];

	// Look-Ahead Interface
	logic [NUM_MACH-1:0]    mem_la_read;
	logic [NUM_MACH-1:0]    mem_la_write;
	logic [31:0] mem_la_addr      [NUM_MACH];
	logic [31:0] mem_la_wdata [NUM_MACH];
	logic [ 3:0] mem_la_wstrb [NUM_MACH];

    // high register interfaces
    logic [31:0] mach_regfifo_rdata [NUM_MACH];
    logic [NUM_MACH-1:0] mach_regfifo_rd;
    logic [31:0] mach_regfifo_wdata [NUM_MACH];
    logic [NUM_MACH-1:0] mach_regfifo_wr;

    logic [NUM_MACH-1:0] quanta_wr;

    logic [31:0] gpio_set  [NUM_MACH];
    logic [31:0] gpio_clr  [NUM_MACH];
    logic [31:0] gpdir_set [NUM_MACH];
    logic [31:0] gpdir_clr [NUM_MACH];
    logic [NUM_MACH-1:0] gpio_set_valid;
    logic [NUM_MACH-1:0] gpio_clr_valid;
    logic [NUM_MACH-1:0] gpdir_set_valid;
    logic [NUM_MACH-1:0] gpbir_clr_valid;

    // modes
    logic [NUM_MACH-1:0]   en;
    logic [NUM_MACH-1:0]   en_sync; // enable has to be synchronized to the ar bit so it hits when clkdivreset, restart hits
    logic                  imem_wr_mode;

    /////////////////////// fifo hookup
    // the signals of the FIFO itself. Note, 4 FIFO regs, 4 machines is just a coincidence. The two
    // are not necessarily the same (machine count is parameterized, FIFO regs is *always* 4).
    logic [31:0]  regfifo_wdata [4];
    logic [3:0]   regfifo_we;
    logic [3:0]   regfifo_writable;
    logic [31:0]  regfifo_rdata [4];
    logic [3:0]   regfifo_re;
    logic [3:0]   regfifo_re;
    logic [3:0]   regfifo_readable;
    logic [3:0]   regfifo_level [4];
    // the readable/writable signals, but selected according to a given machine's target FIFO
    logic [3:0] mach_regfifo_readable;
    logic [3:0] mach_regfifo_writable;

    logic [31:0] host_regfifo_wdata [4];
    logic [3:0]  host_regfifo_we;
    logic [31:0] host_regfifo_rdata [4];
    logic [3:0]  host_regfifo_re;

    logic [3:0] pclk_fifo_event_level[8];
    logic [3:0] host_fifo_event_level[8];
    logic [7:0] host_fifo_event_eq_mask;
    logic [7:0] host_fifo_event_lt_mask;
    logic [7:0] host_fifo_event_gt_mask;
    logic [7:0] pclk_fifo_event_eq_mask;
    logic [7:0] pclk_fifo_event_lt_mask;
    logic [7:0] pclk_fifo_event_gt_mask;

    logic [31:0] fdin       [NUM_MACH];
    logic [31:0] fdin_sync  [NUM_MACH];
    logic [31:0] fdout      [NUM_MACH];
    logic [3:0] push;
    logic [3:0] pull;

    /////////////////////// register bank
    // synchronizers for .ar pulses
    logic ctl_action_sync;
    logic [3:0] push_sync;
    logic [3:0] pull_sync;
    // nc fields
    wire ctl_action_sync_ack;
    logic [31:0] host_mem_rdata_pclk;
    logic [3:0]  pclk_regfifo_level [4];
    always_ff @(posedge pclk) begin
        pclk_regfifo_level <= regfifo_level;
        fdout <= regfifo_rdata;
    end
    always_ff @(posedge aclk) begin
        fdin_sync <= fdin;
        host_fifo_event_level <= pclk_fifo_event_level;
        host_fifo_event_eq_mask <= pclk_fifo_event_eq_mask;
        host_fifo_event_lt_mask <= pclk_fifo_event_lt_mask;
        host_fifo_event_gt_mask <= pclk_fifo_event_gt_mask;
    end

    `apbs_common;
    assign  apbx.prdata = '0 |
            sfr_ctrl         .prdata32 |
            sfr_flevel       .prdata32 |
            sfr_txf0         .prdata32 |
            sfr_txf1         .prdata32 |
            sfr_txf2         .prdata32 |
            sfr_txf3         .prdata32 |
            sfr_rxf0         .prdata32 |
            sfr_rxf1         .prdata32 |
            sfr_rxf2         .prdata32 |
            sfr_rxf3         .prdata32 |
            sfr_elevel0      .prdata32 |
            sfr_elevel1      .prdata32 |
            sfr_etype        .prdata32 |
            sfr_qdiv0        .prdata32 |
            sfr_qdiv1        .prdata32 |
            sfr_qdiv2        .prdata32 |
            sfr_qdiv3        .prdata32 |
            sfr_sync_bypass  .prdata32 |
            sfr_io_oe_inv    .prdata32 |
            sfr_io_o_inv     .prdata32 |
            sfr_io_i_inv     .prdata32 |
            host_mem_rdata_pclk
            ;

    apb_ac2r #(.A('h00), .DW(12))    sfr_ctrl             (.cr({clkdiv_restart, restart, en}), .ar(ctl_action), .self_clear(ctl_action_sync_ack), .prdata32(),.*);

    apb_sr  #(.A('h0C), .DW(16))     sfr_flevel           (.sr({
                                                            pclk_regfifo_level[3], pclk_regfifo_level[2],
                                                            pclk_regfifo_level[1], pclk_regfifo_level[0]}), .prdata32(),.*);
    apb_acr #(.A('h10), .DW(32))     sfr_txf0             (.cr(fdin[0]), .ar(push[0]), .prdata32(),.*);
    apb_acr #(.A('h14), .DW(32))     sfr_txf1             (.cr(fdin[1]), .ar(push[1]), .prdata32(),.*);
    apb_acr #(.A('h18), .DW(32))     sfr_txf2             (.cr(fdin[2]), .ar(push[2]), .prdata32(),.*);
    apb_acr #(.A('h1C), .DW(32))     sfr_txf3             (.cr(fdin[3]), .ar(push[3]), .prdata32(),.*);
    apb_asr #(.A('h20), .DW(32))     sfr_rxf0             (.sr(fdout[0]), .ar(pull[0]), .prdata32(),.*);
    apb_asr #(.A('h24), .DW(32))     sfr_rxf1             (.sr(fdout[1]), .ar(pull[1]), .prdata32(),.*);
    apb_asr #(.A('h28), .DW(32))     sfr_rxf2             (.sr(fdout[2]), .ar(pull[2]), .prdata32(),.*);
    apb_asr #(.A('h2C), .DW(32))     sfr_rxf3             (.sr(fdout[3]), .ar(pull[3]), .prdata32(),.*);

    apb_sr  #(.A('h30), .DW(32))     sfr_elevel0          (.sr({
                                                            pclk_fifo_event_level[3], pclk_fifo_event_level[2],
                                                            pclk_fifo_event_level[1], pclk_fifo_event_level[0],}), .prdata32(),.*);
    apb_sr  #(.A('h34), .DW(32))     sfr_elevel1          (.sr({
                                                            pclk_fifo_event_level[7], pclk_fifo_event_level[6],
                                                            pclk_fifo_event_level[5], pclk_fifo_event_level[4],}), .prdata32(),.*);
    apb_sr  #(.A('h38), .DW(24))     sfr_etype            (.sr({
                                                            pclk_fifo_event_gt_mask, pclk_fifo_event_eq_mask,
                                                            pclk_fifo_event_lt_mask}), .prdata32(),.*);

    apb_cr #(.A('h40), .DW(32))      sfr_qdiv0            (.cr({div_int[0], div_frac[0], unused_div[0]}), .prdata32(),.*);
    apb_cr #(.A('h50), .DW(32))      sfr_qdiv1            (.cr({div_int[1], div_frac[1], unused_div[1]}), .prdata32(),.*);
    apb_cr #(.A('h60), .DW(32))      sfr_qdiv2            (.cr({div_int[2], div_frac[2], unused_div[2]}), .prdata32(),.*);
    apb_cr #(.A('h70), .DW(32))      sfr_qdiv3            (.cr({div_int[3], div_frac[3], unused_div[3]}), .prdata32(),.*);

    apb_cr #(.A('h80), .DW(32))      sfr_sync_bypass      (.cr(sync_bypass), .prdata32(),.*);

    apb_cr #(.A('h180), .DW(32))     sfr_io_oe_inv        (.cr(oe_invert), .prdata32(),.*);
    apb_cr #(.A('h184), .DW(32))     sfr_io_o_inv         (.cr(out_invert), .prdata32(),.*);
    apb_cr #(.A('h188), .DW(32))     sfr_io_i_inv         (.cr(in_invert), .prdata32(),.*);

    cdc_blinded       ctl_action_cdc     (.reset(!resetn), .clk_a(pclk), .clk_b(aclk), .in_a(ctl_action            ), .out_b(ctl_action_sync            ));
    cdc_blinded       ctl_action_ack_cdc (.reset(!resetn), .clk_a(aclk), .clk_b(pclk), .in_a(ctl_action_sync       ), .out_b(ctl_action_sync_ack        ));
    cdc_blinded       push_cdc[3:0]    (.reset(!resetn), .clk_a(pclk), .clk_b(aclk), .in_a(push                  ), .out_b(push_sync                  ));
    cdc_blinded       pull_cdc[3:0]    (.reset(!resetn), .clk_a(pclk), .clk_b(aclk), .in_a(pull                  ), .out_b(pull_sync                  ));

    /////////////////////// machine instantiation & instruction memory
    assign reset = ~resetn;

    logic ram_wr_en;
    logic [3:0] ram_wr_mask;
    logic [MEM_ADDR_BITS-1:0] ram_wr_addr;
    logic [31:0] ram_wr_data;
    logic [31:0] host_mem_wdata_sync[2];
    logic [31:0] host_mem_wdata;
    logic host_mem_wr_stb;
    logic host_mem_wr;
    logic host_mem_wr_d;
    logic host_mem_rd_stb;
    logic host_mem_rd;
    logic host_mem_rd_d;
    logic [1:0] psel_sync;
    logic [1:0] pwrite_sync;
    logic [1:0] penable_sync;
    logic [MEM_ADDR_BITS-1:0] host_mem_addr_sync[2];
    logic [MEM_ADDR_BITS-1:0] host_mem_addr;
    logic [31:0] host_mem_rdata_capture;
    logic [31:0] host_mem_rdata;

    logic [31:0] ram_rd_data [NUM_MACH];
    logic [NUM_MACH-1:0] ram_rd_enable;

    assign imem_wr_mode = en_sync[0]; // when machine 0 is disabled, the host can use its port to read/write
    // mux on port 0 of imem between host and machines
    always_comb begin
        host_mem_wr_d = psel_sync[1] & pwrite_sync[1] & penable_sync[1];
        host_mem_rd_d = psel_sync[1] & !pwrite_sync[1];

        if (imem_wr_mode) begin
            // machine0 is running
            ram_wr_en = mem_la_write[0];
            ram_wr_mask = mem_la_wstrb[0];
            ram_wr_data = ram_la_wdata[0];
            ram_wr_addr = ram_la_addr[0][MEM_ADDR_BITS-1:0];

            ram_rd_addr = mem_la_addr;
            ram_rd_enable = mem_la_read;
        end else begin
            // host can write & read
            ram_wr_en = host_mem_wr_stb;
            ram_wr_mask = 4'b1111; // only full 32-bit writes are allowed, anything else is undefined
            ram_wr_data = host_mem_wdata;
            ram_wr_addr = host_mem_addr;

            ram_rd_addr[0] = host_mem_addr;
            ram_rd_addr[1:3] = mem_la_addr[1:3];
            ram_rd_enable = host_mem_rd_d;
            ram_rd_enable[1:3] = mem_la_read[1:3];
        end
    end
    // apb<->imem
    always_ff @(posedge aclk) begin
        // cdc syncs
        psel_sync[0] <= apbs.psel;
        psel_sync[1] <= psel_sync[0];
        pwrite_sync[0] <= apbs.pwrite;
        pwrite_sync[1] <= pwrite_sync[0];
        penable_sync[0] <= apbs.penable;
        penable_sync[1] <= penable_sync[0];
        host_mem_wdata_sync[0] <= apbs.pwdata;
        host_mem_wdata_sync[1] <= host_mem_wdata_sync[0];
        host_mem_addr_sync[0] <= apbs.paddr[MEM_ADDR_BITS-1:0];
        host_mem_addr_sync[1] <= host_mem_addr_sync[0];

        host_mem_wr <= host_mem_wr_d;
        host_mem_wr_stb <= host_mem_wr_d & !host_mem_wr;
        host_mem_addr <= host_mem_wr_d & !host_mem_wr ? host_mem_addr_sync[1] : host_mem_addr;
        host_mem_wdata <= host_mem_wr_d & !host_mem_wr ? host_mem_wdata_sync[1] : host_mem_wdata;

        host_mem_rd <= host_mem_rd_d;
        host_mem_rd_stb <= host_mem_rd_d & !host_mem_rd;

        host_mem_rdata_capture <= host_mem_rd_d & !host_mem_rd ? ram_rd_data : host_mem_rdata_capture;
    end
    // convert rdata to pclk domain
    always_ff @(posedge pclk) begin
        host_mem_rdata <= host_mem_rdata_capture;
    end
    // only assert when selected
    always_comb begin
        host_mem_rdata_pclk = apbs.psel & apbs.penable & ~apbs.pwrite ? host_mem_rdata : '0;
    end
    // imem
    Ram_1w_4rs #(
        .wordCount(MEM_SIZE_WORDS),
        .wordWidth(32),
        .clockCrossing(1'b0),
        .technology("auto"),
        .readUnderWrite("dontCare"),
        .wrAddressWidth(MEM_ADDR_BITS),
        .wrDataWidth(32),
        .wrMaskWidth(4),
        .wrMaskEnable(1),
        .rdAddressWidth(MEM_ADDR_BITS),
        .rdDataWidth(32),
        .ramname("RAM_QP_1024_32")
    ) imem (
        .wr_clk(aclk),
        .wr_en(ram_wr_en),
        .wr_addr(ram_wr_addr),
        .wr_mask(ram_wr_mask),
        .wr_data(ram_wr_data),
        .rd_clk(aclk),
        .rd_en(ram_rd_enable),
        .rd_addr(ram_rd_addr),
        .rd_data(ram_rd_data),
        .CMBIST(0),
        .CMATPG(0),
    );
    // machine stall signal
    always_comb begin
        generate
            genvar j;
            for(j=0; j<NUM_MACH; j=j+1) begin: stalls
                stall[j] = (
                    quanta_wr[j] & ~penable[j]                         // stall to next quanta
                    | mach_regfifo_rd[j] & !mach_regfifo_readable[j]   // FIFO read but empty
                    | mach_regfifo_wr[j] & !mach_regfifo_writable[j]   // FIFO write but full
                    | stalling_for_event[j]                            // event stall
                ) && en_sync[j];                                       // overall machine enable
            end
        endgenerate
    end

    /////////////////////// fifo hookup
    // fifo -> machine back propagate
    generate
        genvar j;
        for(j=0; j<NUM_MACH; j=j+1) begin: fifo_to_mach
            priority_demux #(
                .DATAW(1),
                .LEVELS(4)
            ) select_readable (
                .stb(mach_regfifo_rd), // 4-bit wide, 1-hot encoding of which fifo to target
                .input(regfifo_readable),
                .output(mach_regfifo_readable[j]), // 1-bit wire, per-machine
            );
            priority_demux #(
                .DATAW(32),
                .LEVELS(4)
            ) select_rdata (
                .stb(mach_regfifo_rd), // 4-bit wide, 1-hot encoding of which fifo to target
                .input(regfifo_rdata), // 4x 32 wide bus
                .output(mach_regfifo_rdata[j]), // 32 wide bus
            );
            priority_demux #(
                .DATAW(1),
                .LEVELS(4)
            ) select_writable (
                .stb(mach_regfifo_wr), // 4-bit wide, 1-hot encoding of which fifo to target
                .input(regfifo_writable),
                .output(mach_regfifo_writable[j]), // 1-bit wire, per-machine
            );
        end
    endgenerate
    // machine+host -> fifo
    generate
        genvar k;
        for(k=0; k<4; k=k+1) begin: mach_to_fifo
            priority_demux #(
                .DATAW(32),
                .LEVELS(NUM_MACH + 1)
            ) select_wdata (
                .stb({
                    mach_regfifo_wr[3][k],
                    mach_regfifo_wr[2][k],
                    mach_regfifo_wr[1][k],
                    mach_regfifo_wr[0][k],
                    push_sync[k],
                }),
                .input({
                    mach_regfifo_wdata[3],
                    mach_regfifo_wdata[2],
                    mach_regfifo_wdata[1],
                    mach_regfifo_wdata[0],
                    fdin_sync[k]
                }),
                .output(regfifo_wdata[k]),
            );
            priority_demux #(
                .DATAW(1),
                .LEVELS(NUM_MACH + 1)
            ) select_wr (
                .stb({
                    mach_regfifo_wr[3][k],
                    mach_regfifo_wr[2][k],
                    mach_regfifo_wr[1][k],
                    mach_regfifo_wr[0][k],
                    push_sync[k],
                }),
                .input({
                    mach_regfifo_wr[3][k],
                    mach_regfifo_wr[2][k],
                    mach_regfifo_wr[1][k],
                    mach_regfifo_wr[0][k],
                    push_sync[k],
                }),
                .output(regfifo_we[k]),
            );
            priority_demux #(
                .DATAW(1),
                .LEVELS(NUM_MACH + 1)
            ) select_wr (
                .stb({
                    mach_regfifo_rd[3][k],
                    mach_regfifo_rd[2][k],
                    mach_regfifo_rd[1][k],
                    mach_regfifo_rd[0][k],
                    pull_sync[k],
                }),
                .input({
                    mach_regfifo_rd[3][k],
                    mach_regfifo_rd[2][k],
                    mach_regfifo_rd[1][k],
                    mach_regfifo_rd[0][k],
                    pull_sync[k],
                }),
                .output(regfifo_re[k]),
            );
        end
    endgenerate

    /////////////////////// event logic
    // TODO: left off here


    /////////////////////// repeated core units
    generate
        genvar j;
        for(j=0; j<NUM_MACH; j=j+1) begin: mach
            always_ff @(posedge aclk) begin
                // this aligns the enable so that it in concurrent with restart/clkdiv_restart
                if (~resetn) begin
                    en_sync[j] <= 0;
                end else if (ctl_action_sync) begin
                    en_sync[j] <= en[j];
                end
            end
            pio_divider clk_divider (
                .clk(aclk),
                .reset(reset | clkdiv_restart[j]),
                .div_int(div_int[j]),
                .div_frac(div_frac[j]),
                .penable(penable[j])
            );
            ICG icg(.CK(aclk),.EN(~stall[j]),.SE(cmsatpg),.CKG(core_clk[j]));
            picorv32 #(
                .ENABLE_COUNTERS(0),
                .ENABLE_COUNTERS64(0),
                .ENABLE_REGS_16_32(1),
                .ENABLE_REGS_DUALPORT(1),
	            .LATCHED_MEM_RDATA(0),
	            .TWO_STAGE_SHIFT(1),
	            .BARREL_SHIFTER(1),
	            .TWO_CYCLE_COMPARE(0),
	            .TWO_CYCLE_ALU(0),
	            .COMPRESSED_ISA(1),
	            .CATCH_MISALIGN(0),
	            .CATCH_ILLINSN(1),
	            .ENABLE_PCPI(0),
	            .ENABLE_MUL(0),
	            .ENABLE_FAST_MUL(0),
	            .ENABLE_DIV(0),
	            .ENABLE_IRQ(0),
	            .ENABLE_IRQ_QREGS(0),
	            .ENABLE_IRQ_TIMER(0),
	            .ENABLE_TRACE(0),
	            .REGS_INIT_ZERO(0),
	            .MASKED_IRQ(32'h 0000_0000),
	            .LATCHED_IRQ(32'h ffff_ffff),
	            .PROGADDR_RESET(j * 4),
	            .PROGADDR_IRQ(j * 4 + 32'h 0000_0010),
	            .STACKADDR(MEM_SIZE_BYTES - 1)
            ) core
            (
                .regfifo_rdata(mach_regfifo_rdata[j]),
                .regfifo_rd(mach_regfifo_rd[j]),
                .regfifo_wdata(mach_regfifo_wdata[j]),
                .regfifo_wr(mach_regfifo_wr[j]),
                .quanta_wr(quanta_wr[j]),
                .gpio_set(gpio_set[j]),
                .gpio_clr(gpio_clr[j]),
                .gpdir_set(gpdir_set[j]),
                .gpdir_clr(gpidr_clr[j]),
                .gpio_set_valid(gpio_set_valid[j]),
                .gpio_clr_valid(gpio_clr_valid[j]),
                .gpdir_set_valid(gpdir_set_valid[j]),
                .gpdir_clr_valid(gpdir_clr_valid[j]),
                .gpio_pins(gpio_in_cleaned),
                .aggregated_events(aggregated_events[j]),
                .stalling_for_event(stalling_for_event[j]),
                .event_set(event_set[j]),
                .event_set_valid(event_set_valid[j]),
                .event_clr(event_clr[j]),
                .event_clr_valid(event_clr_valid[j]),
                .core_id(j),
                .clk_count(clk_count[j]),

                .clk(core_clk[j]),
                .resetn(resetn),
                .trap(trap[j]),
                // .mem_valid(mem_valid[j]),
                // .mem_instr(mem_instr[j]),
                .mem_ready(mem_ready[j]),
                // .mem_addr(mem_addr[j]),
                // .mem_wdata(mem_wdata[j]),
                // .mem_wstrb(mem_wrstrb[j]),
                .mem_rdata(mem_rdata[j]),
                .mem_la_read(mem_la_read[j]),
                .mem_la_write(mem_la_write[j]),
                .mem_la_addr(mem_la_addr[j]),
                .mem_la_wdata(mem_la_wdata[j]),
                .mem_la_wstrb(mem_la_wstrb[j])
            );
        end
    endgenerate
    generate
        genvar k;
        for(k=0; k<4; k=k+1) begin: fifos
            regfifo regfifo(
                .reset(reset),
                .aclk(aclk),
                .wdata(regfifo_wdata[k]),
                .we(regfifo_we[k]),
                .writable(regfifo_writable[k]),
                .re(regfifo_re[k]),
                .readable(regfifo_readable[k]),
                .rdata(regfifo_rdata[k]),
                .level(regfifo_level[k]),
            );
        end
    endgenerate
endmodule

module priority_demux #(
    parameter DATAW = 32,
    parameter LEVELS = 4,
)(
    input [LEVELS-1:0]  stb,
    input [DATAW-1:0]   data_in[LEVELS],
    output [DATAW-1:0]  data_out
)
    always_comb begin
        data_out = 0;
        generate
            genvar i;
            for(i = LEVELS-1; i >= 0; i = i-1) begin: priority
                if (stb[i]) data_out = data_in[i];
            end
        endgenerate
    end
endmodule

module picorv32_regs_bio #(
    parameter NUM_MACH = 4,
    parameter NUM_MACH_BITS = $clog2(NUM_MACH)
)(
    input [31:0]  regfifo_rdata[4],
    output [3:0]  regfifo_rd, // must guarantee one pulse per read, even on successive repeated reads. Machine stalls with pulse asserted if FIFO is empty.
    output [31:0] regfifo_wdata,
    output [3:0]  regfifo_wr, // must guarantee one pulse per write, even on successive repeated writes. Machine stalls with pulse asserted if FIFO is full.

    output        quanta_wr,  // asserted on any write access to r20

    output [31:0] gpio_set,
    output [31:0] gpio_clr,
    output [31:0] gpdir_set,
    output [31:0] gpdir_clr,
    output        gpio_set_valid,
    output        gpio_clr_valid,
    output        gpdir_set_valid,
    output        gpdir_clr_valid,
    input [31:0]  gpio_pins,

    input [31:0]  aggregated_events,
    output        stalling_for_event,
    output [31:0] event_set,
    output        event_set_valid,
    output [31:0] event_clr,
    output        event_clr_valid,

    input [NUM_MACH_BITS-1:0]    core_id,
    input [31 - NUM_MACH_BITS:0] clk_count,

	input clk,
    input reset_n,
    input wen,
	input [5:0] waddr,
	input [5:0] raddr1,
	input [5:0] raddr2,
	input [31:0] wdata,
	output [31:0] rdata1,
	output [31:0] rdata2
);
	logic [31:0] regs [0:14];
    logic [31:0] gpio_mask;
    logic [31:0] event_mask;

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) gpio_mask <= 32'hffff_ffff;
        else if (wen & (waddr == 6'd26)) begin
            gpio_mask <= wdata;
        end

        if (!reset_n) event_mask <= 32'h0;
        else if (wen & (waddr == 6'd27)) begin
            event_mask <= wdata;
        end
    end
    // register file is X on boot
	always_ff @(posedge clk) begin
		if (wen && (waddr[5:4] == 2'b00)) begin
            regs[~waddr[3:0]] <= wdata;
        end
    end

    // write path
    always_comb begin
        regfifo_wr = '0;
        quanta_wr = 0;
        gpio_set_valid = 0;
        gpio_clr_valid = 0;
        gpdir_set = '0;
        gpdir_set_valid = 0;
        gpdir_clr_valid = 0;
        gpio_mask_valid = 0;
        event_set_valid = 0;
        event_clr_valid = 0;

        regfifo_wdata = wdata;
        gpio_set = gpio_mask & wdata;
        gpio_clr = ~gpio_mask | ~wdata;
        gpdir_set = gpio_mask & wdata;
        gpdir_clr = ~gpio_mask | ~wdata;
        event_set = wdata;
        event_clr = wdata;

        stalling_for_event = (event_mask & aggregated_events) == event_mask;

        if (wen) begin
            casez (waddr)
                // 0-15 handled in registered path
                6'b0100??: begin // 16-19
                    regfifo_wr[waddr[1:0]] = 1'b1;
                end
                6'b010100: begin // 20
                    quanta_wr = 1;
                end
                6'b010101: begin // 21
                    gpio_set_valid = 1;
                    gpio_clr_valid = 1;
                end
                6'b010110: begin // 22
                    gpio_set_valid = 1;
                end
                6'b010111: begin // 23
                    gpio_clr_valid = 1;
                end
                6'b011000: begin // 24
                    gpdir_set_valid = 1;
                end
                6'b011001: begin // 25
                    gpdir_clr_valid = 1;
                end
                // 26 is handled in registered path
                // 27 is handled in the registered path
                6'b011100: begin // 28
                    event_set_valid = 1;
                end
                6'b011101: begin // 29
                    event_clr_valid = 1;
                end
                // 30 is ignored
                // 31 is ignored
            endcase
        end
    end

    // read path
    always_comb begin
        regfifo_rd = 4'h0;
        rdata1 = 32'h0;
        rdata2 = 32'h0;

        casez (raddr1)
            6'b00????: begin // 0-15
                rdata1 = regs[~raddr1[3:0]];
            end
            6'b0100??: begin // 16-19
                rdata1 = regfifo_rdata[raddr1[1:0]]
                regfifo_rd[raddr1[1:0]] = 1;
            end
            // 20 undefined
            6'b010101: begin // 21
                rdata1 = gpio_pins;
            end
            // 22-25 undefined
            6'b011010: begin // 26
                rdata1 = gpio_mask;
            end
            6'b011011: begin // 27
                rdata1 = event_mask;
            end
            // 28-29 is undefined
            6'b011110: begin // 30
                rdata1 = aggregated_events;
            end
            6'b011111: begin // 31
                rdata1 = {core_id, clk_count};
            end
        endcase

        casez (raddr2)
            6'b00????: begin // 0-15
                rdata2 = regs[~raddr2[3:0]];
            end
            6'b0100??: begin // 16-19
                rdata2 = regfifo_rdata[raddr2[1:0]]
                regfifo_rd[raddr2] = 1'b1;
            end
            // 20 undefined
            6'b010101: begin // 21
                rdata2 = gpio_pins;
            end
            // 22-25 undefined
            6'b011010: begin // 26
                rdata2 = gpio_mask;
            end
            6'b011011: begin // 27
                rdata2 = event_mask;
            end
            // 28-29 is undefined
            6'b011110: begin // 30
                rdata2 = aggregated_events;
            end
            6'b011111: begin // 31
                rdata2 = {core_id, clk_count};
            end
        endcase
    end
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

// action + control register. Any write to this register will cause a pulse that
// can trigger an action, while also updating the value of the register
// this is a "special case" register because the spec requires select bits of one register
// to also be self-clearing, while others are sticky. :-/
module apb_ac2r
#(
      parameter A=0,
      parameter AW=12,
      parameter DW=12,
      parameter IV=32'h0,
      parameter SFRCNT=1,
//      parameter SRMASK=32'h0,               // set write 1 to clr ( for status reg )
      parameter RMASK=32'h0000_0fff        // read mask to remove undefined bit
//      parameter REXTMASK=32'h0              // read ext mask
)(
        input  logic                          pclk        ,
        input  logic                          resetn      ,
        apbif.slavein                         apbs        ,
        input  bit                            sfrlock     ,
        input  bit                            self_clear,
//        input  bit   [AW-1:0]               sfrpaddr    ,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrprdataext,
//        input  bit   [0:SFRCNT-1][DW-1:0]   sfrsr       ,
        output logic [31:0]                 prdata32    ,
        output logic [0:SFRCNT-1][DW-1:0]   cr          ,
        output bit                          ar
);


    logic[DW-1:0] prdata;
    assign prdata32 = prdata[3:0] | 32'h0;

    logic sfrapbwr;
    logic [7:0] self_clear_bits;
    logic [11:0] regular_bits;
    assign cr = {self_clear_bits[7:0], regular_bits[3:0]};

    always @(posedge pclk or negedge resetn) begin
        if(~resetn)
            self_clear_bits <= 8'h0;
        else begin
            if (self_clear) begin // clear to 0 after the pulse has cleared the synchronizer
                self_clear_bits[7:0] <= 8'h0;
            end else if (ar) begin // load value on write
                self_clear_bits[7:0] <= regular_bits[11:4];
            end else begin // else hold value
                self_clear_bits[7:0] <= self_clear_bits[7:0];
            end
        end
    end

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
            .sfrdata     (regular_bits   )
         );

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
