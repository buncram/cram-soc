
`define RERAM_IF_DEF_SIGNAL(theinput, theoutput) \
    ``theinput``  logic [1:0]            utmi_clk,              \
    ``theoutput`` logic [1:0]            u2p0_external_rst,     \
    ``theoutput`` logic [1:0]  [1:0]     utmi_xcvrselect,       \
    ``theoutput`` logic [1:0]            utmi_termselect,       \
    ``theoutput`` logic [1:0]            utmi_suspendm,         \
    ``theinput``  logic [1:0]  [1:0]     utmi_linestate,        \
    ``theoutput`` logic [1:0]  [1:0]     utmi_opmode,           \
    ``theoutput`` logic [1:0]  [7:0]     utmi_datain7_0,        \
    ``theoutput`` logic [1:0]            utmi_txvalid,          \
    ``theinput``  logic [1:0]            utmi_txready,          \
    ``theinput``  logic [1:0]  [7:0]     utmi_dataout7_0,       \
    ``theinput``  logic [1:0]            utmi_rxvalid,          \
    ``theinput``  logic [1:0]            utmi_rxactive,         \
    ``theinput``  logic [1:0]            utmi_rxerror,          \
    ``theoutput`` logic [1:0]            utmi_dppulldown,       \
    ``theoutput`` logic [1:0]            utmi_dmpulldown,       \
    ``theinput``  logic [1:0]            utmi_hostdisconnect   

`define RERAM_IF_DEF `RERAM_IF_DEF_SIGNAL(input, output)
`define RRC_IF_DEF   `RERAM_IF_DEF_SIGNAL(output, input)


`define RERAM_IF_INST( theindex ) \
    .utmi_clk            ( utmi_clk``[theindex]``            ),\
    .u2p0_external_rst   ( u2p0_external_rst``[theindex]``   ),\
    .utmi_xcvrselect     ( utmi_xcvrselect``[theindex]``     ),\
    .utmi_termselect     ( utmi_termselect``[theindex]``     ),\
    .utmi_suspendm       ( utmi_suspendm``[theindex]``       ),\
    .utmi_linestate      ( utmi_linestate``[theindex]``      ),\
    .utmi_opmode         ( utmi_opmode``[theindex]``         ),\
    .utmi_datain7_0      ( utmi_datain7_0``[theindex]``      ),\
    .utmi_txvalid        ( utmi_txvalid``[theindex]``        ),\
    .utmi_txready        ( utmi_txready``[theindex]``        ),\
    .utmi_dataout7_0     ( utmi_dataout7_0``[theindex]``     ),\
    .utmi_rxvalid        ( utmi_rxvalid``[theindex]``        ),\
    .utmi_rxactive       ( utmi_rxactive``[theindex]``       ),\
    .utmi_rxerror        ( utmi_rxerror``[theindex]``        ),\
    .utmi_dppulldown     ( utmi_dppulldown``[theindex]``     ),\
    .utmi_dmpulldown     ( utmi_dmpulldown``[theindex]``     ),\
    .utmi_hostdisconnect ( utmi_hostdisconnect``[theindex]`` )


module rrc(`RRC_IF_DEF);
endmodule

module coretop1(`RRC_IF_DEF);
    rrc rrc(`RERAM_IF_INST(1:0));
endmodule

module coretop2(`RRC_IF_DEF);
    coretop1 coretop1(`RERAM_IF_INST(1:0));
endmodule

module top();
    logic [1:0]            utmi_clk;
    logic [1:0]            u2p0_external_rst;
    logic [1:0]  [1:0]     utmi_xcvrselect;
    logic [1:0]            utmi_termselect;
    logic [1:0]            utmi_suspendm;
    logic [1:0]  [1:0]     utmi_linestate;
    logic [1:0]  [1:0]     utmi_opmode;
    logic [1:0]  [7:0]     utmi_datain7_0;
    logic [1:0]            utmi_txvalid;
    logic [1:0]            utmi_txready;
    logic [1:0]  [7:0]     utmi_dataout7_0;
    logic [1:0]            utmi_rxvalid;
    logic [1:0]            utmi_rxactive;
    logic [1:0]            utmi_rxerror;
    logic [1:0]            utmi_dppulldown;
    logic [1:0]            utmi_dmpulldown;
    logic [1:0]            utmi_hostdisconnect;
    generate
        for (genvar i = 0; i < 2; i++) begin:r
            rerammacro r(`RERAM_IF_INST(i));
        end
    endgenerate
        coretop2 coretop1(`RERAM_IF_INST(1:0));

endmodule


// from tsmc
module rerammacro(
    input  logic           utmi_clk,            
    output logic           u2p0_external_rst,   
    output logic [1:0]     utmi_xcvrselect,     
    output logic           utmi_termselect,     
    output logic           utmi_suspendm,       
    input  logic [1:0]     utmi_linestate,      
    output logic [1:0]     utmi_opmode,         
    output logic [7:0]     utmi_datain7_0,      
    output logic           utmi_txvalid,        
    input  logic           utmi_txready,        
    input  logic [7:0]     utmi_dataout7_0,     
    input  logic           utmi_rxvalid,        
    input  logic           utmi_rxactive,       
    input  logic           utmi_rxerror,        
    output logic           utmi_dppulldown,     
    output logic           utmi_dmpulldown,     
    input  logic           utmi_hostdisconnect  
    );
endmodule
