`include "template.sv"

package cms_pkg;

// cms: chip mode selection
// ==

  typedef enum logic [7:0] {
    CMS_NONE     = 8'h00   ,
    CMS_VRGN     = 8'h37   ,
    CMS_ATPG     = 8'h5B   ,
    CMS_TEST     = 8'hA6   ,
    CMS_USER     = 8'hFB   ,
    CMS_SCDE     = 8'hFF
  } cmscode_e;

  localparam CMSDW = 128;

  typedef enum logic [CMSDW-1:0] {
    CMSDAT_VRGNMODE     = '0                                 ,
    CMSDAT_TESTMODE     = 'hb7ef9053873f40374a0b57c06027b3b6 ,
    CMSDAT_USERMODE     = 'h3ffc481305e23f3da814ccc35a1f70e2
  } cmsdata_e;

endpackage

import cms_pkg::*;


module cms (
    input logic clk,    // Clock
    input logic resetn, // Clock Enable
    input logic chipresetn,

    input logic [0:2]       cmspad,
    input cms_pkg::cmsdata_e         cmsdata,
//    input cmsdata_e         cmsdata,
    input logic             cmsdatavld,

    output logic            cmsatpg,
    output logic            cmstest,
    output logic            cmsuser,
    output logic            cmsvrgn,
    output logic            cmsscde,

    output logic            cmserror,
    output logic            cmsdone,
    output cms_pkg::cmscode_e        cmscode
);

// cmspad sample and check

    localparam CMSPADCYC = 128;

    bit [1:0][0:2]  cmspadregs;
    bit             cmspadlock;
    bit [7:0]       cmspadcnt;
    bit             cmspaderror;
    bit [2:0]       cmspadout;


    `theregrn( cmspadregs ) <= { cmspadregs, cmspad };
    `theregrn( cmspadlock ) <= ( cmspadcnt == CMSPADCYC );
    `theregrn( cmspadcnt )  <= ( cmspadcnt == CMSPADCYC ) ? cmspadcnt : cmspadcnt+1;

    `theregrn( cmspaderror ) <= ( cmspadregs[1] != cmspadregs[0] ) & cmspadlock ? 1'b1 : cmspaderror;

    assign cmspadout = cmspadregs[1];

// cmsdata pattern

    bit         cmsdataregvld;
    cms_pkg::cmsdata_e   cmsdatareg;

    `theregfull(clk, resetn, cmsdatareg, cms_pkg::CMSDAT_VRGNMODE) <= cmsdatavld ? cmsdata : cmsdatareg;
    `theregrn( cmsdataregvld ) <= cmsdatavld ? 1'b1 :    cmsdataregvld;

// cms
    logic [5:0] cmsfsm;

    `theregfull( clk, resetn, cmsfsm, '0 ) <= ( cmsfsm != '1 ) & cmsdataregvld & cmspadlock ? cmsfsm + 1 : cmsfsm;
    `theregfull( clk, resetn, cmsdone, '0 ) <= ( cmsfsm == '1 );

    cms_pkg::cmscode_e   cmscodepre;


    always@(*)
    casex(cmspadout)
        3'bxx1: cmscodepre = ( cmsdatareg == cms_pkg::CMSDAT_USERMODE ) ? cms_pkg::CMS_USER :
                                                                          cms_pkg::CMS_TEST;
        3'b0x0: cmscodepre = cms_pkg::CMS_USER;
        3'b100: cmscodepre = ( cmsdatareg == cms_pkg::CMSDAT_USERMODE ) ? cms_pkg::CMS_USER :
                                                                          cms_pkg::CMS_ATPG ;
        3'b110: cmscodepre = ( cmsdatareg == cms_pkg::CMSDAT_VRGNMODE ) ? cms_pkg::CMS_VRGN :
                             ( cmsdatareg == cms_pkg::CMSDAT_TESTMODE ) ? cms_pkg::CMS_TEST :
                             ( cmsdatareg == cms_pkg::CMSDAT_USERMODE ) ? cms_pkg::CMS_USER :
                                                                          cms_pkg::CMS_SCDE ;
        default: cmscodepre = cms_pkg::CMS_NONE;
    endcase


`ifdef FPGA
    `theregfull( clk, resetn, cmscode, cms_pkg::CMS_NONE ) <= cms_pkg::CMS_USER ;
    `theregfull(clk, chipresetn, cmsatpg, 1'b0) <= '0;
    `theregrn( cmsvrgn ) <= '0;
    `theregrn( cmstest ) <= '0;
    `theregrn( cmsuser ) <= '1;
    `theregrn( cmsscde ) <= '0;
`else
    `ifdef MPW
        `theregfull(clk, chipresetn, cmsatpg, 1'b0) <= '0;
        `theregrn( cmstest ) <=   cmscode == cms_pkg::CMS_TEST | cmscode == cms_pkg::CMS_VRGN;
    `else
        `theregfull(clk, chipresetn, cmsatpg, 1'b0) <= ( cmscode == cms_pkg::CMS_ATPG ) | cmsatpg;
        `theregrn( cmstest ) <=   cmscode == cms_pkg::CMS_TEST;
    `endif
    `theregfull( clk, resetn, cmscode, cms_pkg::CMS_NONE ) <= cmsdataregvld & cmspadlock ? cmscodepre : cmscode;
    `theregrn( cmsvrgn ) <=   cmscode == cms_pkg::CMS_VRGN;
    `theregrn( cmsuser ) <=   cmscode == cms_pkg::CMS_USER;
    `theregrn( cmsscde ) <= ( cmscode == cms_pkg::CMS_SCDE ) | cmsscde;
`endif


`ifdef SIM

always@(posedge cmsdone)begin
         if( cmsdatareg == cms_pkg::CMSDAT_USERMODE ) $display("::::::::cmscode::::::::::USERMODE ________________",);
    else if( cmsdatareg == cms_pkg::CMSDAT_VRGNMODE ) $display("::::::::cmscode::::::::::VRGNMODE ________________",);
    else if( cmsdatareg == cms_pkg::CMSDAT_TESTMODE ) $display("::::::::cmscode::::::::::TESTMODE ________________",);
    else                                              $display("::::::::cmscode::::::::::SCDEMODE ________________%032x", cmsdatareg);
    $display("::::::::cmspads::::::::::%01x, %01x, %01x ", cmspadout[0], cmspadout[1], cmspadout[2]);
    if( cmscode == cms_pkg::CMS_NONE )$display("::::::::  CMS  :::::::::: CMS_NONE (%02x)", cmscode);
    if( cmscode == cms_pkg::CMS_VRGN )$display("::::::::  CMS  :::::::::: CMS_VRGN (%02x)", cmscode);
    if( cmscode == cms_pkg::CMS_ATPG )$display("::::::::  CMS  :::::::::: CMS_ATPG (%02x)", cmscode);
    if( cmscode == cms_pkg::CMS_TEST )$display("::::::::  CMS  :::::::::: CMS_TEST (%02x)", cmscode);
    if( cmscode == cms_pkg::CMS_USER )$display("::::::::  CMS  :::::::::: CMS_USER (%02x)", cmscode);
    if( cmscode == cms_pkg::CMS_SCDE )$display("::::::::  CMS  :::::::::: CMS_SCDE (%02x)", cmscode);

end


`endif


/*
    logic cmsatpgreg;

    initial begin
        // for sim only
        cmsatpgreg = '0;
        #1; cmsatpgreg = '0;
    end

    assign #0.1 cmsatpg =  cmsatpgreg;
*/

// error

    assign cmserror = cmspaderror;

endmodule: cms
