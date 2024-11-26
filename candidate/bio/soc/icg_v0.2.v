// (c) Copyright CrossBar, Inc. 2024.
//
// This documentation describes Open Hardware and is licensed under the
// [CERN-OHL-W-2.0].
//
// You may redistribute and modify this documentation under the terms of the
// [CERN-OHL- W-2.0 (http://ohwr.org/cernohl)]. This documentation is
// distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY, INCLUDING OF
// MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A PARTICULAR PURPOSE.
// Please see the [CERN-OHL- W-2.0] for applicable conditions.

module ICG(CK,EN,SE,CKG);
input   wire CK,EN,SE;
output  wire CKG;

`ifdef FPGA
// BUFGCE: General Clock Buffer with Clock Enable
// UltraScale
// Xilinx HDL Libraries Guide, version 2014.1
    BUFGCE #(
    .CE_TYPE("SYNC"), // SYNC, ASYNC
    .IS_CE_INVERTED(1'b0), // Programmable inversion on CE
    .IS_I_INVERTED(1'b0) // Programmable inversion on I
    )
    BUFGCE_inst (
    .O(CKG), // 1-bit output: Buffer
    .CE(EN), // 1-bit input: Buffer enable
    .I(CK) // 1-bit input: Buffer
    );
// End of BUFGCE_inst instantiation
`else

`ifdef SIM
    STN_CKGTPLT_V5_2 uicg(.Q(CKG), .CK(CK), .EN(EN), .SE(SE));
`endif

`ifdef SYN

    `ifdef SC9T_ARM
        PREICG_X2B_A9G33 uicg(.ECK(CKG), .CK(CK), .E(EN), .SE(SE));
    `endif

    `ifdef SC7T_ARM
        PREICG_X4B_A7PP140ZTS_C40 uicg(.ECK(CKG), .CK(CK), .E(EN), .SE(SE));
    `endif

    `ifdef SC6T_ARM
        PREICG_X4B_A6P5PP140ZTH_C30 uicg(.ECK(CKG), .CK(CK), .E(EN), .SE(SE));
    `endif

    `ifdef SC9T_TSMC
        CKLNQD8BWP35P140 uicg(.Q(CKG), .CP(CK), .E(EN), .TE(SE));
    `endif

`endif

`endif

endmodule

`ifdef SIM
module STN_CKGTPLT_V5_2 (Q, CK, EN, SE);
input   CK,EN,SE;
output  Q;
wire    CK,EN,Q;

        wire    or_out;
        reg     EN1;

        assign or_out = EN;

        always  @(CK or or_out) if(!CK) EN1 = or_out;

        assign Q = ( SE | EN1 ) & CK;

endmodule
/*
module STN_INV_S_8( A, X);
    input A;
    output X;
    wire A,X;
    assign X=A;
endmodule
*/
`endif


module CLKCELL_BUF ( A, Z );
    input wire A;
    output wire Z;
`ifdef SYN
    `ifdef SC9T_ARM
        BUF_X4M_A9G33(.A(A),.Y(Z));
    `endif

    `ifdef SC7T_ARM
        BUF_X4B_A7PP140ZTS_C40(.A(A),.Y(Z));
    `endif

    `ifdef SC6T_ARM
        BUF_X4M_A6P5PP140ZTH_C30(.A(A),.Y(Z));
    `endif
    `ifdef SC9T_TSMC
        CKBD4BWP35P140 u1 (.I(A),.Z(Z));
    `endif
`else
    assign Z = A;
`endif

endmodule : CLKCELL_BUF



module CLKCELL_MUX2 ( A, B, S, Z );
    input wire A,B,S;
    output wire Z;
`ifdef SYN
    `ifdef SC9T_ARM
        BUF_X4M_A9G33(.A(A),.Y(Z));
    `endif

    `ifdef SC7T_ARM
        BUF_X4B_A7PP140ZTS_C40(.A(A),.Y(Z));
    `endif

    `ifdef SC6T_ARM
        BUF_X4M_A6P5PP140ZTH_C30(.A(A),.Y(Z));
    `endif
    `ifdef SC9T_TSMC
        CKMUX2D4BWP35P140 u1 (.I0(A),.I1(B),.S(S),.Z(Z));
    `endif
`else
    assign Z = S? B: A;
`endif

endmodule : CLKCELL_MUX2

module DATACELL_BUF ( A, Z );
    input wire A;
    output wire Z;
`ifdef SYN
    `ifdef SC9T_ARM
        BUF_X4M_A9G33(.A(A),.Y(Z));
    `endif

    `ifdef SC7T_ARM
        BUF_X4B_A7PP140ZTS_C40(.A(A),.Y(Z));
    `endif

    `ifdef SC6T_ARM
        BUF_X4M_A6P5PP140ZTH_C30(.A(A),.Y(Z));
    `endif
    `ifdef SC9T_TSMC
        BUFFD2BWP40P140HVT u1 (.I(A),.Z(Z));
    `endif
`else
    assign Z = A;
`endif

endmodule : DATACELL_BUF


