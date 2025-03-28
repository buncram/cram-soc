
module PkeCore(
            Clk              ,
            Resetn           ,
            PkeIR            ,
            NLen             ,
            ELen             ,
            PkeStart         ,
            RamPkeDat0       ,
            RamPkeDat1       ,
            PkeInt           ,
            ModInvRdy	     ,

            N0Dat	     ,

            PkeRamRd0        ,
            PkeRamWr0        ,
            PkeRamAddr0      ,
            PkeRamDat0       ,
            PkeRamRd1        ,
            PkeRamWr1        ,
            PkeRamAddr1      ,
            PkeRamDat1       
          );

input  	        Clk;
input           Resetn;   
input  [7:0]    PkeIR;
input  [12:0]   NLen;
input  [12:0]   ELen;
input           PkeStart;
input  [63:0]   RamPkeDat0;
input  [63:0]   RamPkeDat1;
   
input  [63:0]   N0Dat;
output          PkeInt;         
output          ModInvRdy;         
output          PkeRamRd0;
output          PkeRamWr0;
output [8:0]    PkeRamAddr0;
output [63:0]   PkeRamDat0;
output          PkeRamRd1;
output          PkeRamWr1;
output [8:0]    PkeRamAddr1;
output [63:0]   PkeRamDat1;
    
wire 	        Clk;
wire            Resetn;         
wire   [7:0]    PkeIR;
wire   [12:0]   NLen;
wire   [12:0]   ELen;
wire            PkeStart;
wire   [63:0]   RamPkeDat0;
wire   [63:0]   RamPkeDat1;
wire   [63:0]   N0Dat;
wire            PkeInt;         
wire            ModInvRdy;         
wire            PkeRamRd0;
wire            PkeRamWr0;
wire   [8:0]    PkeRamAddr0;
wire   [63:0]   PkeRamDat0;
wire            PkeRamRd1;
wire            PkeRamWr1;
wire   [8:0]    PkeRamAddr1;
wire   [63:0]   PkeRamDat1;

wire            PkeDone;
wire            BasicStart;
wire      [5:0] BasicMode;
wire            BasicDone; 
wire            BasicRamRd1;
wire            BasicRamRd2;
wire            BasicRamWr1;
wire            BasicRamWr2;
wire      [8:0] BasicRamAddr1;
wire      [8:0] BasicRamAddr2;
wire     [63:0] BasicRamDat1;
wire     [63:0] BasicRamDat2;
wire      [2:0] BasicSR;



wire            ModMulStart;
reg             ModMulStartReg;
wire            ModMulBasicStart;
wire      [5:0] ModMulBasicMode;
wire            ModMulRamRdA;
wire            ModMulRamRdB;
wire            ModMulRamRdM;
wire            ModMulRamRdN;
wire            ModMulRamWr1;
wire            ModMulRamWr2;
wire      [8:0] ModMulRamAddr1;
wire      [8:0] ModMulRamAddr2;
wire     [63:0] ModMulRamDat1;
wire     [63:0] ModMulRamDat2;
wire      [8:0] ModMulLongSrcAddr1;
wire      [8:0] ModMulLongSrcAddr2;
wire      [8:0] ModMulLongDstAddr;
wire            ModMulDone;

wire      [8:0] Src0Adr;
wire      [8:0] Src1Adr;
wire      [8:0] DstAdr;
wire            PointEn;
wire            ModMulEn;

wire            ModInvRamRd;
wire      [8:0] ModInvRamAdr;
wire 	  [5:0] PkePBasicMode;

wire            StartHCal;
wire            EccHCal;
wire [5:0]      NLenHigh;
wire [63:0]     J0Reg;
wire            HCalRamRd0;
wire            HCalRamRd1;
wire            HCalRamWr1;
wire [8 :0]     HCalRamAddr0;
wire [8 :0]     HCalRamAddr1;
wire [63:0]     HCalRamDat1;

wire [63:0]     PkeQ;
wire            QCalStart;
wire            PkeStart1;
wire            EndQCal;

always @(posedge Clk or negedge Resetn) 
if(~Resetn)
  ModMulStartReg <= 1'b0;
else
  ModMulStartReg <= ModMulStart;

// ================== main code ================== //

// 1.1 PkeCtl

PkeCtrl  uPkeCtrl(
            .Clk              (Clk),
            .Resetn           (Resetn),
            .PkeStart         (PkeStart1), 
            .PkeIR            (PkeIR),
            .PkeNLen          (NLen),
            .PkeELen          (ELen),
            .PkeDone          (PkeDone),
            .ModInvRdy        (ModInvRdy),

            .LongAlgStart       (BasicStart),
            .LongAlgOp        (PkePBasicMode),
            .LongAlgDone        (BasicDone), 
            .LongAlgSR         (BasicSR),

            .MpcStart      (ModMulStart),
            .MpcDone       (ModMulDone),

	    	.Src0Adr	      (Src0Adr),
            .Src1Adr	      (Src1Adr),
            .PointEn          (PointEn),
            .DstAdr	      	  (DstAdr),

            .ModExpRamRd      (ModInvRamRd),
            .ModExpRamAdr     (ModInvRamAdr),
            .RamModExpDat     (RamPkeDat0) 

            );
//assign   ModInvRamRd  =0;
//assign   ModInvRamAdr  =0;
wire RsaMode;
wire ExpMode;

assign RsaMode = PkeIR[7:4]==4'h1;
assign ExpMode = PkeIR[7:0]==8'h13;
// 1.2 PkeRamMux
PkeRamMux uRamMux(
             .Clk                   (Clk),
             .Resetn                (Resetn),
             .BasicStart            (BasicStart |ModMulBasicStart),
             .BasicDone             (BasicDone),
             .ModMulStart           (ModMulStart),
             .ModMulDone            (ModMulDone),
             .Src0Adr               (Src0Adr),
             .Src1Adr               (Src1Adr),
             .PointEn               (PointEn),
	     .ModMulEn              (ModMulEn),
     
             //.RsaMode               (PkeIR[4]), 
             .RsaMode               (RsaMode), 
             //.ExpMode               (PkeIR[0]&PkeIR[1]),
             .ExpMode               (ExpMode),
             .DstAdr                (DstAdr),

             .ModInvRamRd            (ModInvRamRd),
             .ModInvRamAdr           (ModInvRamAdr),
                                                         
             .BasicRamRd1           (BasicRamRd1),
             .BasicRamRd2           (BasicRamRd2),
             .BasicRamWr1           (BasicRamWr1),
             .BasicRamWr2           (BasicRamWr2),
             .BasicRamAddr1         (BasicRamAddr1),
             .BasicRamAddr2         (BasicRamAddr2),
             .BasicRamDat1          (BasicRamDat1),
             .BasicRamDat2          (BasicRamDat2),
                                                         
             .ModMulRamRdA          (ModMulRamRdA),
             .ModMulRamRdB          (ModMulRamRdB),
             .ModMulRamRdM          (ModMulRamRdM),
             .ModMulRamRdN          (ModMulRamRdN),
             .ModMulRamWr1          (ModMulRamWr1),
             .ModMulRamWr2          (ModMulRamWr2),
             .ModMulRamAddr1        (ModMulRamAddr1),
             .ModMulRamAddr2        (ModMulRamAddr2),
             .ModMulRamDat1         (ModMulRamDat1),
             .ModMulRamDat2         (ModMulRamDat2),
             .ModMulLongSrcAddr1    (ModMulLongSrcAddr1),
             .ModMulLongSrcAddr2    (ModMulLongSrcAddr2),
             .ModMulLongDstAddr     (ModMulLongDstAddr),
                        
             .PkeRamRd0             (PkeRamRd0),
             .PkeRamWr0             (PkeRamWr0),
             .PkeRamAddr0           (PkeRamAddr0),
             .PkeRamDat0            (PkeRamDat0),
             .PkeRamRd1             (PkeRamRd1),
             .PkeRamWr1             (PkeRamWr1),
             .PkeRamAddr1           (PkeRamAddr1),
             .PkeRamDat1            (PkeRamDat1) 
);

wire [7:0]BasicLen;

// 1.5 Basic
com_alg uBasicAlg ( 
            .clk                (Clk),
            .resetn             (Resetn),
            .comalg_wlen 	(BasicLen),
            .comalg_mode        (BasicMode),
            .comalg_start       (BasicStart |ModMulBasicStart),
            .comalg_end         (BasicDone),
            .comalg_status      (BasicSR),
            .ram0_comalg_rdata  (RamPkeDat0),
            .ram1_comalg_rdata  (RamPkeDat1),
        
            .comalg_ram0_rd     (BasicRamRd1),
            .comalg_ram1_rd     (BasicRamRd2),
            .comalg_ram0_wr     (BasicRamWr1),
            .comalg_ram1_wr     (BasicRamWr2),
            .comalg_ram0_addr   (BasicRamAddr1[7:0]),
            .comalg_ram1_addr   (BasicRamAddr2[7:0]),
            .comalg_ram0_wdata  (BasicRamDat1),
            .comalg_ram1_wdata  (BasicRamDat2)
);

assign BasicRamAddr1[8] = 1'b0;
assign BasicRamAddr2[8] = 1'b0;

assign BasicMode = ModMulEn ? ModMulBasicMode : PkePBasicMode;
//assign BasicLen =
//                  ModMulEn & (NLen[5:0]==6'd0)  ? NLen[11:6]    : 
//                  ~ModMulEn & (NLen[5:0]==6'd0) ? NLen[11:6]+1  : 
//                  ModMulEn & (NLen[5:0]!=6'd0)  ? NLen[11:6]+1  : 
//                                                  NLen[11:6]+2  ;
//
//zxjian,20220623,NLen%64=63|62 NLen+2
assign BasicLen =
                  ModMulEn & (NLen[5:0]==6'd0)  ? NLen[12:6]    : 
                  ModMulEn & (NLen[5:0]!=6'd0)  ? NLen[12:6]+1  : 
                  ~ModMulEn & (NLen[5:1]==6'h1F)? NLen[12:6]+2  : 
                                                  NLen[12:6]+1  ;

// 1.6 ModMul

mgmr_mul uModMul (
    .clk                    (Clk),
    .resetn                 (Resetn),
    .mgmr_mul_start   	    (ModMulStartReg),
    .param_J0               (PkeQ),
    .N_word_num             (BasicLen ),
    .ram_mul_dat0           (RamPkeDat0),
    .ram_mul_dat1           (RamPkeDat1),
    .mgmr_mul_end           (ModMulDone),
    .mgmr_mul_ram0_rdA      (ModMulRamRdA),
    .mgmr_mul_ram1_rdB      (ModMulRamRdB),
    .mgmr_mul_ram1_rdM      (ModMulRamRdM),
    .mgmr_mul_ram0_rdN      (ModMulRamRdN),
    .mgmr_mul_ram0_wr       (ModMulRamWr1),
    .mgmr_mul_ram1_wr       (ModMulRamWr2),
    .mgmr_mul_ram0_addr     (ModMulRamAddr1[7:0]),
    .mgmr_mul_ram1_addr     (ModMulRamAddr2[7:0]),
    .mgmr_mul_ram0_dat      (ModMulRamDat1),
    .mgmr_mul_ram1_dat      (ModMulRamDat2),

    .mgmr_mul_comalg_start  (ModMulBasicStart),
    .comalg_end             (BasicDone),
    .mgmr_mul_comalg_mode        (ModMulBasicMode),
    .mgmr_mul_comalg_src0_addr   (ModMulLongSrcAddr1[7:0]),
    .mgmr_mul_comalg_src1_addr   (ModMulLongSrcAddr2[7:0]),
    .mgmr_mul_comalg_dst_addr    (ModMulLongDstAddr[7:0]) 
);

assign ModMulRamAddr1[8] = 1'b0;
assign ModMulRamAddr2[8] = 1'b0;
assign ModMulLongSrcAddr1[8] = 1'b0;
assign ModMulLongSrcAddr2[8] = 1'b0;
assign ModMulLongDstAddr[8] = 1'b0;

assign QCalStart = PkeStart & PkeIR[7:0] == 8'h20;
assign PkeStart1 = PkeStart & ~QCalStart;

//1.7 J0Cal
QRegCal uQCal(
    .Clk		(Clk		),
    .Resetn		(Resetn		),
    .N0Dat		(N0Dat		),
    .StartQCal		(QCalStart	),
    
    .QReg		(PkeQ		),
    .EndQCal		(EndQCal	)
    );

assign PkeInt = PkeDone | EndQCal;

endmodule
//-----------------------------End---------------------------------//

