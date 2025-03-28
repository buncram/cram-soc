
module pmu (

// power
    inout   wire  VDD                 ,
    inout   wire  VSS                 ,
    inout   wire  VDD25               ,
    inout   wire  VDD85A              ,
    inout   wire  VDD85D              ,
    inout   wire  VDDAO               ,

// vddao pin
    input   wire  VR25ENA             ,
    input   wire  VR85AENA            ,
    input   wire  VR85DENA            ,
    output  wire  BGRDY               ,
    output  wire  VR25RDY             ,
    output  wire  VR85ARDY            ,
    output  wire  VR85DRDY            ,
    input   wire  VR85A95ENA          ,
    input   wire  VR85D95ENA          ,
    input   wire  VR85AOSENA          ,
    input   wire  VR85DOSENA          ,
    input   wire  TRM_LATCH_b         ,
    input   wire [6-1:0] TRM_CUR      ,
    input   wire [5-1:0] TRM_CTAT     ,
    input   wire [5-1:0] TRM_PTAT     ,
    input   wire [5-1:0] TRM_D1P2     ,
    input   wire [5-1:0] TRM_DP60     ,
    input   wire  IOUTENA             ,
    input   wire  IBIASENA            ,
    input   wire  POCENA              ,
    output  wire  POR                 ,
    input   wire [3-1:0] PMU_TEST_EN  ,
    input   wire [3-1:0] PMU_TEST_SEL ,
// analog
    inout   wire  PMU_ANA_TEST        ,
    inout   wire  ANA_IN0P1U          ,
    output  wire  POC_IO              ,

// vdd85d
    input   wire  VD09ENA             ,
    input   wire  VD09TL              ,
    input   wire  VD09TH              ,
    output  wire  VD09L               ,
    output  wire  VD09H               ,
    input   wire  VD25ENA             ,
    input   wire  VD25TL              ,
    input   wire  VD25TH              ,
    output  wire  VD25L               ,
    output  wire  VD25H               ,
    input   wire  VD33ENA             ,
    input   wire  VD33TL              ,
    input   wire  VD33TH              ,
    output  wire  VD33L               ,
    output  wire  VD33H

);

`ifdef FPGA

assign BGRDY =     '1;
assign VR25RDY =   '1;
assign VR85ARDY =  '1;
assign VR85DRDY =  '1;
assign POR =       '0;
assign POC_IO =    '0;
assign VD09L =     '1;
assign VD09H =     '1;
assign VD25L =     '1;
assign VD25H =     '1;
assign VD33L =     '1;
assign VD33H =     '1;
assign VDD    =    '1;
assign VSS    =    '0;
assign VDD25  =    '1;
assign VDD85A =    '1;
assign VDD85D =    '1;
assign VDDAO  =    '1;

`endif


`ifdef SIM


    logic reg_BGRDY = '0;     assign BGRDY = reg_BGRDY;
    logic reg_VR25RDY = '0;   assign VR25RDY = reg_VR25RDY;
    logic reg_VR85ARDY = '0;  assign VR85ARDY = reg_VR85ARDY;
    logic reg_VR85DRDY = '0;  assign VR85DRDY = reg_VR85DRDY;
    logic reg_POR = '0;       assign POR = reg_POR;
    logic reg_POC_IO = '0;    assign POC_IO = reg_POC_IO;
    logic reg_VD09L = '0;     assign VD09L = reg_VD09L;
    logic reg_VD09H = '0;     assign VD09H = reg_VD09H;
    logic reg_VD25L = '0;     assign VD25L = reg_VD25L;
    logic reg_VD25H = '0;     assign VD25H = reg_VD25H;
    logic reg_VD33L = '0;     assign VD33L = reg_VD33L;
    logic reg_VD33H = '0;     assign VD33H = reg_VD33H;

// power
    logic pwr_VDD    = '0;    assign VDD    = pwr_VDD    ;
    logic pwr_VSS    = '0;    assign VSS    = pwr_VSS    ;
    logic pwr_VDD25  = '0;    assign VDD25  = pwr_VDD25  ;
    logic pwr_VDD85A = '0;    assign VDD85A = pwr_VDD85A ;
    logic pwr_VDD85D = '0;    assign VDD85D = pwr_VDD85D ;
    logic pwr_VDDAO  = '0;    assign VDDAO  = pwr_VDDAO  ;


    initial  begin
        pwr_VDD = 0;
    #1  pwr_VDD = 1;
    end

    always@(posedge pwr_VDD)   #( 100 `US ) pwr_VDDAO = 1;
    always@(posedge pwr_VDD or posedge VR25ENA  )   #( 300 `US ) pwr_VDD25 =  pwr_VDD & VR25ENA ;
    always@(posedge pwr_VDD or posedge VR85AENA )   #( 200 `US ) pwr_VDD85A = pwr_VDD & VR85AENA;
    always@(posedge pwr_VDD or posedge VR85DENA )   #( 200 `US ) pwr_VDD85D = pwr_VDD & VR85DENA;

    always@( * ) if(~ pwr_VDD)  #( 300 `US ) pwr_VDDAO = '0;
    always@( * ) if(~ pwr_VDD | ~ VR25ENA  )  #( 100 `US ) pwr_VDD25 =  '0;
    always@( * ) if(~ pwr_VDD | ~ VR85AENA )  #( 100 `US ) pwr_VDD85A = '0;
    always@( * ) if(~ pwr_VDD | ~ VR85DENA )  #( 100 `US ) pwr_VDD85D = '0;

    always@(*)
        if(pwr_VDDAO) begin
            reg_POR = '1;
            #( 150 `US );
            reg_POR = '0;
        end else if(~pwr_VDD) begin
            reg_POR =  1;
            @(negedge pwr_VDDAO);
            reg_POR = '0;
        end

        logic pwr_VDD85A_delay20;
    assign #(20 `US) pwr_VDD85A_delay20 =  pwr_VDD85A;
    always@(*) reg_POC_IO = pwr_VDD & ~( pwr_VDD85A_delay20 & pwr_VDD85A );

    always@(posedge pwr_VDD)    #( 700 `US ) reg_BGRDY = '1;
    always@(posedge pwr_VDD25)   #( 400 `US ) reg_VR25RDY = '1;
    always@(posedge pwr_VDD85A)  #( 200 `US ) reg_VR85ARDY = '1;
    always@(posedge pwr_VDD85A)  #( 200 `US ) reg_VR85DRDY = '1;

    always@( * ) if(~ pwr_VDD)    reg_BGRDY = '0;
    always@( * ) if(~ pwr_VDD25)   reg_VR25RDY = '0;
    always@( * ) if(~ pwr_VDD85A)  reg_VR85ARDY = '0;
    always@( * ) if(~ pwr_VDD85A)  reg_VR85DRDY = '0;

    always@(*)begin
        reg_VD09L = VDD25 & VDD85A & ( ~VD09ENA ? '1 : VD09TL ? '1 : VDD85D );
        reg_VD09H = VDD25 & VDD85A & ( ~VD09ENA ? '1 : VD09TH ? '1 : VDD85D );
        reg_VD25L = VDD25 & VDD85A & ( ~VD25ENA ? '1 : VD25TL ? '1 : VDD25 );
        reg_VD25H = VDD25 & VDD85A & ( ~VD25ENA ? '1 : VD25TH ? '1 : VDD25 );
        reg_VD33L = VDD25 & VDD85A & ( ~VD33ENA ? '1 : VD33TL ? '1 : VDD );
        reg_VD33H = VDD25 & VDD85A & ( ~VD33ENA ? '1 : VD33TH ? '1 : VDD );
    end



/*
    logic VDD25_vrdrv, VDD85A_vrdrv, VDD85D_vrdrv, VDDAO_vrdrv;

    assign VSS    = '0;
    assign VDD25  = VDD25_vrdrv;
    assign VDD85A = VDD85A_vrdrv;
    assign VDD85D = VDD85D_vrdrv;
    assign VDDAO  = VDDAO_vrdrv;

always@(*)begin

if( VDD ) begin
    #( 100 `US );
        VDDAO_vrdrv = '1;
        VDD08A_vrdrv = '1;
        VDDAO_vrdrv = '1;
    end
end

end


bit BGRDY_reg = 0;       assign BGRDY = BGRDY_reg;
bit VR25RDY_reg = 0;     assign VR25RDY = VR25RDY_reg;
bit VR85ARDY_reg = 0;    assign VR85ARDY = VR85ARDY_reg;
bit VR85DRDY_reg = 0;    assign VR85DRDY = VR85DRDY_reg;
bit POR_reg = 0;         assign POR = POR_reg;
bit VD09L_reg = 0;       assign VD09L = VD09L_reg;
bit VD09H_reg = 0;       assign VD09H = VD09H_reg;
bit VD25L_reg = 0;       assign VD25L = VD25L_reg;
bit VD25H_reg = 0;       assign VD25H = VD25H_reg;
bit VD33L_reg = 0;       assign VD33L = VD33L_reg;
bit VD33H_reg = 0;       assign VD33H = VD33H_reg;


    output  wire  BGRDY               ,
    output  wire  VR25RDY             ,
    output  wire  VR85ARDY            ,
    output  wire  VR85DRDY            ,
    output  wire  POR                 ,
    output  wire  POC_IO              ,
    output  wire  VD09L               ,
    output  wire  VD09H               ,
    output  wire  VD25L               ,
    output  wire  VD25H               ,
    output  wire  VD33L               ,
    output  wire  VD33H               ,

*/
`endif

endmodule
