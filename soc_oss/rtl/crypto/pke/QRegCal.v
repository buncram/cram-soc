module QRegCal(
       Clk,
       Resetn,
       N0Dat,
       StartQCal,
       
       QReg,
       EndQCal
       );
input         Clk;
input         Resetn;
input  [63:0] N0Dat;
input         StartQCal;

output [63:0] QReg;
output        EndQCal;


wire          Clk;
wire          Resetn;
wire   [63:0] N0Dat;
wire          StartQCal;

wire          EndQCal;


parameter Q_IDLE	= 2'b00;
parameter Q_CALC	= 2'b01;
parameter Q_END		= 2'b10;


//1 signals
//
reg  [1 :0] QState; 
reg  [1 :0] NextQState;
reg  [5 :0] QCnt;
wire [5 :0] NextQCnt;
wire        QCalEnd;

reg  [63:0] DiffReg;
wire [63:0] NextDiff;
wire        QRegBit;

reg  [63:0] QReg;
wire [63:0] NextQReg;

//2 FSM
//
always @(posedge Clk or negedge Resetn)
if(~Resetn)
    QState <= Q_IDLE;
else
    QState <= NextQState;

always @(*) 
case(QState)
    Q_IDLE:
        if(StartQCal)
            NextQState = Q_CALC;
        else
            NextQState = Q_IDLE;
    Q_CALC:
        if(QCalEnd)
            NextQState = Q_END;
        else
            NextQState = Q_CALC;
    Q_END:
            NextQState = Q_IDLE;
    default:
            NextQState = Q_IDLE;
endcase

always @(posedge Clk or negedge Resetn)
if(~Resetn)
    QCnt <= 6'd0;
else
    QCnt <= NextQCnt;

assign NextQCnt = StartQCal      ? 6'd0   :
                  QState==Q_CALC ? QCnt+1 :
                                   QCnt   ;
assign QCalEnd = QCnt == 6'h3f;

assign EndQCal = QState == Q_END;

// 3 DataPath
//
always @(posedge Clk or negedge Resetn)
if(~Resetn)
    DiffReg <= 64'd1;
else
    DiffReg <= NextDiff;

assign NextDiff = StartQCal                   ? 64'd1                :
                  QRegBit & (QState==Q_CALC)  ? (DiffReg + N0Dat)>>1 :
                  ~QRegBit & (QState==Q_CALC) ? DiffReg >>1          :
                                                DiffReg              ;

assign QRegBit  = DiffReg[0];


//4 Output
//

always @(posedge Clk or negedge Resetn)
if(~Resetn)
    QReg <= 64'd1;
else
    QReg <= NextQReg;

assign NextQReg[0]  = (QState == Q_CALC) & (QCnt == 0 ) ? QRegBit : QReg[0];
assign NextQReg[1]  = (QState == Q_CALC) & (QCnt == 1 ) ? QRegBit : QReg[1];
assign NextQReg[2]  = (QState == Q_CALC) & (QCnt == 2 ) ? QRegBit : QReg[2];
assign NextQReg[3]  = (QState == Q_CALC) & (QCnt == 3 ) ? QRegBit : QReg[3];
assign NextQReg[4]  = (QState == Q_CALC) & (QCnt == 4 ) ? QRegBit : QReg[4];
assign NextQReg[5]  = (QState == Q_CALC) & (QCnt == 5 ) ? QRegBit : QReg[5];
assign NextQReg[6]  = (QState == Q_CALC) & (QCnt == 6 ) ? QRegBit : QReg[6];
assign NextQReg[7]  = (QState == Q_CALC) & (QCnt == 7 ) ? QRegBit : QReg[7];
assign NextQReg[8]  = (QState == Q_CALC) & (QCnt == 8 ) ? QRegBit : QReg[8];
assign NextQReg[9]  = (QState == Q_CALC) & (QCnt == 9 ) ? QRegBit : QReg[9];
assign NextQReg[10] = (QState == Q_CALC) & (QCnt == 10) ? QRegBit : QReg[10];
assign NextQReg[11] = (QState == Q_CALC) & (QCnt == 11) ? QRegBit : QReg[11];
assign NextQReg[12] = (QState == Q_CALC) & (QCnt == 12) ? QRegBit : QReg[12];
assign NextQReg[13] = (QState == Q_CALC) & (QCnt == 13) ? QRegBit : QReg[13];
assign NextQReg[14] = (QState == Q_CALC) & (QCnt == 14) ? QRegBit : QReg[14];
assign NextQReg[15] = (QState == Q_CALC) & (QCnt == 15) ? QRegBit : QReg[15];
assign NextQReg[16] = (QState == Q_CALC) & (QCnt == 16) ? QRegBit : QReg[16];
assign NextQReg[17] = (QState == Q_CALC) & (QCnt == 17) ? QRegBit : QReg[17];
assign NextQReg[18] = (QState == Q_CALC) & (QCnt == 18) ? QRegBit : QReg[18];
assign NextQReg[19] = (QState == Q_CALC) & (QCnt == 19) ? QRegBit : QReg[19];
assign NextQReg[20] = (QState == Q_CALC) & (QCnt == 20) ? QRegBit : QReg[20];
assign NextQReg[21] = (QState == Q_CALC) & (QCnt == 21) ? QRegBit : QReg[21];
assign NextQReg[22] = (QState == Q_CALC) & (QCnt == 22) ? QRegBit : QReg[22];
assign NextQReg[23] = (QState == Q_CALC) & (QCnt == 23) ? QRegBit : QReg[23];
assign NextQReg[24] = (QState == Q_CALC) & (QCnt == 24) ? QRegBit : QReg[24];
assign NextQReg[25] = (QState == Q_CALC) & (QCnt == 25) ? QRegBit : QReg[25];
assign NextQReg[26] = (QState == Q_CALC) & (QCnt == 26) ? QRegBit : QReg[26];
assign NextQReg[27] = (QState == Q_CALC) & (QCnt == 27) ? QRegBit : QReg[27];
assign NextQReg[28] = (QState == Q_CALC) & (QCnt == 28) ? QRegBit : QReg[28];
assign NextQReg[29] = (QState == Q_CALC) & (QCnt == 29) ? QRegBit : QReg[29];
assign NextQReg[30] = (QState == Q_CALC) & (QCnt == 30) ? QRegBit : QReg[30];
assign NextQReg[31] = (QState == Q_CALC) & (QCnt == 31) ? QRegBit : QReg[31];
assign NextQReg[32] = (QState == Q_CALC) & (QCnt == 32) ? QRegBit : QReg[32];
assign NextQReg[33] = (QState == Q_CALC) & (QCnt == 33) ? QRegBit : QReg[33];
assign NextQReg[34] = (QState == Q_CALC) & (QCnt == 34) ? QRegBit : QReg[34];
assign NextQReg[35] = (QState == Q_CALC) & (QCnt == 35) ? QRegBit : QReg[35];
assign NextQReg[36] = (QState == Q_CALC) & (QCnt == 36) ? QRegBit : QReg[36];
assign NextQReg[37] = (QState == Q_CALC) & (QCnt == 37) ? QRegBit : QReg[37];
assign NextQReg[38] = (QState == Q_CALC) & (QCnt == 38) ? QRegBit : QReg[38];
assign NextQReg[39] = (QState == Q_CALC) & (QCnt == 39) ? QRegBit : QReg[39];
assign NextQReg[40] = (QState == Q_CALC) & (QCnt == 40) ? QRegBit : QReg[40];
assign NextQReg[41] = (QState == Q_CALC) & (QCnt == 41) ? QRegBit : QReg[41];
assign NextQReg[42] = (QState == Q_CALC) & (QCnt == 42) ? QRegBit : QReg[42];
assign NextQReg[43] = (QState == Q_CALC) & (QCnt == 43) ? QRegBit : QReg[43];
assign NextQReg[44] = (QState == Q_CALC) & (QCnt == 44) ? QRegBit : QReg[44];
assign NextQReg[45] = (QState == Q_CALC) & (QCnt == 45) ? QRegBit : QReg[45];
assign NextQReg[46] = (QState == Q_CALC) & (QCnt == 46) ? QRegBit : QReg[46];
assign NextQReg[47] = (QState == Q_CALC) & (QCnt == 47) ? QRegBit : QReg[47];
assign NextQReg[48] = (QState == Q_CALC) & (QCnt == 48) ? QRegBit : QReg[48];
assign NextQReg[49] = (QState == Q_CALC) & (QCnt == 49) ? QRegBit : QReg[49];
assign NextQReg[50] = (QState == Q_CALC) & (QCnt == 50) ? QRegBit : QReg[50];
assign NextQReg[51] = (QState == Q_CALC) & (QCnt == 51) ? QRegBit : QReg[51];
assign NextQReg[52] = (QState == Q_CALC) & (QCnt == 52) ? QRegBit : QReg[52];
assign NextQReg[53] = (QState == Q_CALC) & (QCnt == 53) ? QRegBit : QReg[53];
assign NextQReg[54] = (QState == Q_CALC) & (QCnt == 54) ? QRegBit : QReg[54];
assign NextQReg[55] = (QState == Q_CALC) & (QCnt == 55) ? QRegBit : QReg[55];
assign NextQReg[56] = (QState == Q_CALC) & (QCnt == 56) ? QRegBit : QReg[56];
assign NextQReg[57] = (QState == Q_CALC) & (QCnt == 57) ? QRegBit : QReg[57];
assign NextQReg[58] = (QState == Q_CALC) & (QCnt == 58) ? QRegBit : QReg[58];
assign NextQReg[59] = (QState == Q_CALC) & (QCnt == 59) ? QRegBit : QReg[59];
assign NextQReg[60] = (QState == Q_CALC) & (QCnt == 60) ? QRegBit : QReg[60];
assign NextQReg[61] = (QState == Q_CALC) & (QCnt == 61) ? QRegBit : QReg[61];
assign NextQReg[62] = (QState == Q_CALC) & (QCnt == 62) ? QRegBit : QReg[62];
assign NextQReg[63] = (QState == Q_CALC) & (QCnt == 63) ? QRegBit : QReg[63];


endmodule
