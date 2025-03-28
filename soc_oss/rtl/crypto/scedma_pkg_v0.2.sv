
`include "template.sv"
x
import sram_pkg::*;

package scedma_pkg;

    parameter AW = 14;  // 16KB at 32bit // the greatest AW of all RAM/size/ptr
    parameter DW = 32;  // 4KB at 32bit
    parameter RAMCNT = 1;
    parameter SEGTYPECNT = 5;
    parameter TRANSCNTW = 30;

    typedef bit[AW-1:0]     adr_t   ;
    typedef bit[DW-1:0]     dat_t   ;

    typedef struct packed {
        bit [7:0]   ramid           ;
        adr_t       ramaw           ;
        adr_t       ramsize         ;
    }ramcfg_t;


// seg define
// ■■■■■■■■■■■■■■■ 

    typedef enum  bit[$clog2(SEGTYPECNT)-1:0] {
        ST_NONE      = 'd0,
        ST_BI        = 'd1,     //        input  data buf
        ST_BO        = 'd2,     //        output data buf
        ST_SI        = 'd3,     // secure input  data buf ( not used )
        ST_SO        = 'd4      // secure output data buf
    } segtype_e;

    typedef struct packed {
		bit [7:0]	segid 		    ;
        segtype_e   segtype         ;
		bit [7:0]	ramsel 		    ;
		adr_t       segaddr 	    ;
        adr_t       segsize 	    ;
        bit 		isfifo		    ;
        bit 		isfifostream	;
        bit [7:0]   fifoid          ; // idx of fifo
    }segcfg_t;

// chnl 
// ■■■■■■■■■■■■■■■ 

    typedef struct packed {
        bit [7:0]   chnlid          ;
        segcfg_t    rpsegcfg        ;
        segcfg_t    wpsegcfg        ;
        adr_t       rpptr_start     ;
        adr_t       wpptr_start     ;

        bit         wpffen          ;
        bit [TRANSCNTW-1:0] transsize       ;
        bit [3:0]   opt_ltx         ;
        bit         opt_xor         ;
        bit         opt_cmpp        ;   // mod 
        bit [255:0] opt_prm         ;   // prime to be mod
    }chnlcfg_t;

    typedef enum  bit[1:0] {
        PT_NONE      = 'd0,
        PT_RO        = 'd1,
        PT_WO        = 'd2,
        PT_RW        = 'd3
    } porttype_e;

  // chnl port

    typedef struct packed {
        segcfg_t       segcfg       ;
        adr_t          segaddr      ;
        adr_t          segptr       ;
        bit            segrd        ;
        bit            segwr        ;
        dat_t          segwdat      ;
        porttype_e     porttype     ;
     }chnlreq_t; 

    typedef struct packed {
        bit            segready     ;
        dat_t          segrdat      ;
        bit            segrdatvld   ;
    }chnlres_t; 

  // for arb only @memc

    typedef struct packed {
        adr_t          segaddr      ;
        adr_t          segptr       ;
        bit            ramrd        ;
        bit            ramwr        ;
        dat_t          ramwdat      ;
    }arbdat_t;


    typedef struct packed {
        bit [7:0]   portid          ;
        bit [SEGTYPECNT-1:0]
                    accessrule  ;   // should be 
    }accessrule_t;

// seg parameter
// ■■■■■■■■■■■■■■■

    localparam [7:0] SEGID_LKEY = 8'd0;
    localparam [7:0] SEGID_KEY  = SEGID_LKEY + 'd1;
    localparam [7:0] SEGID_SKEY = SEGID_KEY  + 'd1;
    localparam [7:0] SEGID_SCRT = SEGID_SKEY + 'd1;
    localparam [7:0] SEGID_MSG  = SEGID_SCRT + 'd1;
    localparam [7:0] SEGID_HOUT = SEGID_MSG  + 'd1;
    localparam [7:0] SEGID_SOB  = SEGID_HOUT + 'd1;
    localparam [7:0] SEGID_PIB  = SEGID_SOB  + 'd1;
    localparam [7:0] SEGID_PSIB = SEGID_PIB  + 'd1;
    localparam [7:0] SEGID_POB  = SEGID_PSIB + 'd1;
    localparam [7:0] SEGID_PSOB = SEGID_POB  + 'd1;
    localparam [7:0] SEGID_AKEY = SEGID_PSOB + 'd1;
    localparam [7:0] SEGID_AIB  = SEGID_AKEY + 'd1;
    localparam [7:0] SEGID_AOB  = SEGID_AIB  + 'd1;
    localparam [7:0] SEGID_RNGA = SEGID_AOB  + 'd1;
    localparam [7:0] SEGID_RNGB = SEGID_RNGA + 'd1;

    localparam adr_t SEGSIZE_LKEY = 'd256/4; localparam adr_t SEGADDR_LKEY = 'd0;
    localparam adr_t SEGSIZE_KEY  = 'd256/4; localparam adr_t SEGADDR_KEY  = SEGADDR_LKEY + SEGSIZE_LKEY;
    localparam adr_t SEGSIZE_SKEY = 'd256/4; localparam adr_t SEGADDR_SKEY = SEGADDR_KEY  + SEGSIZE_KEY ;
    localparam adr_t SEGSIZE_SCRT = 'd256/4; localparam adr_t SEGADDR_SCRT = SEGADDR_SKEY + SEGSIZE_SKEY;
    localparam adr_t SEGSIZE_MSG  = 'd512/4; localparam adr_t SEGADDR_MSG  = SEGADDR_SCRT + SEGSIZE_SCRT;
    localparam adr_t SEGSIZE_HOUT = 'd256/4; localparam adr_t SEGADDR_HOUT = SEGADDR_MSG  + SEGSIZE_MSG ;
    localparam adr_t SEGSIZE_SOB  = 'd256/4; localparam adr_t SEGADDR_SOB  = SEGADDR_HOUT + SEGSIZE_HOUT;
    localparam adr_t SEGSIZE_PIB  = 'd256/4; localparam adr_t SEGADDR_PIB  = SEGADDR_SOB  + SEGSIZE_SOB ;
    localparam adr_t SEGSIZE_PSIB = 'd256/4; localparam adr_t SEGADDR_PSIB = SEGADDR_PIB  + SEGSIZE_PIB ;
    localparam adr_t SEGSIZE_POB  = 'd256/4; localparam adr_t SEGADDR_POB  = SEGADDR_PSIB + SEGSIZE_PSIB;
    localparam adr_t SEGSIZE_PSOB = 'd256/4; localparam adr_t SEGADDR_PSOB = SEGADDR_POB  + SEGSIZE_POB ;
    localparam adr_t SEGSIZE_AKEY = 'd256/4; localparam adr_t SEGADDR_AKEY = SEGADDR_PSOB + SEGSIZE_PSOB;
    localparam adr_t SEGSIZE_AIB  = 'd256/4; localparam adr_t SEGADDR_AIB  = SEGADDR_AKEY + SEGSIZE_AKEY;
    localparam adr_t SEGSIZE_AOB  = 'd256/4; localparam adr_t SEGADDR_AOB  = SEGADDR_AIB  + SEGSIZE_AIB ;
    localparam adr_t SEGSIZE_RNGA = 'd256/4; localparam adr_t SEGADDR_RNGA = SEGADDR_AOB  + SEGSIZE_AOB ;
    localparam adr_t SEGSIZE_RNGB = 'd256/4; localparam adr_t SEGADDR_RNGB = SEGADDR_RNGA + SEGSIZE_RNGA;

    localparam SEGCNT = SEGID_RNGB+1;

    localparam segcfg_t [0:SEGCNT-1] SEGCFGS =
    '{
        '{ segid:SEGID_LKEY , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_LKEY , segsize:SEGSIZE_LKEY ,  isfifo:'1,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_KEY  , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_KEY  , segsize:SEGSIZE_KEY  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_SKEY , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_SKEY , segsize:SEGSIZE_SKEY ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_SCRT , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_SCRT , segsize:SEGSIZE_SCRT ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_MSG  , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_MSG  , segsize:SEGSIZE_MSG  ,  isfifo:'1,  isfifostream:'0,  fifoid: 'd1 },
        '{ segid:SEGID_HOUT , segtype:ST_BO,  ramsel:'d0,  segaddr:SEGADDR_HOUT , segsize:SEGSIZE_HOUT ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_SOB  , segtype:ST_SO,  ramsel:'d0,  segaddr:SEGADDR_SOB  , segsize:SEGSIZE_SOB  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_PIB  , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_PIB  , segsize:SEGSIZE_PIB  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_PSIB , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_PSIB , segsize:SEGSIZE_PSIB ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_POB  , segtype:ST_BO,  ramsel:'d0,  segaddr:SEGADDR_POB  , segsize:SEGSIZE_POB  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_PSOB , segtype:ST_SO,  ramsel:'d0,  segaddr:SEGADDR_PSOB , segsize:SEGSIZE_PSOB ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_AKEY , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_AKEY , segsize:SEGSIZE_AKEY ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
        '{ segid:SEGID_AIB  , segtype:ST_BI,  ramsel:'d0,  segaddr:SEGADDR_AIB  , segsize:SEGSIZE_AIB  ,  isfifo:'1,  isfifostream:'0,  fifoid: 'd2 },
        '{ segid:SEGID_AOB  , segtype:ST_SO,  ramsel:'d0,  segaddr:SEGADDR_AOB  , segsize:SEGSIZE_AOB  ,  isfifo:'1,  isfifostream:'0,  fifoid: 'd3 },
        '{ segid:SEGID_RNGA , segtype:ST_BO,  ramsel:'d0,  segaddr:SEGADDR_RNGA , segsize:SEGSIZE_RNGA ,  isfifo:'1,  isfifostream:'1,  fifoid: 'd4 },
        '{ segid:SEGID_RNGB , segtype:ST_BO,  ramsel:'d0,  segaddr:SEGADDR_RNGB , segsize:SEGSIZE_RNGB ,  isfifo:'1,  isfifostream:'1,  fifoid: 'd5 }
    };
    localparam FFCNT = 6;

    localparam segcfg_t SEG_LKEY = SEGCFGS[SEGID_LKEY];
    localparam segcfg_t SEG_KEY  = SEGCFGS[SEGID_KEY ];
    localparam segcfg_t SEG_SKEY = SEGCFGS[SEGID_SKEY];
    localparam segcfg_t SEG_SCRT = SEGCFGS[SEGID_SCRT];
    localparam segcfg_t SEG_MSG  = SEGCFGS[SEGID_MSG ];
    localparam segcfg_t SEG_HOUT = SEGCFGS[SEGID_HOUT];
    localparam segcfg_t SEG_SOB  = SEGCFGS[SEGID_SOB ];
    localparam segcfg_t SEG_PIB  = SEGCFGS[SEGID_PIB ];
    localparam segcfg_t SEG_PSIB = SEGCFGS[SEGID_PSIB];
    localparam segcfg_t SEG_POB  = SEGCFGS[SEGID_POB ];
    localparam segcfg_t SEG_PSOB = SEGCFGS[SEGID_PSOB];
    localparam segcfg_t SEG_AKEY = SEGCFGS[SEGID_AKEY];
    localparam segcfg_t SEG_AIB  = SEGCFGS[SEGID_AIB ];
    localparam segcfg_t SEG_AOB  = SEGCFGS[SEGID_AOB ];
    localparam segcfg_t SEG_RNGA = SEGCFGS[SEGID_RNGA];
    localparam segcfg_t SEG_RNGB = SEGCFGS[SEGID_RNGB];

    localparam CHNLACCNT = 8;
        // 0/1: ahb:   R/W,
        // 2/3: axi:   R/W
        // 4/5: axi-s: R/W
        // 6/7: ich:   R/W

    typedef struct packed {
        bit [7:0]   segid          ;
        bit [0:CHNLACCNT-1]
                    accessrule  ;   // should be 
    }accessrule_t;

    localparam accessrule_t [0:SEGCNT-1] ACRULEs =
    '{
        '{ segid:SEGID_LKEY , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_KEY  , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_SKEY , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_SCRT , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_MSG  , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_HOUT , accessrule: 8'b01_00_00_10 },
        '{ segid:SEGID_SOB  , accessrule: 8'b10_00_10_00 },
        '{ segid:SEGID_PIB  , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_PSIB , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_POB  , accessrule: 8'b00_00_00_10 },
        '{ segid:SEGID_PSOB , accessrule: 8'b10_00_10_00 },
        '{ segid:SEGID_AKEY , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_AIB  , accessrule: 8'b01_01_01_01 },
        '{ segid:SEGID_AOB  , accessrule: 8'b10_10_10_10 },
        '{ segid:SEGID_RNGA , accessrule: 8'b10_11_10_10 },
        '{ segid:SEGID_RNGB , accessrule: 8'b10_11_10_10 }
    };

    localparam sram_pkg::sramcfg_t [0:RAMCNT-1] SCERAMCFGS = {
    {
        AW: AW,
        DW: 32,
        KW: 32,
        PW: 4,
        WCNT: 1024*8,
        AWX: 5,
        isBWEN: '1,
        isSCMB: '1,
        isPRT:  '1,
        EVITVL:  15
    }
    };


endpackage : scedma_pkg
