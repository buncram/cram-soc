module cgufpgadrp (
    input  logic clk48m,
    input logic [7:0] clkpll0cfg,
    input logic [7:0] clkpll1cfg,
    output logic clk32m,
    output logic clkpll0,
    output logic clkpll1
);

/*
# Dynamic Clock generator for Ultrascale Plus

## Revision

20210312: update for VUP.
20221107: update for VUP Daric project, 48MHz input, 25-200MHz output.

## top module 

which is `dyna_clk` in `dyna_clk_vup4.v`

## info

### parameter

- Input clock frequency: 48MHz
- Output clock frequency range: 25MHz-200MHz
- Input `clk_sel_pins`, 8-bit, valid range: 16-128, (each LSB is 1.5625MHz (25MHz/16))
- Output duty cycle: 50%
- Output clock while frequency switching is STOPPED
- Auxilary Clock:
- - `uart_clk`: output, 14.7456MHz (exact value: 48 * 25 / 81.375 MHz) for UART 128 * 115200 and for MMCM_DRP config
- - clk_50M: internal 50MHz for second stage 
- - clk_12M5: deprecated

### Resources

- MMCME4: 2, on VU13P, each SLR has 4
- BUFG: ~3?
- SYSMONE4: 1, for Overtemperature protection
*/

dyna_clk_vup4 drp32m
(
    .osc_clk        (clk48m),
    .clk_sel_pins   ('d20), // 1.5625 * 20 = 31.25 M
    .hash_clk       (clk32m),
    .uart_clk       (),
    .led            (),
    .o_lock         ()
);

dyna_clk_vup4 drpclkpll0
(
    .osc_clk        (clk48m),
    .clk_sel_pins   (clkpll0cfg),
    .hash_clk       (clkpll0),
    .uart_clk       (),
    .led            (),
    .o_lock         ()
);

dyna_clk_vup4 drpclkpll1
(
    .osc_clk        (clk48m),
    .clk_sel_pins   (clkpll1cfg),
    .hash_clk       (clkpll1),
    .uart_clk       (),
    .led            (),
    .o_lock         ()
);

endmodule