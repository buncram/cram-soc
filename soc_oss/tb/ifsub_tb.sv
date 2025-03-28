`include "template.sv"

module ifsub_tb ();
        parameter DW             = 32;
        parameter AW             = 18; // 256KB
        parameter AW32           = AW-2;
        parameter CAM_DW         = 8;
        parameter PAW            = 16;  //APB slaves are 4KB by default
        parameter TRANS_SIZE     = 20;  //max uDMA transaction size of 1MB
        parameter N_SPIM         = 4;
        parameter N_UART         = 4;
        parameter N_I2C          = 1;
//      parameter N_I2S          = 1;
        parameter N_CAM          = 1;
        parameter N_SDIO         = 1;
        parameter N_FILTER       = 1;
        parameter N_EXT_PER      = 1;
        parameter EVCNT          = 8;
        parameter N_SPIS         = 2;
         bit                 clk;
         bit                 resetn;
         bit                 perclk;
         bit                 cmsatpg;
         bit [EVCNT-1:0]    intr;

         bit pclk;

        apbif                  apbs();
        axiif #(.DW(32))               axim();
        ahbif #(.DW(32))               ahbm();
        ioif       spim_clk_pad[N_SPIM-1:0]();
        ioif       spim_csn0_pad[N_SPIM-1:0]();
        ioif       spim_csn1_pad[N_SPIM-1:0]();
        ioif       spim_csn2_pad[N_SPIM-1:0]();
        ioif       spim_csn3_pad[N_SPIM-1:0]();
        ioif       spim_sd0_pad[N_SPIM-1:0]();
        ioif       spim_sd1_pad[N_SPIM-1:0]();
        ioif       spim_sd2_pad[N_SPIM-1:0]();
        ioif       spim_sd3_pad[N_SPIM-1:0]();
        ioif      i2c_scl_pad[N_I2C-1:0] ();
        ioif      i2c_sda_pad[N_I2C-1:0] ();
        ioif      cam_clk_pad            ();
        ioif      cam_data_pad[CAM_DW-1:0]();
        ioif      cam_hsync_pad            ();
        ioif      cam_vsync_pad            ();
        ioif      uart_rx_pad[N_UART-1:0]();
        ioif      uart_tx_pad[N_UART-1:0]();
        ioif      sdio_clk_pad            ();
        ioif      sdio_cmd_pad            ();
        ioif      sdio_data_pad[3:0]       ();
        ioif      i2ss_sd_pad();
        ioif      i2ss_ws_pad();
        ioif      i2ss_sck_pad();
        ioif      i2sm_sd_pad();
        ioif      i2sm_ws_pad();
        ioif      i2sm_sck_pad();

        ioif      scif_sck_pad();
        ioif      scif_dat_pad();
        
        ioif      spis_clk_pad[N_SPIS-1:0]();
        ioif      spis_cs_pad[N_SPIS-1:0]();
        ioif      spis_mosi_pad[N_SPIS-1:0]();
        ioif      spis_miso_pad[N_SPIS-1:0]();


    integer i, j, k, errcnt, warncnt;
    assign pclk = clk;

  //
  //  dut
  //  ==
    ifsub1 #(.AXIMID4(daric_cfg::AMBAID4_UDMA)) udma(.*);

    axi_ahb_bdg #(.AW(AW),.DW(DW))aab(
        .clk,
        .resetn,
        .axislave(axim),
        .ahbmaster(ahbm)
    );

    ahb_sram #(.AW(11))ahbram(
        .clk,
        .resetn,
        .ahbs(ahbm)
    );

    initial 
    for (int i = 0; i < 512; i++) begin
        ahbram.ramdata[i] = $urandom();
    end

  //
  //  apb drive
  //  ==

    bit             psel         ;
    bit  [PAW-1:0]  paddr        ;
    bit             penable      ;
    bit             pwrite       ;
    bit  [3:0]      pstrb        ;
    bit  [2:0]      pprot        ;
    bit  [DW-1:0]   pwdata       ;
    bit             apbactive    ;
    logic [DW-1:0]   prdata       ;
    logic            pready       ;
    logic            pslverr      ;
    bit  [DW-1:0]           prdatareg;
    wire2apbm apbdrv(.apbm(apbs),.*);

task apbrd();
    input bit  [PAW-1:0]  tpaddr;
    @(negedge pclk);
    psel = 1;
    paddr = tpaddr;
    penable = '1;
    pwrite = '0;
    @(negedge pclk);
    psel = 0;
    penable = '0;
endtask : apbrd

    `thereg(prdatareg) <= ( penable & psel & ~pwrite ) ? prdata : prdatareg;

task apbwr();
    input bit  [PAW-1:0]  tpaddr;
    input bit  [DW-1:0]   tpwdata;
    @(negedge pclk);
    psel = 1;
    paddr = tpaddr;
    penable = '1;
    pwrite = '1;
    pwdata = tpwdata;
    @(negedge pclk);
    psel = 0;
    penable = '0;
    pwrite = '0;
endtask : apbwr

    `timemarker
    `genclk( clk, 6 );
    `genclk( perclk, 10 );
    `maintest( ifsub_tb, ifsub_tb )
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        #( 2 `US );

        apbwr(0,'1);
        apbrd(0);

        apbwr(32'h0a4, 32'h8_0100);
        apbrd(32'h0a4);

        apbwr(32'h090, 32'h0_0100);
        apbwr(32'h094, 32'h8);
        apbwr(32'h098, 32'h10);
        apbwr(32'h0ac, 32'h01);
        apbrd(32'h0a4);

        #(10 `US)         
        apbrd(32'h0b0);

        #( 1 `MS );
    `maintestend

  //
  //  monitor and clk
  //  ==


  //
  //  subtitle
  //  ==
endmodule


    module wire2apbm #(
      parameter PAW=16,
      parameter DW=32
     )(
        apbif.master            apbm,
        input  logic            psel         ,
        input  logic [PAW-1:0]   paddr        ,
        input  logic            penable      ,
        input  logic            pwrite       ,
        input  logic [3:0]      pstrb        ,
        input  logic [2:0]      pprot        ,
        input  logic [31:0]     pwdata       ,
        input  logic            apbactive    ,
        output logic [DW-1:0]   prdata       ,
        output logic            pready       ,
        output logic            pslverr
    );

        assign apbm.psel      = psel          ;
        assign apbm.paddr     = paddr         ;
        assign apbm.penable   = penable       ;
        assign apbm.pwrite    = pwrite        ;
        assign apbm.pstrb     = pstrb         ;
        assign apbm.pprot     = pprot         ;
        assign apbm.pwdata    = pwdata        ;
        assign apbm.apbactive = apbactive     ;
        assign prdata       = apbm.prdata       ;
        assign pready       = apbm.pready       ;
        assign pslverr      = apbm.pslverr      ;

    endmodule
