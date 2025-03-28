module CguGearBox(
	clk,
	resetn,
	GearLmt,
	GearLvl,
	GearLoad,
	GearGen
);

    parameter GEARINIT = 0;

	input		clk;
	input		resetn;
	input	[7:0]	GearLmt;
	input	[7:0]	GearLvl;
	input		GearLoad;
	output		GearGen;


	wire	[8:0]	GearLvlPlus;
	wire	[8:0]	GearLmtPlus;
	reg	[7:0]	GearLmtReg;
	reg	[7:0]	GearLvlReg;
	reg	[7:0]	GearRndCnt;
	reg	[8:0]	GearCounter;
	wire	[7:0]	GearLmtRegPre;
	wire	[7:0]	GearLvlRegPre;
	wire	[7:0]	GearRndCntPre;
	wire	[8:0]	GearCounterPre;
	reg 		GearGen, GearGen0;
	wire		GearCounterOver;


   //	Gear Rnd


	assign GearRndCntPre =  (( GearRndCnt == GearLmtReg )|GearLoad )? 8'h0 : ( GearRndCnt + 8'h1 ) ;

   //	Gear Counter

	assign GearCounterOver = ( GearCounter + GearLvlPlus ) > GearLmtReg;

	always@(posedge clk or negedge resetn)
	if(!resetn)
	begin
//		GearLmtReg <= 8'hff;
		GearLvlReg <= GEARINIT;
		GearRndCnt <= 8'h0;
		GearCounter <= 9'h0;
		GearGen	<= 1'b0;
		GearGen0 <= 1'b0;
		//GearGen1 <= 1'b0;
	end
	else
	begin
//		GearLmtReg <= GearLmtRegPre;
		GearLvlReg <= GearLvlRegPre;
		GearRndCnt <= GearRndCntPre;
		GearCounter <= GearCounterPre;
		//GearGen <= GearCounterOver;
		GearGen0 <= GearCounter[8];
		GearGen <= ( GearLvl == 0 ) ? 0 : GearCounter[8] ^ GearGen0 ;
	end


//	assign GearLmtPlus = ( {2'h0, GearLmtReg } + 10'h0 );
	assign GearLvlPlus = ( {1'b0, GearLvlReg } + 9'h1 );

//	assign GearCounterPre = GearLoad ? 10'h0 : GearCounterOver ? (( GearCounter + GearLvlPlus ) - GearLmtPlus ) : ( GearCounter + GearLvlPlus );
	assign GearCounterPre = GearLoad ? 9'h0 : GearCounter + GearLvlPlus;
//	assign GearLmtRegPre = GearLoad ? GearLmt : GearLmtReg;
    assign GearLmtReg = GearLmt;
	assign GearLvlRegPre = GearLoad ? GearLvl : GearLvlReg;

   //   Gear Out

	//assign GearGen = GearCounterOver;


endmodule
