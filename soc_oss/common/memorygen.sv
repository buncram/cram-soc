##!/bin/bash

MODULENAME=$MODULENAME
AW=$AW
DW=$DW

cat  << EOF
/*
    MODULENAME  = $MODULENAME
    AW          = $AW
    DW          = $DW

*/

module $MODULENAME #(
    AW = $AW,
    DW = $DW
)
(
    input                       clk            ,
    input                       ramen          ,
    input                       ramcs          ,
    input           [AW-1:0]    ramaddr        ,
    input           [DW/8-1:0]  ramwr          ,
    input           [DW-1:0]    ramwdata       ,
    output  logic   [DW-1:0]    ramrdata       
);

    localparam HEIGHT = AW**2;
    localparam DWW = \$clog2(DW);
    localparam WRW = DW/8;

\`ifdef SIM
    bit [0:HEIGHT-1][DW-1:0]  ramdatareg;
    bit [DW-1:0][DWW-3-1:0][7:0]rdatareg;
    logic                     oprd,opwr;

genvar i;
generate
	for( i = 0; i < WRW; i = i + 1) begin: GenRnd
    always@(posedge clk) if(opwr) ramdatareg[ramaddr][i] <= ramwr[i] ? ramwdata[(i+1)*8-1:i*8] : ramdatareg[ramaddr][i];
end
endgenerate

    always@(posedge clk) if(oprd) rdatareg <= oprd ? ramdatareg[ramaddr] : 0;

    assign oprd = ramen * ramcs *  (ramwr==0);
    assign opwr = ramen * ramcs * ~(ramwr==0);

    assign ramrdata = ramen ? rdatareg : {DW{1'bz}};

\`endif

\`ifdef FPGA

// bram 36Kb, simple dual port RAM, v14.7
// Ref UG768(v14.7). 
    localparam BRAMAWPRIM = 15; // 36K(32k) x 1b
    localparam BRAMAW = BRAMAWPRIM - DWW;
    logic [0:fnbramcnt()-1][WRW-1:0] br_ramwr;
    logic [0:fnbramcnt()-1][DW-1:0]  br_ramrdata;
    logic [0:fnbramcnt()-1]          br_ramsel,br_ramselreg;

    function int fnbramcnt();
        int fntmp;
        if( AW > BRAMAW ) fntmp = 2**(AW-BRAMAW);
        else fntmp = 1;
        fnbramcnt = fntmp;
    endfunction

    always@(clk) br_ramselreg <= br_ramsel;

    function bit [DW-1:0] fnbramrdata(input logic [0:fnbramcnt()-1] fnbr_ramselreg, input logic [0:fnbramcnt()-1][DW-1:0]  fnbr_ramrdata);
        bit [DW-1:0] fntmp;
        int fni;
        for(fni = 0; fni < fnbramcnt(); fni = fni + 1 ) fntmp = fntmp | ( fnbr_ramrdata[fni] & {DW{fnbr_ramselreg[fni]}} );
        fnbramrdata = fntmp;
    endfunction

    assign ramrdata = fnbramrdata( br_ramselreg, br_ramrdata );

genvar i;
generate
	for( i = 0; i < fnbramcnt(); i = i + 1) begin: GenRnd

BRAM_SDP_MACRO #( 
    .BRAM_SIZE("36Kb"), // Target BRAM, "18Kb" or "36Kb" 
    .DEVICE("7SERIES"), // Target device: "VIRTEX5", "VIRTEX6", "SPARTAN6", "7SERIES" 
    .WRITE_WIDTH(DW), // Valid values are 1-72 (37-72 only valid when BRAM_SIZE="36Kb") 
    .READ_WIDTH(DW), // Valid values are 1-72 (37-72 only valid when BRAM_SIZE="36Kb") 
    .DO_REG(0), // Optional output register (0 or 1) 
    .INIT_FILE ("NONE"), 
    .SIM_COLLISION_CHECK ("NONE"), // Collision check enable "ALL", "WARNING_ONLY", // "GENERATE_X_ONLY" or "NONE" 
    .SRVAL(0), // Set/Reset value for port output 
    .INIT(0), // Initial values on output port 
    .WRITE_MODE("WRITE_FIRST") // Specify "READ_FIRST" for same clock or synchronous clocks // Specify "WRITE_FIRST for asynchronous clocks on ports 
)  ubram_macro36kb ( 
    .DO             (br_ramrdata[i]), // Output read data port, width defined by READ_WIDTH parameter 
    .DI             (ramwdata), // Input write data port, width defined by WRITE_WIDTH parameter 
    .RDADDR         (ramaddr), // Input read address, width defined by read port depth 
    .RDCLK          (clk), // 1-bit input read clock 
    .RDEN           (1'b1), // 1-bit input read port enable 
    .REGCE          (1'b0), // 1-bit input read output register enable 
    .RST            (1'b1), // 1-bit input reset 
    .WE             (br_ramwr[i]), // Input write enable, width defined by write port depth 
    .WRADDR         (ramaddr), // Input write address, width defined by write port depth 
    .WRCLK          (clk), // 1-bit input write clock 
    .WREN           (1'b1) // 1-bit input write port enable
 ); 

    assign br_ramsel[i] = (( ramaddr >> BRAMAW ) == i );
    assign br_ramwr[i] = br_ramsel[i] ? ramwr : 0;
	end
endgenerate

\`endif

\`ifdef ASIC
\`endif

endmodule
EOF