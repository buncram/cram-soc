
module rerammacro(
        input  rrc_pkg::rri_t rri,
        output rrc_pkg::rro_t rro,
        inout  ANALOG_0
    );


rrn22ull128kx144m32i8r16_d25_shvt_c220530_wrapper u_rram_wrapper(
                .RDONE        (rro.RDONE            ),
                .SET          (rri.SET              ),
                .RESET        (rri.RESET            ),
                .XADR         (rri.XADR             ),
                .YADR         (rri.YADR             ),
                .DOUT         (rro.DOUT             ),
                .DIN          (rri.DIN              ),
                .CFG_MACRO    (rri.CFG_MACRO        ),
                .RST          (rri.RST              ),
                .NAP          (rri.NAP              ),
                .REDEN        (rri.REDEN            ),
                .IFREN1       (rri.IFREN1           ),
                .IFREN        (rri.IFREN            ),
                .XE           (rri.XE               ),
                .YE           (rri.YE               ),
                .READ         (rri.READ             ),
                .PCH_EXT      (rri.PCH_EXT          ),
                .AE           (rri.AE               ),
                .CE           (rri.CE               ),
                .POC_IO       (rri.POC_IO           ),
                .DIN_CR       (rri.DIN_CR           ),
                .DOUT_CR      (rro.DOUT_CR          ),
                .ANALOG_0     (ANALOG_0             )
);

endmodule