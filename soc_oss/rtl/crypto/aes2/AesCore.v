

module AesCore(
                Clk,
		Resetn,
		StartAes,
		AesIR,
		AesLen,
		AesMode,
		
		AesRamRd,
		AesRamWr,
		AesRamAdr,
		AesRamDat,
		RamAesDat,
	
		MaskIn,
	
		AesDone);


input  wire        Clk	;
input  wire        Resetn	;
input  wire        StartAes	;
input  wire [1:0]  AesIR	;
input  wire [1:0]  AesLen	;
input  wire [2:0]  AesMode	;
output wire        AesRamRd	;
output wire        AesRamWr	;
output wire [7:0]  AesRamAdr;
output wire [31:0] AesRamDat;
input  wire [31:0] RamAesDat;
input  wire [31:0] MaskIn   ;
output wire        AesDone	;


wire KeyEn;
wire EncEn;
wire DecEn;
wire FirstRound;
wire LastRound;

wire [31:0] DataIn;
wire [31:0] MaskIn;
wire [31:0] DataOut;

wire [31:0] RoundKeyIn;


AesCtrl uCtrl(
        .Clk      (Clk			),
	.Resetn   (Resetn		),
	.StartAes (StartAes		),
	.AesIR    (AesIR		), //00 for KeyExp, 01 for Enc, 10 for Dec
	.AesLen   (AesLen		), //00 for 128bit, 01 for 192bit, 10 for 256bit
	.AesMode  (AesMode		), //00 for ECB ,01 for CBC , 10 for 
	
	.AesRamRd (AesRamRd		),
	.AesRamWr (AesRamWr		),
	.AesRamAdr(AesRamAdr		),
	.AesRamDat(AesRamDat		),
	.RamAesDat(RamAesDat		),

        .KeyEn    (KeyEn		),
        .EncEn    (EncEn		),
        .DecEn    (DecEn		),
        .DataPathCtrlDat    (DataOut	),
        .CtrlDataPathDat    (DataIn	),
	.LastRound(LastRound		),
	.FirstRound(FirstRound		),
	
	.AesDone  (AesDone		)
	);

 AesDataPath uDataPath(
        .DataIn		(DataIn			),
        .MaskIn		(MaskIn			),
	.RoundKeyIn	(RamAesDat              ), 
	.Enc		(EncEn			),
	.Dec		(DecEn			),
	.KeyEn		(KeyEn			),
	.FirstRound	(FirstRound		),
	.LastRound	(LastRound		),
	.DataOut	(DataOut		)
	);


endmodule
