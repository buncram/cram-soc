
package sce_pkg;



    typedef struct packed {
        coreuser_t  slotowner,
        key_t       keytype,
        acvflag_t   flag
    } acv_t;

    typedef struct packed {
        bit         core_rd,
        bit         core_wr,
        bit         sce_rd,
        bit         sce_wr,
        bit         admin,
        bit         reservedz
    } acvflag_t;





endpackage : scedma_pkg
