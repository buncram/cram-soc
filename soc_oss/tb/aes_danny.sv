
`include "template.sv"

module sce_tb_aes();

    parameter COREUSERCNT = 8;
    parameter type coreuser_t = bit[0:COREUSERCNT-1];
    parameter INTC = 8;
    parameter ERRC = 8;
    localparam AW = 16;
    localparam IDW = 8;
    localparam DW = 32;
    localparam UW = 4;

    `include "sce_tb_v0.1.svh"

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
     bit [15:0]  truststate;

    // logic []


  //
  //  dut
  //  ==

    sce
    dut
    (
        .clk    (clk),
        .clktop (clk),
        .clksceen('1),
      //  .clkpke(clk),
        .clkpkeen('1),
        .resetn (resetn),
        .*
    );


    bit clkaes, aes_start, aes_done;
    bit [127:0] aeskey, aesdin, aesdout;

    aes_cipher_top aes(
        .clk      (clkaes         ),
        .rstn     (resetn      ),
        .ld       ( aes_start      ),
        .done     ( aes_done   ),
        .key      ( aeskey    ),
        .text_in  ( aesdin ),
        .text_out ( aesdout )
    );

    wire2ahbm ahbdrv(.ahbm(ahbs),.*);

  //
  //  monitor and clk
  //  ==

    `genclk( clk, 20 )
    `genclk( clkaes, 2 )
    `timemarker2
    assign hclk = clk;

  //
  //  subtitle
  //  ==


    logic [1:0] cr_reseed_intval = 'h1;
    logic [1:0] cr_gen_intval = 'h2;
    logic [5:0] cr_healthtest_len = 'h20;
    logic [1:0] cr_postproc_opt = 0;
    logic    cr_drng_en = 0;
    logic    cr_hlthtest_en = 0;
    logic    cr_pfilter_en = 0;
    logic    cr_gen_en = 1;
    logic [31:0] cr_postproc;
    assign cr_postproc = 32'h0 | { cr_reseed_intval[1:0],
             cr_gen_intval[1:0],
             cr_healthtest_len[5:0],
             cr_postproc_opt[1:0],
             cr_drng_en,
             cr_hlthtest_en,
             cr_pfilter_en, cr_gen_en };

    initial begin
      #(100 `MS);
    `maintestend

    `maintest(sce_tb_aes,sce_tb_aes)
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );


        $readmemh("../tb/hashram.dua", membuf);

        memwr('0, 512);
        memrd('0, 512);

        sfrwr(GLB_suben, 32'hb );

        sfrwr(HASH_crfunc, 32'hff); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
         sfrwr(GLB_suben, 32'hff );

// aes


        #( 10 `US );
        $display("\n@I:: AES Verification ECB mode\n");

        membuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
        membuf[0:3]= {32'h69c4e0d8,32'h6a7b0430,32'hd8cdb780,32'h70b4c55a};
        memwr(SEGADDR_AIB,4);
        membuf[0:11] = {32'h00010203,'h04050607,'h08090a0b,'h0c0d0e0f,'h296bd6eb,'h2ca90321,'hbbef5f5f,'h4cfc10ec,'h11111111,'h22222222,'h33333333,'h44444444};
        memwr(SEGADDR_AKEY,4);
        //sfrwr(AES_segptr_IV,8);

        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);

        #( 1 `US );
        sfrwr(AES_opt1,32'h0);
        sfrwr(AES_crfunc,32'h2);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
        memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);

        refbuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
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
        refbuf[0:3] = {32'h69c4e0d8,32'h6a7b0430,32'hd8cdb780,32'h70b4c55a};
        checkref(4);


        #( 10 `US );
        $display("\n@I:: AES Verification CBC mode\n");
        membuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
        membuf[0:3]= {32'h684947d5,32'h6fd7ebf7,32'h68cbb35e,32'hbecea695};
        memwr(SEGADDR_AIB,4);
        membuf[0:11] = {32'h00010203,'h04050607,'h08090a0b,'h0c0d0e0f,'h296bd6eb,'h2ca90321,'hbbef5f5f,'h4cfc10ec,'h11111111,'h22222222,'h33333333,'h44444444};
        memwr(SEGADDR_AKEY,12);
        sfrwr(AES_segptr_IV,8);


        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);

        #( 1 `US );
        sfrwr(AES_opt1,32'h0);
        sfrwr(AES_crfunc,32'h2);
        sfrwr(AES_opt,32'h110);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
       // memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);



        refbuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
        checkref(4);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0F0E);
        sfrwr(SDMA_ichcr_transize,32'h4);
        sfrwr(SDMA_chstart_ar,32'h5a);

        #( 10 `US );
      sfrwr(AES_crfunc,32'h1);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
       // memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);
        refbuf[0:3] = {32'h684947d5,32'h6fd7ebf7,32'h68cbb35e,32'hbecea695};
        checkref(4);



         #( 10 `US );
        $display("\n@I:: AES Verification CFB mode\n");
        membuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
        membuf[0:3]= {32'h6b21b2a9,32'h13a7513a,32'h48061454,32'h3d1254ea};
        memwr(SEGADDR_AIB,4);
        membuf[0:11] = {32'h00010203,'h04050607,'h08090a0b,'h0c0d0e0f,'h296bd6eb,'h2ca90321,'hbbef5f5f,'h4cfc10ec,'h11111111,'h22222222,'h33333333,'h44444444};
        memwr(SEGADDR_AKEY,12);
        sfrwr(AES_segptr_IV,8);


        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);

        #( 1 `US );
        sfrwr(AES_opt1,32'h0);
        sfrwr(AES_crfunc,32'h2);
        sfrwr(AES_opt,32'h130);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
        memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);

        refbuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
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
        refbuf[0:3]= {32'h6b21b2a9,32'h13a7513a,32'h48061454,32'h3d1254ea};
        checkref(4);

         #( 10 `US );
        $display("\n@I:: AES Verification OFB mode\n");
        membuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};

        membuf[0:3]= {32'h6bef65c0 ,32'hb7028999 ,32'hf9d95cfa,32'ha5d62c57};
        memwr(SEGADDR_AIB,4);
        membuf[0:11] = {32'h00010203,'h04050607,'h08090a0b,'h0c0d0e0f,'h296bd6eb,'h2ca90321,'hbbef5f5f,'h4cfc10ec,'h11111111,'h22222222,'h33333333,'h44444444};
        memwr(SEGADDR_AKEY,12);
        sfrwr(AES_segptr_IV,8);


        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);

        #( 1 `US );
        sfrwr(AES_opt1,32'h0);
        sfrwr(AES_crfunc,32'h2);
        sfrwr(AES_opt,32'h140);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
        memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);

        refbuf[0:3] = {32'h00112233,32'h44556677,32'h8899aabb,32'hccddeeff};
        checkref(4);

        #( 1 `US );
        sfrwr(SDMA_ichcr_segid,32'h0F0E);
        sfrwr(SDMA_ichcr_transize,32'h4);
        sfrwr(SDMA_chstart_ar,32'h5a);

        #( 10 `US );

        sfrwr(AES_crfunc,32'h1);
        sfrwr(AES_ar,32'h5a);sfrwait(AES_fr, 32'h1, 32'h1); sfrwr(AES_fr, 32'h1);
       // memrd(SEGADDR_AIB,4);
        memrd(SEGADDR_AOB,4);
        refbuf[0:3]= {32'h6bef65c0 ,32'hb7028999 ,32'hf9d95cfa,32'ha5d62c57};
        checkref(4);

    `maintestend

// ■■■■■■■■■■■■■■■


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


