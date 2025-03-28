
`include "template.sv"

module sce_tb_hmac();

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



  //
  //  dut
  //  ==
    logic [4:0] pkefd = 0, PM_PFD = 5'h1;
    logic [4:0] scefd = 0, PM_SFD = 5'h6;
    logic clkpkeen, clksceen, clkpke, clksce;
    logic clktop = 0;
    logic por = 0;

    sce 
    dut
    (
        .clk    (clk),
        .clktop (clktop),
        .clksceen(clksceen),
        .clkpke(clkpke),
        .clkpkeen(clkpkeen),
        .resetn (resetn),
		.*
    );
    
    wire2ahbm ahbdrv(.ahbm(ahbs),.*);

  //
  //  monitor and clk
  //  ==

    `genclk( clktop, 10 )
    `timemarker2
    assign hclk = clk;

    `theregfull( clktop, por, pkefd, 0 ) <= ( pkefd == PM_PFD ) ? '0 : pkefd + 1;
    `theregfull( clktop, por, scefd, 0 ) <= ( scefd == PM_SFD ) ? '0 : scefd + 1;

    assign clkpkeen = ( pkefd == PM_PFD );
    assign clksceen = ( scefd == PM_SFD );

    ICG icgclkpke(.CK (clktop),.SE('0),.EN (clkpkeen),.CKG(clkpke));
    ICG icgclksce(.CK (clktop),.SE('0),.EN (clksceen),.CKG(clksce));

    assign clk = clksce;

  //
  //  subtitle
  //  ==

    initial begin
      #(10000 `MS);
    `maintestend

    `maintest(sce_tb_hmac,sce_tb_hmac)
        resetn = 0;
        por = 0;
        #( 200 );
        por = 1;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );

        #( 500 `US );


// init hash

        $readmemh("hashram.dua", membuf);
        memwr('0, 512);
        memrd('0, 512);


        sfrwr(GLB_suben, 32'hb );

        sfrwr(HASH_crfunc, 32'hff); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);

// hmac256
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

        refbuf[0:7] = 'h92ab4d9a1f3b6152bca9dd9e69af43f4ce99e42fd4e30ff972c48025b9f9cfef;

        sfrwr(HASH_crfunc, 32'h50); sfrrd(HASH_crfunc); 
        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 0);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        checkref(8);

    // pass2
        $display("\n@I:: hmac256 pass2");

        refbuf[0:7] = 'hb0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7;

        sfrwr(HASH_crfunc, 32'h60); sfrrd(HASH_crfunc); 
//        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        checkref(8);







        membuf[ 0  ] = 32'hf2bdc948;
        membuf[ 1  ] = 32'h6a09e667;
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
        membuf[ 16+0  ] = 32'h48692054;
        membuf[ 16+1  ] = 32'h68657265;
        membuf[ 16+2  ] = 32'h80000000;
        membuf[ 16+3  ] = 32'h00000000;
        membuf[ 16+4  ] = 32'h00000000;
        membuf[ 16+5  ] = 32'h00000000;
        membuf[ 16+6  ] = 32'h00000000;
        membuf[ 16+7  ] = 32'h00000000;
        membuf[ 16+8  ] = 32'h00000000;
        membuf[ 16+9  ] = 32'h00000000;
        membuf[ 16+10 ] = 32'h00000000;
        membuf[ 16+11 ] = 32'h00000000;
        membuf[ 16+12 ] = 32'h00000000;
        membuf[ 16+13 ] = 32'h00000000;
        membuf[ 16+14 ] = 32'h00000000;
        membuf[ 16+15 ] = 32'h00000240;
        memwr(SEGADDR_MSG,32);

    // pass1
        $display("\n@I:: hmac256 pass1");

        sfrwr(HASH_crfunc, 32'h50); sfrrd(HASH_crfunc); 
        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 0);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);
        //checkref(8);

    // pass2
        $display("\n@I:: hmac256 pass2");

        sfrwr(HASH_crfunc, 32'h60); sfrrd(HASH_crfunc); 
//        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);



    // pass1
        $display("\n@I:: hmac256 pass1.a");

        sfrwr(HASH_crfunc, 32'h50); sfrrd(HASH_crfunc); 
        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_opt3, 0);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);

    // pass1
        $display("\n@I:: hmac256 pass1.b");

        sfrwr(HASH_segptr_MSG, 'h10);
        sfrwr(HASH_crfunc, 32'h50); sfrrd(HASH_crfunc); 
        sfrwr(HASH_opt1, 0);
        sfrwr(HASH_opt2, 0);
        sfrwr(HASH_opt3, 0);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);



    // pass2
        $display("\n@I:: hmac256 pass2");

        sfrwr(HASH_segptr_MSG, 'h0);
        sfrwr(HASH_crfunc, 32'h60); sfrrd(HASH_crfunc); 
//        sfrwr(HASH_opt1, 1);
        sfrwr(HASH_opt2, 4);
        sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);
        memrd(SEGADDR_HOUT,8);

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


