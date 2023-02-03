`timescale 1ns/1ps

module uart_print #(
    parameter TYPE="generic"
)(
    input logic [7:0] uart_data,
    input logic uart_data_valid,
    input logic resetn,
    input logic clk
);
    // print debug strings
    `define theregfull( theclk, theresetn, theregname, theinitvalue ) \
        always@( posedge theclk or negedge theresetn ) \
        if( ~theresetn) \
            theregname <= theinitvalue; \
        else \
            theregname

    `define theregrn(theregname) \
        `theregfull( clk, resetn, theregname, '0 )

    localparam CHARLEN = 256;
    logic                       charbufwr, charbuffill, charbufclr;
    bit [$clog2(CHARLEN)-1:0]   charbufidx;
    bit [CHARLEN-1:0][7:0]      charbufdat;
    string charbufstring;
    assign charbufwr = uart_data_valid;
    assign charbuffill = charbufwr & ~(( uart_data[7:0] == 'h0d ) | ( uart_data[7:0] == 'h0a ));
    assign charbufclr  = charbufwr &  (( uart_data[7:0] == 'h0d ) | ( uart_data[7:0] == 'h0a ));

    `theregrn( charbufidx ) <= charbufclr ? (CHARLEN-1) : charbuffill ? charbufidx - 1 : charbufidx;
    `theregrn( charbufdat[charbufidx] ) <= charbuffill ? uart_data[7:0] : charbufdat[charbufidx];

    always@( negedge clk )
    if( uart_data_valid )  begin
        if( charbufclr ) begin
            charbufstring = string'(charbufdat);
            $display("[%s] %s", TYPE, charbufstring );
            charbufdat = '0;
        end
    end
endmodule