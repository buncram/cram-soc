`include "template.sv"


module daric_top_tb ();

    bit clk,resetn;
    bit clk1m;
    integer i=0, j=0, k=0, errcnt=0, warncnt=0;

    bit [0:2]   cmspad;
    bit         padresetn;
    bit         clkswd;
    bit         dbgtxd;

    reg pulse1M = 0;

    wire pulse1Mwire; assign pulse1Mwire = pulse1M;
    wire uartloop;
    wire SDIO_CLK;
    wire SDIO_CMD;
    wire [3:0] SDIO_DATA;

    wire USB0PN, USB0PP;
    bit host_USB0PN = 0, host_USB0PP=0;

//    assign USB0PN = host_USB0PN;
//    assign USB0PP = host_USB0PP;

  //
  //  dut
  //  ==
/*
    ioif iopad_A[0: 7]();
    ioif iopad_B[0:15]();
    ioif iopad_C[0:13]();
    ioif iopad_D[0:15]();
    ioif iopad_E[0:15]();
    ioif iopad_F[0: 5]();
*/
    bit clk32k, clk32m, clk24m, clk48m, clk60m;
    OSC_SIM #(31.25*1000)   osc32k ( .EN('1), .CFG('0),      .CKO( clk32k  ) );
    OSC_SIM #(16.6667)   osc60m ( .EN('1), .CFG('0),      .CKO( clk60m  ) );

`ifdef FPGA
    OSC_SIM #(20.83333)        osc32M ( .EN('1), .CFG('0),      .CKO( clk32m  ) );
`else

    OSC_SIM #(41.67)        osc24M ( .EN('1), .CFG('0),      .CKO( clk24m  ) );
    OSC_SIM #(20.835)       osc48M ( .EN('1), .CFG('0),      .CKO( clk48m  ) );
    OSC_SIM #(31.25)        osc32M ( .EN('1), .CFG('0),      .CKO( clk32m  ) );
`endif
    wire PAD_SWDIO;
    wire PC11;
    assign PC11 = clk32m;
    wire PB11, PB12;
    pullup ( PB11 );
    pullup ( PB12 );

wire UTMIPAD_dataout0;         pulldown( UTMIPAD_dataout0 );
wire UTMIPAD_dataout1;         pulldown( UTMIPAD_dataout1 );
wire UTMIPAD_dataout2;         pulldown( UTMIPAD_dataout2 );
wire UTMIPAD_dataout3;         pulldown( UTMIPAD_dataout3 );
wire UTMIPAD_dataout4;         pulldown( UTMIPAD_dataout4 );
wire UTMIPAD_dataout5;         pulldown( UTMIPAD_dataout5 );
wire UTMIPAD_dataout6;         pulldown( UTMIPAD_dataout6 );
wire UTMIPAD_dataout7;         pulldown( UTMIPAD_dataout7 );
wire UTMIPAD_hostdisconnect;   pulldown( UTMIPAD_hostdisconnect );
wire UTMIPAD_linestate0;       pulldown( UTMIPAD_linestate0 );
wire UTMIPAD_linestate1;       pulldown( UTMIPAD_linestate1 );
wire UTMIPAD_rxerror;          pulldown( UTMIPAD_rxerror );
wire UTMIPAD_rxvactive;        pulldown( UTMIPAD_rxvactive );
wire UTMIPAD_rxvalid;          pulldown( UTMIPAD_rxvalid );
wire UTMIPAD_txready;          pulldown( UTMIPAD_txready );
wire UTMIPAD_dmpulldown;       pulldown( UTMIPAD_dmpulldown );
wire UTMIPAD_dppulldown;       pulldown( UTMIPAD_dppulldown );
wire UTMIPAD_opmode0;          pulldown( UTMIPAD_opmode0 );
wire UTMIPAD_opmode1;          pulldown( UTMIPAD_opmode1 );
wire UTMIPAD_suspendm;         pulldown( UTMIPAD_suspendm );
wire UTMIPAD_termselect;       pulldown( UTMIPAD_termselect );
wire UTMIPAD_txvalid;          pulldown( UTMIPAD_txvalid );
wire UTMIPAD_xcvselect0;       pulldown( UTMIPAD_xcvselect0 );
wire UTMIPAD_xcvselect1;       pulldown( UTMIPAD_xcvselect1 );

    daric_top dut(
        .XTAL_IN    (clk32m),
`ifndef FPGA
        .XTAL32K_IN (clk32k),
//        .XTAL24M_IN (clk24m),
        .XTAL48M_IN (clk48m),
        .USB0PN                (USB0PN),
        .USB0PP                (USB0PP),
`else
        .UTMIPAD_clk1          (clk60m),

/*    input  wire*/  .UTMIPAD_dataout0,
/*    input  wire*/  .UTMIPAD_dataout1,
/*    input  wire*/  .UTMIPAD_dataout2,
/*    input  wire*/  .UTMIPAD_dataout3,
/*    input  wire*/  .UTMIPAD_dataout4,
/*    input  wire*/  .UTMIPAD_dataout5,
/*    input  wire*/  .UTMIPAD_dataout6,
/*    input  wire*/  .UTMIPAD_dataout7,
/*    input  wire*/  .UTMIPAD_hostdisconnect,
/*    input  wire*/  .UTMIPAD_linestate0,
/*    input  wire*/  .UTMIPAD_linestate1,
/*    input  wire*/  .UTMIPAD_rxerror,
/*    input  wire*/  .UTMIPAD_rxvactive,
/*    input  wire*/  .UTMIPAD_rxvalid,
/*    input  wire*/  .UTMIPAD_txready,
/*    output wire*/  .UTMIPAD_dmpulldown,
/*    output wire*/  .UTMIPAD_dppulldown,
/*    output wire*/  .UTMIPAD_opmode0,
/*    output wire*/  .UTMIPAD_opmode1,
/*    output wire*/  .UTMIPAD_suspendm,
/*    output wire*/  .UTMIPAD_termselect,
/*    output wire*/  .UTMIPAD_txvalid,
/*    output wire*/  .UTMIPAD_xcvselect0,
/*    output wire*/  .UTMIPAD_xcvselect1,
`endif


        .PA0 (pulse1Mwire),
        .PA3 (uartloop),
        .PA4 (uartloop),

        .PC0 (SDIO_CLK),
        .PC1 (SDIO_CMD),
        .PC2 (SDIO_DATA[0]),
        .PC3 (SDIO_DATA[1]),
        .PC4 (SDIO_DATA[2]),
        .PC5 (SDIO_DATA[3]),

        .PB11,.PB12,.PC11,
/*        input logic       */  // .clkxtl     ( clk          ),
/*        input logic [0:2] */  // .cmspad     ( cmspad       ),
/*        input logic       */  // .chipresetn ( resetn       ),
/*        input logic       */  // .padresetn  ( padresetn    ),
                                // .dbgtxd     ( dbgtxd       ),
/*        input logic       */  // .clkswd     ( clkswd       ),
//            .*
        .PAD_SWDCK  (clk1m),
        .PAD_SWDIO  (PAD_SWDIO),
        .PAD_AOXRSTn ( padresetn )
    );

  //
  //  monitor and clk
  //  ==

    `genclk( clk, 20 )
    `genclk( clk1m, 1000 )
    `timemarker2


`ifdef SDSIM
sdModel sd(
  .sdClk ( SDIO_CLK ),
  .cmd   ( SDIO_CMD ),
  .dat   ( SDIO_DATA)
);
`endif

    initial forever begin
         pulse1M = 0;
        #( 500 `US ) pulse1M = 1;
        #( 1 `US ) pulse1M = 1;
    end

//    assign dut.PA0 = pulse1Mwire;

  //
  //  subtitle
  //  ==

    initial begin
        #(100 `MS);
    `maintestend

    `maintest(daric_top_tb,daric_top_tb)
        #105 resetn = 0; #(1 `MS ) padresetn = 1;

//        #(15 `US) resetn = 0;
        #(1 `US) resetn = 1;

        #(200 `MS);
    `maintestend

    initial resetn = 0;
    initial clk = 0;
    initial clkswd = 0;

    integer fd;
    string MEMFILE = "reram_simcode.bin";
    parameter MEMDEPTH = 2**17; //17
    parameter DATAWIDTH = 64;
    localparam DATA_BYTES = 8;

    reg  [DATAWIDTH-1:0]    data;
    reg  [63:0]  data0,data1,data2,data3,data_i;
    reg   [63:0] bin_data0,bin_data1,bin_data2,bin_data3;
    reg  [255:0] reram_source;
    bit  [255:0] probe_reram_data;
    bit  [143:0] reram_data0,reram_data1;
    integer r,r0,r1,r2,r3;
    bit [143:0] probe_r0;
    bit [143:0] probe_r1;

    logic ifmeminitflag=0;
    nvrcfg_pkg::nvrcms_t thenvrcms = nvrcfg_pkg::defnvrcms;
    nvrcfg_pkg::nvripm_t thenvripm = nvrcfg_pkg::defnvripm;
    nvrcfg_pkg::nvrcfg_t thenvrcfg = nvrcfg_pkg::defnvrcfg;
    logic [0:15][255:0] thenvrdat;
    assign thenvrdat = { thenvrcms, thenvripm, thenvrcfg };
    logic [0:15][255:0] thenvrdat0;

`ifndef FPGA
generate
    for (genvar gvi = 0; gvi < 16; gvi++) begin
        /* code */
assign thenvrdat0[gvi] = {
                        dut.soc.soc_coresub.greram[1].reram.u_rram_wrapper.u_rram.ifren_mem[gvi][144:73],
                        dut.soc.soc_coresub.greram[1].reram.u_rram_wrapper.u_rram.ifren_mem[gvi][71:0],
                        dut.soc.soc_coresub.greram[0].reram.u_rram_wrapper.u_rram.ifren_mem[gvi][144:73],
                        dut.soc.soc_coresub.greram[0].reram.u_rram_wrapper.u_rram.ifren_mem[gvi][71:0]
                        };

    end
endgenerate

  assign probe_r0 = {dut.soc.soc_coresub.greram[0].reram.u_rram_wrapper.u_rram.main_mem[0][144:73],
                      dut.soc.soc_coresub.greram[0].reram.u_rram_wrapper.u_rram.main_mem[0][71:0]};

  assign probe_r1 = {dut.soc.soc_coresub.greram[1].reram.u_rram_wrapper.u_rram.main_mem[0][144:73],
                      dut.soc.soc_coresub.greram[1].reram.u_rram_wrapper.u_rram.main_mem[0][71:0]};

  assign probe_reram_data = {probe_r1[127:0],probe_r0[127:0]};


`endif


initial begin

// ■■■■■■■■■■■
//  nvr cfg customization
// ■■■■■■■■■■■

    thenvrcms.cmsdata1 = cms_pkg::CMSDAT_TESTMODE;
    thenvripm.ipm0 = 256'h190f0000_e7df4435_23d32435_7e20a435_6a428c35_6a428c35_00010203_04050607;

    thenvrcfg.cfgcore.devena = nvrcfg_pkg::cpudevmode;                      // to enable the dev mode
    thenvrcfg.cfgcore.coreselcm7 = nvrcfg_pkg::coreselcm7_code;        // to enable the cm7
    thenvrcfg.cfgcore.coreselvex = '0;//nvrcfg_pkg::coreselvex_code;             // to enable the rv


// ■■■■■■■■■■■
//  init rram
// ■■■■■■■■■■■

          fd = $fopen(MEMFILE,"rb");
          #(10 `US );
//          for(i = 0; (i < MEMDEPTH) && ($fread(data,fd) != -1); i = i + 1)
          for(i = 0; (i < MEMDEPTH ) ; i = i + 1)
          begin

`ifdef FPGA
            r = $fread(data,fd);
            if( i < 4096 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[0].FIRST.u0.mem[i] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[0].FIRST.u0.mem[i],r);
            end
            else if( i < 4096*2 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[1].MIDDLE.um.mem[i&4095] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[1].MIDDLE.um.mem[i&4095],r);
            end
            else if( i < 4096*3 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[2].MIDDLE.um.mem[i&4095] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[2].MIDDLE.um.mem[i&4095],r);
            end
            else if( i < 4096*4 ) begin
                dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[3].MIDDLE.um.mem[i&4095] = swizzle(data);
                $display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.gencolmn[0].genrow[3].MIDDLE.um.mem[i&4095],r);
            end
`else
            #0.01;
            r0 = $fread(data0,fd);
            #0.01;
            r1 = $fread(data1,fd);
            #0.01;
            r2 = $fread(data2,fd);
            #0.01;
            r3 = $fread(data3,fd);
            #0.01;
            bin_data0 = swizzle(data0);
            #0.01;
            bin_data1 = swizzle(data1);
            #0.01;
            bin_data2 = swizzle(data2);
            #0.01;
            bin_data3 = swizzle(data3);
            //data = {data3,data2,data1,data0};
            //dut.soc.soc_coresub.rrc.emuram.ramdat[i] = swizzle(data);
            //$display("%d, %016x, %d",i,dut.soc.soc_coresub.rrc.emuram.ramdat[i],r);
            #0.01;
            reram_source = {swizzle(data3),swizzle(data2),swizzle(data1),swizzle(data0)};
            #0.01;
            reram_data0 = rram_encoder(reram_source[127:0],1);
            reram_data1 = rram_encoder(reram_source[255:128],1);

            #0.01;
            dut.soc.soc_coresub.greram[0].reram.u_rram_wrapper.u_rram.main_mem[i] = {1'b0,reram_data0[143:72],1'b0,reram_data0[71:0]};
            dut.soc.soc_coresub.greram[1].reram.u_rram_wrapper.u_rram.main_mem[i] = {1'b0,reram_data1[143:72],1'b0,reram_data1[71:0]};
            #0.01;
`endif

          end


    // IFR region intital
        #0.01;
`ifndef FPGA
        ifmeminitflag = 1;
        for (int i = 0; i < 16; i++) begin
            #0.01;
            reram_data0 = rram_encoder(thenvrdat[i][127:0],1);
            reram_data1 = rram_encoder(thenvrdat[i][255:128],1);
            #0.01;
            dut.soc.soc_coresub.greram[0].reram.u_rram_wrapper.u_rram.ifren_mem[i] = {1'b0,reram_data0[143:72],1'b0,reram_data0[71:0]};
            dut.soc.soc_coresub.greram[1].reram.u_rram_wrapper.u_rram.ifren_mem[i] = {1'b0,reram_data1[143:72],1'b0,reram_data1[71:0]};
        end
        ifmeminitflag = 0;

`endif

/*
          r = 1;
          i = 0;
          while( (i < MEMDEPTH ) && ( r != 0 ) )
          begin
            r = $fread(data,fd) ;
//            r = $fread(data,fd);
            dut.soc_coresub.rrc.emuram.ramdat[i] = swizzle(data);
            $display("%d, %016x, %d",i,dut.soc_coresub.rrc.emuram.ramdat[i],r);

            i = i + 1;
          end
*/
          $fclose(fd);
          $write("RAM: image max    = %d lines\n",i);
    end
//      dut.soc_coresub.rrc.emuram.sramc.ramdat



  //Function to account for the Big Endianness of $fread
  function [DATAWIDTH-1:0] swizzle;
      input [DATAWIDTH-1:0] data_in;
      integer i;
      begin
        for (i=0; i<DATA_BYTES; i=i+1)
          swizzle[i*8 +:8] = data_in[(DATA_BYTES-1-i)*8 +:8];
      end
  endfunction

  function [143:0] rram_encoder;
    input [127:0] messg;
    input mode;
    bit [15:0] parity;

      parity[0]=    messg[0]^    messg[2]^    messg[3]^    messg[4]^    messg[5]^    messg[6]^    messg[12]^    messg[14]^    messg[15]^
                  messg[16]^    messg[18]^    messg[19]^    messg[20]^    messg[21]^    messg[26]^    messg[28]^    messg[31]^    messg[37]^
    messg[38]^    messg[39]^    messg[40]^    messg[42]^    messg[43]^    messg[44]^    messg[46]^    messg[48]^    messg[49]^    messg[51]^
    messg[52]^    messg[53]^    messg[63]^    messg[65]^    messg[66]^    messg[69]^    messg[70]^    messg[71]^    messg[72]^    messg[73]^
    messg[75]^    messg[76]^    messg[77]^    messg[79]^    messg[82]^    messg[84]^    messg[87]^    messg[89]^    messg[91]^    messg[93]^
    messg[97]^    messg[98]^    messg[101]^    messg[107]^    messg[110]^    messg[111]^    messg[112]^    messg[120]^    messg[121]^
    messg[123]^    messg[124]^    messg[125];

     parity[1]=    messg[0]^    messg[1]^    messg[2]^    messg[7]^    messg[8]^    messg[12]^    messg[13]^    messg[14]^    messg[17]^
    messg[20]^    messg[22]^    messg[24]^    messg[26]^    messg[27]^    messg[28]^    messg[29]^    messg[31]^    messg[32]^    messg[37]^
    messg[41]^    messg[45]^    messg[46]^    messg[47]^    messg[48]^    messg[54]^    messg[57]^    messg[63]^    messg[64]^    messg[65]^
    messg[67]^    messg[73]^    messg[74]^    messg[75]^    messg[78]^    messg[79]^    messg[80]^    messg[82]^    messg[83]^    messg[84]^
    messg[85]^    messg[86]^    messg[87]^    messg[88]^    messg[89]^    messg[90]^    messg[91]^    messg[92]^    messg[93]^    messg[94]^
    messg[96]^    messg[97]^    messg[98]^    messg[99]^    messg[101]^    messg[102]^    messg[107]^    messg[108]^    messg[113]^
    messg[120]^    messg[121]^    messg[122]^    messg[123]^    messg[126]^    messg[127];

     parity[2]=    messg[1]^    messg[2]^    messg[3]^    messg[8]^    messg[9]^    messg[13]^    messg[14]^    messg[15]^    messg[18]^
    messg[20]^    messg[21]^    messg[23]^    messg[27]^    messg[28]^    messg[29]^    messg[30]^    messg[32]^    messg[33]^    messg[38]^
    messg[47]^    messg[48]^    messg[49]^    messg[50]^    messg[51]^    messg[54]^    messg[58]^    messg[64]^    messg[65]^    messg[66]^
    messg[73]^    messg[74]^    messg[75]^    messg[76]^    messg[79]^    messg[80]^    messg[81]^    messg[83]^    messg[86]^    messg[87]^
    messg[89]^    messg[90]^    messg[91]^    messg[92]^    messg[93]^    messg[94]^    messg[95]^    messg[96]^    messg[97]^    messg[99]^
    messg[102]^    messg[103]^    messg[105]^    messg[108]^    messg[109]^    messg[110]^    messg[114]^    messg[122]^    messg[123]^
    messg[124]^    messg[127];

     parity[3]=     messg[2]^    messg[3]^    messg[4]^    messg[8]^    messg[9]^    messg[10]^    messg[14]^    messg[15]^    messg[16]^
    messg[19]^    messg[21]^    messg[22]^    messg[24]^    messg[25]^    messg[28]^    messg[29]^    messg[30]^    messg[32]^    messg[33]^
    messg[34]^    messg[39]^    messg[46]^    messg[48]^    messg[49]^    messg[50]^    messg[52]^    messg[54]^    messg[57]^    messg[59]^
    messg[65]^    messg[66]^    messg[67]^    messg[68]^    messg[69]^    messg[74]^    messg[75]^    messg[76]^    messg[77]^    messg[80]^
    messg[81]^    messg[82]^    messg[84]^    messg[85]^    messg[86]^    messg[87]^    messg[88]^    messg[90]^    messg[91]^    messg[92]^
    messg[93]^    messg[94]^    messg[95]^    messg[97]^    messg[98]^    messg[102]^    messg[103]^    messg[104]^    messg[106]^
    messg[109]^    messg[111]^    messg[114]^    messg[115]^    messg[121]^    messg[123]^    messg[124]^    messg[125]^    messg[127];

     parity[4]=    messg[3]^    messg[4]^    messg[5]^    messg[9]^    messg[10]^    messg[11]^    messg[15]^    messg[16]^    messg[17]^    messg[18]^
    messg[22]^    messg[23]^    messg[26]^    messg[29]^    messg[30]^    messg[33]^    messg[34]^    messg[35]^    messg[40]^    messg[42]^    messg[46]^    messg[47]^
    messg[49]^    messg[50]^    messg[53]^    messg[55]^    messg[58]^    messg[60]^    messg[66]^    messg[67]^    messg[68]^    messg[70]^    messg[73]^
    messg[75]^    messg[76]^    messg[77]^    messg[78]^    messg[81]^    messg[82]^    messg[83]^    messg[84]^    messg[87]^    messg[88]^    messg[89]^
    messg[91]^    messg[92]^    messg[93]^    messg[94]^    messg[95]^    messg[96]^    messg[99]^    messg[102]^    messg[103]^    messg[104]^    messg[105]^
    messg[107]^    messg[110]^    messg[112]^    messg[114]^    messg[115]^    messg[116]^    messg[121]^    messg[122]^    messg[124]^    messg[125]^    messg[126];

     parity[5]=    messg[0]^    messg[2]^    messg[3]^    messg[10]^    messg[11]^    messg[14]^    messg[15]^    messg[17]^    messg[18]^
     messg[20]^    messg[21]^    messg[23]^    messg[24]^    messg[25]^    messg[26]^    messg[27]^    messg[28]^    messg[30]^    messg[32]^
    messg[34]^    messg[35]^    messg[36]^    messg[37]^    messg[38]^    messg[39]^    messg[40]^    messg[42]^    messg[44]^    messg[46]^    messg[47]^
    messg[49]^    messg[52]^    messg[53]^    messg[55]^    messg[56]^    messg[59]^    messg[61]^    messg[63]^    messg[65]^    messg[66]^
    messg[67]^    messg[68]^    messg[70]^    messg[72]^    messg[74]^    messg[75]^    messg[78]^    messg[83]^    messg[84]^    messg[85]^    messg[86]^    messg[87]^
    messg[88]^    messg[90]^    messg[91]^    messg[92]^    messg[94]^    messg[95]^    messg[96]^    messg[100]^    messg[101]^    messg[102]^
    messg[103]^    messg[104]^    messg[105]^    messg[106]^    messg[107]^    messg[108]^    messg[112]^    messg[113]^    messg[115]^    messg[116]^
    messg[117]^    messg[120]^    messg[121]^    messg[122]^    messg[124]^    messg[126];

     parity[6]=    messg[0]^    messg[1]^    messg[2]^    messg[5]^    messg[6]^    messg[8]^    messg[11]^    messg[14]^    messg[22]^    messg[24]^    messg[27]^
    messg[29]^    messg[32]^    messg[33]^    messg[35]^    messg[36]^    messg[45]^    messg[47]^    messg[49]^    messg[50]^    messg[51]^
    messg[52]^    messg[54]^    messg[55]^    messg[56]^    messg[57]^    messg[60]^    messg[62]^    messg[63]^    messg[64]^    messg[65]^
    messg[67]^    messg[69]^    messg[70]^    messg[72]^    messg[73]^    messg[77]^    messg[82]^    messg[86]^    messg[92]^    messg[95]^
    messg[100]^    messg[103]^    messg[104]^    messg[105]^    messg[106]^    messg[108]^    messg[109]^    messg[111]^    messg[112]^    messg[113]^
    messg[116]^    messg[117]^    messg[118]^    messg[120]^    messg[121]^    messg[122]^    messg[124];

     parity[7]=    messg[1]^    messg[2]^    messg[3]^    messg[6]^    messg[7]^    messg[8]^    messg[9]^    messg[12]^    messg[15]^    messg[20]^
    messg[23]^    messg[24]^    messg[25]^    messg[28]^    messg[30]^    messg[31]^    messg[32]^    messg[33]^    messg[34]^    messg[36]^
    messg[37]^    messg[42]^    messg[44]^    messg[48]^    messg[51]^    messg[52]^    messg[53]^    messg[54]^    messg[56]^    messg[58]^    messg[61]^
    messg[63]^    messg[64]^    messg[65]^    messg[66]^    messg[68]^    messg[70]^    messg[71]^    messg[73]^    messg[74]^    messg[78]^
    messg[83]^    messg[84]^    messg[87]^    messg[93]^    messg[98]^    messg[101]^    messg[102]^    messg[104]^    messg[105]^    messg[106]^
    messg[107]^    messg[109]^    messg[110]^    messg[112]^    messg[113]^    messg[114]^    messg[117]^    messg[118]^
    messg[119]^    messg[122]^    messg[123]^    messg[125];

     parity[8]=    messg[0]^    messg[5]^    messg[6]^    messg[7]^    messg[9]^    messg[10]^    messg[12]^    messg[13]^    messg[14]^    messg[15]^    messg[19]^
    messg[25]^    messg[28]^    messg[29]^    messg[32]^    messg[33]^    messg[34]^    messg[35]^    messg[39]^    messg[40]^    messg[44]^    messg[45]^
    messg[48]^    messg[59]^    messg[62]^    messg[63]^    messg[64]^    messg[67]^    messg[70]^    messg[73]^    messg[74]^    messg[76]^    messg[77]^
    messg[82]^    messg[84]^    messg[87]^    messg[89]^    messg[91]^    messg[93]^    messg[94]^    messg[96]^    messg[97]^    messg[99]^    messg[100]^
    messg[101]^    messg[102]^    messg[103]^    messg[106]^    messg[108]^    messg[112]^    messg[113]^    messg[115]^
    messg[118]^    messg[119]^    messg[120]^    messg[125]^    messg[126];

     parity[9]=    messg[0]^    messg[1]^    messg[2]^    messg[3]^    messg[4]^    messg[5]^    messg[7]^    messg[10]^    messg[11]^    messg[12]^    messg[13]^
    messg[19]^    messg[20]^    messg[21]^    messg[24]^    messg[25]^    messg[28]^    messg[29]^    messg[30]^    messg[31]^    messg[33]^    messg[34]^
    messg[35]^    messg[36]^    messg[37]^    messg[38]^    messg[39]^    messg[41]^    messg[43]^    messg[45]^    messg[48]^    messg[50]^
    messg[51]^    messg[52]^    messg[53]^    messg[54]^    messg[55]^    messg[60]^    messg[64]^    messg[66]^    messg[69]^    messg[70]^    messg[72]^
    messg[74]^    messg[76]^    messg[78]^    messg[79]^    messg[82]^    messg[83]^    messg[86]^    messg[87]^    messg[89]^    messg[90]^
    messg[91]^    messg[92]^    messg[93]^    messg[94]^    messg[95]^    messg[96]^    messg[103]^    messg[104]^    messg[105]^    messg[109]^    messg[110]^
    messg[111]^    messg[112]^    messg[113]^    messg[114]^    messg[116]^    messg[119]^    messg[123]^    messg[124]^    messg[125]^    messg[126]^    messg[127];

     parity[10]=    messg[0]^    messg[1]^    messg[11]^    messg[13]^    messg[15]^    messg[16]^    messg[19]^    messg[22]^    messg[24]^    messg[28]^
    messg[29]^    messg[30]^    messg[32]^    messg[34]^    messg[35]^    messg[36]^    messg[42]^    messg[43]^    messg[46]^    messg[48]^    messg[54]^
    messg[55]^    messg[56]^    messg[57]^    messg[61]^    messg[63]^    messg[66]^    messg[67]^    messg[68]^    messg[69]^    messg[72]^    messg[73]^
    messg[76]^    messg[80]^    messg[82]^    messg[83]^    messg[84]^    messg[86]^    messg[89]^    messg[90]^    messg[92]^    messg[94]^    messg[95]^
    messg[96]^    messg[98]^    messg[101]^    messg[104]^    messg[105]^    messg[106]^    messg[107]^    messg[113]^    messg[115]^    messg[117]^
    messg[121]^    messg[123]^    messg[126];

     parity[11]=    messg[0]^    messg[1]^    messg[3]^    messg[4]^    messg[5]^    messg[6]^    messg[15]^    messg[17]^    messg[19]^    messg[20]^    messg[21]^
    messg[23]^    messg[25]^    messg[26]^    messg[28]^    messg[29]^    messg[30]^    messg[31]^    messg[32]^    messg[33]^    messg[35]^
    messg[36]^    messg[38]^    messg[39]^    messg[40]^    messg[41]^    messg[44]^    messg[46]^    messg[47]^    messg[48]^    messg[50]^
    messg[51]^    messg[52]^    messg[53]^    messg[54]^    messg[56]^    messg[58]^    messg[62]^    messg[63]^    messg[64]^    messg[65]^
    messg[66]^    messg[67]^    messg[68]^    messg[69]^    messg[71]^    messg[72]^    messg[74]^    messg[75]^    messg[76]^    messg[79]^
    messg[81]^    messg[82]^    messg[83]^    messg[85]^    messg[86]^    messg[89]^    messg[90]^    messg[95]^    messg[99]^    messg[101]^
    messg[106]^    messg[108]^    messg[110]^    messg[111]^    messg[112]^    messg[116]^    messg[118]^    messg[122]^    messg[123]^    messg[125];

     parity[12]=    messg[1]^    messg[2]^    messg[4]^    messg[5]^    messg[6]^    messg[7]^    messg[8]^    messg[16]^    messg[18]^    messg[21]^
    messg[22]^    messg[25]^    messg[26]^    messg[27]^    messg[29]^    messg[30]^    messg[31]^    messg[33]^    messg[34]^    messg[36]^
    messg[37]^    messg[39]^    messg[40]^    messg[41]^    messg[45]^    messg[46]^    messg[47]^    messg[48]^    messg[49]^    messg[50]^
    messg[52]^    messg[53]^    messg[55]^    messg[59]^    messg[63]^    messg[64]^    messg[65]^    messg[66]^    messg[67]^    messg[70]^
    messg[72]^    messg[75]^    messg[76]^    messg[77]^    messg[80]^    messg[82]^    messg[83]^    messg[84]^    messg[86]^    messg[87]^
    messg[88]^    messg[90]^    messg[91]^    messg[100]^    messg[102]^    messg[105]^    messg[107]^    messg[109]^    messg[110]^    messg[111]^
    messg[112]^    messg[113]^    messg[117]^    messg[119]^    messg[123]^    messg[124]^    messg[126];

     parity[13]=    messg[0]^    messg[4]^    messg[7]^    messg[9]^    messg[12]^    messg[14]^    messg[15]^    messg[16]^    messg[17]^    messg[18]^
    messg[20]^    messg[21]^    messg[22]^    messg[23]^    messg[24]^    messg[27]^    messg[30]^    messg[34]^    messg[35]^    messg[39]^
    messg[41]^    messg[43]^    messg[44]^    messg[46]^    messg[47]^    messg[50]^    messg[52]^    messg[55]^    messg[56]^    messg[60]^
    messg[63]^    messg[64]^    messg[67]^    messg[68]^    messg[70]^    messg[72]^    messg[73]^    messg[75]^    messg[78]^    messg[79]^
    messg[81]^    messg[82]^    messg[83]^    messg[86]^    messg[92]^    messg[93]^    messg[96]^    messg[97]^    messg[103]^    messg[106]^
    messg[107]^    messg[108]^    messg[110]^    messg[113]^    messg[118]^    messg[120]^    messg[121]^    messg[123];

     parity[14]=    messg[0]^    messg[1]^    messg[2]^    messg[3]^    messg[4]^    messg[6]^    messg[10]^    messg[12]^    messg[13]^
    messg[14]^    messg[17]^    messg[18]^    messg[22]^    messg[23]^    messg[26]^    messg[35]^    messg[36]^    messg[37]^    messg[38]^
    messg[39]^    messg[42]^    messg[43]^    messg[45]^    messg[46]^    messg[47]^    messg[49]^    messg[51]^    messg[52]^    messg[55]^
    messg[56]^    messg[61]^    messg[63]^    messg[64]^    messg[66]^    messg[69]^    messg[70]^    messg[72]^    messg[73]^    messg[74]^
    messg[75]^    messg[77]^    messg[80]^    messg[83]^    messg[86]^    messg[89]^    messg[91]^    messg[94]^    messg[96]^    messg[101]^
    messg[104]^    messg[105]^    messg[108]^    messg[109]^    messg[110]^    messg[112]^    messg[119]^    messg[121]^
    messg[122]^    messg[123]^    messg[125];

    parity[15]=    messg[1]^    messg[2]^    messg[3]^    messg[4]^    messg[5]^    messg[7]^    messg[11]^    messg[13]^    messg[14]^    messg[15]^
    messg[18]^    messg[19]^    messg[20]^    messg[23]^    messg[25]^    messg[27]^    messg[31]^    messg[36]^    messg[37]^    messg[38]^
    messg[39]^    messg[40]^    messg[41]^    messg[42]^    messg[43]^    messg[47]^    messg[48]^    messg[50]^    messg[51]^    messg[52]^
    messg[53]^    messg[56]^    messg[62]^    messg[64]^    messg[65]^    messg[67]^    messg[68]^    messg[69]^    messg[70]^    messg[71]^
    messg[74]^    messg[75]^    messg[76]^    messg[78]^    messg[81]^    messg[84]^    messg[85]^    messg[86]^    messg[87]^    messg[88]^
    messg[90]^    messg[92]^    messg[95]^    messg[96]^    messg[97]^    messg[100]^    messg[106]^    messg[109]^    messg[110]^    messg[111]^
    messg[113]^    messg[122]^    messg[123]^    messg[124]^    messg[126]^    messg[127];

    rram_encoder =(mode)?({parity,messg}):(144'd0);

  endfunction


endmodule
