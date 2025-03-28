
module padcell_i #(parameter pu = 1)(
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
    generate
        if(pu) begin: genpu
         PDUW04SDGZ_H p(
            .I      ('0), 
            .OEN    ('1), 
            .REN    ('0), 
            .PAD    (pad), 
            .C      (pi)
        );
        end
        else begin : genpd
         PDDW04SDGZ_H p(
            .I      ('0), 
            .OEN    ('1), 
            .REN    ('0), 
            .PAD    (pad), 
            .C      (pi)
        );
        end
    endgenerate
    `endif

endmodule

module padcell_o #(parameter pu = 1)(
    output wire pad,
    input  logic po
);

    `ifdef FPGA
        assign pad = po;
    `else
         PDUW04SDGZ_H p(
            .I      (po), 
            .OEN    ('0), 
            .REN    ('1), 
            .PAD    (pad), 
            .C      ()
        );
    `endif

endmodule

module padcell_io (
    inout wire pad,
    ioif.load   pio
);

    `ifdef FPGA
        assign pad = pio.oe ? pio.po : 1'bZ;
        assign pio.pi = pad;
        PULLUP pu ( pad );
    `else
         PDUW04SDGZ_H p(
            .I      (pio.po), 
            .OEN    (~pio.oe), 
            .REN    (~pio.pu), 
            .PAD    (pad), 
            .C      (pio.pi)
        );
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



