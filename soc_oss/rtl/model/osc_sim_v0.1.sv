`include "template.sv"

//`timescale 


module OSC_32K (
    input logic EN,
    input logic [6:0] CFG,    
    output bit CKO
);

  `ifdef SIM
        OSC_SIM #(.PERIOD(31250),.CW(7))   u ( .EN(EN), .CFG(CFG),      .CKO( CKO  ) );
  `endif

  `ifdef SYN
        OSC32K_TOP u(.EN(EN), .OSC32K_TM(CFG), .OSC32K_OUT(CKO));
  `endif

endmodule


module OSC_32M (
    input logic EN,
    input logic [6:0] CFG,    
    output bit CKO
);

  `ifdef SIM
        OSC_SIM #(.PERIOD(31.25),.CW(7))   u ( .EN(EN), .CFG(CFG),      .CKO( CKO  ) );
  `endif

  `ifdef SYN
        OSC32M_TOP u(.EN(EN), .OSC32M_TM(CFG), .OSC32M_OUT(CKO));
  `endif

endmodule


`ifdef SIM
module OSC_SIM #(parameter shortreal PERIOD=1, parameter CW=7)(

    input logic EN,
    input logic [CW-1:0] CFG,    

    output bit CKO
);

    initial forever begin CKO = ~CKO & EN ; #(PERIOD/2) ; end
    initial CKO = 0;
endmodule
`endif
