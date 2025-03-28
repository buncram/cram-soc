 `include "template.sv"

module sce_tb_hash();

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

    bit [15:0] truststate;

    reg [4095:0] bigNumber ;

  //
  //  dut
  //  ==

    sce 
    dut
    (
        .clk    (clk),
        .clktop (clk),
        .clksceen('1),
        //.clkpke(clk),
        .clkpkeen('1),
        .resetn (resetn),
		.*
    );
    
    assign probe_pkeram_18d = (sce_tb_hash.dut.pke.ramadd0[8:0] == 'h18d);

  assign probe_pkeram_14a = (sce_tb_hash.dut.pke.ramadd0[8:0] == 'h14a);
    //assign probe_pkeram_14c = (sce_tb_hash.dut.pke.ramadd0[8:0] == 'h18c) && ;

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
      #(2000 `MS);
    `maintestend

    `maintest(sce_tb_hash,sce_tb_hash)
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );

        #( 500 `US );

        membuf[0:3] = { 32'h0123_4567, 32'h89ab_cdef, 32'h76543210, 32'hfedc_ba98 };

        memwr('0, 4);
        memrd('0, 4);

// init hash

        $readmemh("../tb/hashram.dua", membuf);
        memwr('0, 512);
        memrd('0, 512);


        sfrwr(GLB_suben, 32'hb );

        sfrwr(HASH_crfunc, 32'hff); sfrrd(HASH_crfunc); sfrwr(HASH_ar, 32'h5a); sfrwait(HASH_fr, 32'h1, 32'h1); sfrwr(HASH_fr, 32'h1);

 
$display("@I:: X25519");
     sfrwr( GLB_suben, '1);
    sfrwr( PKE_optnw, 32'd255);
      sfrwr( PKE_optew, 32'd255);
        // P:
            membuf[0:7] = {32'hffffffed,32'hffffffff,32'hffffffff,32'hffffffff,32'hffffffff,32'hffffffff,32'hffffffff,32'h7fffffff};
        // A
            //membuf[8:15] = {32'h135978a3,32'h75eb4dca,32'h4141d8ab,32'h00700a4d,32'h7779e898,32'h8cc74079,32'h2b6ffe73,32'h52036cee};
               membuf[8:15] = {32'h1db41,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
            memwr(SEGADDR_PCON,16);
        //H

            membuf[0:8] = {32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h80000000,32'h0};
            memwr(SEGADDR_PCON+16,9);


        sfrwr( PKE_crfunc, 32'h01 );
        sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);

        // Q0X
             membuf[0:7] = {32'h9,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0,32'h0};
            memwr(SEGADDR_PIB,8);
                    //K
            membuf[0:7] = {32'h88322950,32'he45bfb8d, 32'hca14370c ,32'h0a27da9f ,32'h78898045, 32'h08f583d7, 32'hd93c79a1, 32'h687fb314};
            memwr(SEGADDR_PKB, 8);
                 $display("\n@I:: First trans\n");
            sfrwr( PKE_crfunc, 32'h24 ); // I2MD
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            memrd(SEGADDR_POB,8*4);


        #( 1 `US );
            sfrwr(SDMA_ichcr_segid,32'h0b09);
            sfrwr(SDMA_ichcr_transize,32'd32);
            sfrwr(SDMA_ichcr_rpstart, 32'd0);
            sfrwr(SDMA_ichcr_wpstart, 32'd128);
            sfrwr(SDMA_chstart_ar,32'h5a);
                  #( 10 `US );
            memrd(SEGADDR_PIB+128,8);
            sfrwr( PKE_segptr_PIB0, 'd128);

            $display("\n@I:: X25519\n");
            sfrwr( PKE_crfunc, 32'h3b ); // X25519
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            memrd(SEGADDR_POB,8*4);
            
            
                   #( 1 `US );
            sfrwr(SDMA_ichcr_segid,32'h0b09);
            sfrwr(SDMA_ichcr_transize,32'd32);
            sfrwr(SDMA_ichcr_wpstart, 32'd0);
                  sfrwr(SDMA_ichcr_wpstart, 32'd128+8);
            sfrwr(SDMA_chstart_ar,32'h5a);
            #( 10 `US );

                   sfrwr( PKE_segptr_PIB0, 'd128+8);
            $display("\n@I:: M2I Result point\n");
            sfrwr( PKE_crfunc, 32'h27 );
            sfrwr( PKE_ar, 32'h5a ); sfrwait(PKE_fr, 32'h1, 32'h1); sfrwr(PKE_fr, 32'h1);
            memrd(SEGADDR_POB,8*4);
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
   // input reg [256:0] bigNumber = 256'h2a582adbbfb6d60c85997d50c3f2f964fcbe82be4fdce244e89eb9025317f30a;

 input reg [4095:0] bigNumber ;
    for(i = 0; i< 128; i = i+1) begin
        membuf[i] = bigNumber[i*32 +:32];
    end

    for(i = 0; i < 128; i=i+8) begin
      $display("membuf[%0d:%0d] =  {32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h,32'h%0h};", i, i+7,  membuf[i],membuf[i+1], membuf[i+2], membuf[i+3], membuf[i+4], membuf[i+5], membuf[i+6], membuf[i+7]);
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

