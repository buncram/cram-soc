`include "template.sv"

module sce_tb_mase();

    parameter COREUSERCNT = 8;
    parameter type coreuser_t = bit[0:COREUSERCNT-1];
    parameter INTC = 8;
    parameter ERRC = 8;
    localparam AW = 16;
    localparam IDW = 8;
    localparam DW = 32;
    localparam UW = 4;

    `include "../tb/sce_tb_v0.1.svh"

    bit clk;
    bit resetn;
    bit cmsatpg, cmsbist;
    coreuser_t   coreuser=8;
    coreuser_t   sceuser;
    bit        secmode;
    ahbif ahbs();
    axiif axim[0:1]();
    bit [INTC-1:0] intr;
    bit [ERRC-1:0] err;

    integer j=0, k=0, errcnt=0, warncnt=0;


	bit            hsel;
	bit  [AW-1:0]  haddr;
	bit  [1:0]     htrans;
	bit            hwrite;
	bit  [2:0]     hsize;
	bit  [2:0]     hburst;
	bit  [3:0]     hprot;
	bit  [IDW-1:0] hmaster;
	bit   [DW-1:0]  hwdata;
	bit            hmasterlock;
	bit            hreadym=1;
	bit  [UW-1:0]  hauser;
	bit  [UW-1:0]  hwuser;
	bit   [DW-1:0]  hrdata;
	bit            hready;
	bit            hresp;
	logic  [UW-1:0]  hruser;
	bit hdataphase;
	bit [DW-1:0] hrdatareg;
	bit hclk;
    bit [DW-1:0] membuf[0:511];
    bit [0:31][DW-1:0] refbuf;

    bit [DW-1:0] membufx[0:511];
    bit [0:31][DW-1:0] refbufx;

    bit [DW-1:0] le_membuf[0:511]; // little endian membuf

    bit probe_pkeram_18d;
    bit probe_pkeram_14a;
    bit [15:0]  truststate;
    logic ana_rng_0p1u;

  //
  //  dut
  //  ==

    sce
    dut
    (
        .clk    (clk),
        .clktop (clk),
        .clksceen('1),
//        .clkpke(clk),
        .clkpkeen('1),
        .resetn (resetn),
		.*
    );

    assign probe_pkeram_18d = (sce_tb_mase.dut.pke.ramadd0[8:0] == 'h18d);

  assign probe_pkeram_14a = (sce_tb_mase.dut.pke.ramadd0[8:0] == 'h14a);
    //assign probe_pkeram_14c = (sce_tb_mase.dut.pke.ramadd0[8:0] == 'h18c) && ;

    wire2ahbm ahbdrv(.ahbm(ahbs),.*);

  //
  //  monitor and clk
  //  ==

    `genclk( clk, 100 )
    `timemarker2
    assign hclk = clk;

  //
  //  subtitle
  //  ==

    initial begin
      #(1000 `MS);
    `maintestend

    `maintest(sce_tb_mase,sce_tb_mase)
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );

        #( 500 `US );

        membuf[0:3] = { 32'h0123_4567, 32'h89ab_cdef, 32'h76543210, 32'hfedc_ba98 };

        memwr('0, 4);
        memrd('0, 4);

// init hash
/*
        $readmemh("../tb/hashram.dua", membuf);
        memwr('0, 512);
        memrd('0, 512);


        sfrwr(GLB_suben, 32'hb );

        sfrwr(HASH_crfunc, 32'hff); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
*/



// blk2s
    /*
        membuf[ 0  ] = 32'h6b08e647;//32'h6a09e667 ;
        membuf[ 1  ] = 32'hbb67ae85 ;
        membuf[ 2  ] = 32'h3c6ef372 ;
        membuf[ 3  ] = 32'ha54ff53a ;
        membuf[ 4  ] = 32'h510e527f ;
        membuf[ 5  ] = 32'h9b05688c ;
        membuf[ 6  ] = 32'h1f83d9ab ;
        membuf[ 7  ] = 32'h5be0cd19 ;
        membuf[ 8  ] = 32'h0 ;
        membuf[ 9  ] = 32'h0 ;
        membuf[ 10 ] = 32'h0 ;
        membuf[ 11 ] = 32'h0 ;
        membuf[ 12 ] = 32'h0 ^ 32'h3;
        membuf[ 13 ] = 32'h0 ^ 32'h0;
        membuf[ 14 ] = '1 ;
        membuf[ 15 ] = 32'h0 ;
        memwr(SEGADDR_HOUT, 16);

        membuf[ 0:15 ] = {32'h00636261,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};

        memwr(SEGADDR_MSG, 16);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'h80);
    //   sfrwr(HASH_optblk, 'h01234567);
        sfrwr(HASH_segptr_HOUT2, 'h10 );
        sfrwr(HASH_crfunc, 32'h03); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
    //  memrd(SEGADDR_HOUT+16,8);
        memrd(SEGADDR_HOUT+16,8);
        refbuf[0:7] =  256'h508c5e8c327c14e2_e1a72ba34eeb452f_37458b209ed63a29_4d999b4c86675982;
        checkref(8);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'h80);
        sfrwr(HASH_optblk, 'h01234567);
        sfrwr(HASH_crfunc, 32'h03); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT+16,8);
        refbuf[0:7] =  256'h508c5e8c327c14e2_e1a72ba34eeb452f_37458b209ed63a29_4d999b4c86675982;
        checkref(8);

        sfrwr(HASH_opt3, 'h00);
        sfrwr(HASH_segptr_HOUT2, 'h00 );


        $display("\n@I:: BLAKE2S little endian test");
        membuf[ 0  ] = 32'h6b08e647;//32'h6a09e667 ;
        membuf[ 1  ] = 32'hbb67ae85 ;
        membuf[ 2  ] = 32'h3c6ef372 ;
        membuf[ 3  ] = 32'ha54ff53a ;
        membuf[ 4  ] = 32'h510e527f ;
        membuf[ 5  ] = 32'h9b05688c ;
        membuf[ 6  ] = 32'h1f83d9ab ;
        membuf[ 7  ] = 32'h5be0cd19 ;
        membuf[ 8  ] = 32'h0 ;
        membuf[ 9  ] = 32'h0 ;
        membuf[ 10 ] = 32'h0 ;
        membuf[ 11 ] = 32'h0 ;
        membuf[ 12 ] = 32'h0 ^ 32'h3;
        membuf[ 13 ] = 32'h0 ^ 32'h0;
        membuf[ 14 ] = '1 ;
        membuf[ 15 ] = 32'h0 ;
        big_to_little();
        memwr(SEGADDR_HOUT, 16);

        membuf[ 0:15 ] = {32'h00636261,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    big_to_little();
        memwr(SEGADDR_MSG, 16);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'hb0);
        //sfrwr(HASH_optblk, 'h01234567);
        sfrwr(HASH_segptr_HOUT2, 'h10 );
        sfrwr(HASH_crfunc, 32'h03); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        //memrd(SEGADDR_HOUT+16,8);
        memrd(SEGADDR_HOUT+16,8);
        refbuf[0:7] =  256'h508c5e8c327c14e2_e1a72ba34eeb452f_37458b209ed63a29_4d999b4c86675982;
        checkref(8);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'hb0);
        sfrwr(HASH_optblk, 'h01234567);
        sfrwr(HASH_crfunc, 32'h03); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT+16,8);
        refbuf[0:7] =  256'h508c5e8c327c14e2_e1a72ba34eeb452f_37458b209ed63a29_4d999b4c86675982;
        checkref(8);

        sfrwr(HASH_opt3, 'h00);
        sfrwr(HASH_segptr_HOUT2, 'h00 );

        #( 10 `US);

        */

    //        $finish;
    //        refbuf[31:0] =  512'h0d4d1c983fa580ba_e9f6129fb697276a_b7c45a68142f214c_d1a2ffdb6fbb124b_2d79ab2a39c5877d_95cc3345ded552c2_5a92f1dba88ad318_239900d4ed8623b9;
    //508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c86675982
 //   blk2b
    /*

            membuf[ 0  ] = 32'hf2bdc948;//32'hF3BCC908;
            membuf[ 1  ] = 32'h6a09e667;//32'h6A09E667;
            membuf[ 2  ] = 32'h84CAA73B;
            membuf[ 3  ] = 32'hBB67AE85;
            membuf[ 4  ] = 32'hFE94F82B;
            membuf[ 5  ] = 32'h3C6EF372;
            membuf[ 6  ] = 32'h5F1D36F1;
            membuf[ 7  ] = 32'hA54FF53A;
            membuf[ 8  ] = 32'hADE682D1;
            membuf[ 9  ] = 32'h510E527F;
            membuf[ 10 ] = 32'h2B3E6C1F;
            membuf[ 11 ] = 32'h9B05688C;
            membuf[ 12 ] = 32'hFB41BD6B;
            membuf[ 13 ] = 32'h1F83D9AB;
            membuf[ 14 ] = 32'h137E2179;
            membuf[ 15 ] = 32'h5BE0CD19;
            membuf[ 16 + 0  ] =  32'h0;
            membuf[ 16 + 1  ] =  32'h0;
            membuf[ 16 + 2  ] =  32'h0;
            membuf[ 16 + 3  ] =  32'h0;
            membuf[ 16 + 4  ] =  32'h0;
            membuf[ 16 + 5  ] =  32'h0;
            membuf[ 16 + 6  ] =  32'h0;
            membuf[ 16 + 7  ] =  32'h0;
            membuf[ 16 + 8  ] =  32'h3;//t0
            membuf[ 16 + 9  ] =  32'h0;
            membuf[ 16 + 10 ] =  32'h0;
            membuf[ 16 + 11 ] =  32'h0;
            membuf[ 16 + 12 ] =  '1;
            membuf[ 16 + 13 ] =  '1;
            membuf[ 16 + 14 ] =  32'h0;
            membuf[ 16 + 15 ] =  32'h0;
            memwr(SEGADDR_HOUT, 32);


        membuf[ 0:15 ] = {32'h00636261,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[ 16:31 ] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
            memwr(SEGADDR_MSG, 32);

            // membuf[ 0  ] = 32'h6a09e667;//32'h6A09E667;
            // membuf[ 1  ] = 32'hf2bdc948;//32'hF3BCC908;
            // membuf[ 2  ] = 32'hBB67AE85;
            // membuf[ 3  ] = 32'h84CAA73B;
            // membuf[ 4  ] = 32'h3C6EF372;
            // membuf[ 5  ] = 32'hFE94F82B;
            // membuf[ 6  ] = 32'hA54FF53A;
            // membuf[ 7  ] = 32'h5F1D36F1;
            // membuf[ 8  ] = 32'h510E527F;
            // membuf[ 9  ] = 32'hADE682D1;
            // membuf[ 10 ] = 32'h9B05688C;
            // membuf[ 11 ] = 32'h2B3E6C1F;
            // membuf[ 12 ] = 32'h1F83D9AB;
            // membuf[ 13 ] = 32'hFB41BD6B;
            // membuf[ 14 ] = 32'h5BE0CD19;
            // membuf[ 15 ] = 32'h137E2179;
            // membuf[ 16 + 0  ] =  32'h0;
            // membuf[ 16 + 1  ] =  32'h0;
            // membuf[ 16 + 2  ] =  32'h0;
            // membuf[ 16 + 3  ] =  32'h0;
            // membuf[ 16 + 4  ] =  32'h0;
            // membuf[ 16 + 5  ] =  32'h0;
            // membuf[ 16 + 6  ] =  32'h0;
            // membuf[ 16 + 7  ] =  32'h0;
            // membuf[ 16 + 8  ] =  32'h0;
            // membuf[ 16 + 9  ] =  32'h3;//t0
            // membuf[ 16 + 10 ] =  32'h0;
            // membuf[ 16 + 11 ] =  32'h0;
            // membuf[ 16 + 12 ] =  '1;
            // membuf[ 16 + 13 ] =  '1;
            // membuf[ 16 + 14 ] =  32'h0;
            // membuf[ 16 + 15 ] =  32'h0;
            // memwr(SEGADDR_HOUT, 32);

            // membuf[ 0  ] = 32'h0;
            // membuf[ 1  ] = 32'h00636261;
            // membuf[ 2  ] = 32'h0;
            // membuf[ 3  ] = 32'h0;
            // membuf[ 4  ] = 32'h0;
            // membuf[ 5  ] = 32'h0;
            // membuf[ 6  ] = 32'h0;
            // membuf[ 7  ] = 32'h0;
            // membuf[ 8  ] = 32'h0;
            // membuf[ 9  ] = 32'h0;
            // membuf[ 10 ] = 32'h0;
            // membuf[ 11 ] = 32'h0;
            // membuf[ 12 ] = 32'h0;
            // membuf[ 13 ] = 32'h0;
            // membuf[ 14 ] = 32'h0;
            // membuf[ 15 ] = 32'h0;
            // membuf[ 16 + 0  ] =  32'h0;
            // membuf[ 16 + 1  ] =  32'h0;
            // membuf[ 16 + 2  ] =  32'h0;
            // membuf[ 16 + 3  ] =  32'h0;
            // membuf[ 16 + 4  ] =  32'h0;
            // membuf[ 16 + 5  ] =  32'h0;
            // membuf[ 16 + 6  ] =  32'h0;
            // membuf[ 16 + 7  ] =  32'h0;
            // membuf[ 16 + 8  ] =  32'h0;
            // membuf[ 16 + 9  ] =  32'h0;
            // membuf[ 16 + 10 ] =  32'h0;
            // membuf[ 16 + 11 ] =  32'h0;
            // membuf[ 16 + 12 ] =  32'h0;
            // membuf[ 16 + 13 ] =  32'h0;
            // membuf[ 16 + 14 ] =  32'h0;
            // membuf[ 16 + 15 ] =  32'h0;
            // memwr(SEGADDR_MSG, 32);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'h80);
        sfrwr(HASH_crfunc, 32'h04); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,16);
        refbuf[0:15] =  512'h0d4d1c983fa580ba_e9f6129fb697276a_b7c45a68142f214c_d1a2ffdb6fbb124b_2d79ab2a39c5877d_95cc3345ded552c2_5a92f1dba88ad318_239900d4ed8623b9;
        refbuf[0:15] =  512'hba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923;
        checkref(16);
        sfrwr(HASH_opt3, 'h00);

    membuf[ 0  ] = 32'hf2bdc948;//32'hF3BCC908;
            membuf[ 1  ] = 32'h6a09e667;//32'h6A09E667;
            membuf[ 2  ] = 32'h84CAA73B;
            membuf[ 3  ] = 32'hBB67AE85;
            membuf[ 4  ] = 32'hFE94F82B;
            membuf[ 5  ] = 32'h3C6EF372;
            membuf[ 6  ] = 32'h5F1D36F1;
            membuf[ 7  ] = 32'hA54FF53A;
            membuf[ 8  ] = 32'hADE682D1;
            membuf[ 9  ] = 32'h510E527F;
            membuf[ 10 ] = 32'h2B3E6C1F;
            membuf[ 11 ] = 32'h9B05688C;
            membuf[ 12 ] = 32'hFB41BD6B;
            membuf[ 13 ] = 32'h1F83D9AB;
            membuf[ 14 ] = 32'h137E2179;
            membuf[ 15 ] = 32'h5BE0CD19;
            membuf[ 16 + 0  ] =  32'h0;
            membuf[ 16 + 1  ] =  32'h0;
            membuf[ 16 + 2  ] =  32'h0;
            membuf[ 16 + 3  ] =  32'h0;
            membuf[ 16 + 4  ] =  32'h0;
            membuf[ 16 + 5  ] =  32'h0;
            membuf[ 16 + 6  ] =  32'h0;
            membuf[ 16 + 7  ] =  32'h0;
            membuf[ 16 + 8  ] =  32'h3;//t0
            membuf[ 16 + 9  ] =  32'h0;
            membuf[ 16 + 10 ] =  32'h0;
            membuf[ 16 + 11 ] =  32'h0;
            membuf[ 16 + 12 ] =  '1;
            membuf[ 16 + 13 ] =  '1;
            membuf[ 16 + 14 ] =  32'h0;
            membuf[ 16 + 15 ] =  32'h0;
            big_to_little();
            memwr(SEGADDR_HOUT, 32);


        membuf[ 0:15 ] = {32'h00636261,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[ 16:31 ] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
           big_to_little();
            memwr(SEGADDR_MSG, 32);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'hb0);
        sfrwr(HASH_crfunc, 32'h04); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,16);
        refbuf[0:15] =  512'h0d4d1c983fa580ba_e9f6129fb697276a_b7c45a68142f214c_d1a2ffdb6fbb124b_2d79ab2a39c5877d_95cc3345ded552c2_5a92f1dba88ad318_239900d4ed8623b9;
        refbuf[0:15] =  512'hba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923;
        checkref(16);
        sfrwr(HASH_opt3, 'h00);

    //        refbuf[31:0] =  512'h0d4d1c983fa580ba_e9f6129fb697276a_b7c45a68142f214c_d1a2ffdb6fbb124b_2d79ab2a39c5877d_95cc3345ded552c2_5a92f1dba88ad318_239900d4ed8623b9;
        #( 10 `US);
    //        $finish;

    */
/*
// sha256

        membuf[ 0  ] = 32'h61626364;
        membuf[ 1  ] = 32'h62636465;
        membuf[ 2  ] = 32'h63646566;
        membuf[ 3  ] = 32'h64656667;
        membuf[ 4  ] = 32'h65666768;
        membuf[ 5  ] = 32'h66676869;
        membuf[ 6  ] = 32'h6768696a;
        membuf[ 7  ] = 32'h68696a6b;
        membuf[ 8  ] = 32'h696a6b6c;
        membuf[ 9  ] = 32'h6a6b6c6d;
        membuf[ 10 ] = 32'h6b6c6d6e;
        membuf[ 11 ] = 32'h6c6d6e6f;
        membuf[ 12 ] = 32'h6d6e6f70;
        membuf[ 13 ] = 32'h6e6f7071;
        membuf[ 14 ] = 32'h80000000;
        membuf[ 15 ] = 32'h00000000;

        membuf[ 16 ] = 32'h00000000;
        membuf[ 17 ] = 32'h00000000;
        membuf[ 18 ] = 32'h00000000;
        membuf[ 19 ] = 32'h00000000;
        membuf[ 20 ] = 32'h00000000;
        membuf[ 21 ] = 32'h00000000;
        membuf[ 22 ] = 32'h00000000;
        membuf[ 23 ] = 32'h00000000;
        membuf[ 24 ] = 32'h00000000;
        membuf[ 25 ] = 32'h00000000;
        membuf[ 26 ] = 32'h00000000;
        membuf[ 27 ] = 32'h00000000;
        membuf[ 28 ] = 32'h00000000;
        membuf[ 29 ] = 32'h00000000;
        membuf[ 30 ] = 32'h00000000;
        membuf[ 31 ] = 32'h000001c0;

        memwr(SEGADDR_MSG,32);
        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_crfunc, 32'h00); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        refbuf[0:7] = 256'h248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1; checkref(8);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 'h80);
        sfrwr(HASH_crfunc, 32'h00); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        refbuf[0:7] = 256'h85e655d6417a17953363376a624cde5c76e09589cac5f811cc4b32c1f20e533a; checkref(8);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'h20);
        sfrwr(HASH_segptr_MSG, 'h10);
        sfrwr(HASH_crfunc, 32'h00); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        sfrwr(HASH_segptr_MSG, 'h0);
        refbuf[0:7] = 256'h248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1; checkref(8);

    // little endian test
       $display("\n@I:: SHA256 Little endian test");

        membuf[ 0  ] = 32'h61626364;
        membuf[ 1  ] = 32'h62636465;
        membuf[ 2  ] = 32'h63646566;
        membuf[ 3  ] = 32'h64656667;
        membuf[ 4  ] = 32'h65666768;
        membuf[ 5  ] = 32'h66676869;
        membuf[ 6  ] = 32'h6768696a;
        membuf[ 7  ] = 32'h68696a6b;
        membuf[ 8  ] = 32'h696a6b6c;
        membuf[ 9  ] = 32'h6a6b6c6d;
        membuf[ 10 ] = 32'h6b6c6d6e;
        membuf[ 11 ] = 32'h6c6d6e6f;
        membuf[ 12 ] = 32'h6d6e6f70;
        membuf[ 13 ] = 32'h6e6f7071;
        membuf[ 14 ] = 32'h80000000;
        membuf[ 15 ] = 32'h00000000;

        membuf[ 16 ] = 32'h00000000;
        membuf[ 17 ] = 32'h00000000;
        membuf[ 18 ] = 32'h00000000;
        membuf[ 19 ] = 32'h00000000;
        membuf[ 20 ] = 32'h00000000;
        membuf[ 21 ] = 32'h00000000;
        membuf[ 22 ] = 32'h00000000;
        membuf[ 23 ] = 32'h00000000;
        membuf[ 24 ] = 32'h00000000;
        membuf[ 25 ] = 32'h00000000;
        membuf[ 26 ] = 32'h00000000;
        membuf[ 27 ] = 32'h00000000;
        membuf[ 28 ] = 32'h00000000;
        membuf[ 29 ] = 32'h00000000;
        membuf[ 30 ] = 32'h00000000;
        membuf[ 31 ] = 32'h000001c0;
        big_to_little();
        memwr(SEGADDR_MSG,32);
        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_opt2, 4);
         sfrwr(HASH_opt3, 'h10);
        sfrwr(HASH_crfunc, 32'h00); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        refbuf[0:7] = 256'h248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1; checkref(8);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 'h10);
        sfrwr(HASH_crfunc, 32'h00); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        refbuf[0:7] = 256'h85e655d6417a17953363376a624cde5c76e09589cac5f811cc4b32c1f20e533a; checkref(8);

        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 'h10);
        sfrwr(HASH_segptr_MSG, 'h10);
        sfrwr(HASH_crfunc, 32'h00); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        sfrwr(HASH_segptr_MSG, 'h0);
        refbuf[0:7] = 256'h248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1; checkref(8);
    */



// hmac256
    /*
  // pass1
        $display("\n@I:: hmac256 pass1");

        membuf[ 0  ] = 32'h0b0b0b0b;
        membuf[ 1  ] = 32'h0b0b0b0b;
        membuf[ 2  ] = 32'h0b0b0b0b;
        membuf[ 3  ] = 32'h0b0b0b0b;
        membuf[ 4  ] = 32'h0b0b0b0b;
        membuf[ 5  ] = 32'h00000000;
        membuf[ 6  ] = 32'h00000000;
        membuf[ 7  ] = 32'h00000000;
        membuf[ 8  ] = 32'h00000000;
        membuf[ 9  ] = 32'h00000000;
        membuf[ 10 ] = 32'h00000000;
        membuf[ 11 ] = 32'h00000000;
        membuf[ 12 ] = 32'h00000000;
        membuf[ 13 ] = 32'h00000000;
        membuf[ 14 ] = 32'h00000000;
        membuf[ 15 ] = 32'h00000000;

        memwr(SEGADDR_KEY,16);

        membuf[ 0  ] = 32'h48692054;
        membuf[ 1  ] = 32'h68657265;
        membuf[ 2  ] = 32'h80000000;
        membuf[ 3  ] = 32'h00000000;
        membuf[ 4  ] = 32'h00000000;
        membuf[ 5  ] = 32'h00000000;
        membuf[ 6  ] = 32'h00000000;
        membuf[ 7  ] = 32'h00000000;
        membuf[ 8  ] = 32'h00000000;
        membuf[ 9  ] = 32'h00000000;
        membuf[ 10 ] = 32'h00000000;
        membuf[ 11 ] = 32'h00000000;
        membuf[ 12 ] = 32'h00000000;
        membuf[ 13 ] = 32'h00000000;
        membuf[ 14 ] = 32'h00000000;
        membuf[ 15 ] = 32'h00000240;
        memwr(SEGADDR_MSG,16);

        refbuf[0:7] = 256'h92ab4d9a1f3b6152bca9dd9e69af43f4ce99e42fd4e30ff972c48025b9f9cfef;

        sfrwr(HASH_crfunc, 32'h50); sfrrd(HASH_crfunc);
        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 0);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        checkref(8);

    // pass2
        $display("\n@I:: hmac256 pass2");

        refbuf[0:7] = 256'hb0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7;

        sfrwr(HASH_crfunc, 32'h60); sfrrd(HASH_crfunc);
    //        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        checkref(8);


    $display("\n@I:: hmac256 little endian pass1");

        membuf[ 0  ] = 32'h0b0b0b0b;
        membuf[ 1  ] = 32'h0b0b0b0b;
        membuf[ 2  ] = 32'h0b0b0b0b;
        membuf[ 3  ] = 32'h0b0b0b0b;
        membuf[ 4  ] = 32'h0b0b0b0b;
        membuf[ 5  ] = 32'h00000000;
        membuf[ 6  ] = 32'h00000000;
        membuf[ 7  ] = 32'h00000000;
        membuf[ 8  ] = 32'h00000000;
        membuf[ 9  ] = 32'h00000000;
        membuf[ 10 ] = 32'h00000000;
        membuf[ 11 ] = 32'h00000000;
        membuf[ 12 ] = 32'h00000000;
        membuf[ 13 ] = 32'h00000000;
        membuf[ 14 ] = 32'h00000000;
        membuf[ 15 ] = 32'h00000000;
        big_to_little();
        memwr(SEGADDR_KEY,16);

        membuf[ 0  ] = 32'h48692054;
        membuf[ 1  ] = 32'h68657265;
        membuf[ 2  ] = 32'h80000000;
        membuf[ 3  ] = 32'h00000000;
        membuf[ 4  ] = 32'h00000000;
        membuf[ 5  ] = 32'h00000000;
        membuf[ 6  ] = 32'h00000000;
        membuf[ 7  ] = 32'h00000000;
        membuf[ 8  ] = 32'h00000000;
        membuf[ 9  ] = 32'h00000000;
        membuf[ 10 ] = 32'h00000000;
        membuf[ 11 ] = 32'h00000000;
        membuf[ 12 ] = 32'h00000000;
        membuf[ 13 ] = 32'h00000000;
        membuf[ 14 ] = 32'h00000000;
        membuf[ 15 ] = 32'h00000240;
        big_to_little();
        memwr(SEGADDR_MSG,16);

        refbuf[0:7] = 256'h92ab4d9a1f3b6152bca9dd9e69af43f4ce99e42fd4e30ff972c48025b9f9cfef;

        sfrwr(HASH_crfunc, 32'h50); sfrrd(HASH_crfunc);
        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 'h14);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        checkref(8);

    // pass2
        $display("\n@I:: hmac256 little endian pass2");

        refbuf[0:7] = 256'hb0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7;

        sfrwr(HASH_crfunc, 32'h60); sfrrd(HASH_crfunc);
    //        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        checkref(8);

        */

// sha512
    /*
        $display("\n@I:: SHA512");

    membuf[0    ] =  32'h80636261;   // big endian membuf[0    ] = 32'h61626380;
    membuf[1    ] = 32'h00000000;
    membuf[2    ] = 32'h00000000;
    membuf[3    ] = 32'h00000000;
    membuf[4    ] = 32'h00000000;
    membuf[5    ] = 32'h00000000;
    membuf[6    ] = 32'h00000000;
    membuf[7    ] = 32'h00000000;
    membuf[8    ] = 32'h00000000;
    membuf[9    ] = 32'h00000000;
    membuf[10   ] = 32'h00000000;
    membuf[11   ] = 32'h00000000;
    membuf[12   ] = 32'h00000000;
    membuf[13   ] = 32'h00000000;
    membuf[14   ] = 32'h00000000;
    membuf[15   ] = 32'h00000000;
    membuf[0 +16] = 32'h00000000;
    membuf[1 +16] = 32'h00000000;
    membuf[2 +16] = 32'h00000000;
    membuf[3 +16] = 32'h00000000;
    membuf[4 +16] = 32'h00000000;
    membuf[5 +16] = 32'h00000000;
    membuf[6 +16] = 32'h00000000;
    membuf[7 +16] = 32'h00000000;
    membuf[8 +16] = 32'h00000000;
    membuf[9 +16] = 32'h00000000;
    membuf[10+16] = 32'h00000000;
    membuf[11+16] = 32'h00000000;
    membuf[12+16] = 32'h00000000;
    membuf[13+16] = 32'h00000000;
    membuf[14+16] = 32'h00000000;
    membuf[15+16] = 32'h18000000;
        memwr(SEGADDR_MSG,32);
        refbuf[0:15] = 512'hDDAF35A193617ABACC417349AE20413112E6FA4E89A97EA20A9EEEE64B55D39A2192992A274FC1A836BA3C23A3FEEBBD454D4423643CE80E2A9AC94FA54CA49F;
        sfrwr(HASH_crfunc, 32'h1); sfrrd(HASH_crfunc);
        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 'h10);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,16);
        checkref(16);

    //        #( 10 `US ); $finish;

    */
/*
// aes
        #( 10 `US );
        $display("\n@I:: AES");

        membuf[ 0  ] = 32'h00112233;
        membuf[ 1  ] = 32'h44556677;
        membuf[ 2  ] = 32'h8899aabb;
        membuf[ 3  ] = 32'hccddeeff;
        membuf[ 0  ] = 32'h69c4e0d8;
        membuf[ 1  ] = 32'h6a7b0430;
        membuf[ 2  ] = 32'hd8cdb780;
        membuf[ 3  ] = 32'h70b4c55a;
        memwr(SEGADDR_AIB,4);
        membuf[ 0  ] = 32'h00010203;
        membuf[ 1  ] = 32'h04050607;
        membuf[ 2  ] = 32'h08090a0b;
        membuf[ 3  ] = 32'h0c0d0e0f;
        membuf[ 4  ] = 32'h296bd6eb;
        membuf[ 5  ] = 32'h2ca90321;
        membuf[ 6  ] = 32'hbbef5f5f;
        membuf[ 7  ] = 32'h4cfc10ec;
        membuf[ 8  ] = 32'h11111111;
        membuf[ 9  ] = 32'h22222222;
        membuf[ 10 ] = 32'h33333333;
        membuf[ 11 ] = 32'h44444444;
        memwr(SEGADDR_AKEY,12);
        sfrwr(AES_segptr_IV,8);

        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);

        #( 1 `US );
        sfrwr(AES_opt1,32'h0);
        sfrwr(AES_crfunc,32'h2);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
        memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);

        refbuf[0:3] = {
            32'h00112233,
            32'h44556677,
            32'h8899aabb,
            32'hccddeeff
            };
        checkref(4);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0F0E);
        sfrwr(SDMA_ichcr_transize,32'h4);
        sfrwr(SDMA_chstart_ar,32'h5a);

        #( 10 `US );
        sfrwr(AES_crfunc,32'h1);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
        memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);
        refbuf[0:3] = {
            32'h69c4e0d8,
            32'h6a7b0430,
            32'hd8cdb780,
            32'h70b4c55a
            };
        checkref(4);


// pke EDI2MD
        #( 10 `US );
        $display("\n@I:: PKE ED25519");

        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd255 );
        sfrwr( PKE_optew, 32'd255 );

        // P:
        membuf[ 0  ] = 32'hffffffed;
        membuf[ 1  ] = 32'hffffffff;
        membuf[ 2  ] = 32'hffffffff;
        membuf[ 3  ] = 32'hffffffff;
        membuf[ 4  ] = 32'hffffffff;
        membuf[ 5  ] = 32'hffffffff;
        membuf[ 6  ] = 32'hffffffff;
        membuf[ 7  ] = 32'h7fffffff;
        // A
        membuf[ 8  ] = 32'h135978a3;
        membuf[ 9  ] = 32'h75eb4dca;
        membuf[ 10 ] = 32'h4141d8ab;
        membuf[ 11 ] = 32'h00700a4d;
        membuf[ 12 ] = 32'h7779e898;
        membuf[ 13 ] = 32'h8cc74079;
        membuf[ 14 ] = 32'h2b6ffe73;
        membuf[ 15 ] = 32'h52036cee;
        memwr(SEGADDR_PCON,16);
        membuf[ 0  ] = 32'h0;
        membuf[ 1  ] = 32'h0;
        membuf[ 2  ] = 32'h0;
        membuf[ 3  ] = 32'h0;
        membuf[ 4  ] = 32'h0;
        membuf[ 5  ] = 32'h0;
        membuf[ 6  ] = 32'h0;
        membuf[ 7  ] = 32'h80000000;
        membuf[ 8  ] = 32'h0;
        memwr(SEGADDR_PCON+16,9);
    //        membuf[ 0  ] = 32'h1;
    //        membuf[ 1  ] = 32'h0;
    //        membuf[ 2  ] = 32'h0;
    //        membuf[ 3  ] = 32'h0;
    //        membuf[ 4  ] = 32'h0;
    //        membuf[ 5  ] = 32'h0;
    //        membuf[ 6  ] = 32'h0;
    //        membuf[ 7  ] = 32'h0;
    //        memwr(SEGADDR_PCON+16+9,8);

        sfrwr( PKE_crfunc, 32'h01 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        // K
        membuf[ 0  ] = 32'h86837c30;
        membuf[ 1  ] = 32'hcb33284f;
        membuf[ 2  ] = 32'hf12e7a42;
        membuf[ 3  ] = 32'h3c010ac0;
        membuf[ 4  ] = 32'h6827fffd;
        membuf[ 5  ] = 32'ha3c080d9;
        membuf[ 6  ] = 32'h06f020a5;
        membuf[ 7  ] = 32'h4fe94d90;
        memwr(SEGADDR_PKB,8);

    $display("\n@I:: PKE ED25519 - I2MD ");


        // Q0_X,Q0_Y, I2MD
        membuf[ 0   ] = 32'h8f25d51a;
        membuf[ 1   ] = 32'hc9562d60;
        membuf[ 2   ] = 32'h9525a7b2;
        membuf[ 3   ] = 32'h692cc760;
        membuf[ 4   ] = 32'hfdd6dc5c;
        membuf[ 5   ] = 32'hc0a4e231;
        membuf[ 6   ] = 32'hcd6e53fe;
        membuf[ 7   ] = 32'h216936d3;
        membuf[ 0+8 ] = 32'h66666658;
        membuf[ 1+8 ] = 32'h66666666;
        membuf[ 2+8 ] = 32'h66666666;
        membuf[ 3+8 ] = 32'h66666666;
        membuf[ 4+8 ] = 32'h66666666;
        membuf[ 5+8 ] = 32'h66666666;
        membuf[ 6+8 ] = 32'h66666666;
        membuf[ 7+8 ] = 32'h66666666;

        refbuf[0  ] =  32'h3f9da287;
        refbuf[1  ] =  32'he2cabc55;
        refbuf[2  ] =  32'h2396e489;
        refbuf[3  ] =  32'h9ca59856;
        refbuf[4  ] =  32'hade4b5b7;
        refbuf[5  ] =  32'h9879936b;
        refbuf[6  ] =  32'h7e6077d0;
        refbuf[7  ] =  32'h759e2370;
        refbuf[0+8] =  32'h3333334a;
        refbuf[1+8] =  32'h33333333;
        refbuf[2+8] =  32'h33333333;
        refbuf[3+8] =  32'h33333333;
        refbuf[4+8] =  32'h33333333;
        refbuf[5+8] =  32'h33333333;
        refbuf[6+8] =  32'h33333333;
        refbuf[7+8] =  32'h33333333;
        refbuf[0+16] = 32'h00000026;
        refbuf[1+16] = 32'h00000000;
        refbuf[2+16] = 32'h00000000;
        refbuf[3+16] = 32'h00000000;
        refbuf[4+16] = 32'h00000000;
        refbuf[5+16] = 32'h00000000;
        refbuf[6+16] = 32'h00000000;
        refbuf[7+16] = 32'h00000000;
        refbuf[0+24] = 32'h994ae86c;
        refbuf[1+24] = 32'h4f0896aa;
        refbuf[2+24] = 32'hb612506e;
        refbuf[3+24] = 32'he3b7ad11;
        refbuf[4+24] = 32'hf183c492;
        refbuf[5+24] = 32'h46c7a922;
        refbuf[6+24] = 32'hfeb3930d;
        refbuf[7+24] = 32'h5e181c59;

        memwr(SEGADDR_PIB,16);
        sfrwr( PKE_crfunc, 32'h24 ); // I2MD
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        memrd(SEGADDR_POB,8*4);
        checkref(8*4);



        // Q0_X,Q0_Y, I2MD
        membuf[ 0   ] = 32'h8f25d51a;
        membuf[ 1   ] = 32'hc9562d60;
        membuf[ 2   ] = 32'h9525a7b2;
        membuf[ 3   ] = 32'h692cc760;
        membuf[ 4   ] = 32'hfdd6dc5c;
        membuf[ 5   ] = 32'hc0a4e231;
        membuf[ 6   ] = 32'hcd6e53fe;
        membuf[ 7   ] = 32'h216936d3;
        membuf[ 0+8 ] = 32'h66666658;
        membuf[ 1+8 ] = 32'h66666666;
        membuf[ 2+8 ] = 32'h66666666;
        membuf[ 3+8 ] = 32'h66666666;
        membuf[ 4+8 ] = 32'h66666666;
        membuf[ 5+8 ] = 32'h66666666;
        membuf[ 6+8 ] = 32'h66666666;
        membuf[ 7+8 ] = 32'h66666660;

        membuf[ 16+0   ] = 32'h8f25d51a;
        membuf[ 16+1   ] = 32'hc9562d60;
        membuf[ 16+2   ] = 32'h9525a7b2;
        membuf[ 16+3   ] = 32'h692cc760;
        membuf[ 16+4   ] = 32'hfdd6dc5c;
        membuf[ 16+5   ] = 32'hc0a4e231;
        membuf[ 16+6   ] = 32'hcd6e53fe;
        membuf[ 16+7   ] = 32'h216936d3;
        membuf[ 16+0+8 ] = 32'h66666658;
        membuf[ 16+1+8 ] = 32'h66666666;
        membuf[ 16+2+8 ] = 32'h66666666;
        membuf[ 16+3+8 ] = 32'h66666666;
        membuf[ 16+4+8 ] = 32'h66666666;
        membuf[ 16+5+8 ] = 32'h66666666;
        membuf[ 16+6+8 ] = 32'h66666666;
        membuf[ 16+7+8 ] = 32'h66666666;
        refbuf[0  ] =  'h3f9da287;
        refbuf[1  ] =  'he2cabc55;
        refbuf[2  ] =  'h2396e489;
        refbuf[3  ] =  'h9ca59856;
        refbuf[4  ] =  'hade4b5b7;
        refbuf[5  ] =  'h9879936b;
        refbuf[6  ] =  'h7e6077d0;
        refbuf[7  ] =  'h759e2370;
        refbuf[0+8] =  'h3333334a;
        refbuf[1+8] =  'h33333333;
        refbuf[2+8] =  'h33333333;
        refbuf[3+8] =  'h33333333;
        refbuf[4+8] =  'h33333333;
        refbuf[5+8] =  'h33333333;
        refbuf[6+8] =  'h33333333;
        refbuf[7+8] =  'h3333324f;
        refbuf[0+16] = 'h00000026;
        refbuf[1+16] = 'h00000000;
        refbuf[2+16] = 'h00000000;
        refbuf[3+16] = 'h00000000;
        refbuf[4+16] = 'h00000000;
        refbuf[5+16] = 'h00000000;
        refbuf[6+16] = 'h00000000;
        refbuf[7+16] = 'h00000000;
        refbuf[0+24] = 'h9cbb2c7f;
        refbuf[1+24] = 'h9ca50bdc;
        refbuf[2+24] = 'h3296a3b6;
        refbuf[3+24] = 'h0405d58a;
        refbuf[4+24] = 'h253c78ac;
        refbuf[5+24] = 'hb8dcf35b;
        refbuf[6+24] = 'h3ddc02dc;
        refbuf[7+24] = 'h60664cc7;


        memwr(SEGADDR_PIB,32);
        sfrwr( PKE_segptr_PIB0, 32'h0 );
        sfrwr( PKE_crfunc, 32'h24 ); // I2MD
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        memrd(SEGADDR_POB,8*4);
        checkref(8*4);
        #( 1 `US );

    $display("\n@I:: PKE ED25519 - I2MD2 ");

        refbuf[0  ] =  32'h3f9da287;
        refbuf[1  ] =  32'he2cabc55;
        refbuf[2  ] =  32'h2396e489;
        refbuf[3  ] =  32'h9ca59856;
        refbuf[4  ] =  32'hade4b5b7;
        refbuf[5  ] =  32'h9879936b;
        refbuf[6  ] =  32'h7e6077d0;
        refbuf[7  ] =  32'h759e2370;
        refbuf[0+8] =  32'h3333334a;
        refbuf[1+8] =  32'h33333333;
        refbuf[2+8] =  32'h33333333;
        refbuf[3+8] =  32'h33333333;
        refbuf[4+8] =  32'h33333333;
        refbuf[5+8] =  32'h33333333;
        refbuf[6+8] =  32'h33333333;
        refbuf[7+8] =  32'h33333333;
        refbuf[0+16] = 32'h00000026;
        refbuf[1+16] = 32'h00000000;
        refbuf[2+16] = 32'h00000000;
        refbuf[3+16] = 32'h00000000;
        refbuf[4+16] = 32'h00000000;
        refbuf[5+16] = 32'h00000000;
        refbuf[6+16] = 32'h00000000;
        refbuf[7+16] = 32'h00000000;
        refbuf[0+24] = 32'h994ae86c;
        refbuf[1+24] = 32'h4f0896aa;
        refbuf[2+24] = 32'hb612506e;
        refbuf[3+24] = 32'he3b7ad11;
        refbuf[4+24] = 32'hf183c492;
        refbuf[5+24] = 32'h46c7a922;
        refbuf[6+24] = 32'hfeb3930d;
        refbuf[7+24] = 32'h5e181c59;
    //        memwr(SEGADDR_PIB,32);
            sfrwr( PKE_segptr_PIB0, 32'h10 );
            sfrwr( PKE_crfunc, 32'h24 ); // I2MD
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            sfrwr( PKE_segptr_PIB0, 32'h0 );

            memrd(SEGADDR_POB,8*4);
            checkref(8*4);
            #( 1 `US );
    //     $display("\n@I:: PKE ED25519 - I2MA ");

    //         sfrwr( PKE_segptr_PIB0, 32'h0 );
    //         sfrwr( PKE_segptr_PIB1, 32'h10 );
    //         sfrwr( PKE_crfunc, 32'h23 ); // I2MA
    //         sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

    //         memrd(SEGADDR_POB,8*4);
    //         checkref(8*4);
    //         sfrwr( PKE_segptr_PIB1, 32'h0 );
    //         #( 1 `US );


    //#(10 `US ) $finish;
    //EDPD
        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0b09);
        sfrwr(SDMA_ichcr_transize,32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);
        #( 10 `US );

        refbuf[0  ] =  32'h972189f4;
        refbuf[1  ] =  32'h37afef4b;
        refbuf[2  ] =  32'h2cd49994;
        refbuf[3  ] =  32'h4070c6f3;
        refbuf[4  ] =  32'h4444f76a;
        refbuf[5  ] =  32'hc1173625;
        refbuf[6  ] =  32'hee6d891e;
        refbuf[7  ] =  32'h4b5c6e98;
        refbuf[0+8] =  32'hfac0fb07;
        refbuf[1+8] =  32'hb728a843;
        refbuf[2+8] =  32'h7e3b703d;
        refbuf[3+8] =  32'h86a34b2a;
        refbuf[4+8] =  32'hb9e22a85;
        refbuf[5+8] =  32'hb13d7170;
        refbuf[6+8] =  32'h25ddb291;
        refbuf[7+8] =  32'h176e9b58;
        refbuf[0+16] = 32'h5df1062f;
        refbuf[1+16] = 32'hc3511f0e;
        refbuf[2+16] = 32'ha7c2d514;
        refbuf[3+16] = 32'h01606607;
        refbuf[4+16] = 32'hcbdaa1cf;
        refbuf[5+16] = 32'h96af2b24;
        refbuf[6+16] = 32'heefc74f3;
        refbuf[7+16] = 32'h2a015005;
        refbuf[0+24] = 32'h0d78ed3b;
        refbuf[1+24] = 32'h2e9d950e;
        refbuf[2+24] = 32'h51bbc5b8;
        refbuf[3+24] = 32'h40ac787d;
        refbuf[4+24] = 32'hc902b652;
        refbuf[5+24] = 32'ha1e4cd8a;
        refbuf[6+24] = 32'h52f53e27;
        refbuf[7+24] = 32'h156aabe7;

        sfrwr( PKE_crfunc, 32'h26 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        memrd(SEGADDR_POB,8*4);
        checkref(8*4);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0b09);
        sfrwr(SDMA_ichcr_transize,32'd32);
        sfrwr(SDMA_ichcr_wpstart,32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);
        #( 10 `US );
    //EDPM

        sfrwr( PKE_segptr_PIB1, 'd32 );

        sfrwr( PKE_crfunc, 32'h2b );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        memrd(SEGADDR_POB,8*4);

        refbuf[0  ] =  32'h2d066d9e;
        refbuf[1  ] =  32'h30eccbb4;
        refbuf[2  ] =  32'h03735d6a;
        refbuf[3  ] =  32'h01144cb6;
        refbuf[4  ] =  32'h8c900430;
        refbuf[5  ] =  32'h8eb9c012;
        refbuf[6  ] =  32'h2012176b;
        refbuf[7  ] =  32'h544539f8;
        refbuf[0+8] =  32'hccdde4d6;
        refbuf[1+8] =  32'h24d124f8;
        refbuf[2+8] =  32'h9fc385b1;
        refbuf[3+8] =  32'h23c64004;
        refbuf[4+8] =  32'hb4ee4089;
        refbuf[5+8] =  32'h713b604b;
        refbuf[6+8] =  32'h5e447916;
        refbuf[7+8] =  32'h7464294c;
        refbuf[0+16] = 32'h1d19d4f6;
        refbuf[1+16] = 32'h494cd6cb;
        refbuf[2+16] = 32'h1ceb4430;
        refbuf[3+16] = 32'ha4b5a0bf;
        refbuf[4+16] = 32'h0a23bd49;
        refbuf[5+16] = 32'haa9f813c;
        refbuf[6+16] = 32'heedca845;
        refbuf[7+16] = 32'h36381de3;
        refbuf[0+24] = 32'hd728eef6;
        refbuf[1+24] = 32'hda1bec5b;
        refbuf[2+24] = 32'h81094f61;
        refbuf[3+24] = 32'hc5536070;
        refbuf[4+24] = 32'hcb5fb33d;
        refbuf[5+24] = 32'hdd2abf27;
        refbuf[6+24] = 32'hcf5ee31a;
        refbuf[7+24] = 32'h3f9ff638;

       checkref(8*4);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0b09);
        sfrwr(SDMA_ichcr_transize,32'd32);
        sfrwr(SDMA_ichcr_wpstart,32'd0);
        sfrwr(SDMA_chstart_ar,32'h5a);
        #( 10 `US );

        //EDM2I
        sfrwr( PKE_crfunc, 32'h27 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        memrd(SEGADDR_POB,8*2);

        refbuf[0   ] = 32'h777645ce;
        refbuf[1   ] = 32'hb12786bd;
        refbuf[2   ] = 32'h53187c24;
        refbuf[3   ] = 32'hc513d472;
        refbuf[4   ] = 32'h60d0f620;
        refbuf[5   ] = 32'h2297e08d;
        refbuf[6   ] = 32'h2b9d3429;
        refbuf[7   ] = 32'h55d0e09a;
        refbuf[0+8 ] = 32'h01985ad7;
        refbuf[1+8 ] = 32'hb70ab182;
        refbuf[2+8 ] = 32'hd3fe4bd5;
        refbuf[3+8 ] = 32'h3a0764c9;
        refbuf[4+8 ] = 32'hf372e10e;
        refbuf[5+8 ] = 32'h2523a6da;
        refbuf[6+8 ] = 32'h681a02af;
        refbuf[7+8 ] = 32'h1a5107f7;

        checkref(8*2);

       #( 10 `US );
//        $finish;

*/
// pke ECI2MD ECPD ECPM ECM2I

    /*
        #( 10 `US );
        $display("\n@I:: PKE ECC256r1");

        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd256 );
        sfrwr( PKE_optew, 32'd256 );


        // P:
        membuf[ 0  ] = 32'h1F6E5377;
        membuf[ 1  ] = 32'h2013481D;
        membuf[ 2  ] = 32'hD5262028;
        membuf[ 3  ] = 32'h6E3BF623;
        membuf[ 4  ] = 32'h9D838D72;
        membuf[ 5  ] = 32'h3E660A90;
        membuf[ 6  ] = 32'hA1EEA9BC;
        membuf[ 7  ] = 32'hA9FB57DB;
        // A
        membuf[ 8  ] = 32'hF330B5D9;
        membuf[ 9  ] = 32'hE94A4B44;
        membuf[ 10 ] = 32'h26DC5C6C;
        membuf[ 11 ] = 32'hFB8055C1;
        membuf[ 12 ] = 32'h417AFFE7;
        membuf[ 13 ] = 32'hEEF67530;
        membuf[ 14 ] = 32'hFC2C3057;
        membuf[ 15 ] = 32'h7D5A0975;
        memwr(SEGADDR_PCON,16);
        membuf[ 0  ] = 32'h0;
        membuf[ 1  ] = 32'h0;
        membuf[ 2  ] = 32'h0;
        membuf[ 3  ] = 32'h0;
        membuf[ 4  ] = 32'h0;
        membuf[ 5  ] = 32'h0;
        membuf[ 6  ] = 32'h0;
        membuf[ 7  ] = 32'h0;
        membuf[ 8  ] = 32'h1;
        memwr(SEGADDR_PCON+16,9);
    //        membuf[ 0  ] = 32'h1;
    //        membuf[ 1  ] = 32'h0;
    //        membuf[ 2  ] = 32'h0;
    //        membuf[ 3  ] = 32'h0;
    //        membuf[ 4  ] = 32'h0;
    //        membuf[ 5  ] = 32'h0;
    //        membuf[ 6  ] = 32'h0;
    //        membuf[ 7  ] = 32'h0;
    //        memwr(SEGADDR_PCON+16+9,8);

        sfrwr( PKE_crfunc, 32'h01 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        // K
        membuf[ 0  ] = 32'h39804F1D;
        membuf[ 1  ] = 32'hF77B0630;
        membuf[ 2  ] = 32'h41D79950;
        membuf[ 3  ] = 32'h300CB542;
        membuf[ 4  ] = 32'h8271BE38;
        membuf[ 5  ] = 32'hEA338D70;
        membuf[ 6  ] = 32'h00150FF2;
        membuf[ 7  ] = 32'h81DB1EE1;
        memwr(SEGADDR_PKB,8);

        // Q0_X,Q0_Y, I2MD
        membuf[ 0   ] = 32'h9ACE3262;
        membuf[ 1   ] = 32'h3A4453BD;
        membuf[ 2   ] = 32'hE3BD23C2;
        membuf[ 3   ] = 32'hB9DE27E1;
        membuf[ 4   ] = 32'hFC81B7AF;
        membuf[ 5   ] = 32'h2C4B482F;
        membuf[ 6   ] = 32'hCB7E57CB;
        membuf[ 7   ] = 32'h8BD2AEB9;
        membuf[ 0+8 ] = 32'h2F046997;
        membuf[ 1+8 ] = 32'h5C1D54C7;
        membuf[ 2+8 ] = 32'h2DED8E54;
        membuf[ 3+8 ] = 32'hC2774513;
        membuf[ 4+8 ] = 32'h14611DC9;
        membuf[ 5+8 ] = 32'h97F8461A;
        membuf[ 6+8 ] = 32'hC3DAC4FD;
        membuf[ 7+8 ] = 32'h547EF835;

        refbuf[0  ] = 32'h351fd10c;
        refbuf[1  ] = 32'h27c0d92d;
        refbuf[2  ] = 32'hb97cf30a;
        refbuf[3  ] = 32'h80de4d9a;
        refbuf[4  ] = 32'h6b892ad3;
        refbuf[5  ] = 32'h704c311d;
        refbuf[6  ] = 32'h9e119bdf;
        refbuf[7  ] = 32'h8e1f767a;
        refbuf[0+8] = 32'ha0917a17;
        refbuf[1+8] = 32'h9a4fe948;
        refbuf[2+8] = 32'hcd950162;
        refbuf[3+8] = 32'ha618f259;
        refbuf[4+8] = 32'hdfbd8b03;
        refbuf[5+8] = 32'h16fdf6e8;
        refbuf[6+8] = 32'h026eb0a2;
        refbuf[7+8] = 32'h14eb78c6;
        refbuf[0+16] = 32'he091ac89;
        refbuf[1+16] = 32'hdfecb7e2;
        refbuf[2+16] = 32'h2ad9dfd7;
        refbuf[3+16] = 32'h91c409dc;
        refbuf[4+16] = 32'h627c728d;
        refbuf[5+16] = 32'hc199f56f;
        refbuf[6+16] = 32'h5e115643;
        refbuf[7+16] = 32'h5604a824;

        memwr(SEGADDR_PIB,16);
        sfrwr( PKE_crfunc, 32'h04 ); // I2MD
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        memrd(SEGADDR_POB,8*3);
        checkref(8*3);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0b09);
        sfrwr(SDMA_ichcr_transize,32'd24);
        sfrwr(SDMA_chstart_ar,32'h5a);
        #( 10 `US );

        refbuf[0   ] =  32'h4273816f;
        refbuf[1   ] =  32'h522c4f61;
        refbuf[2   ] =  32'h954dee9a;
        refbuf[3   ] =  32'h850f4a8b;
        refbuf[4   ] =  32'h47fb38ca;
        refbuf[5   ] =  32'hcea591e4;
        refbuf[6   ] =  32'h16a0bad9;
        refbuf[7   ] =  32'h2350fb71;
        refbuf[0+8 ] =  32'h118003c1;
        refbuf[1+8 ] =  32'h00f4fb49;
        refbuf[2+8 ] =  32'ha3353283;
        refbuf[3+8 ] =  32'h2c546a9b;
        refbuf[4+8 ] =  32'he7e6f540;
        refbuf[5+8 ] =  32'hf39e663d;
        refbuf[6+8 ] =  32'h08258897;
        refbuf[7+8 ] =  32'h556f60af;
        refbuf[0+16] = 32'h4122f42e;
        refbuf[1+16] = 32'h349fd291;
        refbuf[2+16] = 32'h9b2a02c5;
        refbuf[3+16] = 32'h4c31e4b3;
        refbuf[4+16] = 32'hbf7b1607;
        refbuf[5+16] = 32'h2dfbedd1;
        refbuf[6+16] = 32'h04dd6144;
        refbuf[7+16] = 32'h29d6f18c;

       sfrwr( PKE_crfunc, 32'h06 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        memrd(SEGADDR_POB,8*3);
        checkref(8*3);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0b09);
        sfrwr(SDMA_ichcr_transize,32'd24);
        sfrwr(SDMA_ichcr_wpstart,32'd24);
        sfrwr(SDMA_chstart_ar,32'h5a);
        #( 10 `US );


        sfrwr( PKE_segptr_PIB1, 'd24 );

        sfrwr( PKE_crfunc, 32'h0b );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        memrd(SEGADDR_POB,8*3);

        refbuf[0   ] = 32'hb033388e;
        refbuf[1   ] = 32'ha9274524;
        refbuf[2   ] = 32'hf64bcc5e;
        refbuf[3   ] = 32'h38f92d82;
        refbuf[4   ] = 32'hc150a4d4;
        refbuf[5   ] = 32'h9de08088;
        refbuf[6   ] = 32'hc0b1bf85;
        refbuf[7   ] = 32'ha3b76364;
        refbuf[0+8 ] = 32'hab1aa5e2;
        refbuf[1+8 ] = 32'h0e34b98d;
        refbuf[2+8 ] = 32'h45d1085d;
        refbuf[3+8 ] = 32'h81523076;
        refbuf[4+8 ] = 32'h032908ac;
        refbuf[5+8 ] = 32'haa3c1e36;
        refbuf[6+8 ] = 32'h7ec54bf8;
        refbuf[7+8 ] = 32'h1f4d97af;
        refbuf[0+16] = 32'hb8af1d7d;
        refbuf[1+16] = 32'hee69f560;
        refbuf[2+16] = 32'h2caa4cf1;
        refbuf[3+16] = 32'hb316455a;
        refbuf[4+16] = 32'hb0af23b3;
        refbuf[5+16] = 32'h76a1f23b;
        refbuf[6+16] = 32'hde723ec3;
        refbuf[7+16] = 32'h24433ae5;

       checkref(8*3);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0b09);
        sfrwr(SDMA_ichcr_transize,32'd24);
        sfrwr(SDMA_ichcr_wpstart,32'd0);
        sfrwr(SDMA_chstart_ar,32'h5a);
        #( 10 `US );

        sfrwr( PKE_crfunc, 32'h07 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        memrd(SEGADDR_POB,8*2);

        refbuf[0   ] = 32'he3100be5;
        refbuf[1   ] = 32'h85f929a8;
        refbuf[2   ] = 32'h49e81d9e;
        refbuf[3   ] = 32'hb95e1aaa;
        refbuf[4   ] = 32'h53a8414d;
        refbuf[5   ] = 32'ha1705d99;
        refbuf[6   ] = 32'h3f92bc02;
        refbuf[7   ] = 32'h44106e91;
        refbuf[0+8 ] = 32'heb089bdc;
        refbuf[1+8 ] = 32'hf789ef10;
        refbuf[2+8 ] = 32'h2c272223;
        refbuf[3+8 ] = 32'h00a69fd3;
        refbuf[4+8 ] = 32'hd120f5a9;
        refbuf[5+8 ] = 32'h3ce49cbd;
        refbuf[6+8 ] = 32'h11caccb7;
        refbuf[7+8 ] = 32'h8ab4846f;
        refbuf[0+16] = 32'h0;
        refbuf[1+16] = 32'h0;
        refbuf[2+16] = 32'h0;
        refbuf[3+16] = 32'h0;
        refbuf[4+16] = 32'h0;
        refbuf[5+16] = 32'h0;
        refbuf[6+16] = 32'h0;
        refbuf[7+16] = 32'h0;

        checkref(8*2);


    */


//RSAME 64
     /*
    #( 10 `US );

        $display("\n@I:: RSAME 64");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd64);
        sfrwr( PKE_optew, 32'd64);

        //N the modulo for RSA, product of two large prime
        membuf[0:1] = {32'h000003D9,32'h8fffffbe};
        memwr(SEGADDR_PCON,2);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:2] = {32'h00000000,32'h00000000,32'h1};
            memwr(SEGADDR_PCON+2,3);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

         // RSAME
        // R0
            membuf[0:1] ={32'h1,32'h0};
        // R1
            membuf[2:3] ={32'h4,32'h0};
            memwr(SEGADDR_PIB, 4);
            sfrwr( PKE_segptr_PIB1, 'h2);
        // E
            membuf[0:1] ={32'h1,32'h80000000};
        memwr(SEGADDR_PKB, 2);

            sfrwr( PKE_crfunc, 32'h13 ); // RSAME
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading RSAME result\n");
            memrd(SEGADDR_POB, 2);
            */

// RSA MA 64
    /*
    #( 10 `US );

        $display("\n@I:: RSAMA 64");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd64);
        sfrwr( PKE_optew, 32'd64);

    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);

        //N the modulo for RSA, product of two large prime
        membuf[0:1] = {32'h000003D9,32'h8fffffbe};
        memwr(SEGADDR_PCON,2);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:2] = {32'h00000000,32'h00000000,32'h1};
            memwr(SEGADDR_PCON+2,3);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        // X
               membuf[0:1] = {32'h000003D5,32'h8fffffbe};
        // Y
            membuf[2:3] =  {32'h000003D5,32'h8fffffbe};
            memwr(SEGADDR_PIB, 4);
            sfrwr( PKE_segptr_PIB1, 'h2);
            sfrwr( PKE_crfunc, 32'h19 ); // RSAMM
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading 64 bit RSAMA result\n");
            memrd(SEGADDR_POB, 2);

    */

// RSA MA 4096

    #( 10 `US );
       // split_Big_number();
        $display("\n@I:: RSAMA 64");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd128);
        sfrwr( PKE_optew, 32'd128);

    #( 1 `US);
//        sfrwr(SDMA_ichcr_segid, 32'h0b09);
//        sfrwr(SDMA_ichcr_transize, 32'd32);
//        sfrwr(SDMA_chstart_ar,32'h5a);

        //N the modulo for RSA, product of two large prime
        membuf[0:7] =  {32'h1,32'h9259feeb,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[8:15] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[16:23] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[24:31] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[32:39] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[40:47] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[48:55] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[56:63] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[64:71] =  {32'hbeef9527,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[72:79] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[80:87] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[88:95] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[96:103] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[104:111] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[112:119] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[120:127] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h1,32'h0,32'h0};
        memwr(SEGADDR_PCON,128);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
         membuf[0:8] = {32'h01,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[65:72] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[73:80] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[81:88] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[89:96] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[97:104] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[105:112] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[113:120] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[121:128] = {32'h0,32'h0,32'h2,32'h0,32'h0,32'h0,32'h0,32'h0};
        memwr(SEGADDR_PCON+128,'d129);
        sfrwr( PKE_optmask, 32'h1);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        // X
               membuf[0:7] =  {32'hb,32'hb9025a70,32'hb9025b09,32'hb9025ba2,32'hb9025c3b,32'hb9025cd4,32'hb9025d6d,32'hb9025e06};
            membuf[8:15] =  {32'hb9025e9f,32'hb9025f38,32'hb9025fd1,32'hb902606a,32'hb9026103,32'hb902619c,32'hb9026235,32'hb90262ce};
            membuf[16:23] =  {32'hb9026367,32'hb9026400,32'hb9026499,32'hb9026532,32'hb90265cb,32'hb9026664,32'hb90266fd,32'hb9026796};
            membuf[24:31] =  {32'hb902682f,32'hb90268c8,32'hb9026961,32'hb90269fa,32'hb9026a93,32'hb9026b2c,32'hb9026bc5,32'hb9026c5e};
            membuf[32:39] =  {32'hb9026cf7,32'hb9026d90,32'hb9026e29,32'hb9026ec2,32'hb9026f5b,32'hb9026ff4,32'hb902708d,32'hb9027126};
            membuf[40:47] =  {32'hb90271bf,32'hb9027258,32'hb90272f1,32'hb902738a,32'hb9027423,32'hb90274bc,32'hb9027555,32'hb90275ee};
            membuf[48:55] =  {32'hb9027687,32'hb9027720,32'hb90277b9,32'hb9027852,32'hb90278eb,32'hb9027984,32'hb9027a1d,32'hb9027ab6};
            membuf[56:63] =  {32'hb9027b4f,32'hb9027be8,32'hb9027c81,32'hb9027d1a,32'hb9027db3,32'hb9027e4c,32'hb9027ee5,32'hb9027f7e};
            membuf[64:71] =  {32'hb9028017,32'hb90280b0,32'hb9028149,32'hb90281e2,32'hb902827b,32'hb9028314,32'hb90283ad,32'hb9028446};
            membuf[72:79] =  {32'hb90284df,32'hb9028578,32'hb9028611,32'hb90286aa,32'hb9028743,32'hb90287dc,32'hb9028875,32'hb902890e};
            membuf[80:87] =  {32'hb90289a7,32'hb9028a40,32'hb9028ad9,32'hb9028b72,32'hb9028c0b,32'hb9028ca4,32'hb9028d3d,32'hb9028dd6};
            membuf[88:95] =  {32'hb9028e6f,32'hb9028f08,32'hb9028fa1,32'hb902903a,32'hb90290d3,32'hb902916c,32'hb9029205,32'hb902929e};
            membuf[96:103] =  {32'hb9029337,32'hb90293d0,32'hb9029469,32'hb9029502,32'hb902959b,32'hb9029634,32'hb90296cd,32'hb9029766};
            membuf[104:111] =  {32'hb90297ff,32'hb9029898,32'hb9029931,32'hb90299ca,32'hb9029a63,32'hb9029afc,32'hb9029b95,32'hb9029c2e};
            membuf[112:119] =  {32'hb9029cc7,32'hb9029d60,32'hb9029df9,32'hb9029e92,32'hb9029f2b,32'hb9029fc4,32'hb902a05d,32'hb902a0f6};
            membuf[120:127] =  {32'hb902a18f,32'hb902a228,32'hb902a2c1,32'hb902a35a,32'hb902a3f3,32'h0,32'h0,32'h0};
        // Y
            membuf[128:135] =  {32'h7,32'h7d058ba4,32'h7d058c95,32'h7d058d86,32'h7d058e77,32'h7d058f68,32'h7d059059,32'h7d05914a};
        membuf[136:143] =  {32'h7d05923b,32'h7d05932c,32'h7d05941d,32'h7d05950e,32'h7d0595ff,32'h7d0596f0,32'h7d0597e1,32'h7d0598d2};
        membuf[144:151] =  {32'h7d0599c3,32'h7d059ab4,32'h7d059ba5,32'h7d059c96,32'h7d059d87,32'h7d059e78,32'h7d059f69,32'h7d05a05a};
        membuf[152:159] =  {32'h7d05a14b,32'h7d05a23c,32'h7d05a32d,32'h7d05a41e,32'h7d05a50f,32'h7d05a600,32'h7d05a6f1,32'h7d05a7e2};
        membuf[160:167] =  {32'h7d05a8d3,32'h7d05a9c4,32'h7d05aab5,32'h7d05aba6,32'h7d05ac97,32'h7d05ad88,32'h7d05ae79,32'h7d05af6a};
        membuf[168:175] =  {32'h7d05b05b,32'h7d05b14c,32'h7d05b23d,32'h7d05b32e,32'h7d05b41f,32'h7d05b510,32'h7d05b601,32'h7d05b6f2};
        membuf[176:183] =  {32'h7d05b7e3,32'h7d05b8d4,32'h7d05b9c5,32'h7d05bab6,32'h7d05bba7,32'h7d05bc98,32'h7d05bd89,32'h7d05be7a};
        membuf[184:191] =  {32'h7d05bf6b,32'h7d05c05c,32'h7d05c14d,32'h7d05c23e,32'h7d05c32f,32'h7d05c420,32'h7d05c511,32'h7d05c602};
        membuf[192:199] =  {32'h7d05c6f3,32'h7d05c7e4,32'h7d05c8d5,32'h7d05c9c6,32'h7d05cab7,32'h7d05cba8,32'h7d05cc99,32'h7d05cd8a};
        membuf[200:207] =  {32'h7d05ce7b,32'h7d05cf6c,32'h7d05d05d,32'h7d05d14e,32'h7d05d23f,32'h7d05d330,32'h7d05d421,32'h7d05d512};
        membuf[208:215] =  {32'h7d05d603,32'h7d05d6f4,32'h7d05d7e5,32'h7d05d8d6,32'h7d05d9c7,32'h7d05dab8,32'h7d05dba9,32'h7d05dc9a};
        membuf[216:223] =  {32'h7d05dd8b,32'h7d05de7c,32'h7d05df6d,32'h7d05e05e,32'h7d05e14f,32'h7d05e240,32'h7d05e331,32'h7d05e422};
        membuf[224:231] =  {32'h7d05e513,32'h7d05e604,32'h7d05e6f5,32'h7d05e7e6,32'h7d05e8d7,32'h7d05e9c8,32'h7d05eab9,32'h7d05ebaa};
        membuf[232:239] =  {32'h7d05ec9b,32'h7d05ed8c,32'h7d05ee7d,32'h7d05ef6e,32'h7d05f05f,32'h7d05f150,32'h7d05f241,32'h7d05f332};
        membuf[240:247] =  {32'h7d05f423,32'h7d05f514,32'h7d05f605,32'h7d05f6f6,32'h7d05f7e7,32'h7d05f8d8,32'h7d05f9c9,32'h7d05faba};
        membuf[248:255] =  {32'h7d05fbab,32'h7d05fc9c,32'h7d05fd8d,32'h7d05fe7e,32'h7d05ff6f,32'h0,32'h0,32'h0};
            memwr(SEGADDR_PIB, 256);
            sfrwr( PKE_segptr_PIB1, 'd128);
            sfrwr( PKE_crfunc, 32'h19 ); // RSAMA
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading 64 bit RSAMA result\n");
            memrd(SEGADDR_POB,128);


#(100 `US);
$finish;

// RSA MS 4096
    /*
    #( 10 `US );
        $display("\n@I:: RSAMS 64");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd4001);

    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);

        //N the modulo for RSA, product of two large prime
        membuf[0:7] =  {32'h1,32'h9259feeb,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[8:15] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[16:23] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[24:31] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[32:39] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[40:47] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[48:55] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[56:63] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[64:71] =  {32'hbeef9527,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[72:79] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[80:87] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[88:95] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[96:103] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[104:111] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[112:119] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[120:127] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h1,32'h0,32'h0};
        memwr(SEGADDR_PCON,128);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
         membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[65:72] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[73:80] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[81:88] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[89:96] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[97:104] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[105:112] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[113:120] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[121:128] = {32'h0,32'h0,32'h2,32'h0,32'h0,32'h0,32'h0,32'h0};
        memwr(SEGADDR_PCON+128,'d129);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        // X
               membuf[0:7] =  {32'hb,32'hb9025a70,32'hb9025b09,32'hb9025ba2,32'hb9025c3b,32'hb9025cd4,32'hb9025d6d,32'hb9025e06};
            membuf[8:15] =  {32'hb9025e9f,32'hb9025f38,32'hb9025fd1,32'hb902606a,32'hb9026103,32'hb902619c,32'hb9026235,32'hb90262ce};
            membuf[16:23] =  {32'hb9026367,32'hb9026400,32'hb9026499,32'hb9026532,32'hb90265cb,32'hb9026664,32'hb90266fd,32'hb9026796};
            membuf[24:31] =  {32'hb902682f,32'hb90268c8,32'hb9026961,32'hb90269fa,32'hb9026a93,32'hb9026b2c,32'hb9026bc5,32'hb9026c5e};
            membuf[32:39] =  {32'hb9026cf7,32'hb9026d90,32'hb9026e29,32'hb9026ec2,32'hb9026f5b,32'hb9026ff4,32'hb902708d,32'hb9027126};
            membuf[40:47] =  {32'hb90271bf,32'hb9027258,32'hb90272f1,32'hb902738a,32'hb9027423,32'hb90274bc,32'hb9027555,32'hb90275ee};
            membuf[48:55] =  {32'hb9027687,32'hb9027720,32'hb90277b9,32'hb9027852,32'hb90278eb,32'hb9027984,32'hb9027a1d,32'hb9027ab6};
            membuf[56:63] =  {32'hb9027b4f,32'hb9027be8,32'hb9027c81,32'hb9027d1a,32'hb9027db3,32'hb9027e4c,32'hb9027ee5,32'hb9027f7e};
            membuf[64:71] =  {32'hb9028017,32'hb90280b0,32'hb9028149,32'hb90281e2,32'hb902827b,32'hb9028314,32'hb90283ad,32'hb9028446};
            membuf[72:79] =  {32'hb90284df,32'hb9028578,32'hb9028611,32'hb90286aa,32'hb9028743,32'hb90287dc,32'hb9028875,32'hb902890e};
            membuf[80:87] =  {32'hb90289a7,32'hb9028a40,32'hb9028ad9,32'hb9028b72,32'hb9028c0b,32'hb9028ca4,32'hb9028d3d,32'hb9028dd6};
            membuf[88:95] =  {32'hb9028e6f,32'hb9028f08,32'hb9028fa1,32'hb902903a,32'hb90290d3,32'hb902916c,32'hb9029205,32'hb902929e};
            membuf[96:103] =  {32'hb9029337,32'hb90293d0,32'hb9029469,32'hb9029502,32'hb902959b,32'hb9029634,32'hb90296cd,32'hb9029766};
            membuf[104:111] =  {32'hb90297ff,32'hb9029898,32'hb9029931,32'hb90299ca,32'hb9029a63,32'hb9029afc,32'hb9029b95,32'hb9029c2e};
            membuf[112:119] =  {32'hb9029cc7,32'hb9029d60,32'hb9029df9,32'hb9029e92,32'hb9029f2b,32'hb9029fc4,32'hb902a05d,32'hb902a0f6};
            membuf[120:127] =  {32'hb902a18f,32'hb902a228,32'hb902a2c1,32'hb902a35a,32'hb902a3f3,32'h0,32'h0,32'h0};
        // Y
            membuf[128:135] =  {32'h7,32'h7d058ba4,32'h7d058c95,32'h7d058d86,32'h7d058e77,32'h7d058f68,32'h7d059059,32'h7d05914a};
        membuf[136:143] =  {32'h7d05923b,32'h7d05932c,32'h7d05941d,32'h7d05950e,32'h7d0595ff,32'h7d0596f0,32'h7d0597e1,32'h7d0598d2};
        membuf[144:151] =  {32'h7d0599c3,32'h7d059ab4,32'h7d059ba5,32'h7d059c96,32'h7d059d87,32'h7d059e78,32'h7d059f69,32'h7d05a05a};
        membuf[152:159] =  {32'h7d05a14b,32'h7d05a23c,32'h7d05a32d,32'h7d05a41e,32'h7d05a50f,32'h7d05a600,32'h7d05a6f1,32'h7d05a7e2};
        membuf[160:167] =  {32'h7d05a8d3,32'h7d05a9c4,32'h7d05aab5,32'h7d05aba6,32'h7d05ac97,32'h7d05ad88,32'h7d05ae79,32'h7d05af6a};
        membuf[168:175] =  {32'h7d05b05b,32'h7d05b14c,32'h7d05b23d,32'h7d05b32e,32'h7d05b41f,32'h7d05b510,32'h7d05b601,32'h7d05b6f2};
        membuf[176:183] =  {32'h7d05b7e3,32'h7d05b8d4,32'h7d05b9c5,32'h7d05bab6,32'h7d05bba7,32'h7d05bc98,32'h7d05bd89,32'h7d05be7a};
        membuf[184:191] =  {32'h7d05bf6b,32'h7d05c05c,32'h7d05c14d,32'h7d05c23e,32'h7d05c32f,32'h7d05c420,32'h7d05c511,32'h7d05c602};
        membuf[192:199] =  {32'h7d05c6f3,32'h7d05c7e4,32'h7d05c8d5,32'h7d05c9c6,32'h7d05cab7,32'h7d05cba8,32'h7d05cc99,32'h7d05cd8a};
        membuf[200:207] =  {32'h7d05ce7b,32'h7d05cf6c,32'h7d05d05d,32'h7d05d14e,32'h7d05d23f,32'h7d05d330,32'h7d05d421,32'h7d05d512};
        membuf[208:215] =  {32'h7d05d603,32'h7d05d6f4,32'h7d05d7e5,32'h7d05d8d6,32'h7d05d9c7,32'h7d05dab8,32'h7d05dba9,32'h7d05dc9a};
        membuf[216:223] =  {32'h7d05dd8b,32'h7d05de7c,32'h7d05df6d,32'h7d05e05e,32'h7d05e14f,32'h7d05e240,32'h7d05e331,32'h7d05e422};
        membuf[224:231] =  {32'h7d05e513,32'h7d05e604,32'h7d05e6f5,32'h7d05e7e6,32'h7d05e8d7,32'h7d05e9c8,32'h7d05eab9,32'h7d05ebaa};
        membuf[232:239] =  {32'h7d05ec9b,32'h7d05ed8c,32'h7d05ee7d,32'h7d05ef6e,32'h7d05f05f,32'h7d05f150,32'h7d05f241,32'h7d05f332};
        membuf[240:247] =  {32'h7d05f423,32'h7d05f514,32'h7d05f605,32'h7d05f6f6,32'h7d05f7e7,32'h7d05f8d8,32'h7d05f9c9,32'h7d05faba};
        membuf[248:255] =  {32'h7d05fbab,32'h7d05fc9c,32'h7d05fd8d,32'h7d05fe7e,32'h7d05ff6f,32'h0,32'h0,32'h0};
            memwr(SEGADDR_PIB, 256);
            sfrwr( PKE_segptr_PIB1, 'h128);
            sfrwr( PKE_crfunc, 32'h1A ); // RSAMS
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading 64 bit RSAMS result\n");
            memrd(SEGADDR_POB,128);
        */




// RSA MS 64

    /*
    #( 10 `US );

        $display("\n@I:: RSAMA 64");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd64);
        sfrwr( PKE_optew, 32'd64);

    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);

        //N the modulo for RSA, product of two large prime
        membuf[0:1] = {32'h000003D9,32'h8fffffbe};
        memwr(SEGADDR_PCON,2);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:2] = {32'h00000000,32'h00000000,32'h1};
            memwr(SEGADDR_PCON+2,3);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        // X
               membuf[0:1] = {32'h000003D9,32'h8fffffbe};
        // Y
            membuf[2:3] =  {32'h1,32'h0};
            memwr(SEGADDR_PIB, 4);
            sfrwr( PKE_segptr_PIB1, 'h2);
            sfrwr( PKE_crfunc, 32'h1A ); // RSAMM
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading 64 bit RSAMA result\n");
            memrd(SEGADDR_POB, 2);

            */

// RSAME 2048
    /*
    #( 10 `US );

        $display("\n@I:: RSAME 2048");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd2047);
        sfrwr( PKE_optew, 32'd4);
    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);

    //N the modulo for RSA
    membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[16:23] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[24:31] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[32:39] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[40:47] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[48:55] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[56:63] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h504c796c};

        memwr(SEGADDR_PCON,64);

    //H, to fill the remaining position, need to summarize the pattern

        membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h80000000,32'h0};
    memwr(SEGADDR_PCON+64,65);

        sfrwr( PKE_crfunc, 32'h11); //RSAINIT

        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
    // RSAME
        // R0
            membuf[0:7] ={32'h1,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[8:15] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[16:23] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[24:31] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[32:39] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[40:47] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[48:55] ={32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
            membuf[56:63] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
            // R1
        membuf[64:71] ={32'h589c0865,'hc7c33eee,'h2d180131,'hd49e800d,'h9027fab4,'ha20f82b9,'hdb29ff6a,'h05fa765f};
        membuf[72:79] = {32'h7e258c53,'h26274633,'h7b441475,'h03c10eab,'heb547d58,'hedd99d02,'h7746a188,'h09e1f8b6};
        membuf[80:87] = {32'hdeb1cf17,'hd032d70a,'h764d4818,'h8f862685,'ha378c325,'he77e8571,'h62b4e9da,'h4e1ec85e};
        membuf[88:95] = {32'hc126d958,'h8c4c789c,'h6319a5c3,'h3f5450c4,'h0179d4c5,'h5867c130,'he459de08,'h9c176e7e};
        membuf[96:103] = {32'h6d6eb3bf,'h23ddb111,'h8d91341e,'hdc94148f,'h4d41b8dd,'h08fbd964,'h471e88ba,'hba36733e};
        membuf[104:111] = {32'h2b6f64f4,'h5e4d0a90,'h3b9dfcd0,'h2eacfb0f,'hcfb67718,'hbea15538,'hd1ebf097,'h72e25ec7};
        membuf[112:119] = {32'h4213f49e,'h03040c41,'hb4260582,'hfd058c6c,'hcec21a1b,'h44c3bed2,'hcca91d47,'h8a82b741};
        membuf[120:127] = {32'hfc406c65,'hda6b3e4d,'h411258db,'h110750cd,'h944ca991,'h60c89c5a,'h7e3e1773,'h4b8007d4};

            memwr(SEGADDR_PIB, 128);
            sfrwr( PKE_segptr_PIB1, 'h64);
        // E
             membuf[0:7] =   {32'hd,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};

        memwr(SEGADDR_PKB, 8);

            sfrwr( PKE_crfunc, 32'h13 ); // RSAME
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading RSAME result\n");
            memrd(SEGADDR_POB, 64);

        */

 // RSAMM 4064
    /*

    #( 10 `US );

        $display("\n@I:: RSA 4064");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd4064);
        sfrwr( PKE_optew, 32'd256 );
    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);
    //N
   // split_Big_number();
        membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[16:23] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[24:31] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[32:39] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[40:47] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[48:55] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[56:63] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h504c796c};
        membuf[64:71] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[72:79] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[80:87] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[88:95] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[96:103] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[104:111] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[112:119] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[120:126] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h96b17026};

        memwr(SEGADDR_PCON,127);
    //H, to fill the remaining position, need to summarize the pattern

        membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[65:72] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[73:80] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[81:88] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[89:96] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[97:104] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[105:112] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[113:120] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[121:127] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h1};
        memwr(SEGADDR_PCON+127,'d128);

        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
   //X
           membuf[0:8] = {32'h7,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[65:72] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[73:80] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[81:88] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[89:96] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[97:104] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[105:112] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[113:120] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[121:126] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};


    // Y

        membuf[127:135] =  {32'hb,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[136:143] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[144:151] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[152:159] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[160:167] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[168:175] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[176:183] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[184:191] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[192:199] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[200:207] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[208:215] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[216:223] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[224:231] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[232:239] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[240:247] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[248:253] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};

        memwr(SEGADDR_PIB, 'd254);
        sfrwr( PKE_segptr_PIB1, 'd127);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading 4064 bit RSAMM result\n");
        memrd(SEGADDR_POB, 128);

        */

// 2048bit RSAMM test


    #( 10 `US );

        $display("\n@I:: RSA");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd2047);
        sfrwr( PKE_optew, 32'd256 );
          // sfrwr( GLB_ar,'ha5 );
    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);

    //N the modulo for RSA
        // membuf[0:7] ={32'h504c796c,'h46b17026,'h362e6cbe,'h3b4b5138,'h3be1e6a0,'h6c59c36b,'he2257b0a,'h5c820cd1};
        // membuf[8:15] = {32'h4f6e860a,'h31e6f953,'h0864a33f,'h3597582b,'h7c62335e,'h57197e67,'h7bf9ddb2,'ha9e7c06a};
        // membuf[16:23] = {32'hb86c0a95,'he1effc6d,'h4c3466fb,'hdb130ff8,'h347b64e6,'h13a85e6c,'hd0a3b8f1,'h2ef9507a};
        // membuf[24:31] = {32'h65bedcc7,'h70c4d0ab,'h99168eab,'h45b9ced8,'hfba54ef0,'hbaffbdb1,'h7b9be37f,'h03b11339};
        // membuf[32:39] = {32'hecddd557,'hf55bce46,'h8882f206,'h8b80ec03,'h6a56cb33,'h6316f070,'h135a3616,'h400662df};
        // membuf[40:47] = {32'he740cdfc,'h87ae4e76,'haff16bc6,'hc561c2b1,'h1709b368,'h26e352df,'h2f578b6d,'hfdf095a3};
        // membuf[48:55] = {32'hed5f9b6f,'h40b4a773,'ha7dad0a1,'h0b52a71a,'h9b33de47,'h1a603838,'h690ab73b,'h526703be};
        // membuf[56:63] = {32'h95b31768,'h37663175,'h08b5fa50,'h754ef476,'h8e4e2388,'h5984fdb1,'h58ec9539,'h57640469};

        membuf[0:7] ={32'h57640469,'h58ec9539,'h5984fdb1,'h8e4e2388,'h754ef476,'h08b5fa50,'h37663175,'h95b31768};
        membuf[8:15] = {32'h526703be,'h690ab73b,'h1a603838,'h9b33de47,'h0b52a71a,'ha7dad0a1,'h40b4a773,'hed5f9b6f};
        membuf[16:23] = {32'hfdf095a3,'h2f578b6d,'h26e352df,'h1709b368,'hc561c2b1,'haff16bc6,'h87ae4e76,'he740cdfc};
        membuf[24:31] = {32'h400662df,'h135a3616,'h6316f070,'h6a56cb33,'h8b80ec03,'h8882f206,'hf55bce46,'hecddd557};
        membuf[32:39] = {32'h03b11339,'h7b9be37f,'hbaffbdb1,'hfba54ef0,'h45b9ced8,'h99168eab,'h70c4d0ab,'h65bedcc7};
        membuf[40:47] = {32'h2ef9507a,'hd0a3b8f1,'h13a85e6c,'h347b64e6,'hdb130ff8,'h4c3466fb,'he1effc6d,'hb86c0a95};
        membuf[48:55] = {32'ha9e7c06a,'h7bf9ddb2,'h57197e67,'h7c62335e,'h3597582b,'h0864a33f,'h31e6f953,'h4f6e860a};
        membuf[56:63] = {32'h5c820cd1,'he2257b0a,'h6c59c36b,'h3be1e6a0,'h3b4b5138,'h362e6cbe,'h46b17026,'h504c796c};

        memwr(SEGADDR_PCON,64);
    //H, to fill the remaining position, need to summarize the pattern

    membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h80000000,32'h0};
    memwr(SEGADDR_PCON+64,'d65);

        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
    // R0, X
        // membuf[0:7] ={32'h4b8007d4,'h7e3e1773,'h60c89c5a,'h944ca991,'h110750cd,'h411258db,'hda6b3e4d,'hfc406c65};
        // membuf[8:15] = {32'h8a82b741,'hcca91d47,'h44c3bed2,'hcec21a1b,'hfd058c6c,'hb4260582,'h03040c41,'h4213f49e};
        // membuf[16:23] = {32'h72e25ec7,'hd1ebf097,'hbea15538,'hcfb67718,'h2eacfb0f,'h3b9dfcd0,'h5e4d0a90,'h2b6f64f4};
        // membuf[24:31] = {32'hba36733e,'h471e88ba,'h08fbd964,'h4d41b8dd,'hdc94148f,'h8d91341e,'h23ddb111,'h6d6eb3bf};
        // membuf[32:39] = {32'h9c176e7e,'he459de08,'h5867c130,'h0179d4c5,'h3f5450c4,'h6319a5c3,'h8c4c789c,'hc126d958};
        // membuf[40:47] = {32'h4e1ec85e,'h62b4e9da,'he77e8571,'ha378c325,'h8f862685,'h764d4818,'hd032d70a,'hdeb1cf17};
        // membuf[48:55] = {32'h09e1f8b6,'h7746a188,'hedd99d02,'heb547d58,'h03c10eab,'h7b441475,'h26274633,'h7e258c53};
        // membuf[56:63] = {32'h05fa765f,'hdb29ff6a,'ha20f82b9,'h9027fab4,'hd49e800d,'h2d180131,'hc7c33eee,'h589c0865};

        // membuf[0:7] ={32'h589c0865,'hc7c33eee,'h2d180131,'hd49e800d,'h9027fab4,'ha20f82b9,'hdb29ff6a,'h05fa765f};
        // membuf[8:15] = {32'h7e258c53,'h26274633,'h7b441475,'h03c10eab,'heb547d58,'hedd99d02,'h7746a188,'h09e1f8b6};
        // membuf[16:23] = {32'hdeb1cf17,'hd032d70a,'h764d4818,'h8f862685,'ha378c325,'he77e8571,'h62b4e9da,'h4e1ec85e};
        // membuf[24:31] = {32'hc126d958,'h8c4c789c,'h6319a5c3,'h3f5450c4,'h0179d4c5,'h5867c130,'he459de08,'h9c176e7e};
        // membuf[32:39] = {32'h6d6eb3bf,'h23ddb111,'h8d91341e,'hdc94148f,'h4d41b8dd,'h08fbd964,'h471e88ba,'hba36733e};
        // membuf[40:47] = {32'h2b6f64f4,'h5e4d0a90,'h3b9dfcd0,'h2eacfb0f,'hcfb67718,'hbea15538,'hd1ebf097,'h72e25ec7};
        // membuf[48:55] = {32'h4213f49e,'h03040c41,'hb4260582,'hfd058c6c,'hcec21a1b,'h44c3bed2,'hcca91d47,'h8a82b741};
        // membuf[56:63] = {32'hfc406c65,'hda6b3e4d,'h411258db,'h110750cd,'h944ca991,'h60c89c5a,'h7e3e1773,'h4b8007d4};


        membuf[0:7] ={32'h11,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[8:15] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[16:23] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[24:31] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[32:39] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[40:47] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[48:55] ={32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[56:63] =  {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};

    // R1, Y
        // membuf[64:71] ={32'h589c0865,'hc7c33eee,'h2d180131,'hd49e800d,'h9027fab4,'ha20f82b9,'hdb29ff6a,'h05fa765f};
        // membuf[72:79] = {32'h7e258c53,'h26274633,'h7b441475,'h03c10eab,'heb547d58,'hedd99d02,'h7746a188,'h09e1f8b6};
        // membuf[80:87] = {32'hdeb1cf17,'hd032d70a,'h764d4818,'h8f862685,'ha378c325,'he77e8571,'h62b4e9da,'h4e1ec85e};
        // membuf[88:95] = {32'hc126d958,'h8c4c789c,'h6319a5c3,'h3f5450c4,'h0179d4c5,'h5867c130,'he459de08,'h9c176e7e};
        // membuf[96:103] = {32'h6d6eb3bf,'h23ddb111,'h8d91341e,'hdc94148f,'h4d41b8dd,'h08fbd964,'h471e88ba,'hba36733e};
        // membuf[104:111] = {32'h2b6f64f4,'h5e4d0a90,'h3b9dfcd0,'h2eacfb0f,'hcfb67718,'hbea15538,'hd1ebf097,'h72e25ec7};
        // membuf[112:119] = {32'h4213f49e,'h03040c41,'hb4260582,'hfd058c6c,'hcec21a1b,'h44c3bed2,'hcca91d47,'h8a82b741};
        // membuf[120:127] = {32'hfc406c65,'hda6b3e4d,'h411258db,'h110750cd,'h944ca991,'h60c89c5a,'h7e3e1773,'h4b8007d4};

      membuf[64:71] ={32'h13,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[72:79] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[80:87] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[88:95] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[96:103] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[104:111] ={32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[112:119] ={32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000000};
        membuf[120:127] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};

        memwr(SEGADDR_PIB, 'd128);
        sfrwr( PKE_segptr_PIB1, 'd64);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading RSAMM result\n");
        memrd(SEGADDR_POB, 64);




// RSAMM 4096 bit
    /*
    #( 10 `US );

        $display("\n@I:: RSA 4096");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd4096);
        sfrwr( PKE_optew, 32'd256 );
    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);
    //N
   // split_Big_number();
        membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[16:23] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[24:31] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[32:39] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[40:47] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[48:55] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[56:63] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h504c796c};
        membuf[64:71] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[72:79] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[80:87] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[88:95] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[96:103] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[104:111] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[112:119] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[120:127] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h904c796c};




        memwr(SEGADDR_PCON,128);
    //H, to fill the remaining position, need to summarize the pattern

        membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[65:72] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[73:80] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[81:88] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[89:96] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[97:104] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[105:112] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[113:120] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
        membuf[121:128] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h1};
        memwr(SEGADDR_PCON+128,'d129);

        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
   //X
        membuf[0:7] =  {32'h589c0865,32'hc7c33eee,32'h2d180131,32'hd49e800d,32'h9027fab4,32'ha20f82b9,32'hdb29ff6a,32'h5fa765f};
        membuf[8:15] =  {32'h7e258c53,32'h26274633,32'h7b441475,32'h3c10eab,32'heb547d58,32'hedd99d02,32'h7746a188,32'h9e1f8b6};
        membuf[16:23] =  {32'hdeb1cf17,32'hd032d70a,32'h764d4818,32'h8f862685,32'ha378c325,32'he77e8571,32'h62b4e9da,32'h4e1ec85e};
        membuf[24:31] =  {32'hc126d958,32'h8c4c789c,32'h6319a5c3,32'h3f5450c4,32'h179d4c5,32'h5867c130,32'he459de08,32'h9c176e7e};
        membuf[32:39] =  {32'h6d6eb3bf,32'h23ddb111,32'h8d91341e,32'hdc94148f,32'h4d41b8dd,32'h8fbd964,32'h471e88ba,32'hba36733e};
        membuf[40:47] =  {32'h2b6f64f4,32'h5e4d0a90,32'h3b9dfcd0,32'h2eacfb0f,32'hcfb67718,32'hbea15538,32'hd1ebf097,32'h72e25ec7};
        membuf[48:55] =  {32'h4213f49e,32'h3040c41,32'hb4260582,32'hfd058c6c,32'hcec21a1b,32'h44c3bed2,32'hcca91d47,32'h8a82b741};
        membuf[56:63] =  {32'hfc406c65,32'hda6b3e4d,32'h411258db,32'h110750cd,32'h944ca991,32'h60c89c5a,32'h7e3e1773,32'h4b8007d4};
        membuf[64:71] =  {32'h589c0865,32'hc7c33eee,32'h2d180131,32'hd49e800d,32'h9027fab4,32'ha20f82b9,32'hdb29ff6a,32'h5fa765f};
        membuf[72:79] =  {32'h7e258c53,32'h26274633,32'h7b441475,32'h3c10eab,32'heb547d58,32'hedd99d02,32'h7746a188,32'h9e1f8b6};
        membuf[80:87] =  {32'hdeb1cf17,32'hd032d70a,32'h764d4818,32'h8f862685,32'ha378c325,32'he77e8571,32'h62b4e9da,32'h4e1ec85e};
        membuf[88:95] =  {32'hc126d958,32'h8c4c789c,32'h6319a5c3,32'h3f5450c4,32'h179d4c5,32'h5867c130,32'he459de08,32'h9c176e7e};
        membuf[96:103] =  {32'h6d6eb3bf,32'h23ddb111,32'h8d91341e,32'hdc94148f,32'h4d41b8dd,32'h8fbd964,32'h471e88ba,32'hba36733e};
        membuf[104:111] =  {32'h2b6f64f4,32'h5e4d0a90,32'h3b9dfcd0,32'h2eacfb0f,32'hcfb67718,32'hbea15538,32'hd1ebf097,32'h72e25ec7};
        membuf[112:119] =  {32'h4213f49e,32'h3040c41,32'hb4260582,32'hfd058c6c,32'hcec21a1b,32'h44c3bed2,32'hcca91d47,32'h8a82b741};
        membuf[120:127] =  {32'hfc406c65,32'hda6b3e4d,32'h411258db,32'h110750cd,32'h944ca991,32'h60c89c5a,32'h7e3e1773,32'h4b8007d4};


    // Y

        membuf[128:135] =  {32'h589c0865,32'hc7c33eee,32'h2d180131,32'hd49e800d,32'h9027fab4,32'ha20f82b9,32'hdb29ff6a,32'h5fa765f};
        membuf[136:143] =  {32'h7e258c53,32'h26274633,32'h7b441475,32'h3c10eab,32'heb547d58,32'hedd99d02,32'h7746a188,32'h9e1f8b6};
        membuf[144:151] =  {32'hdeb1cf17,32'hd032d70a,32'h764d4818,32'h8f862685,32'ha378c325,32'he77e8571,32'h62b4e9da,32'h4e1ec85e};
        membuf[152:159] =  {32'hc126d958,32'h8c4c789c,32'h6319a5c3,32'h3f5450c4,32'h179d4c5,32'h5867c130,32'he459de08,32'h9c176e7e};
        membuf[160:167] =  {32'h6d6eb3bf,32'h23ddb111,32'h8d91341e,32'hdc94148f,32'h4d41b8dd,32'h8fbd964,32'h471e88ba,32'hba36733e};
        membuf[168:175] =  {32'h2b6f64f4,32'h5e4d0a90,32'h3b9dfcd0,32'h2eacfb0f,32'hcfb67718,32'hbea15538,32'hd1ebf097,32'h72e25ec7};
        membuf[176:183] =  {32'h4213f49e,32'h3040c41,32'hb4260582,32'hfd058c6c,32'hcec21a1b,32'h44c3bed2,32'hcca91d47,32'h8a82b741};
        membuf[184:191] =  {32'hfc406c65,32'hda6b3e4d,32'h411258db,32'h110750cd,32'h944ca991,32'h60c89c5a,32'h7e3e1773,32'h4b8007d4};
        membuf[192:199] =  {32'h589c0865,32'hc7c33eee,32'h2d180131,32'hd49e800d,32'h9027fab4,32'ha20f82b9,32'hdb29ff6a,32'h5fa765f};
        membuf[200:207] =  {32'h7e258c53,32'h26274633,32'h7b441475,32'h3c10eab,32'heb547d58,32'hedd99d02,32'h7746a188,32'h9e1f8b6};
        membuf[208:215] =  {32'hdeb1cf17,32'hd032d70a,32'h764d4818,32'h8f862685,32'ha378c325,32'he77e8571,32'h62b4e9da,32'h4e1ec85e};
        membuf[216:223] =  {32'hc126d958,32'h8c4c789c,32'h6319a5c3,32'h3f5450c4,32'h179d4c5,32'h5867c130,32'he459de08,32'h9c176e7e};
        membuf[224:231] =  {32'h6d6eb3bf,32'h23ddb111,32'h8d91341e,32'hdc94148f,32'h4d41b8dd,32'h8fbd964,32'h471e88ba,32'hba36733e};
        membuf[232:239] =  {32'h2b6f64f4,32'h5e4d0a90,32'h3b9dfcd0,32'h2eacfb0f,32'hcfb67718,32'hbea15538,32'hd1ebf097,32'h72e25ec7};
        membuf[240:247] =  {32'h4213f49e,32'h3040c41,32'hb4260582,32'hfd058c6c,32'hcec21a1b,32'h44c3bed2,32'hcca91d47,32'h8a82b741};
        membuf[248:255] =  {32'hfc406c65,32'hda6b3e4d,32'h411258db,32'h110750cd,32'h944ca991,32'h60c89c5a,32'h7e3e1773,32'h4b8007d4};

        memwr(SEGADDR_PIB, 'd256);
        sfrwr( PKE_segptr_PIB1, 'd128);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading 4096 bit RSAMM result\n");
        memrd(SEGADDR_POB, 128);

    */

// GCD 4096
    /*
    #( 10 `US );

        $display("\n@I:: GCD 4096");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd4096);
        sfrwr( PKE_optew, 32'd256 );
    #( 1 `US);
        sfrwr(SDMA_ichcr_segid, 32'h0b09);
        sfrwr(SDMA_ichcr_transize, 32'd32);
        sfrwr(SDMA_chstart_ar,32'h5a);
    //N
   // split_Big_number();
        membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[16:23] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[24:31] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[32:39] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[40:47] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[48:55] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[56:63] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h504c796c};
        membuf[64:71] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[72:79] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[80:87] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[88:95] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[96:103] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[104:111] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[112:119] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[120:127] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h904c796c};

        memwr(SEGADDR_PCON,128);
    //H, to fill the remaining position, need to summarize the pattern

    membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[9:16] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[17:24] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[25:32] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[33:40] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[41:48] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[49:56] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[57:64] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[65:72] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[73:80] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[81:88] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[89:96] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[97:104] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[105:112] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[113:120] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
    membuf[121:128] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h1};
    memwr(SEGADDR_PCON+128,'d129);

        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
   //X
        membuf[0:7] =  {32'h589c0865,32'hc7c33eee,32'h2d180131,32'hd49e800d,32'h9027fab4,32'ha20f82b9,32'hdb29ff6a,32'h5fa765f};
        membuf[8:15] =  {32'h7e258c53,32'h26274633,32'h7b441475,32'h3c10eab,32'heb547d58,32'hedd99d02,32'h7746a188,32'h9e1f8b6};
        membuf[16:23] =  {32'hdeb1cf17,32'hd032d70a,32'h764d4818,32'h8f862685,32'ha378c325,32'he77e8571,32'h62b4e9da,32'h4e1ec85e};
        membuf[24:31] =  {32'hc126d958,32'h8c4c789c,32'h6319a5c3,32'h3f5450c4,32'h179d4c5,32'h5867c130,32'he459de08,32'h9c176e7e};
        membuf[32:39] =  {32'h6d6eb3bf,32'h23ddb111,32'h8d91341e,32'hdc94148f,32'h4d41b8dd,32'h8fbd964,32'h471e88ba,32'hba36733e};
        membuf[40:47] =  {32'h2b6f64f4,32'h5e4d0a90,32'h3b9dfcd0,32'h2eacfb0f,32'hcfb67718,32'hbea15538,32'hd1ebf097,32'h72e25ec7};
        membuf[48:55] =  {32'h4213f49e,32'h3040c41,32'hb4260582,32'hfd058c6c,32'hcec21a1b,32'h44c3bed2,32'hcca91d47,32'h8a82b741};
        membuf[56:63] =  {32'hfc406c65,32'hda6b3e4d,32'h411258db,32'h110750cd,32'h944ca991,32'h60c89c5a,32'h7e3e1773,32'h4b8007d4};
        membuf[64:71] =  {32'h589c0865,32'hc7c33eee,32'h2d180131,32'hd49e800d,32'h9027fab4,32'ha20f82b9,32'hdb29ff6a,32'h5fa765f};
        membuf[72:79] =  {32'h7e258c53,32'h26274633,32'h7b441475,32'h3c10eab,32'heb547d58,32'hedd99d02,32'h7746a188,32'h9e1f8b6};
        membuf[80:87] =  {32'hdeb1cf17,32'hd032d70a,32'h764d4818,32'h8f862685,32'ha378c325,32'he77e8571,32'h62b4e9da,32'h4e1ec85e};
        membuf[88:95] =  {32'hc126d958,32'h8c4c789c,32'h6319a5c3,32'h3f5450c4,32'h179d4c5,32'h5867c130,32'he459de08,32'h9c176e7e};
        membuf[96:103] =  {32'h6d6eb3bf,32'h23ddb111,32'h8d91341e,32'hdc94148f,32'h4d41b8dd,32'h8fbd964,32'h471e88ba,32'hba36733e};
        membuf[104:111] =  {32'h2b6f64f4,32'h5e4d0a90,32'h3b9dfcd0,32'h2eacfb0f,32'hcfb67718,32'hbea15538,32'hd1ebf097,32'h72e25ec7};
        membuf[112:119] =  {32'h4213f49e,32'h3040c41,32'hb4260582,32'hfd058c6c,32'hcec21a1b,32'h44c3bed2,32'hcca91d47,32'h8a82b741};
        membuf[120:127] =  {32'hfc406c65,32'hda6b3e4d,32'h411258db,32'h110750cd,32'h944ca991,32'h60c89c5a,32'h7e3e1773,32'h4b8007d4};

    // Y

       membuf[128:135] =  {32'h57640466,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[136:143] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[144:151] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[152:159] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[160:167] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[168:175] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[176:183] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[184:191] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h504c796c};
        membuf[192:199] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[200:207] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[208:215] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[216:223] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[224:231] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[232:239] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[240:247] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[248:255] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h404c796c};
        memwr(SEGADDR_PIB, 'd256);
        sfrwr( PKE_segptr_PIB1, 'd128);
        sfrwr( PKE_crfunc, 32'h51 ); // GCD
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading 4096 bit GCD result\n");
        memrd(SEGADDR_POB, 128);

        */

//MODINV 512
    /*
    #(10 `US);
    $display("\n@I:: INVINIT 512");
    sfrwr( GLB_suben, '1);
    sfrwr( PKE_optnw, 32'd512);
    // P:
        membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};

        memwr(SEGADDR_PCON, 16);

        sfrwr( PKE_crfunc, 32'hfe);
        sfrrd(PKE_crfunc);
        sfrwr( PKE_ar, 32'h5a); sfrwait( PKE_fr, 32'h1, 32'h1); sfrwr( PKE_fr, 32'h1);

    // U
        membuf[0:2] ={32'h11,32'h00000000,32'h00000001};
        memwr(SEGADDR_PIB, 3);

        sfrwr( PKE_crfunc, 32'h18); // MODINV
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading MODINV result\n");
        memrd(SEGADDR_POB,16);
    */

// MODINV 1024
    /*
    #(10 `US);
    $display("\n@I:: INVINIT 1024");
    sfrwr( GLB_suben, '1);
    sfrwr( PKE_optnw, 32'd1024);
    // P:
        membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[16:23] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[24:31] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};

        memwr(SEGADDR_PCON, 32);

        sfrwr( PKE_crfunc, 32'hfe);
        sfrrd(PKE_crfunc);
        sfrwr( PKE_ar, 32'h5a); sfrwait( PKE_fr, 32'h1, 32'h1); sfrwr( PKE_fr, 32'h1);

    // U
        membuf[0:2] ={32'h11,32'h00000000,32'h00000001};
        memwr(SEGADDR_PIB, 3);

        sfrwr( PKE_crfunc, 32'h18); // MODINV
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading MODINV result\n");
        memrd(SEGADDR_POB,32);
    */


// MODINV 4096

    /*
    #(10 `US);
    $display("\n@I:: INVINIT 4096");
    sfrwr( GLB_suben, '1);

    sfrwr( PKE_optnw, 32'd4096);
    sfrwr (PKE_optew, 32'd4096);

        #( 1 `US);
            sfrwr(SDMA_ichcr_segid, 32'h0b09);
            sfrwr(SDMA_ichcr_transize, 32'd32);
            sfrwr(SDMA_chstart_ar,32'h5a);

            // P:
              membuf[0:7] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[8:15] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[16:23] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[24:31] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[32:39] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[40:47] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[48:55] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[56:63] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h904c796c};
        membuf[64:71] =  {32'h57640469,32'h58ec9539,32'h5984fdb1,32'h8e4e2388,32'h754ef476,32'h8b5fa50,32'h37663175,32'h95b31768};
        membuf[72:79] =  {32'h526703be,32'h690ab73b,32'h1a603838,32'h9b33de47,32'hb52a71a,32'ha7dad0a1,32'h40b4a773,32'hed5f9b6f};
        membuf[80:87] =  {32'hfdf095a3,32'h2f578b6d,32'h26e352df,32'h1709b368,32'hc561c2b1,32'haff16bc6,32'h87ae4e76,32'he740cdfc};
        membuf[88:95] =  {32'h400662df,32'h135a3616,32'h6316f070,32'h6a56cb33,32'h8b80ec03,32'h8882f206,32'hf55bce46,32'hecddd557};
        membuf[96:103] =  {32'h3b11339,32'h7b9be37f,32'hbaffbdb1,32'hfba54ef0,32'h45b9ced8,32'h99168eab,32'h70c4d0ab,32'h65bedcc7};
        membuf[104:111] =  {32'h2ef9507a,32'hd0a3b8f1,32'h13a85e6c,32'h347b64e6,32'hdb130ff8,32'h4c3466fb,32'he1effc6d,32'hb86c0a95};
        membuf[112:119] =  {32'ha9e7c06a,32'h7bf9ddb2,32'h57197e67,32'h7c62335e,32'h3597582b,32'h864a33f,32'h31e6f953,32'h4f6e860a};
        membuf[120:127] =  {32'h5c820cd1,32'he2257b0a,32'h6c59c36b,32'h3be1e6a0,32'h3b4b5138,32'h362e6cbe,32'h46b17026,32'h904c796c};
          //       membuf[0:7] ={32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h00000007};

            memwr(SEGADDR_PCON, 128);

            sfrwr( PKE_crfunc, 32'hfe);
             sfrrd(PKE_crfunc);
            sfrwr( PKE_ar, 32'h5a); sfrwait( PKE_fr, 32'h1, 32'h1); sfrwr( PKE_fr, 32'h1);

        // memrd(SEGADDR_PCON, 8);

            // U
         membuf[0:2] ={32'h11,32'h00000000,32'h00000001};
     memwr(SEGADDR_PIB, 3);

        sfrwr( PKE_crfunc, 32'h18); // MODINV
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        sfrrd(PKE_sr);
        $write("\n Reading MODINV result\n");
          memrd(SEGADDR_POB, 128);

        */


`maintestend





    task memwr();
        input bit [AW-1:0] thaddr;
        input integer tc;
        $write("@::memwr:: %04x <- ", SCERAM_BA+thaddr );
        for (int i = 0; i < tc; i++) begin
            ahbwr( (SCERAM_BA+thaddr+i)*4, membuf[i] );
            $write(" %08x", membuf[i]);
            @(negedge hclk);
        end
        $write("\n");
    endtask

    task memrd();
        input bit [AW-1:0] thaddr;
        input integer tc;
        $write("@::memrd:: %04x -> ", SCERAM_BA+thaddr );
        for (int i = 0; i < tc; i++) begin
            ahbrd( (SCERAM_BA+thaddr+i)*4 );
            @(negedge hdataphase);
            @(negedge hclk);
            membuf[i] = hrdatareg;
            $write(" %08x", membuf[i]);
        end
        $write("\n");
    endtask

    task checkref();
        input integer tc;
        bit checkbit=1;
        checkbit='1;
        $write("@::chref:: ---- -> " );
        for (int i = 0; i < tc; i++) begin
            $write(" %08x", refbuf[i]);
            checkbit = checkbit & ( refbuf[i] == membuf[i] );
            if(refbuf[i] != membuf[i] )
            $write("@E: !!![%02d][ref:%08x][mem:%08x]\n",i,refbuf[i],membuf[i]);
        end
        $write(" %01x \n", checkbit);
        if(checkbit)
            $write("@i: PASS!\n");
        else begin
            errcnt = errcnt + 1;
            $write("@E: FAIL! %0d\n", errcnt);
           // $stop;
        end
        sfrrd(GLB_tickcyc);
        sfrrd(GLB_tickcnt);
    endtask

    task sfrwr();
        input bit  [AW-1:0]  thaddr;
        input bit  [DW-1:0]  thwdata;
        $write("@::sfrwr:: %04x <- ", thaddr );
        ahbwr( thaddr, thwdata );
        $write(" %08x", thwdata );
        $write("\n");
    endtask


    task sfrrd();
        input bit [AW-1:0] thaddr;
        $write("@::sfrrd:: %04x -> ", thaddr );
        ahbrd( thaddr );
        @(negedge hdataphase);
        @(negedge hclk);
        $write(" %08x", hrdatareg);
        $write("\n");
    endtask

    task sfrwait();
        input bit [AW-1:0] thaddr;
        input bit [DW-1:0] texpdata;
        input bit [DW-1:0] tmask;

        $write("@::sfrwait:: %04x == %08x & %08x ? ", thaddr, texpdata, tmask );
        while(1)begin
            ahbrd( thaddr );
            @(negedge hdataphase);
            @(negedge hclk);
            if( hrdatareg & tmask == texpdata ) break;
        end
        $write(" done! ");
        $write("\n");
    endtask



// ■■■■■■■■■■■■■■■

    task ahbrd();
        input bit  [AW-1:0]  thaddr;
        @( posedge hready & clk);
        @(negedge hclk);
        hsel = 1;
        haddr = thaddr;
        htrans = 'h2;
        hwrite = '0;
        @(negedge hclk);
        hsel = 0;
    endtask : ahbrd

        `theregrn(hdataphase) <= ( hsel & hreadym & |htrans & ~hwrite ) ? 1 : hready ? 0 : hdataphase;
        `theregrn(hrdatareg) <= hdataphase & hready ? hrdata : hrdatareg;

    task ahbwr();
        input bit  [AW-1:0]  thaddr;
        input bit  [DW-1:0]   thwdata;
        @( posedge hready & clk);
        @(negedge hclk);
        hsel = 1;
        haddr = thaddr;
        htrans = 'h2;
        hwrite = '1;
        hwdata = thwdata;
        @(negedge hclk);
        hsel = 0;
        hwrite = '0;
    endtask : ahbwr

    task big_to_little();
    integer i;

        for(i = 0; i <512; i = i+1)begin
    le_membuf[i] = {membuf[i][7:0],membuf[i][15:8],membuf[i][23:16],membuf[i][31:24]};
        end

        for(i = 0; i <512; i = i+1)begin
            membuf[i] = le_membuf[i];
        end
    endtask

        task split_Big_number();
    integer i;
    reg [4095:0] bigNumber = 4096'h0x7d05ff6f7d05fe7e7d05fd8d7d05fc9c7d05fbab7d05faba7d05f9c97d05f8d87d05f7e77d05f6f67d05f6057d05f5147d05f4237d05f3327d05f2417d05f1507d05f05f7d05ef6e7d05ee7d7d05ed8c7d05ec9b7d05ebaa7d05eab97d05e9c87d05e8d77d05e7e67d05e6f57d05e6047d05e5137d05e4227d05e3317d05e2407d05e14f7d05e05e7d05df6d7d05de7c7d05dd8b7d05dc9a7d05dba97d05dab87d05d9c77d05d8d67d05d7e57d05d6f47d05d6037d05d5127d05d4217d05d3307d05d23f7d05d14e7d05d05d7d05cf6c7d05ce7b7d05cd8a7d05cc997d05cba87d05cab77d05c9c67d05c8d57d05c7e47d05c6f37d05c6027d05c5117d05c4207d05c32f7d05c23e7d05c14d7d05c05c7d05bf6b7d05be7a7d05bd897d05bc987d05bba77d05bab67d05b9c57d05b8d47d05b7e37d05b6f27d05b6017d05b5107d05b41f7d05b32e7d05b23d7d05b14c7d05b05b7d05af6a7d05ae797d05ad887d05ac977d05aba67d05aab57d05a9c47d05a8d37d05a7e27d05a6f17d05a6007d05a50f7d05a41e7d05a32d7d05a23c7d05a14b7d05a05a7d059f697d059e787d059d877d059c967d059ba57d059ab47d0599c37d0598d27d0597e17d0596f07d0595ff7d05950e7d05941d7d05932c7d05923b7d05914a7d0590597d058f687d058e777d058d867d058c957d058ba400000007;

    for(i = 0; i< 128; i = i+1) begin
        membuf[i] = bigNumber[i*32 +:32];
    end

    for(i = 0; i < 128; i=i+8) begin
      $display("membuf[%0d:%0d] =  {32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h};", i+128, i+7+128,  membuf[i],membuf[i+1], membuf[i+2], membuf[i+3], membuf[i+4], membuf[i+5], membuf[i+6], membuf[i+7]);
    end
    endtask


endmodule


    module wire2ahbm #(
      parameter AW=16,
      parameter DW=32,
      parameter IDW=8,
      parameter UW=4
     )(
        ahbif.master          ahbm,
        input wire            hsel,
        input wire  [AW-1:0]  haddr,
        input wire  [1:0]     htrans,
        input wire            hwrite,
        input wire  [2:0]     hsize,
        input wire  [2:0]     hburst,
        input wire  [3:0]     hprot,
        input wire  [IDW-1:0] hmaster,
        input bit   [DW-1:0]  hwdata,
        input wire            hmasterlock,
        input wire            hreadym,
        input wire  [UW-1:0]  hauser,
        input wire  [UW-1:0]  hwuser,
        output bit   [DW-1:0]  hrdata,
        output wire            hready,
        output wire            hresp,
        output logic  [UW-1:0]  hruser
    );
        assign ahbm.hsel = hsel;
        assign ahbm.haddr = haddr;
        assign ahbm.htrans = htrans;
        assign ahbm.hwrite = hwrite;
        assign ahbm.hsize = hsize;
        assign ahbm.hburst = hburst;
        assign ahbm.hprot = hprot;
        assign ahbm.hmaster = hmaster;
        assign ahbm.hwdata = hwdata;
        assign ahbm.hmasterlock = hmasterlock;
        assign ahbm.hreadym = hreadym;
        assign ahbm.hauser = hauser;
        assign ahbm.hwuser = hwuser;

        assign hrdata = ahbm.hrdata;
        assign hready = ahbm.hready;
        assign hresp = ahbm.hresp;
        assign hruser = ahbm.hruser;

    endmodule


