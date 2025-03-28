
    localparam scedma_pkg::segcfg_t RSASEG_X      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h000, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t RSASEG_R1     = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h080, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t RSASEG_N      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h100, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t RSASEG_E      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h180, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t RSASEG_Y      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h200, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t RSASEG_H      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h300, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };

    localparam scedma_pkg::segcfg_t INVSEG_U      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: `d128#, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t INVSEG_P      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: `d128#, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t INVSEG_UT     = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: `d128#, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t INVSEG_PT     = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: `d128#, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t INVSEG_D      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: `d128#, isfifo:'0, isfifostream:0, fifoid:'0 };

    localparam scedma_pkg::segcfg_t ECCSEG_Q0X    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h000, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q0Y    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h200, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q0Z    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h024, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q0T    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q1X    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h012, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q1Y    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h212, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q1Z    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h224, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_Q1T    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h0##, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P0X    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h39e, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P0Y    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h19e, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P0Z    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h1d4, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P0T    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h3d4, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P1X    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h3c2, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P1Y    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h1c2, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P1Z    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h1e6, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P1T    = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h3e6, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_A      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h036, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_P      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h048, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_K      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h05a, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_U      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h18c, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_H      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h236, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_PT     = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h248, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_M      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h25a, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_CON1   = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h26c, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_INVU   = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h368, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t ECCSEG_UT     = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h38c, segsize: 'd144, isfifo:'0, isfifostream:0, fifoid:'0 };

    localparam scedma_pkg::segcfg_t GCDSEG_A      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h000, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t GCDSEG_B      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h200, segsize: 'd128, isfifo:'0, isfifostream:0, fifoid:'0 };
