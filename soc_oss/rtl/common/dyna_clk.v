//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date:    18:42:09 05/05/2013 
// Design Name: 
// Module Name:    dyna_clk 
// Project Name: 
// Target Devices: 
// Tool versions: 
// Description: 
//
// Dependencies: 
//
// Revision: 
// Revision 0.01 - File Created
//          0.10 - upgrade
// Additional Comments: 
//
//////////////////////////////////////////////////////////////////////////////////

`default_nettype none
`ifdef SIM
//`include "/opt/Xilinx/Vivado/currentversion/data/verilog/src/unisims/XADC.v"
`include "/opt/Xilinx/Vivado/currentversion/data/verilog/src/unisims/PLLE2_BASE.v"
`include "/opt/Xilinx/Vivado/currentversion/data/verilog/src/unisims/PLLE2_ADV.v"
`endif



(* use_dsp = "yes" *) module dyna_clk(
        osc_clk,
        hash_clk,
        uart_clk,
        led,
        clk_sel_pins
    );
    parameter C_FAMILY = "KINTEX7";
//    parameter DELAY_ORDER = 23;
//    localparam DEFAULT_D = 8'd5;        //
//    localparam DEFAULT_M = 5'd13;       // 48/(5+1)*(14+1) = 112MHz 
//    localparam LOWEST_M = 5'd3;         //  48/(5+1)*(3+1) = 32MHz, limit 32MHz 
//    localparam LOWEST_M_SEL = 5'd13;    // lowest M for SEL==4'h0,  48/(5+1)*(13+1) = 112MHz
                                        // Do NOT exceed 5'b10000 or 5'd16
    localparam MODE_TH = 16;            // mode switch , 48/(5+1)*(16+2) = 144
    localparam FS_ADC = C_FAMILY == "KINTEX7" ? 4096 : 1024;
    input   wire          osc_clk;
    input   wire    [7:0] clk_sel_pins;
    output  wire          hash_clk;
    output  wire          uart_clk;
    output  wire          led;
    
   localparam ALM_MSB = (C_FAMILY == "VERTEX5") ? 2 : 7;

function integer mdegc_to_int16 (input reg[63:0] temp_mdegc);
begin : F_degc_to_int
    integer code;
    if (C_FAMILY == "VERTEX5") begin
       code = ((temp_mdegc + 273_150) * 1024 / 503_975) << 6;
    end else if (C_FAMILY == "KINTEX7") begin
       code = ((temp_mdegc + 273_150) * 4096 / 503_975) << 4;
    end
    mdegc_to_int16 = code > 65535 ? 65535 : code;
end
endfunction

function integer mv_to_int16 (input reg[32:0]  mv);
begin : F_mv_to_int
    integer code;
    if (C_FAMILY == "VERTEX5") begin
       code = (mv * 1024 / 3000) << 6;
    end else if (C_FAMILY == "KINTEX7") begin
       code = (mv * 4096 / 3000) << 4;
    end
    mv_to_int16 = code > 65535 ? 65535 : code;
end
endfunction

    `define JSTF        6
    `define TEMP_80     (16'd717 << `JSTF)
    `define TEMP_75     (16'd707 << `JSTF)
    `define TEMP_70     (16'd697 << `JSTF)
    `define TEMP_65     (16'd687 << `JSTF)
    `define TEMP_60     (16'd677 << `JSTF)
    `define TEMP_55     (16'd667 << `JSTF)
    // v = code * 2.93 (mV)
    `define VINT_1100   (16'd375 << `JSTF)
    `define VINT_900    (16'd307 << `JSTF)	
    
//    reg     [DELAY_ORDER:0]  sysmon_delay = 0;    // appr 32M / 4.608M = 7sec
    //wire    [DELAY_ORDER:0]  sysmon_delay_next; 
//    reg     [4:0]   cksw = DEFAULT_M;
//    wire    [4:0]   cksw_next; 
//    reg     [4:0]   clk_sel = 5'b00000; // bit 4 is always 1, for 17x-32x mul,
                                       // 136-256MHz for clkin=240MHz,div=30
    // TODO XADC Enhanced Linearity Mode
    wire    [7:0]   alm;
    // XADC: Dual 12-Bit 1MSPS Analog-to-Digital Converter 
    // 7 Series 
    // Xilinx HDL Libraries Guide, version 14.7 
//    XADC #( // INIT_40 - INIT_42: XADC configuration registers 
//        .INIT_40(16'h3000), // average 256, continuous sampling mode
//        .INIT_41(16'h20f0), // Continuous sequence mode, enable calibration, disable unused alarms
//        .INIT_42(16'h0200), // ADCCLK = DCLK div 2
//        // 43..47, test reg
//        // INIT_48 - INIT_4F: Sequence Registers 
//        .INIT_48(16'h4701), // [0] Cal, [8] temp, [9] VCCINT, [10] VCCAUX, [14] VCCBRAM
//        .INIT_49(16'h0000), // other channel off
//        
//        .INIT_4A(16'h0000), // avg enable / disable, 1=enable or disable??
//        .INIT_4B(16'h0000), // avg en/dis
//        
//        .INIT_4C(16'h0000), // analog input mode, 0 is unipolar
//        .INIT_4D(16'h0000), // dito
//        
//        .INIT_4F(16'h4701), // "1" sets settling time to 10ADCCLK, 
//        .INIT_4E(16'h0000), // Sequence register 6 
//        // INIT_50 - INIT_58, INIT5C: Alarm Limit Registers 
//        .INIT_50(mdegc_to_int16(70_000)), // Temp upper
//        .INIT_51(mv_to_int16(    1_050)), // VCCINT upper 
//        .INIT_52(mv_to_int16(    2_000)), // VCCAUX upper
//        .INIT_53(mdegc_to_int16(90_000)), // OT limit
//        .INIT_54(mdegc_to_int16(65_000)), // Temp lower
//        .INIT_55(mv_to_int16(      970)), // VCCINT lower            
//        .INIT_56(mv_to_int16(    1_650)), // VCCAUX lower 
//        .INIT_57(mdegc_to_int16(75_000)), // OT alarm reset
//        .INIT_58(mv_to_int16(    1_050)), // VCCBRAM upper
//        .INIT_5C(mv_to_int16(      970)), // VCCBRAM lower
//        // Simulation attributes: Set for proper simulation behavior 
//        .SIM_DEVICE("7SERIES"), // Select target device (values) 
//        .SIM_MONITOR_FILE("xadc_sim.txt") // Analog simulation data file name 
//    ) XADC_inst ( // ALARMS: 8-bit (each) output: ALM, OT 
//        .ALM(alm),  // 8-bit output: Output alarm for  [0] temp, [1]Vccint, [2] Vccaux and [3]Vccbram 
//                    // [6:4] only for Zynq7000, [7] OR of [6:0] 
//        .OT(), // 1-bit output: Over-Temperature alarm
//        // Dynamic Reconfiguration Port (DRP): 16-bit (each) output: Dynamic Reconfiguration Ports 
//        .DO(), // 16-bit output: DRP output data bus 
//        .DRDY(), // 1-bit output: DRP data ready // STATUS: 1-bit (each) output: XADC status ports 
//        .BUSY(), // 1-bit output: ADC busy output 
//        .CHANNEL(), // 5-bit output: Channel selection outputs 
//        .EOC(), // 1-bit output: End of Conversion 
//        .EOS(), // 1-bit output: End of Sequence 
//        .JTAGBUSY(), // 1-bit output: JTAG DRP transaction in progress output 
//        .JTAGLOCKED(), // 1-bit output: JTAG requested DRP port lock 
//        .JTAGMODIFIED(), // 1-bit output: JTAG Write to the DRP has occurred 
//        .MUXADDR(), // 5-bit output: External MUX channel decode // Auxiliary Analog-Input Pairs: 16-bit (each) input: VAUXP[15:0], VAUXN[15:0] 
//        .VAUXN(16'h0000), // 16-bit input: N-side auxiliary analog input 
//        .VAUXP(16'h0000), // 16-bit input: P-side auxiliary analog input 
//        // CONTROL and CLOCK: 1-bit (each) input: Reset, conversion start and clock inputs 
//        .CONVST(1'b0), // 1-bit input: Convert start input 
//        .CONVSTCLK(1'b0), // 1-bit input: Convert start input 
//        .RESET(1'b0), // 1-bit input: Active-high reset // Dedicated Analog Input Pair: 1-bit (each) input: VP/VN 
//        .VN(1'b0), // 1-bit input: N-side analog input 
//        .VP(1'b0), // 1-bit input: P-side analog input // Dynamic Reconfiguration Port (DRP): 7-bit (each) input: Dynamic Reconfiguration Ports 
//        .DADDR(7'h00), // 7-bit input: DRP address bus 
//        .DCLK(uart_clk), // 1-bit input: DRP clock 
//        .DEN(1'b0), // 1-bit input: DRP enable signal 
//        .DI(16'h0000), // 16-bit input: DRP input data bus 
//        .DWE(1'b0) // 1-bit input: DRP write enable 
//    ); // End of XADC_inst instantiation

    // 48*24 / 125 = 9.216MHz
    pll_7s  #(
        .INPUT_T(16.6), // use the value in UCF to avoid annoying warninga
        .M(24), .D(1), 
        .DO0(125)
    ) pll_uart(
        .CLK_IN(osc_clk), 
        .CLKOUT0(uart_clk),
        .CLKOUT1(),
        .CLKOUT2(),
        .CLKOUT3(),
        .CLKOUT4(),
        .CLKOUT5(),
        .LOCKED_OUT()
    ); 


    (* keep = "no" *) wire        locked;

    //TODO
    pll_7s_drp #(
        .DELAY_MSB(23)
    ) u_pll_drp(
       .clk_in    (osc_clk),
       .clk_out   (hash_clk),
       .i_m_target(clk_sel_pins),
//       .alm_in    (|alm),
       .alm_in    (1'b0),  // debug, seems like there's a unrelated alarm
       .cfg_clk   (uart_clk  ),
       .locked_out(locked)
       // TODO critical alarm in
    );

    
    // LED outout for hash_clk / 1.0e6
    (* use_dsp48 = "yes" *) reg [19:0] led_ctr = 0;
    reg ledreg = 0;
    always @(posedge hash_clk) begin
        if (499_999 == led_ctr) begin
            led_ctr <= 0;
            ledreg <= ~ledreg;
        end else begin
            led_ctr <= led_ctr + 1;
        end
    end
    assign led = ledreg;

endmodule


module pll_7s(CLK_IN, 
                   CLKOUT0, 
                   CLKOUT1, 
                   CLKOUT2, 
                   CLKOUT3, 
                   CLKOUT4, 
                   CLKOUT5, 
                   LOCKED_OUT
);

    parameter INPUT_T = 20.833; 
    parameter D = 1; 
    parameter M = 12;
    // clk_in / D * M must be within [800MHz, 1600MHz]!
    // 60MHz / 1 * 12 = 720MHz     
    // 60MHz / 5 * 48 = 576MHz     
    parameter DO0 = 4;
    parameter DO1 = 4;
    parameter DO2 = 4;
    parameter DO3 = 4;
    parameter DO4 = 4;
    parameter DO5 = 4;
    // 720MHz / 4 = 180MHz (for hash mining)
    // 576 MHz / 125 = 4.608MHz  (== 115200 * 8 * 5, for uart)
    
    input wire CLK_IN;
    output wire CLKOUT0;
    output wire CLKOUT1;
    output wire CLKOUT2;
    output wire CLKOUT3;
    output wire CLKOUT4;
    output wire CLKOUT5;
    output wire LOCKED_OUT;
   
    wire CLKFBOUT_CLKFBIN;
    //wire CLK_IBUFG;
    wire CLKOUT0_UNBUF;
    wire CLKOUT1_UNBUF;
    wire CLKOUT2_UNBUF;
    wire CLKOUT3_UNBUF;
    wire CLKOUT4_UNBUF;
    wire CLKOUT5_UNBUF;
    
    //assign CLK_IBUFG = CLK_IN;
    BUFG  CLKOUT0_BUFG_INST (.I(CLKOUT0_UNBUF), .O(CLKOUT0));
    BUFG  CLKOUT1_BUFG_INST (.I(CLKOUT1_UNBUF), .O(CLKOUT1));
    BUFG  CLKOUT2_BUFG_INST (.I(CLKOUT2_UNBUF), .O(CLKOUT2));
    BUFG  CLKOUT3_BUFG_INST (.I(CLKOUT3_UNBUF), .O(CLKOUT3));
    BUFG  CLKOUT4_BUFG_INST (.I(CLKOUT4_UNBUF), .O(CLKOUT4));
    BUFG  CLKOUT5_BUFG_INST (.I(CLKOUT5_UNBUF), .O(CLKOUT5));


    // PLLE2_BASE: Base Phase Locked Loop (PLL) 
    // 7 Series 
    // Xilinx HDL Libraries Guide, version 14.7 
    PLLE2_BASE #( 
        .BANDWIDTH("OPTIMIZED"), // OPTIMIZED, HIGH, LOW 
        .CLKFBOUT_MULT(M), // Multiply value for all CLKOUT, (2-64) 
        .DIVCLK_DIVIDE(D), // Master division value, (1-56) 
        .CLKFBOUT_PHASE(0.0), // Phase offset in degrees of CLKFB, (-360.000-360.000). 
        .CLKIN1_PERIOD(INPUT_T),  // Input clock period in ns to ps resolution (i.e. 33.333 is 30 MHz). 
        // CLKOUT0_DIVIDE - CLKOUT5_DIVIDE: Divide amount for each CLKOUT (1-128) 
        .CLKOUT0_DIVIDE(DO0), 
        .CLKOUT1_DIVIDE(DO1), 
        .CLKOUT2_DIVIDE(DO2), 
        .CLKOUT3_DIVIDE(DO3), 
        .CLKOUT4_DIVIDE(DO4), 
        .CLKOUT5_DIVIDE(DO5), // CLKOUT0_DUTY_CYCLE - CLKOUT5_DUTY_CYCLE: Duty cycle for each CLKOUT (0.001-0.999). 
        .CLKOUT0_DUTY_CYCLE(0.5), 
        .CLKOUT1_DUTY_CYCLE(0.5), 
        .CLKOUT2_DUTY_CYCLE(0.5), 
        .CLKOUT3_DUTY_CYCLE(0.5), 
        .CLKOUT4_DUTY_CYCLE(0.5), 
        .CLKOUT5_DUTY_CYCLE(0.5), // CLKOUT0_PHASE - CLKOUT5_PHASE: Phase offset for each CLKOUT (-360.000-360.000). 
        .CLKOUT0_PHASE(0.0), 
        .CLKOUT1_PHASE(0.0), 
        .CLKOUT2_PHASE(0.0), 
        .CLKOUT3_PHASE(0.0), 
        .CLKOUT4_PHASE(0.0), 
        .CLKOUT5_PHASE(0.0), 
        .REF_JITTER1(0.0), // Reference input jitter in UI, (0.000-0.999). 
        .STARTUP_WAIT("FALSE") // Delay DONE until PLL Locks, ("TRUE"/"FALSE") 
    ) PLLE2_BASE_inst ( // Clock Outputs: 1-bit (each) output: User configurable clock outputs 
        .CLKOUT0(CLKOUT0_UNBUF), // 1-bit output: CLKOUT0 
        .CLKOUT1(CLKOUT1_UNBUF), // 1-bit output: CLKOUT1 
        .CLKOUT2(CLKOUT2_UNBUF), // 1-bit output: CLKOUT2 
        .CLKOUT3(CLKOUT3_UNBUF), // 1-bit output: CLKOUT3 
        .CLKOUT4(CLKOUT4_UNBUF), // 1-bit output: CLKOUT4 
        .CLKOUT5(CLKOUT5_UNBUF), // 1-bit output: CLKOUT5 // Feedback Clocks: 1-bit (each) output: Clock feedback ports 
        .CLKFBOUT(CLKFBOUT_CLKFBIN), // 1-bit output: Feedback clock 
        .LOCKED(LOCKED_OUT), // 1-bit output: LOCK 
        .CLKIN1(CLK_IN), // 1-bit input: Input clock // Control Ports: 1-bit (each) input: PLL control ports 
        .PWRDWN(1'b0), // 1-bit input: Power-down 
        .RST(1'b0 /*RST*/), // 1-bit input: Reset // Feedback Clocks: 1-bit (each) input: Clock feedback ports 
        .CLKFBIN(CLKFBOUT_CLKFBIN) // 1-bit input: Feedback clock 
    ); // End of PLLE2_BASE_inst instantiation

endmodule


module pll_7s_drp(
    clk_in, 
    clk_out, 
    i_m_target,
    alm_in,
    cfg_clk,
    locked_out
    // TODO critical alarm in
);

    parameter INPUT_T = 20.833; 
    //for 17x-32x mul, 48/2*34..48/2*64 = 816..1536
    // for 48MHz input , PLL_D = 2, OD = 4, M = {34..64}, F_out = {204,384}, step = 12MHz
    //                   PLL_D = 2, OD = 8, M = {34..64}, F_out = {102,192}, step =  6MHz
    //                                           ^^^^^^- 33 is slightly lower than 792MHz limit, shoudl work
    // for 
    parameter DEFAULT_D =  2; 
    parameter DEFAULT_OD = 4; 
    parameter DEFAULT_M = 40; // NOTICE! Default M, D and OD is ONLY for timing constraint & report
    parameter BANDWIDTH = "LOW";
    localparam LOWEST_M = 17;

    localparam M_TARGET_MIN = 17;
    localparam M_TARGET_MAX = 128;
    parameter START_MUL = LOWEST_M; // 48 / 2 *17-128, start at lowest freq for safety and power transient's sake
    parameter DELAY_MSB = 24;


// This function takes the divide value and outputs the necessary lock values
function [39:0] lut_pll_lock
   (
      input [6:0] clk_fb_div // Max divide is 64
   );

   localparam [40 * 64 - 1: 0]   lookup = {
         // This table is composed of:
         // LockRefDly_LockFBDly_LockCnt_LockSatHigh_UnlockCnt
         40'b00110_00110_1111101000_1111101001_0000000001,  //  1, never happens
         40'b00110_00110_1111101000_1111101001_0000000001,
         40'b01000_01000_1111101000_1111101001_0000000001,
         40'b01011_01011_1111101000_1111101001_0000000001,
         40'b01110_01110_1111101000_1111101001_0000000001,
         40'b10001_10001_1111101000_1111101001_0000000001,
         40'b10011_10011_1111101000_1111101001_0000000001,
         40'b10110_10110_1111101000_1111101001_0000000001,
         40'b11001_11001_1111101000_1111101001_0000000001,
         40'b11100_11100_1111101000_1111101001_0000000001,  // 10
         40'b11111_11111_1110000100_1111101001_0000000001,
         40'b11111_11111_1100111001_1111101001_0000000001,
         40'b11111_11111_1011101110_1111101001_0000000001,
         40'b11111_11111_1010111100_1111101001_0000000001,
         40'b11111_11111_1010001010_1111101001_0000000001,
         40'b11111_11111_1001110001_1111101001_0000000001,
         40'b11111_11111_1000111111_1111101001_0000000001,
         40'b11111_11111_1000100110_1111101001_0000000001,
         40'b11111_11111_1000001101_1111101001_0000000001,
         40'b11111_11111_0111110100_1111101001_0000000001,  // 20
         40'b11111_11111_0111011011_1111101001_0000000001,
         40'b11111_11111_0111000010_1111101001_0000000001,
         40'b11111_11111_0110101001_1111101001_0000000001,
         40'b11111_11111_0110010000_1111101001_0000000001,
         40'b11111_11111_0110010000_1111101001_0000000001,
         40'b11111_11111_0101110111_1111101001_0000000001,
         40'b11111_11111_0101011110_1111101001_0000000001,
         40'b11111_11111_0101011110_1111101001_0000000001,
         40'b11111_11111_0101000101_1111101001_0000000001,
         40'b11111_11111_0101000101_1111101001_0000000001,  // 30
         40'b11111_11111_0100101100_1111101001_0000000001,
         40'b11111_11111_0100101100_1111101001_0000000001,
         40'b11111_11111_0100101100_1111101001_0000000001,
         40'b11111_11111_0100010011_1111101001_0000000001,
         40'b11111_11111_0100010011_1111101001_0000000001,
         40'b11111_11111_0100010011_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,  // 40
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,  // 50
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,  // 60
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001,
         40'b11111_11111_0011111010_1111101001_0000000001   // 64
      }; // lock_LUT
    begin
        // Set lookup_entry with the explicit bits from lookup with a part select
        lut_pll_lock = lookup[ ((64 - clk_fb_div) * 40) +: 40];
    end
`ifdef DEBUG
      $display("lock_lookup: %b", lut_pll_lock);
`endif

endfunction

// This function takes the divide value and the bandwidth setting of the PLL
//  and outputs the digital filter settings necessary.
function [9:0] lut_pll_filter
   (
      input [6:0] clk_fb_div, // Max divide is 64
      input [8*9:0] BANDWIDTH
   );
//   reg [9:0] lookup_entry;
      
    localparam [10 * 64 - 1:0] lookup_low = {
        // CP_RES_LFHF
        10'b0010_1111_00,
        10'b0010_1111_00,
        10'b0010_0111_00,
        10'b0010_1101_00,
        10'b0010_0101_00,
        10'b0010_0101_00,
        10'b0010_1001_00,
        10'b0010_1110_00,
        10'b0010_1110_00,
        10'b0010_0001_00,
        10'b0010_0001_00,
        10'b0010_0110_00,
        10'b0010_0110_00,
        10'b0010_0110_00,
        10'b0010_0110_00,
        10'b0010_1010_00,
        10'b0010_1010_00,
        10'b0010_1010_00,
        10'b0010_1010_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_1100_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0010_0010_00,
        10'b0011_1100_00,
        10'b0011_1100_00,
        10'b0011_1100_00,
        10'b0011_1100_00,
        10'b0011_1100_00,
        10'b0011_1100_00,
        10'b0011_1100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00
    };

   localparam [10 * 64 - 1:0] lookup_high = {
        // CP_RES_LFHF
        10'b0011_0111_00,
        10'b0011_0111_00,
        10'b0101_1111_00,
        10'b0111_1111_00,
        10'b0111_1011_00,
        10'b1101_0111_00,
        10'b1110_1011_00,
        10'b1110_1101_00,
        10'b1111_1101_00,
        10'b1111_0111_00,
        10'b1111_1011_00,
        10'b1111_1101_00,
        10'b1111_0011_00,
        10'b1110_0101_00,
        10'b1111_0101_00,
        10'b1111_0101_00,
        10'b1111_0101_00,
        10'b1111_0101_00,
        10'b0111_0110_00,
        10'b0111_0110_00,
        10'b0111_0110_00,
        10'b0111_0110_00,
        10'b0101_1100_00,
        10'b0101_1100_00,
        10'b0101_1100_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b1100_0001_00,
        10'b0100_0010_00,
        10'b0100_0010_00,
        10'b0100_0010_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0011_0100_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0010_1000_00,
        10'b0100_1100_00,
        10'b0100_1100_00,
        10'b0100_1100_00,
        10'b0100_1100_00,
        10'b0100_1100_00,
        10'b0100_1100_00,
        10'b0100_1100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00,
        10'b0010_0100_00
    };
    
    // Set lookup_entry with the explicit bits from lookup with a part select
    if(BANDWIDTH == "LOW") begin
        // Low Bandwidth
        lut_pll_filter = lookup_low[ ((64 - clk_fb_div)*10) +: 10];
    end else begin
        // High or optimized bandwidth
        lut_pll_filter = lookup_high[ ((64 - clk_fb_div)*10) +: 10];
    end

`ifdef DEBUG
    $display("filter_lookup: %b", pll_filter_lookup);
`endif
endfunction


    // clk_in / D * M must be within [800MHz, 1600MHz]!
    localparam OTHER_DO = 128; // unused DO
    // 720MHz / 4 = 180MHz (for hash mining)
    // 576 MHz / 125 = 4.608MHz  (== 115200 * 8 * 5, for uart)
    
    input   wire            clk_in;
    input   wire            alm_in;
    input   wire    [7:0]   i_m_target; // valid range 17..128, not all values covered
    output  wire            clk_out;
    output  wire            locked_out;
    input   wire            cfg_clk;

    wire clk_fb_wire;
    wire clkout0_unbuf;
    
    //assign CLK_IBUFG = CLK_IN;
    BUFG  CLKOUT0_BUFG_INST (.I(clkout0_unbuf), .O(clk_out));

    // value path
    // I_M_TAEGET -> clk_mul_next -> clk_mul -> {mul_high/low, lock_lut, filter_lut, odiv_high/low} 
    reg     [7:0]   clk_mul = START_MUL;
    wire    [7:0]   clk_mul_next;
    reg     [7:0]   clk_sel = START_MUL; 
    
    assign clk_mul_next = 
        (|(alm_in)) ? 
            (clk_mul > LOWEST_M ? clk_mul - 1: clk_mul)  // reduce clock freq, until as low as 2x
            :
            ((clk_mul > clk_sel)?
                (clk_mul - 1)
                :
                ((clk_mul < clk_sel)?
                    (clk_mul + 1)
                    :
                    clk_mul
                )
            );

    // approx. 32M / 9.216M = 3.64sec
    reg  [DELAY_MSB : 0]  sysmon_delay = {(DELAY_MSB+1){1'b1}};  // reconfig right after boot
    reg  dcm_dfs_mode = 1'b0;
    always @(posedge cfg_clk) begin
        sysmon_delay <= sysmon_delay + 1;

        if (0 == sysmon_delay)
        begin
            clk_mul <= clk_mul_next;
            clk_sel <= (i_m_target < M_TARGET_MIN) ? M_TARGET_MIN 
                : 
                (i_m_target > M_TARGET_MAX) ? M_TARGET_MAX : i_m_target;
        end
    end
    
    // TODO DRP signals
    wire [7:0] mul_val;

    wire [39:0] lock_lut;
    wire [9:0] filter_lut;

    assign lock_lut   = lut_pll_lock(mul_val);
    assign filter_lut = lut_pll_filter(mul_val, BANDWIDTH);

    reg     [7:0]   clk_mul_save = DEFAULT_M;
    wire    drp_we;
    wire    drp_re;
    wire    drp_den;
    wire    drp_drdy;
    reg     drp_drdy_ext = 0; // drdy extended, to relax the bus, optional
    wire    drp_change;
    reg     [3:0]   drp_idx = 0;
    reg     drp_start = 0, drp_sel = 0, drp_wait = 0, drp_read = 0, drp_write = 0;
    wire    [6:0]   drp_daddr;
    reg     [15:0]  drp_do_r;
    wire    [15:0]  drp_do;
    //reg     [15:0]  drp_di;
    wire    [15:0]  drp_di;
    
    reg [5:0] odiv_high = DEFAULT_OD, odiv_low = DEFAULT_OD, mul_high = DEFAULT_M,  mul_low = DEFAULT_M;

    // valid mul is 2~64, 48/2*32 =768 is less than 800(violation), 
    // but 68 is larger than 64(HW limit), so we have to swithch at 32, 
    // Use a 50MHz OSC (or 26MHz VCTCXO) in the future for not violating VCO lower freq lim.
    assign mul_val = (clk_mul > 64) ? (clk_mul >> 1)
        :
        ((clk_mul > 32) ? clk_mul : (clk_mul << 1));
    wire [5:0] div_val;
    assign div_val = (clk_mul > 64) ? 4
        :
        ((clk_mul > 32) ? 8 : 16);
        
    always @(posedge cfg_clk) begin
        mul_high  <= (mul_val >> 1);  // TODO 0.5?
        mul_low   <= (mul_val >> 1);
        odiv_high <= (div_val >> 1);
        odiv_low  <= (div_val >> 1);        
    end
    
    // number of affecgted reg, co0 reg1,, cfbo reg 1, lock_lut[1,2,3], flt[1,2]
    localparam DRP_LAST_REG = 8;
    wire [7 + 16 - 1 : 0 ] drp_regs[0:DRP_LAST_REG];

    assign drp_regs[0] = {7'h28, 16'hffff};
    assign drp_regs[1] = {7'h08, {3'b0, drp_do_r[12], odiv_high, odiv_low}};  // co0 reg1 only, co0 reg2 not used
    assign drp_regs[2] = {7'h14, {3'b0, drp_do_r[12], mul_high,  mul_low}};
    assign drp_regs[3] = {7'h18, {drp_do_r[15:10], lock_lut[29:20]}};
    assign drp_regs[4] = {7'h19, {drp_do_r[15], lock_lut[34:30], lock_lut[ 9: 0]}};
    assign drp_regs[5] = {7'h1a, {drp_do_r[15], lock_lut[39:35], lock_lut[19:10]}};
    assign drp_regs[6] = {7'h4e, {filter_lut[9], drp_do_r[14:13], filter_lut[8:7], drp_do_r[10:9], filter_lut[6],  drp_do_r[7:0]}};
    assign drp_regs[7] = {7'h4f, {filter_lut[5], drp_do_r[14:13], filter_lut[4:3], drp_do_r[10:9], filter_lut[2:1],  drp_do_r[6:5], filter_lut[0], drp_do_r[3:0]}};
    assign drp_regs[8] = {7'h28, 16'h0000}; // need this?

    assign drp_di    = drp_regs[drp_idx][15:0];
    assign drp_daddr = drp_regs[drp_idx][22:16];
    
    assign drp_change = (clk_mul_save != clk_mul);
    
    (* clock_signal = "yes" *) 
    wire dcm_clkfx;
    wire pll_rst;
    assign drp_re = drp_read  & (!(drp_wait | drp_drdy_ext));
    assign drp_we = drp_write & (!(drp_wait | drp_drdy_ext));
    assign drp_den = drp_re | drp_we; // | (dcm_drdy & drpfsm[1] & drpfsm[2]);
    assign pll_rst = drp_start | drp_sel;

    // TODO DRP control
    always @(posedge cfg_clk) begin
        drp_drdy_ext <= drp_drdy;
        if (drp_read & drp_drdy) begin
            drp_do_r <= drp_do;
        end

        casex ({drp_change, drp_start, drp_sel, drp_read, drp_write, drp_wait, drp_drdy, drp_drdy_ext})
            // c ss rwt rd
            8'b1_00_000_00: begin  // change -> begin
                drp_start <= 1; 
                drp_idx  <= 0;
            end
            8'bx_10_000_xx: begin  // begin -> reset, idle
                drp_sel <= 1;
                drp_start <= 0;
                drp_idx <= 0;
            end            
            8'bx_x1_000_xx: begin  // idle -> to read
                drp_read <= 1;
            end
            8'bx_x1_100_xx: begin  // read -> to read wait
                drp_wait <= 1;
            end          
            8'bx_x1_xx1_x0: begin  // wait, and wait for drdy_ext, read or write
            end
            8'bx_x1_101_x1: begin  // wait -> write
                drp_read <= 0;
                drp_wait <= 0;
                drp_write <= 1;
            end
            8'bx_x1_010_xx: begin  // write -> write wait
                drp_wait <= 1;
            end
            8'bx_x1_011_x1: begin  // write wait -> next
                drp_wait <= 0;
                drp_write <= 0;
                if (drp_idx != DRP_LAST_REG) begin // next
                    drp_idx <= drp_idx + 1;
                end else begin // finish
                    clk_mul_save <= clk_mul;
                    drp_start <= 0;
                    drp_sel <= 0;
                    drp_read <= 0;
                end
            end
            default: begin
                {drp_start, drp_sel, drp_read, drp_write, drp_wait} <= 0;
            end
        endcase
    end

    // PLLE2_BASE: Base Phase Locked Loop (PLL) 
    // 7 Series 
    // Xilinx HDL Libraries Guide, version 14.7 
    PLLE2_ADV #( 
        .BANDWIDTH(BANDWIDTH), // OPTIMIZED, HIGH, LOW 
        .CLKFBOUT_MULT(DEFAULT_M), // Multiply value for all CLKOUT, (2-64) 
        .DIVCLK_DIVIDE(DEFAULT_D), // Master division value, (1-56) 
        .CLKFBOUT_PHASE(0.0), // Phase offset in degrees of CLKFB, (-360.000-360.000). 
        .CLKIN1_PERIOD(INPUT_T),  // Input clock period in ns to ps resolution (i.e. 33.333 is 30 MHz). 
        .CLKIN2_PERIOD(INPUT_T),
        // CLKOUT0_DIVIDE - CLKOUT5_DIVIDE: Divide amount for each CLKOUT (1-128) 
        .CLKOUT0_DIVIDE(DEFAULT_OD), 
        .CLKOUT1_DIVIDE(OTHER_DO), 
        .CLKOUT2_DIVIDE(OTHER_DO), 
        .CLKOUT3_DIVIDE(OTHER_DO), 
        .CLKOUT4_DIVIDE(OTHER_DO), 
        .CLKOUT5_DIVIDE(OTHER_DO), 
        // CLKOUT0_DUTY_CYCLE - CLKOUT5_DUTY_CYCLE: Duty cycle for each CLKOUT (0.001-0.999). 
        .CLKOUT0_DUTY_CYCLE(0.5), 
        .CLKOUT1_DUTY_CYCLE(0.5), 
        .CLKOUT2_DUTY_CYCLE(0.5), 
        .CLKOUT3_DUTY_CYCLE(0.5), 
        .CLKOUT4_DUTY_CYCLE(0.5), 
        .CLKOUT5_DUTY_CYCLE(0.5), 
        // CLKOUT0_PHASE - CLKOUT5_PHASE: Phase offset for each CLKOUT (-360.000-360.000). 
        .CLKOUT0_PHASE(0.0), 
        .CLKOUT1_PHASE(0.0), 
        .CLKOUT2_PHASE(0.0), 
        .CLKOUT3_PHASE(0.0), 
        .CLKOUT4_PHASE(0.0), 
        .CLKOUT5_PHASE(0.0), 
        .COMPENSATION("INTERNAL"), // ZHOLD, BUF_IN, EXTERNAL, INTERNAL
        .REF_JITTER1(0.0), // Reference input jitter in UI, (0.000-0.999). 
        .REF_JITTER2(0.0), // Reference input jitter in UI, (0.000-0.999). 
        .STARTUP_WAIT("FALSE") // Delay DONE until PLL Locks, ("TRUE"/"FALSE") 
    ) u_plle2 ( // Clock Outputs: 1-bit (each) output: User configurable clock outputs 
        .CLKIN1(clk_in), // 1-bit input: Input clock // Control Ports: 1-bit (each) input: PLL control ports 
        .CLKIN2(clk_in), // 1-bit input: Input clock // Control Ports: 1-bit (each) input: PLL control ports 
        .CLKINSEL(1'b0), // 1-bit input: Input clock // Control Ports: 1-bit (each) input: PLL control ports 
        .CLKFBOUT(clk_fb_wire), // 1-bit output: Feedback clock 
        .CLKFBIN( clk_fb_wire), // 1-bit input: Feedback clock 
        
        .CLKOUT0(clkout0_unbuf), // 1-bit output: CLKOUT0 
        .CLKOUT1(), // 1-bit output: CLKOUT1 
        .CLKOUT2(), // 1-bit output: CLKOUT2 
        .CLKOUT3(), // 1-bit output: CLKOUT3 
        .CLKOUT4(), // 1-bit output: CLKOUT4 
        .CLKOUT5(), // 1-bit output: CLKOUT5 
        
        .PWRDWN(1'b0), // 1-bit input: Power-down 
        // Feedback Clocks: 1-bit (each) output: Clock feedback ports 
        .LOCKED(locked_out), // 1-bit output: LOCK 
     
        // DRP Ports: 16-bit (each) output: Dynamic reconfiguration ports 
        .DO(drp_do), // 16-bit output: DRP data 
        .DRDY(drp_drdy), // 1-bit output: DRP ready 
        // Status Ports: 1-bit (each) output: MMCM status ports 
        .DADDR(drp_daddr), // 7-bit input: DRP address 
        .DCLK(cfg_clk), // 1-bit input: DRP clock 
        .DEN(drp_den), // 1-bit input: DRP enable 
        .DI(drp_di), // 16-bit input: DRP data 
        .DWE(drp_we), // 1-bit input: DRP write enable 
        
        .RST(pll_rst) // 1-bit input: Reset // Feedback Clocks: 1-bit (each) input: Clock feedback ports 
    ); // End of PLLE2_BASE_inst instantiation

endmodule


