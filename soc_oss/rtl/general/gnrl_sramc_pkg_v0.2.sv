`include "template.sv"

package sram_pkg;
/*
    parameter AW = 8,
    parameter DW = 32,
    parameter KW = DW,
    parameter PW = DW/8,
    parameter WCNT = 2**AW,
    parameter BWEN = 1'b1 // byte write enable
*/

    typedef struct packed {
        int     AW;
        int     DW;
        int     KW;
        int     PW;
        int     WCNT;

        int     AWX;

        bit     isBWEN;
        bit     isSCMB;
        bit     isPRT;
        int     EVITVL;

    }sramcfg_t;

    localparam sramcfg_t samplecfg = '{
        AW: 10,
        DW: 32,
        KW: 32,
        PW: 4,
        WCNT: 1024,
        AWX: 5,
        isBWEN: '1,
        isSCMB: '1,
        isPRT:  '1,
        EVITVL:  15
    };

endpackage : sram_pkg
