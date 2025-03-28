module hashcore_tb();

    bit clk,resetn;
    integer i=0, j, k, errcnt, warncnt;

    parameter AW = 10;
    bit             start;
    bit             done;
    hashtype_t      cfg_hashtype = 0;
    bit             cfg_firsthash;
    bit             cfg_finalhash;
    bit  [AW-1:0]    ramaddr;
    bit  [31:0]      ramrdat;
    bit              ramwr;
    bit  [31:0]      ramwdat;
    bit             busy;
  //
  //  dut
  //  ==
    hashcore
    dut
    (
        .clk    (clk),
        .resetn (resetn),
        .start  (start),
        .busy   (busy),
        .done   (done),
        .cfg_hashtype (cfg_hashtype),
        .cfg_firsthash(cfg_firsthash),
        .cfg_finalhash(cfg_finalhash),

        .ramaddr(ramaddr),
        .ramrdat32(ramrdat),
        .ramwr32(ramwr),
        .ramwdat32(ramwdat)
    );

    hashram #(.INITFILE("hashram.dua")) hashram(.*);


  //
  //  monitor and clk
  //  ==

    `genclk( clk, 100 )
    `timemarker2
    initial begin
        #(10 `MS);
    `maintestend

    logic [0:63][31:0]  sim_ww;
    logic [0:63][63:0]  sim_ww64;
    logic [0: 7][31:0]  sim_st;
    logic [0: 7][63:0]  sim_st64;

    bit     [0:7][31:0]   hashcmp;
    bit     [0:7][63:0]   hashcmp64;

    assign sim_ww[0:63]  = hashram.vramdat[ 'h200 + 32 + 0 :'h200 + 32 + 63  ];
    assign sim_ww64[0:63]  = hashram.vramdat[ 'h200 + 32 + 0 :'h200 + 32 + 63 + 64  ];
    assign sim_st[0: 7]  = hashram.vramdat[ 'h200 + 0 :'h200 +  7  ];
    assign sim_st64[0: 7]  = hashram.vramdat[ 'h200 + 0 :'h200 +  15  ];

  //
  //  subtitle
  //  ==

    `maintest(hashcore_tb,hashcore_tb)
        #105 resetn = 1;


        cfg_hashtype = 0; 

        hashram.ramdat[ 'h200 + 32 + 0  ] =  32'h61626364 ;
        hashram.ramdat[ 'h200 + 32 + 1  ] =  32'h62636465 ;
        hashram.ramdat[ 'h200 + 32 + 2  ] =  32'h63646566 ;
        hashram.ramdat[ 'h200 + 32 + 3  ] =  32'h64656667 ;
        hashram.ramdat[ 'h200 + 32 + 4  ] =  32'h65666768 ;
        hashram.ramdat[ 'h200 + 32 + 5  ] =  32'h66676869 ;
        hashram.ramdat[ 'h200 + 32 + 6  ] =  32'h6768696a ;
        hashram.ramdat[ 'h200 + 32 + 7  ] =  32'h68696a6b ;
        hashram.ramdat[ 'h200 + 32 + 8  ] =  32'h696a6b6c ;
        hashram.ramdat[ 'h200 + 32 + 9  ] =  32'h6a6b6c6d ;
        hashram.ramdat[ 'h200 + 32 + 10 ]  = 32'h6b6c6d6e;
        hashram.ramdat[ 'h200 + 32 + 11 ]  = 32'h6c6d6e6f;
        hashram.ramdat[ 'h200 + 32 + 12 ]  = 32'h6d6e6f70;
        hashram.ramdat[ 'h200 + 32 + 13 ]  = 32'h6e6f7071;
        hashram.ramdat[ 'h200 + 32 + 14 ]  = 32'h80000000;
        hashram.ramdat[ 'h200 + 32 + 15 ]  = 32'h00000000;

//        for( i = 0; i < 9; i++) begin

        i = 0;

        #(1 `US) ;
        cfg_hashtype = ( cfg_hashtype + 1 ) % 9;
        cfg_firsthash = 1; cfg_finalhash = 0; 
        #(1 `US) @( negedge clk ) start = 1; @( negedge clk ) start = 0;
        @(posedge done);
        #(1 `US);

//        $write("@i:H    = \n"); for(i=0;i<8; i++)$write(" %08x ", hh[i]); $write("\n");
       hashcmp =   { 32'h85e655d6, 32'h417a1795, 32'h3363376a, 32'h624cde5c, 32'h76e09589, 32'hcac5f811, 32'hcc4b32c1, 32'hf20e533a };

        $write("@i:W    = \n"); for(i=0;i<8 ;i++)$write(" %08x ", sim_ww[i]); $write("\n"); for(i=8;i<16 ;i++)$write(" %08x ", sim_ww[i]); $write("\n");
        $write("@i:Hout = \n"); for(i=0;i<8; i++)$write(" %08x ", sim_st[i]); $write("\n");
        $write("@i:Hcmp = \n"); for(i=0;i<8; i++)$write(" %08x ", hashcmp[i]); $write("\n");

        hashram.ramdat[ 'h200 + 32 + 2*0  :  'h200 + 32 + 2*0  + 1 ] =  { 32'h61626380 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*1  :  'h200 + 32 + 2*1  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*2  :  'h200 + 32 + 2*2  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*3  :  'h200 + 32 + 2*3  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*4  :  'h200 + 32 + 2*4  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*5  :  'h200 + 32 + 2*5  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*6  :  'h200 + 32 + 2*6  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*7  :  'h200 + 32 + 2*7  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*8  :  'h200 + 32 + 2*8  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*9  :  'h200 + 32 + 2*9  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*10 :  'h200 + 32 + 2*10 + 1 ]  = { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*11 :  'h200 + 32 + 2*11 + 1 ]  = { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*12 :  'h200 + 32 + 2*12 + 1 ]  = { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*13 :  'h200 + 32 + 2*13 + 1 ]  = { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*14 :  'h200 + 32 + 2*14 + 1 ]  = { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*15 :  'h200 + 32 + 2*15 + 1 ]  = { 32'h00000000 , 32'h00000018 } ;

        i = 0;

        #(1 `US) ;
        cfg_hashtype = ( cfg_hashtype + 1 ) % 9;
        cfg_firsthash = 1; cfg_finalhash = 0; 
        #(1 `US) @( negedge clk ) start = 1; @( negedge clk ) start = 0;
        @(posedge done);
       #(1 `US);
        hashcmp64 = 512'hDDAF35A193617ABACC417349AE20413112E6FA4E89A97EA20A9EEEE64B55D39A2192992A274FC1A836BA3C23A3FEEBBD454D4423643CE80E2A9AC94FA54CA49F;

        $write("@i:W    = \n"); for(i=0;i<8 ;i++)$write(" %016x ", sim_ww64[i]); $write("\n"); for(i=8;i<16 ;i++)$write(" %016x ", sim_ww64[i]); $write("\n");
        $write("@i:Hout = \n"); for(i=0;i<8; i++)$write(" %016x ", sim_st64[i]); $write("\n");
        $write("@i:Hcmp = \n"); for(i=0;i<8; i++)$write(" %016x ", hashcmp64[i]); $write("\n");


//  blk2s
//  ==
        #(1 `US) ;
        cfg_hashtype = HT_BLK2s ;
        cfg_firsthash = 1; cfg_finalhash = 1; 

        hashram.ramdat[ 'h200 + 0  ] =  32'h6b08e647;//32'h6a09e667 ;
        hashram.ramdat[ 'h200 + 1  ] =  32'hbb67ae85 ;
        hashram.ramdat[ 'h200 + 2  ] =  32'h3c6ef372 ;
        hashram.ramdat[ 'h200 + 3  ] =  32'ha54ff53a ;
        hashram.ramdat[ 'h200 + 4  ] =  32'h510e527f ;
        hashram.ramdat[ 'h200 + 5  ] =  32'h9b05688c ;
        hashram.ramdat[ 'h200 + 6  ] =  32'h1f83d9ab ;
        hashram.ramdat[ 'h200 + 7  ] =  32'h5be0cd19 ;
        hashram.ramdat[ 'h200 + 8  ] =  32'h0 ;
        hashram.ramdat[ 'h200 + 9  ] =  32'h0 ;
        hashram.ramdat[ 'h200 + 10 ] =  32'h0 ;
        hashram.ramdat[ 'h200 + 11 ] =  32'h0 ;
        hashram.ramdat[ 'h200 + 12 ] =  32'h0 ^ 32'h3;
        hashram.ramdat[ 'h200 + 13 ] =  32'h0 ^ 32'h0;
        hashram.ramdat[ 'h200 + 14 ] =  32'h0 ;
        hashram.ramdat[ 'h200 + 15 ] =  32'h0 ;

        hashram.ramdat[ 'h200 + 32 + 2*0  :  'h200 + 32 + 2*0  + 1 ] =  { 32'h00636261 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*1  :  'h200 + 32 + 2*1  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*2  :  'h200 + 32 + 2*2  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*3  :  'h200 + 32 + 2*3  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*4  :  'h200 + 32 + 2*4  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*5  :  'h200 + 32 + 2*5  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*6  :  'h200 + 32 + 2*6  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*7  :  'h200 + 32 + 2*7  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;

        #(1 `US) @( negedge clk ) start = 1; @( negedge clk ) start = 0;
        @(posedge done);
       #(1 `US);

       hashcmp =  256'h508c5e8c327c14e2_e1a72ba34eeb452f_37458b209ed63a29_4d999b4c86675982;
//       hashcmp =   { 32'h85e655d6, 32'h417a1795, 32'h3363376a, 32'h624cde5c, 32'h76e09589, 32'hcac5f811, 32'hcc4b32c1, 32'hf20e533a };

        $write("@i:W    = \n"); for(i=0;i<8 ;i++)$write(" %08x ", sim_ww[i]); $write("\n"); for(i=8;i<16 ;i++)$write(" %08x ", sim_ww[i]); $write("\n");
        $write("@i:Hout = \n"); for(i=0;i<8; i++)$write(" %08x ", sim_st[i]); $write("\n");
        $write("@i:Hcmp = \n"); for(i=0;i<8; i++)$write(" %08x ", hashcmp[i]); $write("\n");
       #(1 `US);

//  blk2b
//  ==
// 0x6a09e667f2bdc948 with param
        hashram.ramdat[ 'h200 + 0  ] = 32'h6a09e667;//32'h6A09E667;
        hashram.ramdat[ 'h200 + 1  ] = 32'hf2bdc948;//32'hF3BCC908;
        hashram.ramdat[ 'h200 + 2  ] = 32'hBB67AE85;
        hashram.ramdat[ 'h200 + 3  ] = 32'h84CAA73B;
        hashram.ramdat[ 'h200 + 4  ] = 32'h3C6EF372;
        hashram.ramdat[ 'h200 + 5  ] = 32'hFE94F82B;
        hashram.ramdat[ 'h200 + 6  ] = 32'hA54FF53A;
        hashram.ramdat[ 'h200 + 7  ] = 32'h5F1D36F1;
        hashram.ramdat[ 'h200 + 8  ] = 32'h510E527F;
        hashram.ramdat[ 'h200 + 9  ] = 32'hADE682D1;
        hashram.ramdat[ 'h200 + 10 ] = 32'h9B05688C;
        hashram.ramdat[ 'h200 + 11 ] = 32'h2B3E6C1F;
        hashram.ramdat[ 'h200 + 12 ] = 32'h1F83D9AB;
        hashram.ramdat[ 'h200 + 13 ] = 32'hFB41BD6B;
        hashram.ramdat[ 'h200 + 14 ] = 32'h5BE0CD19;
        hashram.ramdat[ 'h200 + 15 ] = 32'h137E2179;

        hashram.ramdat[ 'h200 + 16 + 0  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 1  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 2  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 3  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 4  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 5  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 6  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 7  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 8  ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 9  ] =  32'h3;//t0
        hashram.ramdat[ 'h200 + 16 + 10 ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 11 ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 12 ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 13 ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 14 ] =  32'h0;
        hashram.ramdat[ 'h200 + 16 + 15 ] =  32'h0;

        hashram.ramdat[ 'h200 + 32 + 2*0  :  'h200 + 32 + 2*0  + 1 ] =  { 32'h00000000 , 32'h00636261 } ;
        hashram.ramdat[ 'h200 + 32 + 2*1  :  'h200 + 32 + 2*1  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*2  :  'h200 + 32 + 2*2  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*3  :  'h200 + 32 + 2*3  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*4  :  'h200 + 32 + 2*4  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*5  :  'h200 + 32 + 2*5  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*6  :  'h200 + 32 + 2*6  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*7  :  'h200 + 32 + 2*7  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*8  :  'h200 + 32 + 2*8  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*9  :  'h200 + 32 + 2*9  + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*10 :  'h200 + 32 + 2*10 + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*11 :  'h200 + 32 + 2*11 + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*12 :  'h200 + 32 + 2*12 + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*13 :  'h200 + 32 + 2*13 + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*14 :  'h200 + 32 + 2*14 + 1 ] =  { 32'h00000000 , 32'h00000000 } ;
        hashram.ramdat[ 'h200 + 32 + 2*15 :  'h200 + 32 + 2*15 + 1 ] =  { 32'h00000000 , 32'h00000000 } ;

       hashcmp64 =  512'h0d4d1c983fa580ba_e9f6129fb697276a_b7c45a68142f214c_d1a2ffdb6fbb124b_2d79ab2a39c5877d_95cc3345ded552c2_5a92f1dba88ad318_239900d4ed8623b9;
        #(1 `US) ;
        cfg_hashtype = HT_BLK2b ;
        cfg_firsthash = 1; cfg_finalhash = 1; 
        #(1 `US) @( negedge clk ) start = 1; @( negedge clk ) start = 0;
        @(posedge done);
       #(1 `US);
        $write("@i:W    = \n"); for(i=0;i<8 ;i++)$write(" %016x ", sim_ww64[i]); $write("\n"); for(i=8;i<16 ;i++)$write(" %016x ", sim_ww64[i]); $write("\n");
        $write("@i:Hout = \n"); for(i=0;i<8; i++)$write(" %016x ", sim_st64[i]); $write("\n");
        $write("@i:Hcmp = \n"); for(i=0;i<8; i++)$write(" %016x ", hashcmp64[i]); $write("\n");
       #(1 `US);

//  blk3
//  ==
        #(1 `US) ;
        cfg_hashtype = HT_BLK3 ;
        cfg_firsthash = 1; cfg_finalhash = 0; 
        #(1 `US) @( negedge clk ) start = 1; @( negedge clk ) start = 0; @(posedge done);
        #(1 `US);

/*
        #(1 `US) ;
        cfg_firsthash = 0; cfg_finalhash = 1; 
        #(1 `US) @( negedge clk ) start = 1; @( negedge clk ) start = 0;
        @(posedge done);
//        end        

*/

        $display("done");

        #(1 `MS);
    `maintestend

endmodule
