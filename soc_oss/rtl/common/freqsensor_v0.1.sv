module freqsensor(
    input   bit     clk,
    input   bit     resetn,
    input   bit     clkdut,
    input   bit     freqsensoren,
    output  bit [15:0]  freqvalue
);

parameter clkfreqkvalue = 11520;
bit freq1mtrig, freqvalueupdate, freq1mtog, dutfreqtrig, freq1mtogs0, freq1mtogs1;
bit [15:0] freq1mcnt, freqvalue, freqvaluehs, dutfreqcnt;

assign freq1mtrig = ( freq1mcnt == clkfreqkvalue );
assign freqvalueupdate = ( freq1mcnt == ( clkfreqkvalue / 2 ) ) ;
always@(posedge clk)  freq1mtog <= freq1mtrig ? ~ freq1mtog : freq1mtog;
always@(posedge clk)  freq1mcnt <= freq1mtrig ? 0 : freq1mcnt + 1;
always@(posedge clk)  freqvalue <= freqvalueupdate ? freqvaluehs : freqvalue;

always@(posedge clkdut) dutfreqcnt <= dutfreqtrig ? 0 : dutfreqcnt + 1;
always@(posedge clkdut) dutfreqtrig <= freq1mtogs0 ^ freq1mtogs1;
always@(posedge clkdut) freq1mtogs0 <= freq1mtog;
always@(posedge clkdut) freq1mtogs1 <= freq1mtogs0;
always@(posedge clkdut) freqvaluehs <= dutfreqtrig ? dutfreqcnt : freqvaluehs;


endmodule
