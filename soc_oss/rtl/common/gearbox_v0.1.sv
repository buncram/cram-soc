module CguGearBox(
	clk,
	clkenin,
	resetn,
	GearLmt,
	GearLvl,
	GearLoad,
	GearGen
);

	input	logic 		clk;
	input 	logic  		clkenin,
	input	logic 		resetn;
	input	logic [7:0]	GearLmt;
	input	logic [7:0]	GearLvl;
	input	logic 		GearLoad;
	output	logic 		GearGen;


	wire	[9:0]	GearLvlPlus;
	wire	[9:0]	GearLmtPlus;
	reg 	[7:0]	GearLmtReg;
	reg 	[7:0]	GearLvlReg;
	reg 	[7:0]	GearRndCnt;
	reg 	[7:0]	GearCounter;
	wire	[7:0]	GearLmtRegPre;
	wire	[7:0]	GearLvlRegPre;
	wire	[7:0]	GearRndCntPre;
	wire	[9:0]	GearCounterPre;
	wire			GearGen;
	wire			GearCounterOver;
	
	
   //	Gear Rnd


	assign GearRndCntPre =  (( GearRndCnt == GearLmtReg )|GearLoad )? 8'h0 : ( GearRndCnt + 8'h1 ) ;

   //	Gear Counter

	assign GearCounterOver = ( GearCounter + GearLvlPlus ) > GearLmtReg;

	always@(posedge clk or negedge resetn)
	if(!resetn)
	begin
		GearLmtReg <= 8'hff;
		GearLvlReg <= 8'h0;
		GearRndCnt <= 8'h0;
		GearCounter <= 10'h0;
	end
	else
	begin
		GearLmtReg  <= clkenin ? GearLmtRegPre : GearLmtReg;
		GearLvlReg  <= clkenin ? GearLvlRegPre : GearLvlReg ;
		GearRndCnt  <= clkenin ? GearRndCntPre : GearRndCnt ;
		GearCounter <= clkenin ? GearCounterPre : GearCounter;
	end


	assign GearLmtPlus = ( {2'h0, GearLmtReg } + 10'h0 );
	assign GearLvlPlus = ( {2'h0, GearLvlReg } + 10'h0 );

	assign GearCounterPre = GearLoad ? 10'h0 : GearCounterOver ? (( GearCounter + GearLvlPlus ) - GearLmtPlus ) : ( GearCounter + GearLvlPlus );
	assign GearLmtRegPre = GearLoad ? GearLmt : GearLmtReg;
	assign GearLvlRegPre = GearLoad ? GearLvl : GearLvlReg;

   //   Gear Out

	assign GearGen = GearCounterOver;


endmodule
