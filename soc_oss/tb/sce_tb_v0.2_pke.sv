
`include "template.sv"

module sce_tb_pke();

    parameter COREUSERCNT = 8;
    parameter type coreuser_t = bit[0:COREUSERCNT-1];
    parameter INTC = 8;
    parameter ERRC = 8;
    localparam AW = 16;
    localparam IDW = 8;
    localparam DW = 32;
    localparam UW = 4;
    integer i;

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
    logic clkpke, ana_rng_0p1u, truststate;


  //
  //  dut
  //  ==

    sce
    dut
    (
        .clk    (clk),
        .clktop (clk),
        .clksceen('1),
        .clkpke(clk),
        .clkpkeen('1),
        .resetn (resetn),
		.*
    );

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

    `maintest(sce_tb_pke,sce_tb_pke)
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 20 `US );

        #( 500 `US );

// init hash

//        $readmemh("hashram.dua", membuf);
//        memwr('0, 512);
//        memrd('0, 512);


        sfrwr(GLB_suben, 32'hff );

// rsa multi
    #( 10 `US );
    $display("\n@I:: MODINV");

        i = 0;
        membuf[i] = 'h6c794c90; i++;
        membuf[i] = 'h2670b146; i++;
        membuf[i] = 'hbe6c2e36; i++;
        membuf[i] = 'h38514b3b; i++;
        membuf[i] = 'ha0e6e13b; i++;
        membuf[i] = 'h6bc3596c; i++;
        membuf[i] = 'h0a7b25e2; i++;
        membuf[i] = 'hd10c825c; i++;
        membuf[i] = 'h0a866e4f; i++;
        membuf[i] = 'h53f9e631; i++;
        membuf[i] = 'h3fa36408; i++;
        membuf[i] = 'h2b589735; i++;
        membuf[i] = 'h5e33627c; i++;
        membuf[i] = 'h677e1957; i++;
        membuf[i] = 'hb2ddf97b; i++;
        membuf[i] = 'h6ac0e7a9; i++;
        membuf[i] = 'h950a6cb8; i++;
        membuf[i] = 'h6dfcefe1; i++;
        membuf[i] = 'hfb66344c; i++;
        membuf[i] = 'hf80f13db; i++;
        membuf[i] = 'he6647b34; i++;
        membuf[i] = 'h6c5ea813; i++;
        membuf[i] = 'hf1b8a3d0; i++;
        membuf[i] = 'h7a50f92e; i++;
        membuf[i] = 'hc7dcbe65; i++;
        membuf[i] = 'habd0c470; i++;
        membuf[i] = 'hab8e1699; i++;
        membuf[i] = 'hd8ceb945; i++;
        membuf[i] = 'hf04ea5fb; i++;
        membuf[i] = 'hb1bdffba; i++;
        membuf[i] = 'h7fe39b7b; i++;
        membuf[i] = 'h3913b103; i++;
        membuf[i] = 'h57d5ddec; i++;
        membuf[i] = 'h46ce5bf5; i++;
        membuf[i] = 'h06f28288; i++;
        membuf[i] = 'h03ec808b; i++;
        membuf[i] = 'h33cb566a; i++;
        membuf[i] = 'h70f01663; i++;
        membuf[i] = 'h16365a13; i++;
        membuf[i] = 'hdf620640; i++;
        membuf[i] = 'hfccd40e7; i++;
        membuf[i] = 'h764eae87; i++;
        membuf[i] = 'hc66bf1af; i++;
        membuf[i] = 'hb1c261c5; i++;
        membuf[i] = 'h68b30917; i++;
        membuf[i] = 'hdf52e326; i++;
        membuf[i] = 'h6d8b572f; i++;
        membuf[i] = 'ha395f0fd; i++;
        membuf[i] = 'h6f9b5fed; i++;
        membuf[i] = 'h73a7b440; i++;
        membuf[i] = 'ha1d0daa7; i++;
        membuf[i] = 'h1aa7520b; i++;
        membuf[i] = 'h47de339b; i++;
        membuf[i] = 'h3838601a; i++;
        membuf[i] = 'h3bb70a69; i++;
        membuf[i] = 'hbe036752; i++;
        membuf[i] = 'h6817b395; i++;
        membuf[i] = 'h75316637; i++;
        membuf[i] = 'h50fab508; i++;
        membuf[i] = 'h76f44e75; i++;
        membuf[i] = 'h88234e8e; i++;
        membuf[i] = 'hb1fd8459; i++;
        membuf[i] = 'h3995ec58; i++;
        membuf[i] = 'h69046457;

        memwr( SEGADDR_PCON,    64 );
        memwr( SEGADDR_PCON+64, 64 );

        membuf[0] = 'h01000000;
        for (i = 0; i < 128; i++) membuf[i+1] = '0;
        memwr( SEGADDR_PCON+128, 129 );
        memrd( SEGADDR_PCON+128, 129 );

        sfrwr( PKE_optnw, 'd4096 );
        sfrwr( PKE_optew, 'd4096 );
        sfrwr( PKE_optltx, 'hff );
        sfrwr( PKE_crfunc, 32'h11 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        membuf[0] = 'h11000000;
        for (i = 0; i < 127; i++) membuf[i+1] = '0;


//        memrd( SEGADDR_POB, 0 );

        #( 500 `US );
$finish;

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


