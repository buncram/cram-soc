`include "template.sv"
//`include "icg.v"

module cgucore
    #(
        parameter ICNT = 3,
        parameter OCNT = 5,
        parameter FDW = 8,
        parameter GEARLMT = 2**FDW
    )
    (
        input   logic [0:ICNT-1]    clksrc,
        input   logic               cmsatpg,
        input   logic               porresetn,
        input   logic               resetn,
        input   logic               clksyssel,
        input   logic               clktopsel,
        input   logic               clktopselupdate,
        output  logic               clksys,
        output  logic               clktop,

        input   logic               clktopenin,
        input   bit   [0:OCNT-1][FDW-1:0]  fd,
        input   bit                 fdload,
        output  logic [0:OCNT-1]    clkout,
        output  logic [0:OCNT-1]    clkouten,
        output  logic [0:OCNT-1]    clkouten_atparent
    );

    bit         clk;
    bit         clksysselreg,  clktopselreg;
    bit         clksysselreg0, clktopselreg0;
    bit [0:1]   clksysselen, clksys0, clktopselen, clktop0;

// clksys select
// == 


//    `theregfull(clk, porresetn, clksysselreg0, 1'b0) <= clksrcselupdate ? clksyssel : clksysselreg0;
    `theregfull(clksys, porresetn, clksysselreg0, 1'b0) <= clksyssel;
    `theregfull(clksys, porresetn, clksysselreg , 1'b0) <= clksysselreg0 ;

    cgudyncswt uclksyssel(
        .clk0   (clksrc[0]),
        .clk1   (clksrc[1]),
        .resetn (porresetn),
        .clksel (clksysselreg),
        .clk0en (clksysselen[0]),
        .clk1en (clksysselen[1])
    );

    ICG uclksys0 ( .CK (clksrc[0]), .EN ( clksysselen[0] ), .SE(cmsatpg), .CKG ( clksys0[0] ));
    ICG uclksys1 ( .CK (clksrc[1]), .EN ( clksysselen[1] ), .SE(cmsatpg), .CKG ( clksys0[1] ));
    assign clksys = |clksys0 ;

// clktop select
// == 

    assign clk = clktop;

    `theregrn( clktopselreg0 ) <= clktopselupdate ? clktopsel : clktopselreg0;
    `theregrn( clktopselreg )  <= clktopselreg0;

    cgudyncswt uclktopsel(
        .clk0   (clksys),
        .clk1   (clksrc[2]),
        .resetn (resetn),
        .clksel (clktopselreg),
        .clk0en (clktopselen[0]),
        .clk1en (clktopselen[1])
    );

    ICG uclktop0 ( .CK (clksys   ), .EN ( clktopselen[0] ), .SE(cmsatpg), .CKG ( clktop0[0] ));
    ICG uclktop1 ( .CK (clksrc[2]), .EN ( clktopselen[1] ), .SE(cmsatpg), .CKG ( clktop0[1] ));
    assign clktop = |clktop0 ;

// clkout 0 : gear
// == 

    logic [0:OCNT-1]     iclken;
    logic [0:OCNT-1]     oclken;
    logic [0:OCNT-1]     oclken_aticlk;

    assign clkouten = oclken;

    CguGearBox fd0(
        .clk        (clktop     ),
        .resetn     (resetn     ),
//        .clkenin    (clktopenin ),
        .GearLmt    (GEARLMT[7:0] ),
        .GearLvl    (fd[0]      ),
        .GearLoad   (fdload     ),
        .GearGen    (oclken[0]  )
    );

    ICG fdicg ( .CK (clktop   ), .EN ( oclken[0] ), .SE(cmsatpg), .CKG ( clkout[0] ));

    assign iclken[0] = 1'b0;
    assign clkouten_atparent[0] = 1'b1;

// clkout 1~4 : fdsync
// == 
    logic [1:OCNT-1] oclkencheck;
    logic [1:OCNT-1] clkoutcheck;
    bit   [1:OCNT-1][15:0] checkcnt0, checkcnt1;
    logic [1:OCNT-1]checkfdsync_enatparenterror ;
    genvar gvi;

    generate
       for(gvi=1;gvi<OCNT;gvi++) begin: genfd
            cgufdsync fd(
                .clk            (clktop),
                .resetn         (resetn),
                .clk0en         (clktopenin),
                .clk1en         (iclken[gvi]),
                .fd2            (fd[gvi]),
                .fdload         (fdload),
                .clk2en         (oclken[gvi]),
                .clk2en_atclk1  (clkouten_atparent[gvi])
            );
            assign oclkencheck[gvi] = oclken[gvi] & ~iclken[gvi];
            `ifdef SIM
                always@(posedge clktop)
                    if(oclkencheck[gvi]) $display("%t %m clkfdsync wrong!", $time );

            ICG fdicgcheck ( .CK (clkout[gvi-1]   ), .EN ( clkouten_atparent[gvi] ), .SE(cmsatpg), .CKG ( clkoutcheck[gvi] ));
            always@(posedge clkout[gvi])      checkcnt0[gvi] <= checkcnt0[gvi] + 1;
            always@(posedge clkoutcheck[gvi]) checkcnt1[gvi] <= checkcnt1[gvi] + 1;
            always@(posedge clktop) checkfdsync_enatparenterror[gvi] <= ~( checkcnt0[gvi] == checkcnt1[gvi] );
            always@(posedge clktop)
                if(checkfdsync_enatparenterror[gvi]) $display("%t %m checkfdsync_en_atparent_error wrong!", $time );

            `endif
            assign iclken[gvi] = oclken[gvi-1];
            ICG fdicg ( .CK (clktop   ), .EN ( oclken[gvi] ), .SE(cmsatpg), .CKG ( clkout[gvi] ));
        end
    endgenerate
/*
*/
endmodule:cgucore


// sim testbench
// ==

`ifdef SIMCGUCORE

module cgucoretb();

        parameter ICNT = 3;
        parameter OCNT = 5;
        parameter FDW = 8;
        parameter GEARLMT = 2**FDW;

    bit [0:ICNT-1]    clksrc;
    bit               clk, resetn;
    bit               clksrcselupdate;
    bit               clksyssel;
    bit               clktopsel;
    bit               clksys;
    bit               clktop;
    bit               clktopenin=1;
    bit   [0:OCNT-1][FDW-1:0]  fd;
    bit                 fdload;
    logic [0:OCNT-1]    clkout;
    logic [0:OCNT-1]    clkouten_atparent;
    integer i;

    `genclk( clksrc[0], 40 );   // osc
    `genclk( clksrc[1], 41 );   // xtal
    `genclk( clksrc[2], 23  );  // clkpll


cgucore  #(
        .ICNT ( 3 ),
        .OCNT ( 5 ),
        .FDW ( 8 ),
        .GEARLMT ( 2**8 )
    )dut(.*);

//    `timemarker
    integer tmms=0, tmus=0; 
    initial forever #( 1 `US ) tmus = ( tmus == 1000 ) ? 0 : tmus + 1 ; 
    initial forever #( 1 `MS ) tmms = tmms + 1 ; 
    always@( tmms ) $display("------------------------------------[%0dms][%4h][%4h][%4h][%4h][%d]------------------------------------", tmms,
            dut.checkcnt0[1],
            dut.checkcnt0[2],
            dut.checkcnt0[3],
            dut.checkcnt0[4],
            (dut.checkcnt0[4]+dut.checkcnt0[1]+dut.checkcnt0[2]+dut.checkcnt0[3])-
            (dut.checkcnt1[4]+dut.checkcnt1[1]+dut.checkcnt1[2]+dut.checkcnt1[3])
        ) ;

    `ifndef NOFSDB
    initial begin 
        #(10 `MS); `maintestend
    `endif 

    `maintest( cgucoretb, cgucoretb )
        #( 1`US ); #100 resetn = 1;

        #( 100 `US ); clksyssel = 1; clktopsel = 0; #( 1 `US );@(negedge clksys);clksrcselupdate=1;@(negedge clksys);clksrcselupdate=0;
        #( 100 `US ); clksyssel = 1; clktopsel = 1; #( 1 `US );@(negedge clksys);clksrcselupdate=1;@(negedge clksys);clksrcselupdate=0;
        #( 100 `US ); clksyssel = 1; clktopsel = 0; #( 1 `US );@(negedge clksys);clksrcselupdate=1;@(negedge clksys);clksrcselupdate=0;
        #( 100 `US ); clksyssel = 0; clktopsel = 0; #( 1 `US );@(negedge clksys);clksrcselupdate=1;@(negedge clksys);clksrcselupdate=0;

        #( 100 `US ); 


    `ifdef NOFSDB
        for( i = 0; i < 1000; i=0)begin
    `else 
        for( i = 0; i < 1000; i++)begin
    `endif
            fd[0] = $urandom();
            fd[1] = $urandom();
            fd[2] = $urandom();
            fd[3] = $urandom();
            fd[4] = $urandom();

            #( 1 `US );@(negedge clktop );fdload=1;@(negedge clktop);fdload=0;
        #( 100 `US ); 
        end
        #( 100 `US ); 

    `maintestend

endmodule



`endif

