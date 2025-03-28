`include "template.sv"

import scedma_pkg::*;

module scedma_ac #(
    parameter CHNLCNT = scedma_pkg::CHNLACCNT,
    parameter accessrule_t [0:scedma_pkg::SEGCNT-1] ACRULEs = scedma_pkg::ACRULEs
)(
    input   logic                   clk,
    input   logic                   resetn,
    input   logic                   acenable,      
    input   chnlreq_t [0:CHNLCNT-1] chnlinreq   ,
    output  chnlres_t [0:CHNLCNT-1] chnlinres   ,
    output  chnlreq_t [0:CHNLCNT-1] chnloutreq  ,
    input   chnlres_t [0:CHNLCNT-1] chnloutres  ,
    output  logic [0:CHNLCNT-1]             acerr
);

    logic [0:CHNLCNT-1][7:0]    chnlinsegid;
    logic [0:CHNLCNT-1]         chnlac;
    logic [0:CHNLCNT-1]         errsr, errsw;

generate
    for(genvar i = 0; i < CHNLCNT; i++) begin: genac

        assign chnlinsegid[i] = chnlinreq[i].segcfg.segid;

`ifdef MPW
        assign chnlac[i] = '1;//~acenable | ACRULEs[chnlinsegid[i]][i];
`else
        assign chnlac[i] = ~acenable | ACRULEs[chnlinsegid[i]][i];
`endif
        assign chnloutreq[i].segcfg    = chnlinreq[i].segcfg    ;
        assign chnloutreq[i].segaddr   = chnlinreq[i].segaddr   ;
        assign chnloutreq[i].segptr    = chnlinreq[i].segptr    ;
        assign chnloutreq[i].segrd     = chnlinreq[i].segrd  & chnlac[i]   ;
        assign chnloutreq[i].segwr     = chnlinreq[i].segwr  & chnlac[i]   ;
        assign chnloutreq[i].segwdat   = chnlinreq[i].segwdat   ;
        assign chnloutreq[i].porttype  = chnlinreq[i].porttype  ;

        assign chnlinres[i].segready   = chnlac[i] ? chnloutres[i].segready   : '1 ;
        assign chnlinres[i].segrdat    = chnlac[i] ? chnloutres[i].segrdat    : '0 ;
        assign chnlinres[i].segrdatvld = chnlac[i] ? chnloutres[i].segrdatvld : '1 ;

        `theregrn( errsr[i] ) <= chnloutreq[i].segrd & ~ACRULEs[chnlinsegid[i]][i];
        `theregrn( errsw[i] ) <= chnloutreq[i].segwr & ~ACRULEs[chnlinsegid[i]][i];
    end
endgenerate

    assign acerr = errsr | errsw;



endmodule : scedma_ac
