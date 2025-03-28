`include "template.sv"

module sce_tb_rsa2();

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

    `maintest( sce_tb_rsa2, sce_tb_rsa2)
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );

        #( 500 `US );
   `ifdef NOFSDB
while(1) begin
`endif
        membuf[0:3] = { 32'h0123_4567, 32'h89ab_cdef, 32'h76543210, 32'hfedc_ba98 };

        memwr('0, 4);
        memrd('0, 4);

    $display("\n@I:: RSAMM 119 normal");
            sfrwr( GLB_suben,'1 );
            sfrwr( PKE_optnw, 32'd119);

        //N the modulo for RSA, product of two large prime
            membuf[0:3] = {32'h00000001,32'h0, 32'h0eef9527,32'h59feeb};
            memwr(SEGADDR_PCON,4);

        //H, to fill the remaining position, H = 1 << NLen
            membuf[0:3] = {32'h00000000,32'h00000000,32'h0,32'h00800000};
            memwr(SEGADDR_PCON+4,4);
            sfrwr( PKE_crfunc, 32'h11); //RSAINIT
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        // X
            membuf[0:3] ={32'h9527beef,32'h9527beef, 32'h9527beef, 32'h1beeb};
        // Y
            membuf[4:7] ={32'hbeca7319, 32'hbeca7319,32'hbeca7319, 32'h487209};
            memwr(SEGADDR_PIB, 8);
            sfrwr( PKE_segptr_PIB1, 'd4);

            sfrwr( PKE_crfunc, 32'h12); // RSAMM
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading 119 bit RSAMM result\n");
            memrd(SEGADDR_POB, 4);


 $display("\n@I:: RSAMM 119 bypass");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd119);

        //N the modulo for RSA, product of two large prime
        membuf[0:3] = {32'h00000001,32'h0, 32'h0eef9527,32'h59feeb};
        memwr(SEGADDR_PCON,4);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:3] = {32'h00000000,32'h00000000,32'h0,32'h00800000};
            memwr(SEGADDR_PCON+4,4);
            sfrwr(PKE_optmask, 32'h1);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);


        memrd('h1100,4);
        memrd('h1500,8);

            sfrwr(GLB_ar, 32'ha5);
            #( 1000 `US );
        //N the modulo for RSA, product of two large prime
            membuf[0:3] = {32'h00000001,32'h0, 32'h0eef9527,32'h59feeb};
            memwr('h1100,4);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
            membuf[0:3] = {32'h00000000,32'h00000000,32'h0,32'h00800000};
            memwr('h1500,4);
            sfrwr( PKE_crfunc, 32'h11ff); //RSAINIT
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

    // X
         membuf[0:3] ={32'h9527beef,32'h9527beef, 32'h9527beef, 32'h1beeb};
           memwr('h1000,4);
    // Y
            membuf[0:3] ={32'hbeca7319, 32'hbeca7319,32'hbeca7319, 32'h487209};
            memwr('h1400,4);
            sfrwr( PKE_crfunc, 32'h12ff ); // RSAMM
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            $write("\n Reading 119 bit RSAMM bypass mode result\n");
            memrd('h1000, 4);



//RSAMM 92
    /*
   $display("\n@I:: RSAMM 92");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd92);
        //sfrwr( PKE_optew, 32'd96);

        //N the modulo for RSA, product of two large prime
        membuf[0:2] = {32'h1,32'h0, 32'h0eef9527};memwr(SEGADDR_PCON,3);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:2] = {32'h00000000,32'h00000000,32'h10000000};memwr(SEGADDR_PCON+3,3);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        //X
            membuf[0:2] ={32'h9527beef,32'h9527beef, 32'h0001beeb};
            // membuf[0:2] ={32'hb,32'h0, 32'h0};
        // Y
        membuf[3:5] ={32'hbeca7319,32'hbeca7319, 32'h00487209};memwr(SEGADDR_PIB, 6);
         //membuf[3:5] ={32'h7,32'h0, 32'h0};
          sfrwr( PKE_segptr_PIB1, 'h3);
         memwr(SEGADDR_PIB, 6);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM+_
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading RSAMM 92 bit clock count\n");
        sfrrd(GLB_tickcnt);
        $write("\n Reading RSAMM 92 bit result\n");
        memrd(SEGADDR_POB, 4);//refbuf[0:1] = {32'h41f4cca4,32'h7479d8cf}; checkref(2);
          

  $display("\n@I:: RSAMM 92 zero padding");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd92);
        //sfrwr( PKE_optew, 32'd96);

        //N the modulo for RSA, product of two large prime
        membuf[0:3] = {32'h1,32'h0, 32'h0eef9527, 32'h0};memwr(SEGADDR_PCON,4);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:3] = {32'h00000000,32'h00000000,32'h10000000,32'h0};memwr(SEGADDR_PCON+4,4);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        //X
           // membuf[0:3] ={32'h9527beef,32'h9527beef, 32'h0001beeb, 32'h0};
             membuf[0:3] ={32'hb,32'h0, 32'h0};
        // Y
       // membuf[4:7] ={32'hbeca7319,32'hbeca7319, 32'h00487209,32'h0};memwr(SEGADDR_PIB, 8);
         membuf[4:7] ={32'h7,32'h0, 32'h0};
          sfrwr( PKE_segptr_PIB1, 'd4);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading RSAMM 92 bit clock count\n");
        sfrrd(GLB_tickcnt);
        $write("\n Reading RSAMM 92 bit result\n");
        memrd(SEGADDR_POB, 4);//refbuf[0:1] = {32'h41f4cca4,32'h7479d8cf}; checkref(2);


    
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
    // X
        membuf[0:1] ={32'hb,32'h0};
    // Y
        membuf[2:3] ={32'h7,32'h0};
        memwr(SEGADDR_PIB, 4);
        sfrwr( PKE_segptr_PIB1, 'h2);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading 64 bit RSAMM result\n");
        memrd(SEGADDR_POB, 2);
        */
    /*
          $display("\n@I:: RSAME 65");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd65);
        sfrwr( PKE_optew, 32'd65);

        //N the modulo for RSA, product of two large prime
        membuf[0:2] = {32'h000003D9,32'h8fffffbe,32'h1};
        memwr(SEGADDR_PCON,3);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:3] = {32'h00000000,32'h00000000,32'h2,32'h0};
            memwr(SEGADDR_PCON+3,4);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
       //X
           // membuf[0:2] ={32'h9527beef,32'h9527beef, 32'h0001beeb};
             membuf[0:2] ={32'hb,32'h0, 32'h0};
        // Y
        //membuf[3:5] ={32'hbeca7319,32'hbeca7319, 32'h00487209};memwr(SEGADDR_PIB, 6);
         membuf[3:5] ={32'h7,32'h0, 32'h0};
         memwr(SEGADDR_PIB, 6);
        sfrwr( PKE_segptr_PIB1, 'd3);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading 65 bit RSAMM result\n");
        memrd(SEGADDR_POB, 2);


         $display("\n@I:: RSAMM 128");
        sfrwr( GLB_suben,'1 );
        sfrwr( PKE_optnw, 32'd128);
        sfrwr( PKE_optew, 32'd128);

        //N the modulo for RSA, product of two large prime
        membuf[0:3] = {32'h000003D9,32'h0,32'h0,32'h8fffffbe};
        memwr(SEGADDR_PCON,4);

        //H, to fill the remaining position, need to summarize the pattern, H = 1 << NLen
        membuf[0:4] = {32'h00000000,32'h00000000,32'h0,32'h0,32'h1};
            memwr(SEGADDR_PCON+4,5);
        sfrwr( PKE_crfunc, 32'h11); //RSAINIT
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
       //X
             membuf[0:3] ={32'hb,32'h0, 32'h0,32'h0};
        // Y
         membuf[4:7] ={32'h7,32'h0, 32'h0,32'h0};
         memwr(SEGADDR_PIB, 8);
        sfrwr( PKE_segptr_PIB1, 'd4);
        sfrwr( PKE_crfunc, 32'h12 ); // RSAMM
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
        $write("\n Reading 128 bit RSAMM result\n");
        memrd(SEGADDR_POB, 4);

    */
 

`ifdef NOFSDB
end
`endif

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
