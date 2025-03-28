
module padcell_i #(parameter pu = 1,
    parameter pd = 0)(
    input wire pad,
    output logic pi
);

    `ifdef FPGA
        assign pi = pad;
//    generate
//        if(pu) begin: genpu
//            PULLUP  pu ( pad );
//        end
//        else begin : genpd
//            PULLDOWN pd ( pad );
//        end
//    endgenerate
    `else
//    generate
        PDDWUW0408SDGH_H p(
            .C      (pi),
            .DS     (1'b0),
            .I      (1'b0),
            .IE     (1'b1),
            .OEN    (1'b1), 
            .PAD    (pad), 
            .PU     (pu[0]), 
            .PD     (pd[0]) 
        );
//        if(pu) begin: genpu
//         PDUW04SDGZ_H p(
//            .I      ('0), 
//            .OEN    ('1), 
//            .REN    ('0), 
//            .PAD    (pad), 
//            .C      (pi)
//        );
//        end
//        else begin : genpd
//         PDDW04SDGZ_H p(
//            .I      ('0), 
//            .OEN    ('1), 
//            .REN    ('0), 
//            .PAD    (pad), 
//            .C      (pi)
//        );
//        end
//    endgenerate
    `endif

endmodule

module padcell_o #(parameter pu = 1)(
    output wire pad,
    input  logic po
);

    `ifdef FPGA
        assign pad = po;
    `else
        PDDWUW0408SDGH_H p(
            .C      (),
            .DS     (1'b0),
            .I      (po),
            .IE     (1'b1),
            .OEN    (1'b0), 
            .PAD    (pad), 
            .PU     (1'b0), 
            .PD     (1'b0) 
        );
//         PDUW04SDGZ_H p(
//            .I      (po), 
//            .OEN    ('0), 
//            .REN    ('1), 
//            .PAD    (pad), 
//            .C      ()
//        );
    `endif

endmodule

module padcell_io (
    inout wire pad,
    ioif.load   pio
);

    `ifdef FPGA
        assign pad = pio.oe ? pio.po : 1'bZ;
        assign pio.pi = pad;
        // PULLUP pu ( pad );
    `else
        PDDWUW0408SDGH_H p(
            .C      (pio.pi),
            .DS     (1'b0),
            .I      (pio.po),
            .IE     (1'b1),
            .OEN    (~pio.oe), 
            .PAD    (pad), 
            .PU     (pio.pu), 
            .PD     (1'b0) 
        );
//         PDUW04SDGZ_H p(
//            .I      (pio.po), 
//            .OEN    (~pio.oe), 
//            .REN    (~pio.pu), 
//            .PAD    (pad), 
//            .C      (pio.pi)
//        );
    `endif

endmodule

module padcell_xtal (
    input logic padxin,
    output logic padxout,
    output logic pc
);

    `ifdef FPGA
        assign pc = padxin;
    `else
         PDXOEDG8E_H p(
            .DS0    ('0), 
            .DS1    ('0), 
            .DS2    ('0), 
            .XE     ('1), 
            .XIN    (padxin), 
            .XC     (pc),
            .XOUT   (padxout) 
        );
    `endif

endmodule



