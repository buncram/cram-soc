module uarttx (
    input   logic   clk,
    input   logic   resetn,
    input   logic   sclk,
    input   logic   [7:0]   etu,
    input   logic   txwr,
    input   logic   [31:0]  txword,
    output  logic   tx
    );
        parameter int fifosize = 16;
        reg      [7:0]   txfipt,txfopt;
        wire     [7:0]   txfiptpre,txfoptpre,txfiptnext;
        wire            txfifoempty;
        wire            txfiwr;
        wire    [31:0]  txfidata;
        wire            txford;
        reg     [31:0]  txfodata;
        wire    [31:0]  txfodatapre,txfiforamdout;
        wire            txwordstart;
        reg     [3:0]   txwordfsm;
        wire    [3:0]   txwordfsmnext;
        reg             txresetn;
        wire            txresetnpre;
        wire    [7:0]   txdata;
        reg             txstart;
        wire            txstartpre;
        reg     [3:0]   txfsm;
        wire    [3:0]   txfsmnext;
        wire            txdone,txbusy;
        reg     [11:0]  txstreamdata=12'hfff;
        wire            txstreambit;
        reg             txetu0,txetu1,txetu2,txetu;
        reg     [7:0]   txetucnt_sclk;
        wire    [7:0]   txetucnt_sclkpre;
        reg             txetu_sclk;
        wire            txetu_sclkpre;

  //
  // fifo
  // ====

        always@(posedge clk or negedge resetn)
        if(!resetn)
            txfipt <= 0;
        else
            txfipt <= txfiptpre;

        assign txfiptpre = txfiwr ? txfiptnext : txfipt;
        assign txfiptnext = ((txfipt==(fifosize-1))? 0 : txfipt+1) ;

        always@(posedge clk or negedge resetn)
        if(!resetn)
            txfopt <= 0;
        else
            txfopt <= txfoptpre;

        assign txfoptpre = txford ? ((txfopt==(fifosize-1))? 0: txfopt+1) : txfopt;

        assign txfifoempty = ( txfipt == txfopt );
        assign txfifofull = ( txfiptnext == txfopt );

`ifdef SIM
	always@(posedge txfifofull)
	$display("!---fifo: fifofull!");
`endif


        assign txfiwr = txwr;
        assign txfidata = txword;

        assign txford = ( txwordfsm == 4'h2 );

        always@(posedge clk or negedge resetn)
        if(!resetn)
            txfodata <= 32'h0;
        else
            txfodata <= txfodatapre;

        assign txfodatapre = ( txwordfsm == 4'h3 ) ? txfiforamdout : txfodata;

        assign txwordstart = ( txwordfsm == 4'h0 ) & ~txfifoempty;

  //
  // DPRAM
  // ====

        FIFODPRAM32 
        `ifndef FPGA
        #(.fifosize(fifosize)) 
        `endif
        txfifodpram(
            .clka(clk),
            .wea(txfiwr),
            .addra(txfipt),
            .dina(txfidata),
            .clkb(clk),
            .addrb(txfopt),
            .doutb(txfiforamdout)
        );

  //
  // txword send
  // ====

        always@(posedge clk or negedge resetn)
        if(!resetn)
                txwordfsm <= 4'h0;
        else
                txwordfsm <= txwordfsmnext;

        assign txwordfsmnext =  txwordstart ? 4'h1 :
                                ( txwordfsm == 4'h1 ) ? 4'h2 :
                                ( txwordfsm == 4'h2 ) ? 4'h3 :
                                ( txwordfsm == 4'h3 ) ? 4'h4 :
                                ( txwordfsm == 4'h7 ) & txdone ? 4'h0 :
                                txwordfsm[2] & txdone ? txwordfsm + 1 : txwordfsm;

        always@(posedge clk or negedge resetn)
        if(!resetn)
                txresetn <= 1'b0;
        else
                txresetn <= txresetnpre;

        assign txresetnpre = ( txwordfsm == 4'h1 ) ? 1'b1 : (( txwordfsm == 4'h7 ) & txdone ) ? 1'b0 : txresetn;

        assign txdata = ( txwordfsm[1:0] == 2'h0 ) ? txfodata[31:24]:
                        ( txwordfsm[1:0] == 2'h1 ) ? txfodata[23:16]:
                        ( txwordfsm[1:0] == 2'h2 ) ? txfodata[15: 8]:
                                                     txfodata[ 7: 0];


        always@(posedge clk or negedge resetn)
        if(!resetn)
                txstart <= 1'b0;
        else
                txstart <= txstartpre;

        assign txstartpre = ( txwordfsm == 4'h3 ) | ( txdone & (( txwordfsm == 4'h4 ) | ( txwordfsm == 4'h5 ) | ( txwordfsm == 4'h6 )));



  //
  // tx byte send
  // ====


        always@(posedge clk or negedge txresetn)
        if(!txresetn)
                txfsm <= 4'h0;
        else
                txfsm <= txfsmnext;

        assign txfsmnext = txdone ? 4'h0 : txetu ? ( txfsm + 4'h1 ) : txfsm;

        assign txdone = ( txfsm == 4'h9 ) & txetu;
        assign txbusy = ~( txfsm == 4'h0 );

        always@(posedge clk or negedge txresetn) 
        if(!txresetn)
                txstreamdata <= 12'hfff;
            else
                txstreamdata <= txstart ? { 1'b1, 1'b1, txdata, 1'b0,1'b1} : txetu ? { 1'b1, txstreamdata[11:1] }: txstreamdata;

        assign txstreambit = txstreamdata[0];

        
        // txetu

        always@(posedge clk or negedge txresetn)
        if(!txresetn)
        begin
                txetu0 <= 1'b0;
                txetu1 <= 1'b0;
                txetu2 <= 1'b0;
                txetu  <= 1'b0;
                tx     <= 1'b1;
        end
        else
        begin
                txetu0 <= txetu_sclk;
                txetu1 <= txetu0;
                txetu2 <= txetu1;
                txetu  <= txetu1 & ~txetu2 ;
                tx     <= txstreambit;
        end

        always@(posedge sclk or negedge txresetn)
        if(!txresetn)
                txetucnt_sclk <= 0;
        else
                txetucnt_sclk <= txetucnt_sclkpre;

        always@(posedge sclk or negedge txresetn)
        if(!txresetn)
                txetu_sclk <= 0;
        else
                txetu_sclk <= txetu_sclkpre;

        assign txetucnt_sclkpre = ( txetucnt_sclk == etu ) ? 0 : txetucnt_sclk + 1;
        assign txetu_sclkpre = ( txetucnt_sclk == etu );

endmodule
// -------------------------------------------------------------------------------------------------------------------------------
`ifdef SIM
/*
module FIFODPRAM32 (
            input   logic   clka,
            input   logic   wea,
            input   logic   [7:0]   addra,
            input   logic   [31:0]  dina,
            input   logic           clkb,
            input   logic   [7:0]   addrb,
            output  logic   [31:0]  doutb
        );

        parameter int fifosize = 16;
    logic   [(fifosize-1):0][31:0]  ramdata;

  generate 
    genvar j;
        for( j = 0 ; j < fifosize ; j = j + 1 ) begin: ramdatareg
            always@(posedge clka) ramdata[j] <= wea & ( addra == j ) ? dina : ramdata[j];
        end
  endgenerate

    always@(posedge clka) doutb <= ramdata[addrb];

endmodule
*/
`endif

// -------------------------------------------------------------------------------------------------------------------------------


module itfrxword(
    input   logic   clk, 
    input   logic   resetn,
    input   logic   rxdone,
    input   logic   [7:0] rxdata,
    input   logic   rxto,
    output  logic   [7:0] rxwordno,
    output  logic   rxworddone,
    output  logic   [31:0] rxword
    );
 
    logic   [1:0]   rxdatafsm,rxdatafsmnext;
    logic   [31:0]  rxwordpre;
    logic   [7:0]   rxwordlen;

        always@(posedge clk or negedge resetn)
        if(!resetn)
                rxword <= 32'H0;
        else
                rxword <= rxwordpre;

        assign rxwordpre[31:24] = ( rxdatafsm[1:0] == 2'h0 ) & rxdone ? rxdata : rxword[31:24];
        assign rxwordpre[23:16] = ( rxdatafsm[1:0] == 2'h1 ) & rxdone ? rxdata : rxword[23:16];
        assign rxwordpre[15: 8] = ( rxdatafsm[1:0] == 2'h2 ) & rxdone ? rxdata : rxword[15:8];
        assign rxwordpre[ 7: 0] = ( rxdatafsm[1:0] == 2'h3 ) & rxdone ? rxdata : rxword[7:0];

        always@(posedge clk or negedge resetn)
        if(!resetn)
                rxdatafsm <= 2'h0;
        else
                rxdatafsm <= rxdatafsmnext;

        assign rxdatafsmnext = rxto ? 0 : rxdone ? rxdatafsm + 1 : rxdatafsm;

        always@(posedge clk or negedge resetn)
        if(!resetn)
                rxwordno <=  8'hff ;
        else
                rxwordno <= rxto ?  8'hff : ( rxdatafsm[1:0] == 2'h3 ) & rxdone ? (( rxwordno == rxwordlen ) ? 8'hff : rxwordno + 1 ): rxwordno;

        always@(posedge clk or negedge resetn)
        if(!resetn)
                rxwordlen <=  8'hff ;
        else
                rxwordlen <= rxto ?  8'hff : ( rxdatafsm[1:0] == 2'h3 ) & rxdone & ( rxwordno == rxwordlen ) ? rxword[15:8] : rxwordlen;

        always@(posedge clk or negedge resetn)
        if(!resetn)
                rxworddone <=  0 ;
        else
                rxworddone <= ( rxdatafsm[1:0] == 2'h3 ) & rxdone;

endmodule

module itfrxbyte(

        clk             ,
        resetn          ,

        sclk            ,
        etu             ,
        rx              ,
        rxdone          ,
        rxdata          ,
        rxto
        );

        input           clk             ;
        input           resetn          ;
        input           sclk            ;
        input   [7:0]   etu             ;
        input           rx              ;
        output          rxdone          ;
        output  [7:0]   rxdata       ;
        output          rxto          ;


        wire           clk             ;
        wire           resetn          ;
        wire           sclk            ;
        wire    [7:0]   etu             ;
        reg            rxto          ;
        wire           rx              ;
        reg             rx0,rx1,rx2,rx3,rxnegedge;
        wire            rxstreambit;
        logic           rxetu0,rxetu1,rxetu2,rxetu,rxhalfetu0,rxhalfetu1,rxhalfetu2;
        reg     [7:0]   rxetucnt_sclk;
        wire    [7:0]   rxetucnt_sclkpre;
        reg             rxetu_sclk,rxhalfetu_sclk;
        wire            rxetu_sclkpre;
        wire    [7:0]   rxetupm;
        wire            rxhalfetu_sclkpre;
        reg     [3:0]   rxfsm;
        wire    [3:0]   rxfsmnext;
        wire            rxdone;
        wire            rxsample;
        reg     [7:0]   rxdata=0;
        wire    [7:0]   rxdatapre;
        reg     [15:0]   rxtocnt;
        wire    [15:0]   rxtocntpre;
        logic            rxhalfetu;

        always@(posedge clk)
        begin
                rx0 <= rx;
                rx1 <= rx0;
                rx2 <= rx1;
                rx3 <= rx2;
                rxnegedge <= ~rx0 & rx1;
        end

        assign rxstreambit = rx3;

    //  rxetu

        always@(posedge clk or negedge resetn)
        if(!resetn)
        begin
                rxetu0 <= 1'b0;
                rxetu1 <= 1'b0;
                rxetu2 <= 1'b0;
                rxetu  <= 1'b0;
                rxhalfetu0 <= 1'b0;
                rxhalfetu1 <= 1'b0;
                rxhalfetu2 <= 1'b0;
                //rxhalfetu  <= 1'b0;
                rxto <= 1'b0;
                rxtocnt <= 8'h0;
        end
        else
        begin
                rxetu0 <= rxetu_sclk;
                rxetu1 <= rxetu0;
                rxetu2 <= rxetu1;
                //rxetu  <= rxetu1 & ~rxetu2 ; // ASYNC
                rxetu  <= rxetu_sclk & ~rxetu0 ; //  SYNC
                rxhalfetu0 <= rxhalfetu_sclk;
                rxhalfetu1 <= rxhalfetu0;
                rxhalfetu2 <= rxhalfetu1;
                //rxhalfetu  <= rxhalfetu1 & ~rxhalfetu2 ; // ASYNC
                rxto <= ( rxtocnt[15:0] == { etu[7:0],8'hfe } );
                rxtocnt <= rxtocntpre;
        end
        assign rxhalfetu = rxhalfetu_sclk; // SYNC
        assign rxtocntpre = ~( rxfsm == 4'h0 ) ? 0 : ( rxtocnt[15:0] == { etu[7:0],8'hff } ) ? rxtocnt : rxtocnt + 1;

//        assign rxhalfetu = rxhalfetu_sclk;
//        assign rxetu = rxetu_sclk;

        always@(posedge sclk or negedge resetn)
        if(!resetn)
        begin
                rxetucnt_sclk <= 0;
                rxetu_sclk <= 0;
                rxhalfetu_sclk <= 0;
        end
        else
        begin
                rxetucnt_sclk <= rxetucnt_sclkpre;
                rxetu_sclk <= rxetu_sclkpre;
                rxhalfetu_sclk <= rxhalfetu_sclkpre;
        end

        assign rxetucnt_sclkpre = ( rxfsm == 4'h0 ) | ( rxetucnt_sclk == etu ) ? 0 : rxetucnt_sclk + 1;
        assign rxetu_sclkpre = ( rxetucnt_sclk == etu );
        assign rxetupm = etu;
        assign rxhalfetu_sclkpre = ( rxetucnt_sclk == rxetupm[7:1] );

    //  rxfsm

        always@(posedge clk or negedge resetn)
        if(!resetn)
                rxfsm <= 4'h0;
        else
                rxfsm <= rxfsmnext;

        assign rxfsmnext = ( rxfsm == 4'h0 ) & rxnegedge ? 4'h1 : 
                           ( rxfsm == 4'ha ) & rxhalfetu ? 4'hb :
                           ( rxfsm == 4'hb ) & rxstreambit ? 4'h0 :
                           ~( rxfsm == 4'hb ) & rxetu ? ( rxfsm + 4'h1 ) : rxfsm;

        assign rxdone = ( rxfsm == 4'ha ) & rxhalfetu;
        assign rxsample = rxhalfetu & ~( rxfsm == 4'ha );
        always@(posedge clk)
                rxdata <= rxdatapre;

        assign rxdatapre = rxsample ? { rxstreambit, rxdata[7:1] } : rxdata;

endmodule

module pwm_pulse_fade (clk, trigger, out);
    parameter LEVEL_BITS = 12; // 8-bit ramp is too steppy
    parameter FADE_SHIFT = 11;

    input wire trigger;
    input wire clk;
    output wire out;

    localparam FADE_MSB = 31;
    (* use_dsp = "yes" *) reg [LEVEL_BITS-1:0] pwm_ramp = 0;
    (* use_dsp = "yes" *)  reg [FADE_MSB - 1:0] fade_counter = 0;
    always @(posedge clk) begin
        pwm_ramp <= pwm_ramp + 1;  // ramp generator
        if (1'b1 == trigger) 
            fade_counter <= {(FADE_MSB + 1){1'b1}};
        else 
            // for 12-bit ramp and 12-bit shift, tau = F_clk / (1 << (LEVEL_BITS + FADE_SHIFT))
            // time from full brightness 255 to 0 is 5.5 * tau
            // for F_clk = 9.216MHz, 8 + 15 or similar combination is reasonable 
            if ((|fade_counter) & (0 == pwm_ramp)) // logarithm fade out
                fade_counter <= fade_counter - (fade_counter >> FADE_SHIFT);
    end 

    wire [LEVEL_BITS - 1:0] level;
    assign level = fade_counter[FADE_MSB - 1 -: LEVEL_BITS];
    assign out = (pwm_ramp < level);  // reg this if necessary
endmodule
