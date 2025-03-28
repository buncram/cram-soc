module bmxcore
#(
)(
    input bit           aclk,
    input bit           hclk,
    input bit           resetn,
    input bit           cmsatpg,

    axiif.slave         cm7_axim,
    ahbif.slave         cm7_ahbp,
    axiif.slave         sce_axi32,
    ahbif.slave         mdma_ahb32,

    axiif.master        rrc_axi64,
    axiif.master        sram0_axi64,
    axiif.master        sram1_axi64,
    axiif.master        qfc_axi64,

    ahbif.master        cm7_ahbs,
    ahbif.master        core_ahb32,
    ahbif.master        bmxif_ahb32
);

    typedef axi_pkg::xbar_rule_32_t       rule32_t; // Has to be the same width as axi addr

    parameter XIDW  = 8;     // axi id width
    parameter XUDW  = 8;     // axi userdata width
    parameter XLENW = 8;     // axi len width

    parameter HIDW  = 4;     // axi id width
    parameter HUDW  = 4;     // axi userdata width

//  sce demux
//  ===

     axiif #(
        .AW     ( 32     ),
        .DW     ( 32     ),
        .LENW   ( XLENW  ),
        .IDW    ( XIDW   ),
        .UW     ( XUDW   )
      ) sce_axidemux[1:0] (), nic_s2(), mdma_axi(), tmpnull();

     AXI_BUS #(
        .AXI_ADDR_WIDTH     ( 32     ),
        .AXI_DATA_WIDTH     ( 32     ),
        .AXI_ID_WIDTH       ( XIDW   ),
        .AXI_USER_WIDTH     ( XUDW   )
      ) sce_axi32_pulp[1:0](), sce_axidemux_pulp[0:1] (), nic_s2_pulp(), aximux_slave_pulp[0:1]();//, mdma_axi_pulp();

    localparam axi_pkg::xbar_cfg_t sce_axi_demux_cfg = '{
         NoSlvPorts:         2,
         NoMstPorts:         2,
         MaxMstTrans:        1,
         MaxSlvTrans:        1,
         FallThrough:        1'b0,
         LatencyMode:        axi_pkg::NO_LATENCY,
         AxiIdWidthSlvPorts: XIDW+1,
         AxiIdUsedSlvPorts:  XIDW,
         UniqueIds:          1'b0,
         AxiAddrWidth:       32,
         AxiDataWidth:       32,
         NoAddrRules:        2
    };
  
    localparam rule32_t [sce_axi_demux_cfg.NoAddrRules-1:0] sce_axi_demux_map = '{
        '{idx: 32'd1 , start_addr: 32'h0000_0000, end_addr: 32'h6000_0000}, // to ahb_bmx33
        '{idx: 32'd0 , start_addr: 32'h6000_0000, end_addr: 32'ha000_0000}  // to nic_1
    };

    axitrans_axi2pulp at0( .axis(sce_axi32), .axim(sce_axi32_pulp[0]) );

    axim_null at0a( .aximaster(tmpnull));
    axitrans_axi2pulp at0b( .axis(tmpnull), .axim(sce_axi32_pulp[1]) );

    axi_xbar_intf #(
      .AXI_USER_WIDTH         ( XUDW  ),
      .Cfg                    ( sce_axi_demux_cfg ),
      .ATOPS                  ( 1'b0 ),
      .rule_t                 ( rule32_t   )
    ) sce_axi_demux (
      .clk_i                  ( hclk     ),
      .rst_ni                 ( resetn  ),
      .test_i                 ( cmsatpg    ),
      .slv_ports              ( sce_axi32_pulp  ),
      .mst_ports              ( sce_axidemux_pulp   ),
      .addr_map_i             ( sce_axi_demux_map ),
      .en_default_mst_port_i  ( '0      ),
      .default_mst_port_i     ( '0      )
    );

//    axitrans_pulp2axi at1( .axis(sce_axidemux_pulp[0]), .axim(sce_axidemux[0]) );

//  sce demux - axi-ahb bridge
//  ===

    ahbif   sce_ahb();

    axitrans_pulp2axi at2( .axis(sce_axidemux_pulp[1]), .axim(sce_axidemux[1]) );

    axi_ahb_bdg #(
        .AW (32),
        .DW (32)
    ) sce_ahb_bdg (
        .clk                    ( hclk     ),
        .resetn                 ( resetn  ),
        .axislave               ( sce_axidemux[1] ),
        .ahbmaster              ( sce_ahb )
    );

//  dma demux
//  ===

     ahbif   mdma_ahb32demux[1:0]();

     AXI_BUS #(
        .AXI_ADDR_WIDTH ( 32     ),
        .AXI_DATA_WIDTH ( 32     ),
        .AXI_ID_WIDTH   ( XIDW   ),
        .AXI_USER_WIDTH ( XUDW   )
     ) mdma_axi_pulp ();

    localparam rule32_t [1:0] mdma_ahb_demux_map = '{
        '{idx: 32'd1 , start_addr: 32'h0000_0000, end_addr: 32'h6000_0000}, // to ahb_bmx33
        '{idx: 32'd0 , start_addr: 32'h6100_0000, end_addr: 32'ha000_0000}  // to nic_1, but no ReRAM
    };

    ahb_demux_map #(
        .SLVCNT                 ( 2  ),
        .DW                     ( 32 ),
        .AW                     ( 32 ),
        .UW                     ( HUDW ),
        .ADDRMAP                ( mdma_ahb_demux_map )
    ) mdma_ahb_demux (
        .hclk                   ( hclk     ),
        .resetn                 ( resetn  ),
        .ahbslave               ( mdma_ahb32 ),
        .ahbmaster              ( mdma_ahb32demux )
    );

    ahb_axi_bdg #(
      .AW (32),
      .DW (32)
    ) dma_axi_bdg (
      .clk                    ( hclk    ),
      .resetn                 ( resetn  ),
      .ahbslave               ( mdma_ahb32demux[0] ),
      .aximaster              ( mdma_axi )
    );

//  sce/dma mux
//  ===

    axitrans_axi2pulp at4(.axis(mdma_axi),.axim(mdma_axi_pulp));

    axithru_pulp at5(.axis(mdma_axi_pulp),          .axim(aximux_slave_pulp[0]) );
    axithru_pulp at6(.axis(sce_axidemux_pulp[0]),   .axim(aximux_slave_pulp[1]) );

    axi_mux_intf #(
      .SLV_AXI_ID_WIDTH       (XIDW),
      .MST_AXI_ID_WIDTH       (XIDW+1),
      .AXI_ADDR_WIDTH         (32),
      .AXI_DATA_WIDTH         (32),
      .AXI_USER_WIDTH         (XUDW),
      .NO_SLV_PORTS           (2),
      .MAX_W_TRANS            (1),
      .FALL_THROUGH           (1'b0),
      .SPILL_AW               (1'b0),
      .SPILL_W                (1'b0),
      .SPILL_B                (1'b0),
      .SPILL_AR               (1'b0),
      .SPILL_R                (1'b0)
    ) axi_mux (
      .clk_i                  ( hclk    ),
      .rst_ni                 ( resetn  ),
      .test_i                 ( cmsatpg    ),
      .slv                    ( aximux_slave_pulp ),
      .mst                    ( nic_s2_pulp )
    );

    axitrans_pulp2axi at3(.axis(nic_s2_pulp),.axim(nic_s2));

//  ahb bmx
//  ===

    ahbif bmx33s[0:2](), bmx33m[0:2]();

    ahb_thru bmx33s0 ( .ahbslave(sce_ahb),            .ahbmaster( bmx33s[0] ));
    ahb_thru bmx33s1 ( .ahbslave(mdma_ahb32demux[1]), .ahbmaster( bmx33s[1] ));
    ahb_thru bmx33s2 ( .ahbslave(cm7_ahbp),           .ahbmaster( bmx33s[2] ));
    ahb_thru bmx33m0 ( .ahbslave(bmx33m[0])         , .ahbmaster(cm7_ahbs ));
    ahb_thru bmx33m1 ( .ahbslave(bmx33m[1])         , .ahbmaster(core_ahb32 ));
    ahb_thru bmx33m2 ( .ahbslave(bmx33m[2])         , .ahbmaster(bmxif_ahb32 ));
    

  ahb_bmx33_intf #(
  ) ahb_bmx33 (
    .hclk                    ( hclk    ),
    .resetn                 ( resetn  ),
    .ahbs               ( bmx33s ),
    .ahbm               ( bmx33m )
  );

//  nic 400
//  ===

  nic1_intf nic(
    .clk0       (aclk),
    .clk1       (hclk),
    .resetn     (resetn),
    .s0         (cm7_axim),
    .s2         (nic_s2),
    .m0         (rrc_axi64),
    .m1         (sram0_axi64),
    .m2         (sram1_axi64),
    .m3         (qfc_axi64)
  );

endmodule

//`ifdef __BMXCORE_DUMMYTB

    module dummytb_bmxcore();
        bit     aclk, hclk, resetn;

    axiif        cm7_axim();
    ahbif        cm7_ahbp();
    axiif        sce_axi32();
    ahbif        mdma_ahb32();

    axiif        rrc_axi64();
    axiif        sram0_axi64();
    axiif        sram1_axi64();
    axiif        qfc_axi64();

    ahbif        cm7_ahbs();
    ahbif        core_ahb32();
    ahbif        bmxif_ahb32();

    logic cmsatpg = 1;

 bmxcore dut
(
/*    input bit           */.aclk(aclk),
/*    input bit           */.hclk(hclk),
/*    input bit           */.resetn(resetn),
                            .cmsatpg    (cmsatpg),
/*    axiif.slave         */.cm7_axim(cm7_axim),
/*    ahbif.slave         */.cm7_ahbp(cm7_ahbp),
/*    axiif.slave         */.sce_axi32(sce_axi32),
/*    ahbif.slave         */.mdma_ahb32(mdma_ahb32),
/*    axiif.master        */.rrc_axi64(rrc_axi64),
/*    axiif.master        */.sram0_axi64(sram0_axi64),
/*    axiif.master        */.sram1_axi64(sram1_axi64),
/*    axiif.master        */.qfc_axi64(qfc_axi64),
/*    ahbif.master        */.cm7_ahbs(cm7_ahbs),
/*    ahbif.master        */.core_ahb32(core_ahb32),
/*    ahbif.master        */.bmxif_ahb32(bmxif_ahb32)
);

    axim_null ucm7_axim(cm7_axim);
    ahbm_null ucm7_ahbp(cm7_ahbp);
    axim_null usce_axi32(sce_axi32);
    ahbm_null umdma_ahb32(mdma_ahb32);
    axis_null urrc_axi64(rrc_axi64);
    axis_null usram0_axi64(sram0_axi64);
    axis_null usram1_axi64(sram1_axi64);
    axis_null uqfc_axi64(qfc_axi64);
    ahbs_null ucm7_ahbs(cm7_ahbs);
    ahbs_null ucore_ahb32(core_ahb32);
    ahbs_null ubmxif_ahb32(bmxif_ahb32);

//    AXI_BUS   xslv();
//    AXI_BUS   xmst[1:0]();

//    axi_demux_intf#( .NO_MST_PORTS(2) ) u1
//    ( .clk_i('0), .rst_ni('0), .slv(xslv), .mst(xmst));


    endmodule
//`endif
