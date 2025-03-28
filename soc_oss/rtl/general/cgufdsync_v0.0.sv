`include "template.sv"

module cgufdsync

// 1, fdsync makes clk2 is fixed freq-div to clk0. 
// 2, clk2en are sync with clk1en.
// 3, when clk2fd is faster than clk1fd. the actual clk2en equal clk1en.
// 4, clk0 is clk gated by clk0en.

// output clk2en: is used for clk2 icg@clk
// output clk2en_atclk1: is used for clk2 domain for essential reg-ce, e.g., 'penable'.

#(
    parameter FDW = 8
)(
    input   bit             clk,
    input   bit             resetn,
    input   bit             clk0en,
    input   bit             clk1en,
    input   bit [FDW-1:0]   fd2,
    output  bit             clk2en,
    output  bit             clk2en_atclk1
);

    bit [FDW-1:0]   fd2cnt, fd2reg;
    bit             fd2hit, fd2hitreg;
    bit             clk2en0;



    `theregrn( fd2cnt ) <= clk0en ? ( fd2hit ? 0 : fd2cnt + 1 ) : fd2cnt;
    `theregrn( fd2reg ) <= clk0en ? ( fd2hit ? fd2 : fd2reg ) : fd2reg;
    assign fd2hit = ( fd2cnt == fd2reg );
    
    `theregrn( fd2hitreg ) <= clk0en ? ( fd2hit ? 1'b1 : clk2en0 ? 1'b0 : fd2hitreg ) : fd2hitreg;
    
    assign clk2en0 = fd2hitreg & clk1en;
    
    `theregrn( clk2en_atclk1 ) <= clk0en ? ( clk2en0 ? 1'b1 : clk1en ? 1'b0 : clk2en_atclk1 ) : clk2en_atclk1;
    
    assign clk2en = clk2en_atclk1 & clk1en & clk0en;

endmodule

`ifdef SIMcgufdsynctb
`include "icg.v"
module cgufdsynctb();
    bit         clk,clk1,clk2,resetn,dut1clken,dut2clken;
    bit [7:0]   dut1fd,dut2fd;
    integer     clkcnt=0, dut1clkcnt=0, dut2clkcnt=0, dut1mincycle=0, dut2mincycle=0;
    bit clk0en;
    assign clk0en = 1;

    cgufdsync dut1(
        .clk1en  (1'b1),
        .fd2     (dut1fd),
        .clk2en  (dut1clken),
        .*
    );
    
    cgufdsync dut2(
        .clk1en  (dut1clken),
        .fd2     (dut2fd),
        .clk2en  (dut2clken),
        .*
    );
    
    ICG u1 ( .CK (clk), .EN ( dut1clken ), .CKG ( clk1 ));
    ICG u2 ( .CK (clk), .EN ( dut2clken ), .CKG ( clk2 ));
    
    `timemarker
    `genclk( clk, 10 );
    `maintest( thetestbasic, cgufdsynctb )
        resetn = 0;
        #( 2 `US );
        resetn = 1;
        
        #( 1 `US );
        
        for( dut1fd = 0; dut1fd < 8; dut1fd = dut1fd + 1 ) begin
            for( dut2fd = 4; dut2fd < 32; dut2fd = dut2fd + 1 ) begin
                #( 100 `US );
                $display("@i: fd1:%x(%d), fd2:%x(%d), clkcnt=%d, dut1clkcnt=%d,%d dut2clkcnt=%d,%d .", dut1fd, clkcnt/(dut1fd+1), dut2fd, clkcnt/(dut2fd+1), clkcnt, dut1clkcnt, dut1clkcnt-clkcnt/(dut1fd+1), dut2clkcnt, clkcnt/(dut2fd+1)+1-dut2clkcnt);
                clkcnt = 0; dut1clkcnt = 0; dut2clkcnt = 0;
                dut1mincycle = 1000;
                dut2mincycle = 1000;
            end
        end
    
        #( 10 `US );
    `maintestend
    
    `thereg( clkcnt ) <= clkcnt + 1;
    `thereg( dut1clkcnt ) <= dut1clkcnt + dut1clken;
    `thereg( dut2clkcnt ) <= dut2clkcnt + dut2clken;

endmodule
`endif
