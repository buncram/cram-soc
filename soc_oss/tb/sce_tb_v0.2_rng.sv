
`include "template.sv"

module sce_tb_rng();

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

    logic clkpke, ana_rng_0p1u;
    logic [15:0] truststate;


  //
  //  dut
  //  ==

    sce
    dut
    (
        .clk    (clk),
        .clktop (clk),
        .clksceen('1),
        .clkpkeen('1),
        .resetn (resetn),
        .*
    );

    wire2ahbm ahbdrv(.ahbm(ahbs),.*);

  //
  //  monitor and clk
  //  ==

    `genclk( clk, 20 )
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

    `maintest(sce_tb_rng,sce_tb_rng)
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );

        #( 500 `US );

        sfrwr(GLB_suben, 32'hff );
        sfrwr(GLB_ffen, 32'h30 );

    // rng
        ahbwr( RNG_sfr_crsrc, 'hffff );
        ahbwr( RNG_sfr_crana, 'hffff );
        ahbwr( RNG_sfr_opt, 'h10020 );
        ahbwr( RNG_sfr_chain0, 'hffffffff );
        ahbwr( RNG_sfr_chain1, 'hffffffff );

        cr_reseed_intval[1:0] = 'h1;
        cr_gen_intval[1:0] = 'h2;
        cr_healthtest_len[5:0] = 'h20;
        cr_postproc_opt[1:0] = 0;
        cr_drng_en = 0;
        cr_hlthtest_en = 0;
        cr_pfilter_en = 0;
        cr_gen_en = 1;


$display("~~~~~~~~~1.GET Raw Data~~~~~~~~~");
$display("analog data go through digilization and save in buf");
$display("read buf_dataout out when 256 bits buf is full");
        cr_pfilter_en = 0;
        ahbwr( RNG_sfr_pp, cr_postproc );
        sfrwait(RNG_sfr_sr, 32'h1000000, 32'h1000000);
        for (int i = 0; i < 8; i++) sfrrd( RNG_sfr_buf );

        #( 10 `US );
        cr_pfilter_en = 1;
        ahbwr( RNG_sfr_pp, cr_postproc );
        sfrwait(RNG_sfr_sr, 32'h1000000, 32'h1000000);
        for (int i = 0; i < 8; i++) sfrrd( RNG_sfr_buf );

        #( 10 `US );

$display("~~~~~~~~~2.TRNG+LFSR~~~~~~~~~");
$display("analog data go through digilization then output to LFSR process");
$display("read rngcore_dataout when process is done");

        cr_drng_en=1'b0;
        cr_postproc_opt=2'd0;
       ahbwr( RNG_sfr_pp, cr_postproc );
        ahbwr( RNG_sfr_opt, 'h10010 );
        sfrwr( RNG_sfr_ar, 32'h5a );
        sfrwait(RNG_sfr_fr, 32'h1, 32'h1); sfrwr( RNG_sfr_fr, 32'h1);
        sfrwr( RNG_sfr_ar, 32'ha5);
        memrd( SEGADDR_RNGB, 16*4 );

$display("~~~~~~~~~3.TRNG+AUTO~~~~~~~~~");
$display("analog data go through digilization then output to AES-128 auto process");
$display("read rngcore_dataout when process is done");
        cr_drng_en=1'b0;
        cr_postproc_opt=2'd2;
       ahbwr( RNG_sfr_pp, cr_postproc );
        ahbwr( RNG_sfr_opt, 'h10010 );
        sfrwr( RNG_sfr_ar, 32'h5a );
        sfrwait(RNG_sfr_fr, 32'h1, 32'h1);  sfrwr( RNG_sfr_fr, 32'h1);
        sfrwr( RNG_sfr_ar, 32'ha5);
        memrd( SEGADDR_RNGB, 16*4 );

$display("~~~~~~~~~4.DRNG+confiugre~~~~~~~~~");
$display("In DRNG mode, 256 bits seed configure into buf");
$display("prepare 32*8 bit random bits");
    $display("----DRNG+Write buf----");
        cr_drng_en=1'b1;
        cr_postproc_opt=2'd0;
       ahbwr( RNG_sfr_pp, cr_postproc );
        sfrwr( RNG_sfr_ar, 32'h5a );
    $display("----DRNG+Read buf----");
        for (int i = 0; i < 8; i++) sfrwr( RNG_sfr_buf, $random() );
        #(10 `US);
$display("In DRNG mode,256 bits seed read out from buf");
$display("----DRNG+Read buf----");
        for (int i = 0; i < 8; i++) sfrrd( RNG_sfr_buf);
        sfrwr( RNG_sfr_ar, 32'ha5 );
        #(10 `US);

$display("~~~~~~~~~5.DRNG+LFSR~~~~~~~~~");
$display("In DRNG mode, 256 bits seed configure into buf then output to LFSR process");
$display("read rngcore_dataout when process is done");
    cr_drng_en=1'b1;
    cr_postproc_opt=2'd0;
    cr_reseed_intval=2'd2;
        #(1 `US);
        ahbwr( RNG_sfr_pp, cr_postproc );
        ahbwr( RNG_sfr_opt, 'h10010 );
        #(1 `US);
        sfrwr( RNG_sfr_ar, 32'h5a );
$display("----DRNG write seed----");
        for (int i = 0; i < 8; i++) sfrwr( RNG_sfr_buf, $random() );
        #(10 `US);
        sfrwait(RNG_sfr_fr, 32'h1, 32'h1);  sfrwr( RNG_sfr_fr, 32'h1);
        sfrwr( RNG_sfr_ar, 32'ha5);
$display("----BUF ready----");
        memrd( SEGADDR_RNGB, 16*4 );

$display("~~~~~~~~~6.TRNG+MANUAL~~~~~~~~~");
$display("In DRNG mode, 256 bits seed configure into buf then output to AES128 manual process");
$display("AES128 manual process include AES-128 encrytion calculation twice");
$display("read rngcore_dataout when process is done");
    cr_postproc_opt=2'd1;

for (int x = 0; x < 16; x++) begin
        #(1 `US);
        ahbwr( RNG_sfr_pp, cr_postproc );
        #(1 `US);
    for (int i = 0; i < 8; i++) sfrwr( RNG_sfr_buf, $random() );
        #(1 `US);
        ahbwr( RNG_sfr_opt, 'h2 );
        sfrwr( RNG_sfr_ar, 32'h5a );
        sfrwait(RNG_sfr_fr, 32'h1, 32'h1);  sfrwr( RNG_sfr_fr, 32'h1);
        sfrwr( RNG_sfr_ar, 32'ha5);
        memrd( SEGADDR_RNGB, 2*4 );
end

$display("~~~~~~~~~7.HealthTest~~~~~~~~~");
$display("Healthtest check data output from digilization ");
$display("Once continuously 1 or 0 bits over healthtest_length, healthtest_err is set");
    cr_pfilter_en=1'h0;
    cr_hlthtest_en=1'h1;
        #(1 `US);
        ahbwr( RNG_sfr_pp, cr_postproc );
        ahbwr( RNG_sfr_crsrc, 'hfffd );
        for (int i = 0; i < 20; i++) begin #( 10 `US ) sfrrd( RNG_sfr_sr ); end
        ahbwr( RNG_sfr_crsrc, '1 );

        #( 100 `US );
        ahbwr( RNG_sfr_pp, cr_postproc );
        ahbwr( RNG_sfr_crsrc, 'hfffd );
        for (int i = 0; i < 20; i++) begin #( 10 `US ) sfrrd( RNG_sfr_sr ); end
        ahbwr( RNG_sfr_crsrc, '1 );




        #( 100 `US ); $finish;

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
        //    #(10 `US); $write( "hrdatareg:%08x; tmask:%08x; texp:%08x\n", hrdatareg, tmask, texpdata);
            if( hrdatareg && tmask == texpdata ) break;
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


