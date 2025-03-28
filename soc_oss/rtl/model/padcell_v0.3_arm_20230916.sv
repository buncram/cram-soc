//import pad_pkg::*;


module padcell_io #(
    parameter bit H = '1,
    parameter bit ANA = '0
)(
    inout wire pad,
    input padcfg_arm_t thecfg,
    output wire ai,
    ioif.load   pio
);

    `ifdef FPGA
        assign pad = pio.oe ? pio.po : 1'bZ;
        assign pio.pi = pad;
//        PULLUP pu ( pad );
    `else

    generate
        if( ~ANA && H )begin:ghd
            PBIDIR_33_33_FS_DR_H p(
                .PAD    (pad),
                .Y      (pio.pi),
                .IE     ('1),
                .IS     (thecfg.schmsel),
                .PE     (pio.pu),
                .PS     ('1),
                .A      (pio.po),
                .OE     (pio.oe),
                .DS0    (thecfg.drvsel[0]),
                .DS1    (thecfg.drvsel[1]),
                .SR     (thecfg.slewslow),
                .PO     (),     //useless
                .POE    ('0),   //useless
                .RTO    ('1),
                .SNS    ('1)
             );
        end
        if( ~ANA && ~H ) begin:gvd
            PBIDIR_33_33_FS_DR_V p(
                .PAD    (pad),
                .Y      (pio.pi),
                .IE     ('1),
                .IS     (thecfg.schmsel),
                .PE     (pio.pu),
                .PS     ('1),
                .A      (pio.po),
                .OE     (pio.oe),
                .DS0    (thecfg.drvsel[0]),
                .DS1    (thecfg.drvsel[1]),
                .SR     (thecfg.slewslow),
                .PO     (),     //useless
                .POE    ('0),   //useless
                .RTO    ('1),
                .SNS    ('1)
             );
        end
        if( ANA && H )begin:gha
            PBIDIRANAC_33_50_FS_DR_H p(
                .PAD    (pad),
                .Y      (pio.pi),
                .IE     ('1),
                .IS     (thecfg.schmsel),
                .PE     (pio.pu),
                .PS     ('1),
                .A      (pio.po),
                .OE     (pio.oe),
                .DS0    (thecfg.drvsel[0]),
                .DS1    (thecfg.drvsel[1]),
                .SR     (thecfg.slewslow),
                .MODE   (thecfg.anamode),
                .Y_IOV  (ai),
                .PO     (),     //useless
                .POE    ('0),   //useless
                .RTO    ('1),
                .SNS    ('1)
             );
        end
        if( ANA && ~H )begin:gva
            PBIDIRANAC_33_50_FS_DR_V p(
                .PAD    (pad),
                .Y      (pio.pi),
                .IE     ('1),
                .IS     (thecfg.schmsel),
                .PE     (pio.pu),
                .PS     ('1),
                .A      (pio.po),
                .OE     (pio.oe),
                .DS0    (thecfg.drvsel[0]),
                .DS1    (thecfg.drvsel[1]),
                .SR     (thecfg.slewslow),
                .MODE   (thecfg.anamode),
                .Y_IOV  (ai),
                .PO     (),     //useless
                .POE    ('0),   //useless
                .RTO    ('1),
                .SNS    ('1)
             );
        end
    endgenerate
    `endif

endmodule



module padcell_i #(
    parameter pu = 1,
    parameter pd = 0,
    parameter H = 1
)(
    input wire pad,
    input padcfg_arm_t thecfg,
    output logic pi
);

    `ifdef FPGA
        assign pi = pad;
        /*
    generate
        if(pu) begin: genpu
            PULLUP  pu ( pad );
        end
        else begin : genpd
            PULLDOWN pd ( pad );
        end
    endgenerate
    */
    `else

    generate
        if(H)begin:gh
            PINCRS_33_33_NT_DR_H p(
                .PAD    (pad),
                .Y      (pi),
                .PO     (),
                .IE     ('1),
                .IS     (thecfg.schmsel),
                .POE    ('0),
                .PE     (pu[0]|pd[0]),
                .PS     (pu[0]),
                .RTO    ('1),
                .SNS    ('1)
             );
        end
        else begin:gv
            PINCRS_33_33_NT_DR_V p(
                .PAD    (pad),
                .Y      (pi),
                .PO     (),
                .IE     ('1),
                .IS     (thecfg.schmsel),
                .POE    ('0),
                .PE     (pu[0]|pd[0]),
                .PS     (pu[0]),
                .RTO    ('1),
                .SNS    ('1)
             );
        end
    endgenerate

    `endif

endmodule

module padcell_o #(
    parameter pu = 1,
    parameter H = 1
)(
    output wire pad,
    input  padcfg_arm_t thecfg,
    input  logic po
);

//    assign thecfg.schmsel = '0;
//    assign thecfg.anamode = '0;
//    assign thecfg.slewslow = '0;
//    assign thecfg.drvsel = '0;

    ioif pio();
    assign pio.po = po;
    assign pio.pu = '1;
    assign pio.oe = '1;

    padcell_io #(.H(H))p(
        .pad,
        .thecfg,
        .ai(),
        .pio
    );

endmodule

module padcell_xtal #(
    parameter H    = 1,
    parameter X33k = 0
)(
    input wire padxin,
    inout wire padxout,
    input padcfg_arm_t thecfg,
    output wire pc
);

    `ifdef FPGA
        assign pc = padxin;
    `else
    generate

        if( X33k ) begin: g33k
            if( H )begin:gh
                POSC1_33_33_NT_DR_H p(
                    .CK(pc),
                    .CK_IOV(),
                    .PO(),
                    .PADO(padxout),
                    .PADI(padxin),
                    .E0('1),
                    .POE('0),
                    .RTO('1),
                    .SNS('1),
                    .TE('0),
                    .DS ()
                 );
            end
            else begin:gv
                POSC1_33_33_NT_DR_V p(
                    .CK(pc),
                    .CK_IOV(),
                    .PO(),
                    .PADO(padxout),
                    .PADI(padxin),
                    .E0('1),
                    .POE('0),
                    .RTO('1),
                    .SNS('1),
                    .TE('0),
                    .DS ()
                 );
            end
        end
        else begin: gelse
            if( H )begin:gh
                POSCP_33_33_NT_DR_H p(
                    .CK(pc),
                    .CK_IOV(),
                    .PO(),
                    .PADO(padxout),
                    .PADI(padxin),
                    .E0('1),
                    .POE('0),
                    .RTO('1),
                    .SF0(thecfg.drvsel[0]),
                    .SF1(thecfg.drvsel[1]),
                    .SNS('1),
                    .SP(),
                    .TE('0)
                 );
            end
            else begin:gv
                POSCP_33_33_NT_DR_V p(
                    .CK(pc),
                    .CK_IOV(),
                    .PO(),
                    .PADO(padxout),
                    .PADI(padxin),
                    .E0('1),
                    .POE('0),
                    .RTO('1),
                    .SF0(thecfg.drvsel[0]),
                    .SF1(thecfg.drvsel[1]),
                    .SNS('1),
                    .SP(),
                    .TE('0)
                 );
            end
        end
    endgenerate
    `endif

endmodule



