`include "template.sv"

//import hash_pkg::*;
import scedma_pkg::*;

module aes #(

        parameter RAW = 8,
        parameter ERRCNT = 8,
        parameter INTCNT = 8
    )(

    input  logic clk, resetn, cmsatpg, cmsbist,

    apbif.slavein           apbs,
    apbif.slave             apbx,    
    output  chnlreq_t       chnl_rpreq, chnl_wpreq   ,
    input   chnlres_t       chnl_rpres, chnl_wpres   ,

    input [31:0]    maskin,
    output logic [0:ERRCNT-1]      err,
    output logic [0:INTCNT-1]      intr
);
// localparam
// ■■■■■■■■■■■■■■■

    localparam [7:0] SEGID_AKEY = scedma_pkg::SEGID_AKEY ;
    localparam [7:0] SEGID_AIB  = scedma_pkg::SEGID_AIB  ;
    localparam [7:0] SEGID_AOB  = scedma_pkg::SEGID_AOB  ;

    localparam segcfg_t SEG_AKEY = scedma_pkg::SEGCFGS[SEGID_AKEY];
    localparam segcfg_t SEG_AIB  = scedma_pkg::SEGCFGS[SEGID_AIB ];
    localparam segcfg_t SEG_AOB  = scedma_pkg::SEGCFGS[SEGID_AOB ];

//  cr_func

//    localparam AF_INIT    = 8'hff ;
    localparam AF_KS      = 2'h0 ;
    localparam AF_ENC     = 2'h1 ;
    localparam AF_DEC     = 2'h2 ;

    localparam MFSM_IDLE      = 8'h00;
    localparam MFSM_DONE      = 8'hff;
    localparam MFSM_AF        = 8'h01;
    localparam MFSM_LD_K      = 8'h10;
    localparam MFSM_LD_D      = 8'h11;
    localparam MFSM_LD_IV     = 8'h12;
    localparam MFSM_WB_D      = 8'h30;

    localparam scedma_pkg::segcfg_t AESSEG_K      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h08, segsize: 'd8, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t AESSEG_I      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h00, segsize: 'd4, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t AESSEG_O      = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h44, segsize: 'd4, isfifo:'0, isfifostream:0, fifoid:'0 };
    localparam scedma_pkg::segcfg_t AESSEG_IV     = '{ segid:'0, segtype:ST_NONE, ramsel:'0, segaddr: 'h04, segsize: 'd4, isfifo:'0, isfifostream:0, fifoid:'0 };

// opt
    localparam MODE_ECB = 3'h0;
    localparam MODE_CBC = 3'h1;
    localparam MODE_CTR = 3'h2;
    localparam MODE_CFB = 3'h3;
    localparam MODE_OFB = 3'h4;


// typedef
// ■■■■■■■■■■■■■■■

    bit [7:0]   cr_func;
//    bit [###]   cr_opt;

    logic [7:0] mfsm, mfsmnext;
    bit mfsmtog, mfsmdone, mfsm_done;
    logic acore_start, chnli_start, chnlo_start;
    logic acore_busy,  chnli_busy, chnlo_busy;
    logic acore_done,  chnli_done, chnlo_done;
    logic [1:0] ramerror;
    logic acore_en, chnli_en, chnlo_en;
    chnlreq_t chnlo_rpreq, chnli_wpreq;
    chnlres_t chnlo_rpres, chnli_wpres, aramres;
    chnlcfg_t chnli_cfg, chnlo_cfg;
    bit [7:0] chnlo_intr, chnli_intr;
    bit optlock;

    logic [RAW-1:0] chnl_ramadd, chnl_segptr;
    logic  chnl_ramrd, chnl_ramwr;
    dat_t  chnl_ramwdat;
    logic [31:0] ramwdat, ramrdat, acore_ramrdat, acore_ramwdat;
    logic [3:0] chnl_ramwrs, ramwr;
    logic [RAW-1:0]  ramadd, acore_ramadd;
    logic ramrd, acore_ramrd, acore_ramwr;
    logic ramready;
    
    logic [8:0] opt_d32, opt_key32;
    bit [3:0] opt_mode, opt_klen;
    bit start, busy, done;
    bit [0:3][31:0] opt_iv;
    adr_t [0:3] cr_segptrstart;
    logic opt_mode_ofb;
    logic [15:0] opt_aescnt;
// apb
// ■■■■■■■■■■■■■■■

    logic apbrd, apbwr;
    logic pclk;
    logic sfrlock;
    assign pclk = clk;

    `theregrn( sfrlock ) <= optlock ? 1'b1 : mfsm_done ? '0 : sfrlock;

    `apbs_common;
    assign apbx.prdata = '0
                        | sfr_crfunc.prdata32 | sfr_srmfsm.prdata32 | sfr_fr.prdata32
                        | sfr_opt.prdata32 | sfr_opt1.prdata32 | sfr_segptr.prdata32
                        ;

    apb_cr #(.A('h00), .DW(8))      sfr_crfunc      (.cr(cr_func), .prdata32(),.*);
    apb_ar #(.A('h04), .AR(32'h5a)) sfr_ar          (.ar(start),.*);
    apb_sr #(.A('h08), .DW(8))      sfr_srmfsm      (.sr(mfsm), .prdata32(),.*);
    apb_fr #(.A('h0c), .DW(4))      sfr_fr          (.fr({chnli_done, chnlo_done, acore_done, mfsm_done}), .prdata32(),.*);

    apb_cr #(.A('h10), .DW(8))       sfr_opt        (.cr({opt_mode,opt_klen}), .prdata32(),.*); //## width?
    apb_cr #(.A('h10), .DW(16))      sfr_opt1       (.cr(opt_aescnt), .prdata32(),.*); 

//    apb_cr #(.A('h14), .DW(32), .SFRCNT(4))      sfr_optiv      (.cr(opt_iv), .prdata32(),.*); //## width?

    apb_cr #(.A('h30), .DW(scedma_pkg::AW), .SFRCNT(4) )      sfr_segptr    (.cr(cr_segptrstart), .prdata32(),.*); //## width?

    assign optlock = ( start & ( mfsm == MFSM_IDLE));
    assign opt_mode_ofb = ( opt_mode == MODE_OFB );

// mfsm
// ■■■■■■■■■■■■■■■
    `theregfull(clk, resetn, mfsm, MFSM_IDLE ) <= ( start & ( mfsm == MFSM_IDLE)) | mfsmdone ? mfsmnext : mfsm;
    `theregrn( mfsmtog ) <= ( mfsm != mfsmnext ) & ( mfsmnext != MFSM_IDLE );
    assign mfsm_done = ( mfsm == MFSM_DONE );

    always_comb begin
        mfsmnext = mfsm;
        mfsmdone = '0;
        case (cr_func)
            AF_KS:
                case( mfsm )
                    MFSM_IDLE:
                                                mfsmnext = MFSM_LD_K;
                    MFSM_LD_K : begin
                                                mfsmnext = MFSM_AF;         mfsmdone = chnli_done;
                        end
                    MFSM_AF : begin
                                                mfsmnext = MFSM_DONE;       mfsmdone = acore_done;
                        end
                endcase
            AF_ENC, AF_DEC:
                case( mfsm )
                    MFSM_IDLE:
                                                mfsmnext = MFSM_LD_IV;
                    MFSM_LD_IV : begin
                                                mfsmnext = MFSM_LD_D;       mfsmdone = chnli_done;
                        end
                    MFSM_LD_D : begin
                                                mfsmnext = MFSM_AF;         mfsmdone = chnli_done;
                        end
                    MFSM_AF : begin
                                                mfsmnext = MFSM_WB_D;       mfsmdone = acore_done;
                        end
                    MFSM_WB_D : begin
                                                mfsmnext = MFSM_DONE;       mfsmdone = chnlo_done;
                        end
                endcase
            default : 
                begin
                    mfsmnext = mfsm;
                    mfsmdone = '1;
                end
        endcase 
    end

// subcore, chnl
// ■■■■■■■■■■■■■■■

    `theregrn( acore_start ) <= mfsmtog & ( mfsm == MFSM_AF  );

    assign acore_en = ( mfsm == MFSM_AF );

    logic [31:0] acore_ramwdat1, acore_ramwdat0;

    AesCore acore(
            .Clk              (clk),
            .Resetn           (resetn),
            .StartAes         (acore_start),
            .AesDone          (acore_done),
            .AesIR            (cr_func[1:0]),
            .AesLen           (opt_klen[1:0]),
            .AesMode          (opt_mode[2:0]),

//        	.IVector0 		  (opt_iv[0]),
//        	.IVector1 		  (opt_iv[1]),
//        	.IVector2 		  (opt_iv[2]),
//        	.IVector3 		  (opt_iv[3]),

            .AesRamRd         (acore_ramrd),
            .AesRamWr         (acore_ramwr),
            .AesRamAdr        (acore_ramadd),
            .AesRamDat        (acore_ramwdat),
//            .AesRamDat1       (acore_ramwdat1),
            .RamAesDat        (acore_ramrdat),
            //##

            .MaskIn           (maskin)
		);

//    assign acore_ramwdat = opt_mode_ofb ? acore_ramwdat1 : acore_ramwdat0;


    // 1KB x 1, 32bit data, 7 bit addr .

// chnl behavior
// ■■■■■■■■■■■■■■■

    `theregrn( chnli_start ) <= mfsmtog & chnli_en;
    `theregrn( chnlo_start ) <= mfsmtog & chnlo_en;

    localparam PTRID_IV   = 0;
    localparam PTRID_AKEY = 1;
    localparam PTRID_AIB  = 2;
    localparam PTRID_AOB  = 3;

	assign opt_key32 = ( opt_klen == 0 ) ? 4 : ( opt_klen == 1 ) ? 6 : 8;
	assign opt_d32 = 4;
    always_comb begin
        chnli_en = '0;
        case(mfsm)
            MFSM_LD_K      :  
                begin
                    chnli_cfg.wpsegcfg = AESSEG_K;
			        chnli_cfg.rpptr_start = cr_segptrstart[PTRID_AKEY];
			        chnli_cfg.rpsegcfg = SEG_AKEY;
			        chnli_cfg.transsize = opt_key32;
                    chnli_en = '1;
                end
            MFSM_LD_D      :   
                begin
                    chnli_cfg.wpsegcfg = AESSEG_I;
			        chnli_cfg.rpptr_start = cr_segptrstart[PTRID_AIB];
			        chnli_cfg.rpsegcfg = SEG_AIB;
			        chnli_cfg.transsize = opt_d32;
                    chnli_en = '1;
                end
            MFSM_LD_IV      :   
                begin
                    chnli_cfg.wpsegcfg = AESSEG_IV;
                    chnli_cfg.rpptr_start = cr_segptrstart[PTRID_IV];
                    chnli_cfg.rpsegcfg = SEG_AKEY;
                    chnli_cfg.transsize = opt_d32;
                    chnli_en = '1;
                end
            default : /* default */
                begin
                    chnli_cfg.wpsegcfg = AESSEG_I;
			        chnli_cfg.rpptr_start = cr_segptrstart[PTRID_AIB];
			        chnli_cfg.rpsegcfg = SEG_AIB;
			        chnli_cfg.transsize = opt_d32;
                    chnli_en = '0;
                end
        endcase
    end

    assign chnli_cfg.chnlid = '0;
    assign chnli_cfg.wpptr_start = '0;
    assign chnli_cfg.opt_ltx = '0;
    assign chnli_cfg.opt_xor = '0;
    assign chnli_cfg.opt_cmpp = '0;
    assign chnli_cfg.opt_prm = '0;
    assign chnli_cfg.wpffen = '0;

    always_comb begin
        case(mfsm)
            MFSM_WB_D:
                begin
                    chnlo_cfg.rpsegcfg = AESSEG_O;
                    chnlo_cfg.wpsegcfg = SEG_AOB;
                    chnlo_cfg.wpptr_start = cr_segptrstart[PTRID_AOB];
                    chnlo_cfg.transsize = opt_d32;
                    chnlo_en = '1;
                end
            default : /* default */
                begin
                    chnlo_cfg.rpsegcfg = AESSEG_O;
                    chnlo_cfg.wpsegcfg = SEG_AOB;
                    chnlo_cfg.wpptr_start = cr_segptrstart[PTRID_AOB];
                    chnlo_cfg.transsize = opt_d32;
                    chnlo_en = '0;
                end
        endcase
    end

    assign chnlo_cfg.chnlid = '0;
    assign chnlo_cfg.rpptr_start = '0;
    assign chnlo_cfg.opt_ltx = '0;
    assign chnlo_cfg.opt_xor = '0;
    assign chnlo_cfg.opt_cmpp = '0;
    assign chnlo_cfg.opt_prm = '0;
    assign chnlo_cfg.wpffen = '0;

// chnl instance
// ■■■■■■■■■■■■■■■

    scedma_chnl chnli(
        .clk,
        .resetn,
        .thecfg   (chnli_cfg   ),
        .start    (chnli_start ),
        .busy     (chnli_busy  ),
        .done     (chnli_done  ),
        .rpreq    (chnl_rpreq  ), 
        .rpres    (chnl_rpres  ),
        .wpreq    (chnli_wpreq ), 
        .wpres    (chnli_wpres ),
        .intr     (chnli_intr  )
    );

    scedma_chnl chnlo(
        .clk,
        .resetn,
        .thecfg   (chnlo_cfg   ),
        .start    (chnlo_start ),
        .busy     (chnlo_busy  ),
        .done     (chnlo_done  ),
        .rpreq    (chnlo_rpreq ),
        .rpres    (chnlo_rpres ),
        .wpreq    (chnl_wpreq  ),
        .wpres    (chnl_wpres  ),
        .intr     (chnlo_intr  )
    );

// mem data path
// ■■■■■■■■■■■■■■■

    assign chnl_ramadd = chnlo_en ?  chnlo_rpreq.segaddr + chnlo_rpreq.segptr  : 
                                     chnli_wpreq.segaddr + chnli_wpreq.segptr  ; //chnli_en ?

    assign chnl_segptr = chnlo_en ?  chnlo_rpreq.segptr  : 
                                     chnli_wpreq.segptr  ; //chnli_en ?

    assign chnl_ramrd  = chnli_wpreq.segrd | chnlo_rpreq.segrd;
    assign chnl_ramwr  = chnli_wpreq.segwr | chnlo_rpreq.segwr;
    assign chnl_ramwrs = {4{chnl_ramwr}};

    assign chnl_ramwdat = chnlo_en ?  chnlo_rpreq.segwdat  : 
                                      chnli_wpreq.segwdat  ; //chnli_en ?

    assign ramrd = acore_en ? acore_ramrd : chnl_ramrd;
    assign ramwr = acore_en ? {4{acore_ramwr}} : chnl_ramwr;
    assign ramadd = acore_en ? acore_ramadd : chnl_ramadd;
    assign ramwdat = acore_en ? acore_ramwdat : { chnl_ramwdat, chnl_ramwdat };

    assign chnli_wpres = aramres;
    assign chnlo_rpres = aramres;

    assign acore_ramrdat = ramrdat;
    assign aramres.segready = '1;  // pram always ready
    assign aramres.segrdat  = ramrdat ;
    assign aramres.segrdatvld = '1;

// mem
// ■■■■■■■■■■■■■■■

    localparam sramcfg_t thecfg = {
        AW: RAW,
        DW: 32,
        KW: 32,
        PW: 4,
        WCNT: 256,
        AWX: 5,
        isBWEN: '1,
        isSCMB: '1,
        isPRT:  '1,
        EVITVL:  15
    };

    cryptoram #(
        .ramname    ("ARAM"), // HRAM, PRAM, ARAM, SCERAM
        .thecfg     (thecfg)
    )m(
        .clk, .resetn, .cmsatpg, .cmsbist,
        .ramaddr (ramadd[RAW-1:0] ),
        .ramen('1),
        .ramrd(ramrd),
        .ramwr(ramwr),
        .ramwdat(ramwdat),
        .ramrdat(ramrdat),
        .ramready(ramready),
        .ramerror(ramerror)
    );

// err/intr
// ■■■■■■■■■■■■■■■

    `theregrn( intr[0] ) <= ( mfsm == MFSM_DONE );
    `theregrn( err[0:1] ) <= ramerror;

endmodule

module dummytb_aes ();

    logic clk, resetn, cmsatpg, cmsbist;
    apbif apbs();
    chnlreq_t       chnl_rpreq, chnl_wpreq;
    chnlres_t       chnl_rpres, chnl_wpres;
    logic [0:8-1]      err;
    logic [0:8-1]      intr;
    logic [31:0] maskin;
    aes u1(
        .apbs(apbs),
        .apbx(apbs),
        .*
    );



endmodule

