// sha3 round


module sha3core#(
        parameter AW = 10
    )(
        input  logic               clk,
        input  logic               resetn,

        input  hashcfg_t           thecfg,

        input  logic [7:0]         hashrnd,
        input  logic [5:0]         hashrndcyc,
        input  logic [5:0]         hashrndcycpl1,
        input  logic [5:0]         hashrndcycpl2,

        input  logic [31:0]   ramrdatreg,
        output logic [AW-1:0] ramptr,
        output logic          ramwr,
        output logic [31:0]   ramwdat,

        input  logic [0:4][63:0]   sty,
        output logic [0:4]         stywr,
        output logic [0:4][63:0]   stypre

    );

    assign rambase = '0;
    assign ramptr = '0;
    assign ramwr = '0;
    assign ramwdat = '0;
    assign vregwr = '0;
    assign vregpre = '0;



mfsm_a1_c, subfsmcnt = 25// vertical xor
mfsm_a1_d, subfsmcnt = 1 // D[x,z]
mfsm_a1_a, subfsmcnt = 50// A

mfsm_a2_r/rot/w 0/1/2/3/4


mfsm_a3_r

endmodule
