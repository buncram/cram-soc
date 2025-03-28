

`define sram_sp_uhde_inst       \
         .ema         (3'b010),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (3'b001),  \
         .rawl        (1'b1),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1),    \
         .stov        (1'b0)

`define sram_sp_hde_inst        \
         .ema         (3'b100),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (3'b001),  \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1),    \
         .stov        (1'b0)

`define rf_sp_hde_inst          \
         .ema         (3'b100),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (2'b01),   \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1)

`define rf_2p_hdc_inst          \
         .emaa        (3'b011),  \
         .emab        (3'b101),  \
         .emasa       (1'b0),    \
         .stov        (1'b0),    \
         .wabl        (1'b0),    \
         .wablm       (2'b00),   \
         .ret1n       (1'b1)


`define sram_sp_svt_inst       \
         .ema         (3'b100),  \
         .emaw        (2'b01),   \
         .emas        (1'b0),    \
         .ret1n       (1'b1),    \
         .wabl        (1'b0),    \
         .wablm       (2'b00),   \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .stov        (1'b0)



// configurable

//         .ema         (3'b100),  \
`define sram_sp_svt_inst_tcm       \
         .ema         (sramtrm[2:0]),  \
         .emaw        (2'b01),   \
         .emas        (1'b0),    \
         .ret1n       (1'b1),    \
         .wabl        (1'b0),    \
         .wablm       (2'b00),   \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .stov        (1'b0)

//         .ema         (3'b100),  \
`define sram_sp_hde_inst_tcm       \
         .ema         (sramtrm[2:0]),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (3'b001),  \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1),    \
         .stov        (1'b0)

//         .ema         (3'b100),  \
`define rf_sp_hde_inst_cache          \
         .ema         (sramtrm[2:0]),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (2'b01),   \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1)

//         .ema         (3'b100),  \
`define sram_sp_hde_inst_sram1        \
         .ema         (sramtrm[2:0]),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (3'b001),  \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1),    \
         .stov        (1'b0)

//         .ema         (3'b010),  \
`define sram_sp_uhde_inst_sram0       \
         .ema         (sramtrm[2:0]),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (3'b001),  \
         .rawl        (1'b1),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1),    \
         .stov        (1'b0)

//         .emaa        (3'b011),  \
`define rf_2p_hdc_inst_vex          \
         .emaa        (sramtrm[2:0]),  \
         .emab        (3'b101),  \
         .emasa       (1'b0),    \
         .stov        (1'b0),    \
         .wabl        (1'b0),    \
         .wablm       (2'b00),   \
         .ret1n       (1'b1)

`define rf_sp_hde_inst_bio          \
         .ema         (sramtrm[2:0]),  \
         .emaw        (2'b00),   \
         .emas        (1'b0),    \
         .wabl        (1'b1),    \
         .wablm       (2'b01),   \
         .rawl        (1'b0),    \
         .rawlm       (2'b00),   \
         .ret1n       (1'b1)
