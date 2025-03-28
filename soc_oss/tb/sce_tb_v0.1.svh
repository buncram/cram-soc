

    localparam [AW-1:0] SCERAM_BA = 16'h000 ;

    localparam [AW-1:0] GLB_scemode = 16'h000 + 16'h8000;
    localparam [AW-1:0] GLB_suben   = 16'h004 + 16'h8000;
    localparam [AW-1:0] GLB_ahbs    = 16'h008 + 16'h8000;
    localparam [AW-1:0] GLB_srbusy  = 16'h010 + 16'h8000;
    localparam [AW-1:0] GLB_frdone  = 16'h014 + 16'h8000;
    localparam [AW-1:0] GLB_frerr   = 16'h018 + 16'h8000;
    localparam [AW-1:0] GLB_ar      = 16'h01c + 16'h8000;
    localparam [AW-1:0] GLB_ffen    = 16'h030 + 16'h8000;
    localparam [AW-1:0] GLB_ffclr   = 16'h034 + 16'h8000;
    localparam [AW-1:0] GLB_ffcnt0  = 16'h040 + 16'h8000;
    localparam [AW-1:0] GLB_ffcnt1  = 16'h044 + 16'h8000;
    localparam [AW-1:0] GLB_ffcnt2  = 16'h048 + 16'h8000;
    localparam [AW-1:0] GLB_ffcnt3  = 16'h04c + 16'h8000;
    localparam [AW-1:0] GLB_ffcnt4  = 16'h050 + 16'h8000;
    localparam [AW-1:0] GLB_ffcnt5  = 16'h054 + 16'h8000;
    localparam [AW-1:0] GLB_tickcyc = 16'h020 + 16'h8000;
    localparam [AW-1:0] GLB_tickcnt = 16'h024 + 16'h8000;

    localparam [AW-1:0 ] SDMA_chstart_ar      = 16'h9000;
    localparam [AW-1:0 ] SDMA_xchcr_func      = 16'h9010;
    localparam [AW-1:0 ] SDMA_xchcr_opt       = 16'h9014;
    localparam [AW-1:0 ] SDMA_xchcr_axstart   = 16'h9018;
    localparam [AW-1:0 ] SDMA_xchcr_segid     = 16'h901c;
    localparam [AW-1:0 ] SDMA_xchcr_segstart  = 16'h9020;
    localparam [AW-1:0 ] SDMA_xchcr_transize  = 16'h9024;
    localparam [AW-1:0 ] SDMA_schcr_func      = 16'h9030;
    localparam [AW-1:0 ] SDMA_schcr_opt       = 16'h9034;
    localparam [AW-1:0 ] SDMA_schcr_axstart   = 16'h9038;
    localparam [AW-1:0 ] SDMA_schcr_segid     = 16'h903c;
    localparam [AW-1:0 ] SDMA_schcr_segstart  = 16'h9040;
    localparam [AW-1:0 ] SDMA_schcr_transize  = 16'h9044;
    localparam [AW-1:0 ] SDMA_ichcr_opt       = 16'h9050;
    localparam [AW-1:0 ] SDMA_ichcr_segid     = 16'h9054;
    localparam [AW-1:0 ] SDMA_ichcr_rpstart   = 16'h9058;
    localparam [AW-1:0 ] SDMA_ichcr_wpstart   = 16'h905c;
    localparam [AW-1:0 ] SDMA_ichcr_transize  = 16'h9060;

    localparam [AW-1:0] HASH_crfunc      = 16'hb000;// + 16'hc000;
    localparam [AW-1:0] HASH_ar          = 16'hb004;// + 16'hc000;
    localparam [AW-1:0] HASH_srmfsm      = 16'hb008;// + 16'hc000;
    localparam [AW-1:0] HASH_fr          = 16'hb00c;// + 16'hc000;
    localparam [AW-1:0] HASH_opt1        = 16'hb010;// + 16'hc000;
    localparam [AW-1:0] HASH_opt2        = 16'hb014;// + 16'hc000;
    localparam [AW-1:0] HASH_opt3        = 16'hb018;// + 16'hc000;
    localparam [AW-1:0] HASH_optblk      = 16'hb01C;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_LKEY = 16'hb020;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_KEY  = 16'hb024;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_SKEY = 16'hb028;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_SCRT = 16'hb02c;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_MSG  = 16'hb030;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_HOUT = 16'hb034;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_SOB  = 16'hb038;// + 16'hc000;
    localparam [AW-1:0] HASH_segptr_HOUT2 = 16'hb03C;// + 16'hc000;

    localparam [AW-1:0] AES_crfunc       = 16'hd000;
    localparam [AW-1:0] AES_ar           = 16'hd004;
    localparam [AW-1:0] AES_srmfsm       = 16'hd008;
    localparam [AW-1:0] AES_fr           = 16'hd00c;
    localparam [AW-1:0] AES_opt          = 16'hd010;
    localparam [AW-1:0] AES_opt1         = 16'hd014;
    localparam [AW-1:0] AES_optltx       = 16'hd018;
    localparam [AW-1:0] AES_segptr_IV    = 16'hd030;
    localparam [AW-1:0] AES_segptr_AKEY  = 16'hd034;
    localparam [AW-1:0] AES_segptr_AIB   = 16'hd038;
    localparam [AW-1:0] AES_segptr_AOB   = 16'hd03c;

    localparam [AW-1:0] PKE_crfunc       = 16'hc000;
    localparam [AW-1:0] PKE_ar           = 16'hc004;
    localparam [AW-1:0] PKE_sr           = 16'hc008;
    localparam [AW-1:0] PKE_fr           = 16'hc00c;
    localparam [AW-1:0] PKE_optnw        = 16'hc010;
    localparam [AW-1:0] PKE_optew        = 16'hc014;
    localparam [AW-1:0] PKE_optrw        = 16'hc018;
    localparam [AW-1:0] PKE_optltx       = 16'hc01c;
    localparam [AW-1:0] PKE_optmask      = 16'hc020;
    localparam [AW-1:0] PKE_segptr_PCON  = 16'hc030;
    localparam [AW-1:0] PKE_segptr_PIB0  = 16'hc034;
    localparam [AW-1:0] PKE_segptr_PIB1  = 16'hc038;
    localparam [AW-1:0] PKE_segptr_PKB   = 16'hc03c;
    localparam [AW-1:0] PKE_segptr_POB   = 16'hc040;

    localparam [AW-1:0] RNG_sfr_crsrc    = 16'he000;
    localparam [AW-1:0] RNG_sfr_crana    = 16'he004;
    localparam [AW-1:0] RNG_sfr_pp       = 16'he008;
    localparam [AW-1:0] RNG_sfr_opt      = 16'he00c;
    localparam [AW-1:0] RNG_sfr_ar       = 16'he014;
    localparam [AW-1:0] RNG_sfr_sr       = 16'he010;
    localparam [AW-1:0] RNG_sfr_fr       = 16'he018;
    localparam [AW-1:0] RNG_sfr_drpsz    = 16'he020;
    localparam [AW-1:0] RNG_sfr_drgen    = 16'he024;
    localparam [AW-1:0] RNG_sfr_drreseed = 16'he028;
    localparam [AW-1:0] RNG_sfr_buf      = 16'he030;
    localparam [AW-1:0] RNG_sfr_chain0   = 16'he040;
    localparam [AW-1:0] RNG_sfr_chain1   = 16'he044;





//        '{ segid:SEGID_PCON , segtype:ST_BI,  ramsel: '0/*'d1*/,  segaddr:SEGADDR_PCON , segsize:SEGSIZE_PCON ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
//        '{ segid:SEGID_PKB  , segtype:ST_BI,  ramsel: '0/*'d1*/,  segaddr:SEGADDR_PKB  , segsize:SEGSIZE_PKB  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
//        '{ segid:SEGID_PIB  , segtype:ST_BI,  ramsel: '0/*'d1*/,  segaddr:SEGADDR_PIB  , segsize:SEGSIZE_PIB  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
//        '{ segid:SEGID_PSIB , segtype:ST_BI,  ramsel: '0/*'d1*/,  segaddr:SEGADDR_PSIB , segsize:SEGSIZE_PSIB ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
//        '{ segid:SEGID_POB  , segtype:ST_BO,  ramsel: '0/*'d1*/,  segaddr:SEGADDR_POB  , segsize:SEGSIZE_POB  ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
//        '{ segid:SEGID_PSOB , segtype:ST_SO,  ramsel: '0/*'d1*/,  segaddr:SEGADDR_PSOB , segsize:SEGSIZE_PSOB ,  isfifo:'0,  isfifostream:'0,  fifoid: 'd0 },
