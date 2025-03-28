`define alwaysclkrst    always@( posedge clk or negedge resetn ) if( ~resetn) 
`define alwaysclk       always@( posedge clk )

module miner_control #(parameter [3:0][31:0] sfrinit={32'h0, 32'h0, 32'h0, 32'h0})(

    input   logic   clk,
//    input   logic   clkcore,
    input   logic   resetn,
    
    input   logic [3:0] uartrx,
    output  logic [3:0] uarttx,

    output  bit [7:0]   devid,
    output  bit [3:0][31:0]     sfrreg,
//    output  bit [7:0][31:0]     hh,
//    output  bit [15:0][31:0]    mm,

    output bit          setworktog,
    output bit [7:0]    setworkno,
    output bit [31:0]   setworkdata,
    output bit          minestart,
   
    output  logic           mineresetn,
    output  logic           minenonceresetn,
    input   logic           minebingotog,
    input   logic   [63:0]  minegold,
    output  logic           ledout
    );

    logic   [3:0]       rxdone, rxto;
    bit     [3:0][7:0]  rxdata,rxwordno;
    logic   [3:0]       rxworddone,rxworddonereg,rxcmdvld;
    bit     [3:0][3:0][7:0] rxword,rxcmdword;
    bit     [3:0][7:0]  rxcmd,rxcmdid,rxcmdlen,rxcmdext;
    bit     [3:0][6:0]  forceresetcnt;
    logic   [3:0]       forcereset;
    logic   [7:0]       etu;//##
    bit [3:0][3:0][31:0]    rxcmddata;
    bit [15:0][31:0]    mm0;

    assign etu = sfrreg[1][23:16];


  generate 
    genvar i;
        for( i = 0 ; i < 4 ; i = i + 1 ) begin: rx
    itfrxbyte urxbyte( .clk (clk), .resetn (resetn), .sclk (clk), .etu(etu), .rx (uartrx[i]), .rxdone (rxdone[i]), .rxdata (rxdata[i]), .rxto (rxto[i]));
    itfrxword urxword( .clk (clk), .resetn (resetn), .rxdone (rxdone[i]), .rxdata (rxdata[i]), .rxto (rxto[i]), .rxwordno (rxwordno[i]), .rxworddone (rxworddone[i]), .rxword (rxword[i]));
    `alwaysclk rxworddonereg[i] <= rxworddone[i];
    `alwaysclk rxcmdvld[i] <= rxworddonereg[i] & ( rxwordno[i] == rxcmdlen[i] );
    assign rxcmd[i] = rxcmdword[i][3];
    assign rxcmdid[i] = rxcmdword[i][2];
    assign rxcmdlen[i] = rxcmdword[i][1];
    assign rxcmdext[i] = rxcmdword[i][0];
    always@( posedge clk or negedge resetn )
    if( ~resetn) 
        begin
            forceresetcnt[i]<=0;
            forcereset[i]<=0;
        end
    else
        begin
            forceresetcnt[i] <= rxto[i] ? 0 :  rxdone[i] ? ( ( rxdata[i] == 0 ) ? forceresetcnt[i] + 1 : 0 ) : forceresetcnt[i] ;
            forcereset[i] <= ( forceresetcnt[i] == 64 );
        end
    `alwaysclk rxcmdword[i]<= ( rxwordno[i] == 8'hff ) & rxworddone[i] ? rxword[i] : rxcmdword[i];
    `alwaysclk rxcmddata[i][0] <= ( rxwordno[i] == 8'h00 ) & rxworddone[i] ? rxword[i] : rxcmddata[i][0];
    `alwaysclk rxcmddata[i][1] <= ( rxwordno[i] == 8'h01 ) & rxworddone[i] ? rxword[i] : rxcmddata[i][1];
    `alwaysclk rxcmddata[i][2] <= ( rxwordno[i] == 8'h02 ) & rxworddone[i] ? rxword[i] : rxcmddata[i][2];


        end
  endgenerate

  //
  //  cmd
  //  ==

    logic               cmdreset, cmdresetnonce, cmdgetsfr, cmdsetsfr0, cmdsetsfr1, cmdsetwork, cmdsetid;
    bit [7:0]           cmdext, cmdid;
    bit [31:0]          getsfrdata;
    bit [23:0][31:0]    rxcmdwork;
    //bit [7:0]           devid;
    bit [1:0]           cmdidmet;
    parameter   PM_RXCMD_RESET = 8'h5a;
    parameter   PM_RXCMD_RESETNONCE = 8'h5b;
    parameter   PM_RXCMD_SETSFR = 8'ha0;
    parameter   PM_RXCMD_GETSFR = 8'h90;
    parameter   PM_RXCMD_SETWORK = 8'h30;
    parameter   PM_RXCMD_SETID = 8'haa;
    parameter   PM_RXCMD_SUBMIT = 8'h31;

    assign cmdidmet[0] = ( rxcmdid[0] == 8'hff ) | ( rxcmdid[0] == devid );
    assign cmdidmet[1] = ( rxcmdid[1] == 8'hff ) | ( rxcmdid[1] == devid );

// cmd: set/get sfr, setwork, 

    always@( posedge clk or negedge resetn )
    if( ~resetn) 
        begin
            cmdreset<=0;
            cmdresetnonce<=0;
            cmdsetsfr0<=0;
            cmdsetsfr1<=0;
            cmdgetsfr<=0;
            cmdsetwork<=0;
            cmdsetid<=0;
            cmdext <= 0;
            cmdid <= 0;
        end
    else
        begin
            cmdreset <=         (( rxcmd[0] == PM_RXCMD_RESET ) & rxcmdvld[0] & cmdidmet[0] & ( rxcmdext[0] == PM_RXCMD_RESET ) ) |
                                (( rxcmd[1] == PM_RXCMD_RESET ) & rxcmdvld[1] & cmdidmet[1] & ( rxcmdext[1] == PM_RXCMD_RESET ) );
            cmdresetnonce <=    (( rxcmd[0] == PM_RXCMD_RESETNONCE ) & rxcmdvld[0] & cmdidmet[0] & ( rxcmdext[0] == PM_RXCMD_RESETNONCE ) ) |
                                (( rxcmd[1] == PM_RXCMD_RESETNONCE ) & rxcmdvld[1] & cmdidmet[1] & ( rxcmdext[1] == PM_RXCMD_RESETNONCE ) );
            cmdsetsfr0 <=       (( rxcmd[0] == PM_RXCMD_SETSFR  ) & rxcmdvld[0] & cmdidmet[0] ) ;
            cmdsetsfr1 <=       (( rxcmd[1] == PM_RXCMD_SETSFR  ) & rxcmdvld[1] & cmdidmet[1] ) ;
            cmdgetsfr <=        (( rxcmd[0] == PM_RXCMD_GETSFR  ) & rxcmdvld[0] & cmdidmet[0] ) |
                                (( rxcmd[1] == PM_RXCMD_GETSFR  ) & rxcmdvld[1] & cmdidmet[1] ) ;
            cmdsetwork <=       (( rxcmd[0] == PM_RXCMD_SETWORK ) & rxcmdvld[0] & cmdidmet[0] ) ;
            cmdsetid  <=        (( rxcmd[0] == PM_RXCMD_SETID   ) & rxcmdvld[0] ) |
                                (( rxcmd[1] == PM_RXCMD_SETID   ) & rxcmdvld[1] ) ;
            cmdext <=           ( rxcmdvld[0] & cmdidmet[0] ) ? rxcmdext[0] :
                                ( rxcmdvld[1] & cmdidmet[1] ) ? rxcmdext[1] : cmdext;
            cmdid  <=           ( rxcmdvld[0] ) ? rxcmdid[0] :
                                ( rxcmdvld[1] ) ? rxcmdid[1] : cmdid;
        end

    always@( posedge clk or negedge resetn )
    if( ~resetn) 
        begin
            sfrreg <= sfrinit;
        end
    else
        begin
            sfrreg[0] <= ( cmdsetsfr0 & ( rxcmdext[0] == 8'h0 ) ) ? rxcmddata[0][0] :
                         ( cmdsetsfr1 & ( rxcmdext[1] == 8'h0 ) ) ? rxcmddata[1][0] :
                                    sfrreg[0] ;
            sfrreg[1] <= ( cmdsetsfr0 & ( rxcmdext[0] == 8'h1 ) ) ? rxcmddata[0][0] :
                         ( cmdsetsfr1 & ( rxcmdext[1] == 8'h1 ) ) ? rxcmddata[1][0] : 
                                    sfrreg[1] ;
            sfrreg[2] <= ( cmdsetsfr0 & ( rxcmdext[0] == 8'h2 ) ) ? rxcmddata[0][0] :
                         ( cmdsetsfr1 & ( rxcmdext[1] == 8'h2 ) ) ? rxcmddata[1][0] :
                                    sfrreg[2] ;
            sfrreg[3] <= ( cmdsetsfr0 & ( rxcmdext[0] == 8'h3 ) ) ? rxcmddata[0][0] :
                         ( cmdsetsfr1 & ( rxcmdext[1] == 8'h3 ) ) ? rxcmddata[1][0] : 
                                    sfrreg[3] ;
        end

    // getsfr
    assign getsfrdata = ( cmdext == 0 ) ? sfrreg[0] : ( cmdext == 1 ) ? sfrreg[1] : ( cmdext == 2 ) ? sfrreg[2] : sfrreg[3] ;

    // setwork

    assign setworkno= rxwordno[0];
    assign setworkdata = rxword[0];
    `alwaysclk setworktog <= ( rxcmd[0] == PM_RXCMD_SETWORK ) & cmdidmet[0] & rxworddone[0] ? ~setworktog : setworktog;
    `alwaysclk minestart <= cmdsetwork;

    // setid
    `alwaysclkrst devid <= 0; else devid <= cmdsetid ? cmdid : devid;

`ifdef SIM
	always@(devid)
	$display("#====Ctrl: devid is set:[%2x].", devid );
`endif


  //
  //  nonce
  //  ==
    
    bit minebingo0,minebingo2,minebingo0sync,minebingo0sync0;
    `alwaysclk minebingo0sync0 <= minebingotog;
    `alwaysclk minebingo0sync  <= minebingo0sync0;
    `alwaysclk minebingo2 <= minebingo0sync0 ^ minebingo0sync;

    // reset
        assign mineresetn = ~cmdreset & ~|forcereset;
        assign minenonceresetn = ~cmdresetnonce;
`ifdef SIM
	always@(posedge minebingo2)
	$display("#====Ctrl: bingo! gold:[%8x,%8x].", minegold[31:0], minegold[63:32] );
`endif

  //
  //  tx
  //  ==
  //  ch1, pass thru cmd
  //  ch2, pass thru/tx getsfr
  //  ch3, pass thru/tx gold

    logic   [3:1]       tx1start,tx0start;
    bit     [3:1][7:0]  tx1len,tx0len;
    bit     [3:1][31:0] tx1word,tx0word;
    logic   [3:1][3:0]  tx0fsm,tx1fsm;
    logic   [3:1]       tx0rw,tx1rw,tx0done,tx1done,tx0vld,tx1vld;
    logic   [3:1]       txrdy,txwr;
    logic   [3:1][31:0] txword;
    logic   [3:1][31:0] rxcmdword_incid;
    logic   [3:1][7:0]       devidinc;
    assign tx1start[1] = 0; 
    assign tx1len[1] = 0;
    assign tx1word[1] = 0;

    assign tx1start[2] = cmdgetsfr; 
    assign tx1len[2] = 2;
    assign tx1word[2] = ( tx1fsm[2] == 1 ) ? { PM_RXCMD_GETSFR, devid[7:0], 8'h0, cmdext } : getsfrdata;

    assign tx1start[3] = minebingo2; 
    assign tx1len[3] = 3;
    assign tx1word[3] = ( tx1fsm[3] == 1 ) ? { PM_RXCMD_SUBMIT, devid[7:0], 8'h01, 8'h5a } : ( tx1fsm[3] == 2 ) ? minegold[31:0] : minegold[63:32] ;

    localparam int fifosize[3:1] = {255,8,8};

  generate 
    genvar j;
        for( j = 1 ; j < 4 ; j = j + 1 ) begin: tx

    assign tx0start[j] = rxcmdvld[j];
    assign tx0len[j] = rxcmdlen[j]+2;
    assign tx0word[j] = ( tx0fsm[j] == 1 ) ? rxcmdword_incid[j] : ( tx0fsm[j] == 2 ) ? rxcmddata[j][0] : ( tx0fsm[j] == 3 ) ? rxcmddata[j][1] : rxcmddata[j][2];
    assign rxcmdword_incid[j] = cmdsetid ? { rxcmdword[j][3], devidinc[j], rxcmdword[j][1:0] } : rxcmdword[j];
    assign devidinc[j] = rxcmdword[j][2] + 1;

    `alwaysclkrst  tx0fsm[j] <= 0; else tx0fsm[j] <= ( tx0fsm[j] == 0 ) & tx0start[j] ? 1 : tx0done[j] ? 0 : tx0rw[j] ? tx0fsm[j] + 1 : tx0fsm[j];
    `alwaysclkrst  tx1fsm[j] <= 0; else tx1fsm[j] <= ( tx1fsm[j] == 0 ) & tx1start[j] ? 1 : tx1done[j] ? 0 : tx1rw[j] ? tx1fsm[j] + 1 : tx1fsm[j];

    `alwaysclkrst tx0rw[j] <= 0; else tx0rw[j] <= tx0vld[j] &   txrdy[j] & ~tx0rw[j]? 1 : tx0done[j] ? 0 : tx0rw[j];
    `alwaysclkrst tx1rw[j] <= 0; else tx1rw[j] <= tx1vld[j] & ~tx0vld[j] & ~tx1rw[j]? 1 : tx1done[j] ? 0 : tx1rw[j];

    assign tx0done[j] = ( tx0fsm[j] == tx0len[j] ) & tx0rw[j];
    assign tx1done[j] = ( tx1fsm[j] == tx1len[j] ) & tx1rw[j];
    assign tx0vld[j] = ~( tx0fsm[j] == 0 ) | tx0start[j];
    assign tx1vld[j] = ~( tx1fsm[j] == 0 ) | tx1start[j];

    assign txrdy[j] = ~tx0rw[j] & ~tx1rw[j];
    assign txword[j] = tx0rw[j] ? tx0word[j] : tx1word[j];
    assign txwr[j] = tx0rw[j] | tx1rw[j];

    uarttx  #(.fifosize(fifosize[j]))
    uuarttx(
        .clk        (clk),
        .resetn     (resetn),
        .sclk       (clk),
        .etu        (etu),
        .txwr       (txwr[j]),
        .txword     (txword[j]),
        .tx         (uarttx[j])
        );
        end
  endgenerate

    assign uarttx[0] = uartrx[0];
//
    pwm_pulse_fade uLED(clk, minebingo2, ledout);

endmodule
// -------------------------------------------------------------------------------------------------------------------------------
