`timescale 1ns/10ps
module rerammacro_blackbox();
        wire [0:1] ANALOG_0;

        RRN22ULL128KX144M32I8R16_D25_SHVT_C220530 u0(
              .RDONE             (), 
              .DOUT              (),
              .DOUT_CR           (),
              .SET               ('0),
              .RESET             ('0),
              .XADR              ('0),
              .YADR              ('0),
              .DIN               ('0),
              .CFG_MACRO         ('0),
              .RST               ('0),
              .NAP               ('0),
              .REDEN             ('0),
              .IFREN1            ('0),
              .IFREN             ('0),
              .XE                ('0),
              .YE                ('0),
              .READ              ('0),
              .PCH_EXT           ('0),
              .AE                ('0),
              .CE                ('0),
              .POC_IO            ('0),
              .DIN_CR            ('0),
              .ANALOG_0          (ANALOG_0[0])
        );

        RRN22ULL128KX144M32I8R16_D25_SHVT_C220530 u1(
              .RDONE             (), 
              .DOUT              (),
              .DOUT_CR           (),

              .SET               ('0),
              .RESET             ('0),
              .XADR              ('0),
              .YADR              ('0),
              .DIN               ('0),
              .CFG_MACRO         ('0),
              .RST               ('0),
              .NAP               ('0),
              .REDEN             ('0),
              .IFREN1            ('0),
              .IFREN             ('0),
              .XE                ('0),
              .YE                ('0),
              .READ              ('0),
              .PCH_EXT           ('0),
              .AE                ('0),
              .CE                ('0),
              .POC_IO            ('0),
              .DIN_CR            ('0),
              .ANALOG_0          (ANALOG_0[1])
        );

endmodule
