// ============================================================
module emb_v2(
    hclk,     //hclk
    aclk,     //register clk
    r0clk,    //ram0 clk
    r1clk,    //ram1 clk
    r2clk,    //ram2 clk
    r3clk,    //ram3 clk
    pkeclk,   //pke logic clk
    aesclk,   //aes logic clk
    hresetn,
    haddr,
    hsel,
    htrans,
    hwrite,
    hsize,
    hburst,
    hwdata,
    hrdata,
    hready_in, 
    hready_out,
    hresp
);

input        hclk	;
input        aclk	;
input        r0clk	;
input        r1clk	;
input        r2clk	;
input        r3clk	;
input        pkeclk	;
input        aesclk	;
input        hresetn	;
input [31:0] haddr	;
input        hsel	;
input [1:0]  htrans	;
input        hwrite	;
input [2:0]  hsize	;
input [2:0]  hburst	;
input [31:0] hwdata	;
output[31:0] hrdata	;
input        hready_in	; 
output       hready_out	;
output       hresp	;

//0 internal signals
wire [13:0]  a	  	;
wire [31:0]  d	  	;
wire         ce	  	;
wire [3:0]   we	  	;
wire [31:0]  q	  	;
reg  [11:0]  aReg	;
wire [11:0]  NextaReg	;
reg  [7 :0]  PkeStatus	;
reg  [31 :0] PkeJ0Low	;
reg  [31 :0] PkeJ0High	;

wire [31 :0] NextPkeJ0Low	;
wire [31 :0] NextPkeJ0High	;
wire [63:0]  PkeJ0	;
//wire [63:0]  PkeQ	;

reg  [31 :0] PkeIR	;
reg  [11:0]  NLen        ;
reg  [11:0]  ELen        ;

reg          PkeDone	;
wire         ModInvRdy	;

wire         NextPkeDone;
wire         NextModInvRdy;
wire         PkeStart	;

wire         PkeInt	;
wire         ModInvPkeRdy;

wire [7 :0]  NextPkeStatus;
wire [31 :0] NextPkeIR	;

wire [11 :0] NextNLen	;
wire [11 :0] NextELen	;

	
wire         PkeRamRd0	;
wire         PkeRamWr0	;
wire [7 :0]  PkeRamAddr0;
wire [63:0]  PkeRamDat0	;

wire         PkeRamRd1	;
wire         PkeRamWr1	;
wire [7:0]   PkeRamAddr1;
wire [63:0]  PkeRamDat1	;

wire [63:0]  RamPkeDat0	;
wire [63:0]  aRam0PDatOut;
wire [63:0]  RamPkeDat1	;
wire [63:0]  aRam1PDatOut;

wire [31 :0] aRam00HDatOut;
wire [31 :0] aRam01HDatOut;
wire [31 :0] aRam10HDatOut;
wire [31 :0] aRam11HDatOut;

wire         aRam00HEn_tmp	;
wire [31 :0] aRam00HWr_tmp	;
wire [7  :0] aRam00HAdr_tmp	;
wire [31 :0] aRam00HDatIn_tmp   ;

wire         aRam01HEn_tmp	;
wire [31 :0] aRam01HWr_tmp	;
wire [7  :0] aRam01HAdr_tmp	;
wire [31 :0] aRam01HDatIn_tmp   ;

wire         aRam0PEn_tmp	;
wire         aRam0PWr_tmp	;
wire [7  :0] aRam0PAdr_tmp	;
wire [63 :0] aRam0PDatIn_tmp    ;

wire         aRam10HEn_tmp	;
wire [31 :0] aRam10HWr_tmp	;
wire [7  :0] aRam10HAdr_tmp	;
wire [31 :0] aRam10HDatIn_tmp   ;

wire         aRam11HEn_tmp	;
wire [31 :0] aRam11HWr_tmp	;
wire [7  :0] aRam11HAdr_tmp	;
wire [31 :0] aRam11HDatIn_tmp   ;

wire         aRam1PEn_tmp	;
wire         aRam1PWr_tmp	;
wire [7  :0] aRam1PAdr_tmp	;
wire [63 :0] aRam1PDatIn_tmp    ;

reg           PkeEn	;
wire          NextPkeEn	;

wire          aclk	;
wire          r0clk	;
wire          r1clk	;
wire          r2clk	;
wire          r3clk	;
wire          pkeclk	;
wire          aesclk	;

wire  [31 :0] Ram00Dout	;
wire          Ram00CeN	;
wire  [31 :0] Ram00WeN	;
wire  [7  :0] Ram00Adr	;
wire  [31 :0] Ram00Din	;

wire  [31 :0] Ram01Dout	;
wire          Ram01CeN	;
wire  [31 :0] Ram01WeN	;
wire  [7  :0] Ram01Adr	;
wire  [31 :0] Ram01Din	;

wire  [31 :0] Ram10Dout	;
wire          Ram10CeN	;
wire  [31 :0] Ram10WeN	;
wire  [7  :0] Ram10Adr	;
wire  [31 :0] Ram10Din	;

wire  [31 :0] Ram11Dout	;
wire          Ram11CeN	;
wire  [31 :0] Ram11WeN	;
wire  [7  :0] Ram11Adr	;
wire  [31 :0] Ram11Din	;

wire          StartAes	;
wire  [1  :0] AesIR	;
wire  [1  :0] AesLen	;
wire  [2  :0] AesMode	;
wire          AesOFB	;
wire          AesDone	;

reg           AesEn	;
wire          NextAesEn	;

wire          AesRamRd 	;
wire          AesRamWr 	;
wire  [7:0]   AesRamAdr	;
wire  [31:0]  AesRamDat	;
wire  [31:0]  AesRamDat1;
wire  [31:0]  RamAesDat	;

reg   [127:0] XReg;
reg   [127:0] YReg;
reg   [127:0] ZReg;
reg   [127:0] IVector;
wire  [127:0] NextXReg;
wire  [127:0] NextYReg;
wire  [127:0] NextZReg;
wire  [127:0] NextIVector;
wire  [127:0] ZOut;
wire          GHashStart;
wire          GHashDone;

//
wire  [2   :0] ShaMode_256;
wire  [2   :0] ShaMode_512;
wire  [511 :0] MIn_256;
wire  [255 :0] HIn_256; 
wire  [255 :0] HOut_256; 
wire  [1023:0] MIn_512;
wire  [511 :0] HIn_512;
wire  [511 :0] HOut_512;
wire           EndSha256;
wire           EndSha512;

reg   [1023:0] MInReg;
reg   [511 :0] HInReg; 
reg   [511 :0] HOutReg; 
wire  [1023:0] NextMInReg;
wire  [511 :0] NextHInReg; 
wire  [511 :0] NextHOutReg; 

wire           Sha256En;
wire           Sha512En;
wire           ShaStart_256;
wire           ShaStart_512;

wire  [7 :0]  AesRam00Adr;
wire  [7 :0]  AesRam01Adr;
wire  [7 :0]  AesRam10Adr;
wire  [7 :0]  AesRam11Adr;

wire          AesRamce;
wire          AesRam0Sel;

wire          AesRam00ce;
wire          AesRam01ce;
wire          AesRam10ce;
wire          AesRam11ce;

wire          AesRam00we;
wire          AesRam01we;
wire          AesRam10we;
wire          AesRam11we;

reg           AesRam00ce_reg ;
reg           AesRam01ce_reg ;
reg           AesRam10ce_reg ;
reg           AesRam11ce_reg ;

//zxjian,20220522
//assign AesRam00Adr  = AesRamAdr[7:1] + 8'hE8;
//assign AesRam01Adr  = AesRamAdr[7:1] + 8'hE8;
assign AesRam00Adr  = AesOFB ? AesRamAdr[7:1] + 8'hC6 : AesRamAdr[7:1] + 8'hE8;
assign AesRam01Adr  = AesOFB ? AesRamAdr[7:1] + 8'hC6 : AesRamAdr[7:1] + 8'hE8;

assign AesRam10Adr  = AesRamAdr[7:1] + 8'hD0;
assign AesRam11Adr  = AesRamAdr[7:1] + 8'hD0;

assign AesRamce     = AesRamRd |AesRamWr;
assign AesRam0Sel   = AesRamAdr < 8'h30;
//zxjian,20220522
//assign AesRam00ce   =  AesRam0Sel & AesRamce & ~AesRamAdr[0]; 
//assign AesRam01ce   =  AesRam0Sel & AesRamce &  AesRamAdr[0]; 
assign AesRam00ce   =  (AesRam0Sel | AesOFB) & AesRamce & ~AesRamAdr[0]; 
assign AesRam01ce   =  (AesRam0Sel | AesOFB) & AesRamce &  AesRamAdr[0]; 
assign AesRam10ce   = ~AesRam0Sel & AesRamce & ~AesRamAdr[0]; 
assign AesRam11ce   = ~AesRam0Sel & AesRamce &  AesRamAdr[0]; 

//zxjian,20220522
//assign AesRam00we   =  AesRam0Sel & AesRamWr & ~AesRamAdr[0]; 
//assign AesRam01we   =  AesRam0Sel & AesRamWr &  AesRamAdr[0]; 
assign AesOFB       = (PkeIR[20:18] == 3'b100) & AesRamWr;
assign AesRam00we   =  (AesRam0Sel | AesOFB) & AesRamWr & ~AesRamAdr[0]; 
assign AesRam01we   =  (AesRam0Sel | AesOFB) & AesRamWr &  AesRamAdr[0]; 
assign AesRam10we   = ~AesRam0Sel & AesRamWr & ~AesRamAdr[0]; 
assign AesRam11we   = ~AesRam0Sel & AesRamWr &  AesRamAdr[0]; 


always @(posedge aesclk or negedge hresetn)
if(~hresetn)
    begin
        AesRam00ce_reg <= 1'b0;
        AesRam01ce_reg <= 1'b0;
        AesRam10ce_reg <= 1'b0;
        AesRam11ce_reg <= 1'b0;
    end
else
    begin
        AesRam00ce_reg <= AesRam00ce;
        AesRam01ce_reg <= AesRam01ce;
        AesRam10ce_reg <= AesRam10ce;
        AesRam11ce_reg <= AesRam11ce;
    end

//1 AhbSlave 
//
//
cmsdk_ahb_to_sram #(
   .AW       ( 16))// Address width
 uemb_v2(
   .HCLK		(hclk		), // system bus clock
   .HRESETn		(hresetn	), // system bus reset
   .HSEL		(hsel		), // AHB peripheral select
   .HREADY		(hready_in      ), // AHB ready input
   .HTRANS		(htrans		), // AHB transfer type
   .HSIZE		(hsize		), // AHB hsize
   .HWRITE		(hwrite		), // AHB hwrite
   .HADDR		(haddr[15:0]    ), // AHB address bus
   .HWDATA		(hwdata		), // AHB write data bus
   .HREADYOUT		(hready_out	), // AHB ready output to S->M mux
   .HRESP		(hresp		), // AHB response
   .HRDATA		(hrdata		), // AHB read data bus

   .SRAMRDATA		(q	 	), // SRAM Read Data
   .SRAMADDR		(a       	), // SRAM address
   .SRAMWEN		(we      	), // SRAM write enable (active high)
   .SRAMWDATA		(d		), // SRAM write data
   .SRAMCS		(ce		) // SRAM Chip Select  (active high)
   );


//2 Registers
assign PkeStart = ~PkeStatus[0] & NextPkeStatus[0];
assign AesLen   = PkeIR[17:16];
assign AesMode  = PkeIR[20:18];
assign AesIR    = PkeIR[1:0];
assign AesStart = ~PkeStatus[0] & NextPkeStatus[0] & PkeIR[6:2]==5'h10;
assign GHashStart = ~PkeStatus[0] & NextPkeStatus[0] & PkeIR[7:0] == 8'h80;

always @(posedge aclk or negedge hresetn)
  if(~hresetn)
    begin
        PkeStatus   <= 8'h00;
	PkeIR	    <= 32'h0000_0000;
        NLen        <= 12'h000;
        ELen        <= 12'h000;
	PkeJ0Low    <= 32'h0000_0000;
	PkeJ0High   <= 32'h0000_0000;
        XReg        <= 128'h0;
        YReg        <= 128'h0;
        ZReg        <= 128'h0;
        IVector     <= 128'h0;

	PkeDone	    <= 1'b0;
	//ModInvRdy   <= 1'b0;
	aReg        <= 12'h0;

	end
  else
    begin                     
       
	PkeStatus   <=NextPkeStatus;
	PkeIR	    <=NextPkeIR;
        NLen        <= NextNLen;
        ELen        <= NextELen;
	PkeJ0Low    <= NextPkeJ0Low;
	PkeJ0High   <= NextPkeJ0High;

        XReg        <= NextXReg;
        YReg        <= NextYReg;
        ZReg        <= NextZReg;
        IVector     <= NextIVector;

	PkeDone	    <=NextPkeDone;
	//ModInvRdy   <=NextModInvRdy;
	aReg        <=NextaReg;

	end
assign NextaReg =ce ? a : aReg;

assign NextPkeStatus[7:0] = ((a==14'h040) & ce &we[0]) ? d[7  :0] : PkeStatus[7:0];
assign NextPkeIR[7:0]     = ((a==14'h041) & ce &we[0]) ? d[7  :0] : PkeIR[7  :0] ;
assign NextPkeIR[15:8]    = ((a==14'h041) & ce &we[1]) ? d[15 :8] : PkeIR[15 :8] ;
assign NextPkeIR[23:16]   = ((a==14'h041) & ce &we[2]) ? d[23 :16]: PkeIR[23 :16];
assign NextPkeIR[31:24]   = ((a==14'h041) & ce &we[3]) ? d[31 :24]: PkeIR[31 :24];
assign NextPkeJ0Low[7:0]  = ((a==14'h042) & ce &we[0]) ? d[7  :0] : PkeJ0Low[7:0];
assign NextPkeJ0Low[15:8] = ((a==14'h042) & ce &we[1]) ? d[15 :8] : PkeJ0Low[15:8];
assign NextPkeJ0Low[23:16]= ((a==14'h042) & ce &we[2]) ? d[23:16] : PkeJ0Low[23:16];
assign NextPkeJ0Low[31:24]= ((a==14'h042) & ce &we[3]) ? d[31:24] : PkeJ0Low[31:24];
assign NextPkeJ0High[7:0]  = ((a==14'h043) & ce &we[0]) ? d[7  :0] : PkeJ0High[7:0];
assign NextPkeJ0High[15:8] = ((a==14'h043) & ce &we[1]) ? d[15 :8] : PkeJ0High[15:8];
assign NextPkeJ0High[23:16]= ((a==14'h043) & ce &we[2]) ? d[23:16] : PkeJ0High[23:16];
assign NextPkeJ0High[31:24]= ((a==14'h043) & ce &we[3]) ? d[31:24] : PkeJ0High[31:24];

assign NextNLen[7:0]   	= ((a==14'h044) & ce &we[0]) ? d[7:0]  : NLen[7:0];
assign NextNLen[11:8]  	= ((a==14'h044) & ce &we[1]) ? d[11:8] : NLen[11:8];
assign NextELen[7:0]   	= ((a==14'h045) & ce &we[0]) ? d[7:0]  : ELen[7:0];
assign NextELen[11:8]  	= ((a==14'h045) & ce &we[1]) ? d[11:8] : ELen[11:8];


assign NextXReg[7:0]    = ((a==14'h048) &ce &we[0]) ? d[7  :0] : XReg[7:0]    ; 
assign NextXReg[15:8]   = ((a==14'h048) &ce &we[0]) ? d[15 :8] : XReg[15:8]   ; 
assign NextXReg[23:16]  = ((a==14'h048) &ce &we[0]) ? d[23:16] : XReg[23:16]  ; 
assign NextXReg[31:24]  = ((a==14'h048) &ce &we[0]) ? d[31:24] : XReg[31:24]  ; 
                                                                              
assign NextXReg[39:32]  = ((a==14'h049) &ce &we[0]) ? d[7  :0] : XReg[39:32]  ; 
assign NextXReg[47:40]  = ((a==14'h049) &ce &we[0]) ? d[15 :8] : XReg[47:40]  ; 
assign NextXReg[55:48]  = ((a==14'h049) &ce &we[0]) ? d[23:16] : XReg[55:48]  ; 
assign NextXReg[63:56]  = ((a==14'h049) &ce &we[0]) ? d[31:24] : XReg[63:56]  ; 
                                                                              
assign NextXReg[71:64]  = ((a==14'h04a) &ce &we[0]) ? d[7  :0] : XReg[71:64]  ; 
assign NextXReg[79:72]  = ((a==14'h04a) &ce &we[0]) ? d[15 :8] : XReg[79:72]  ; 
assign NextXReg[87:80]  = ((a==14'h04a) &ce &we[0]) ? d[23:16] : XReg[87:80]  ; 
assign NextXReg[95:88]  = ((a==14'h04a) &ce &we[0]) ? d[31:24] : XReg[95:88]  ; 
                                                                              
assign NextXReg[103:96] = ((a==14'h04b) &ce &we[0]) ? d[7  :0] : XReg[103:96] ; 
assign NextXReg[111:104]= ((a==14'h04b) &ce &we[0]) ? d[15 :8] : XReg[111:104]; 
assign NextXReg[119:112]= ((a==14'h04b) &ce &we[0]) ? d[23:16] : XReg[119:112]; 
assign NextXReg[127:120]= ((a==14'h04b) &ce &we[0]) ? d[31:24] : XReg[127:120];


assign NextYReg[7:0]    = ((a==14'h04c) &ce &we[0]) ? d[7  :0] : YReg[7:0]    ; 
assign NextYReg[15:8]   = ((a==14'h04c) &ce &we[0]) ? d[15 :8] : YReg[15:8]   ; 
assign NextYReg[23:16]  = ((a==14'h04c) &ce &we[0]) ? d[23:16] : YReg[23:16]  ; 
assign NextYReg[31:24]  = ((a==14'h04c) &ce &we[0]) ? d[31:24] : YReg[31:24]  ; 
                                                                            
assign NextYReg[39:32]  = ((a==14'h04d) &ce &we[0]) ? d[7  :0] : YReg[39:32]  ; 
assign NextYReg[47:40]  = ((a==14'h04d) &ce &we[0]) ? d[15 :8] : YReg[47:40]  ; 
assign NextYReg[55:48]  = ((a==14'h04d) &ce &we[0]) ? d[23:16] : YReg[55:48]  ; 
assign NextYReg[63:56]  = ((a==14'h04d) &ce &we[0]) ? d[31:24] : YReg[63:56]  ; 
                                                                            
assign NextYReg[71:64]  = ((a==14'h04e) &ce &we[0]) ? d[7  :0] : YReg[71:64] ; 
assign NextYReg[79:72]  = ((a==14'h04e) &ce &we[0]) ? d[15 :8] : YReg[79:72] ; 
assign NextYReg[87:80]  = ((a==14'h04e) &ce &we[0]) ? d[23:16] : YReg[87:80] ; 
assign NextYReg[95:88]  = ((a==14'h04e) &ce &we[0]) ? d[31:24] : YReg[95:88] ; 
                                                                            
assign NextYReg[103:96] = ((a==14'h04f) &ce &we[0]) ? d[7  :0] : YReg[103:96] ; 
assign NextYReg[111:104]= ((a==14'h04f) &ce &we[0]) ? d[15 :8] : YReg[111:104]; 
assign NextYReg[119:112]= ((a==14'h04f) &ce &we[0]) ? d[23:16] : YReg[119:112]; 
assign NextYReg[127:120]= ((a==14'h04f) &ce &we[0]) ? d[31:24] : YReg[127:120];

assign NextZReg  = GHashDone ? ZOut : ZReg;

assign NextIVector[7:0]    = ((a==14'h054) &ce &we[0]) ? d[7  :0] :
                                                                    IVector[7:0]    ; 
assign NextIVector[15:8]   = ((a==14'h054) &ce &we[0]) ? d[15 :8] : 
                                                                    IVector[15:8]   ; 
assign NextIVector[23:16]  = ((a==14'h054) &ce &we[0]) ? d[23:16] : 
                                                                    IVector[23:16]  ; 
assign NextIVector[31:24]  = ((a==14'h054) &ce &we[0]) ? d[31:24] : 
                                                                    IVector[31:24]  ; 
                                                                            
assign NextIVector[39:32]  = ((a==14'h055) &ce &we[0]) ? d[7  :0] : 
                                                                    IVector[39:32]  ; 
assign NextIVector[47:40]  = ((a==14'h055) &ce &we[0]) ? d[15 :8] : 
                                                                    IVector[47:40]  ; 
assign NextIVector[55:48]  = ((a==14'h055) &ce &we[0]) ? d[23:16] : 
                                                                    IVector[55:48]  ; 
assign NextIVector[63:56]  = ((a==14'h055) &ce &we[0]) ? d[31:24] : 
                                                                    IVector[63:56]  ; 
                                                                            
assign NextIVector[71:64]  = ((a==14'h056) &ce &we[0]) ? d[7  :0] : 
                                                                    IVector[71:64] ; 
assign NextIVector[79:72]  = ((a==14'h056) &ce &we[0]) ? d[15 :8] : 
                                                                    IVector[79:72] ; 
assign NextIVector[87:80]  = ((a==14'h056) &ce &we[0]) ? d[23:16] : 
                                                                    IVector[87:80] ; 
assign NextIVector[95:88]  = ((a==14'h056) &ce &we[0]) ? d[31:24] : 
                                                                    IVector[95:88] ; 
                                                                            
assign NextIVector[103:96] = ((a==14'h057) &ce &we[0]) ? d[7  :0] : 
                                                                    IVector[103:96] ; 
assign NextIVector[111:104]= ((a==14'h057) &ce &we[0]) ? d[15 :8] : 
                                                                    IVector[111:104]; 
assign NextIVector[119:112]= ((a==14'h057) &ce &we[0]) ? d[23:16] : 
                                                                    IVector[119:112]; 
assign NextIVector[127:120]= ((a==14'h057) &ce &we[0]) ? d[31:24] : 
                                                                    IVector[127:120];

assign ShaMode_256 = PkeIR[26:24];
assign ShaMode_512 = PkeIR[26:24];
assign MIn_256     = MInReg[511:0];
assign HIn_256 	   = HInReg[255:0];

assign MIn_512     = MInReg[1023:0];
assign HIn_512 	   = HInReg[511:0];

always @(posedge aclk or negedge hresetn)
if(~hresetn)
begin
    MInReg  <= 1024'h0;
    HInReg  <= 512'h0;
    HOutReg <= 512'h0;
end
else
begin
    MInReg  <= NextMInReg;
    HInReg  <= NextHInReg;
    HOutReg <= NextHOutReg;
end


assign NextMInReg[31      :    0] =((a==14'h080) &ce &we[0]) ? d[31:0] : MInReg[31      :    0]; 
assign NextMInReg[32*2-1  :   32] =((a==14'h081) &ce &we[0]) ? d[31:0] : MInReg[32*2-1  :   32]; 
assign NextMInReg[32*3-1  : 32*2] =((a==14'h082) &ce &we[0]) ? d[31:0] : MInReg[32*3-1  : 32*2]; 
assign NextMInReg[32*4-1  : 32*3] =((a==14'h083) &ce &we[0]) ? d[31:0] : MInReg[32*4-1  : 32*3]; 
assign NextMInReg[32*5-1  : 32*4] =((a==14'h084) &ce &we[0]) ? d[31:0] : MInReg[32*5-1  : 32*4]; 
assign NextMInReg[32*6-1  : 32*5] =((a==14'h085) &ce &we[0]) ? d[31:0] : MInReg[32*6-1  : 32*5]; 
assign NextMInReg[32*7-1  : 32*6] =((a==14'h086) &ce &we[0]) ? d[31:0] : MInReg[32*7-1  : 32*6]; 
assign NextMInReg[32*8-1  : 32*7] =((a==14'h087) &ce &we[0]) ? d[31:0] : MInReg[32*8-1  : 32*7]; 
assign NextMInReg[32*9-1  : 32*8] =((a==14'h088) &ce &we[0]) ? d[31:0] : MInReg[32*9-1  : 32*8]; 
assign NextMInReg[32*10-1 : 32*9] =((a==14'h089) &ce &we[0]) ? d[31:0] : MInReg[32*10-1 : 32*9]; 
assign NextMInReg[32*11-1 :32*10] =((a==14'h08a) &ce &we[0]) ? d[31:0] : MInReg[32*11-1 :32*10]; 
assign NextMInReg[32*12-1 :32*11] =((a==14'h08b) &ce &we[0]) ? d[31:0] : MInReg[32*12-1 :32*11]; 
assign NextMInReg[32*13-1 :32*12] =((a==14'h08c) &ce &we[0]) ? d[31:0] : MInReg[32*13-1 :32*12]; 
assign NextMInReg[32*14-1 :32*13] =((a==14'h08d) &ce &we[0]) ? d[31:0] : MInReg[32*14-1 :32*13]; 
assign NextMInReg[32*15-1 :32*14] =((a==14'h08e) &ce &we[0]) ? d[31:0] : MInReg[32*15-1 :32*14]; 
assign NextMInReg[32*16-1 :32*15] =((a==14'h08f) &ce &we[0]) ? d[31:0] : MInReg[32*16-1 :32*15]; 


assign NextMInReg[32*17-1 :32*16] =((a==14'h090) &ce &we[0]) ? d[31:0] : MInReg[32*17-1 :32*16]; 
assign NextMInReg[32*18-1 :32*17] =((a==14'h091) &ce &we[0]) ? d[31:0] : MInReg[32*18-1 :32*17]; 
assign NextMInReg[32*19-1 :32*18] =((a==14'h092) &ce &we[0]) ? d[31:0] : MInReg[32*19-1 :32*18]; 
assign NextMInReg[32*20-1 :32*19] =((a==14'h093) &ce &we[0]) ? d[31:0] : MInReg[32*20-1 :32*19]; 
assign NextMInReg[32*21-1 :32*20] =((a==14'h094) &ce &we[0]) ? d[31:0] : MInReg[32*21-1 :32*20]; 
assign NextMInReg[32*22-1 :32*21] =((a==14'h095) &ce &we[0]) ? d[31:0] : MInReg[32*22-1 :32*21]; 
assign NextMInReg[32*23-1 :32*22] =((a==14'h096) &ce &we[0]) ? d[31:0] : MInReg[32*23-1 :32*22]; 
assign NextMInReg[32*24-1 :32*23] =((a==14'h097) &ce &we[0]) ? d[31:0] : MInReg[32*24-1 :32*23]; 
assign NextMInReg[32*25-1 :32*24] =((a==14'h098) &ce &we[0]) ? d[31:0] : MInReg[32*25-1 :32*24]; 
assign NextMInReg[32*26-1 :32*25] =((a==14'h099) &ce &we[0]) ? d[31:0] : MInReg[32*26-1 :32*25]; 
assign NextMInReg[32*27-1 :32*26] =((a==14'h09a) &ce &we[0]) ? d[31:0] : MInReg[32*27-1 :32*26]; 
assign NextMInReg[32*28-1 :32*27] =((a==14'h09b) &ce &we[0]) ? d[31:0] : MInReg[32*28-1 :32*27]; 
assign NextMInReg[32*29-1 :32*28] =((a==14'h09c) &ce &we[0]) ? d[31:0] : MInReg[32*29-1 :32*28]; 
assign NextMInReg[32*30-1 :32*29] =((a==14'h09d) &ce &we[0]) ? d[31:0] : MInReg[32*30-1 :32*29]; 
assign NextMInReg[32*31-1 :32*30] =((a==14'h09e) &ce &we[0]) ? d[31:0] : MInReg[32*31-1 :32*30]; 
assign NextMInReg[32*32-1 :32*31] =((a==14'h09f) &ce &we[0]) ? d[31:0] : MInReg[32*32-1 :32*31]; 


assign NextHInReg[31      :    0] =((a==14'h0a0) &ce &we[0]) ? d[31:0] : HInReg[31      :    0]; 
assign NextHInReg[32*2-1  :   32] =((a==14'h0a1) &ce &we[0]) ? d[31:0] : HInReg[32*2-1  :   32]; 
assign NextHInReg[32*3-1  : 32*2] =((a==14'h0a2) &ce &we[0]) ? d[31:0] : HInReg[32*3-1  : 32*2]; 
assign NextHInReg[32*4-1  : 32*3] =((a==14'h0a3) &ce &we[0]) ? d[31:0] : HInReg[32*4-1  : 32*3]; 
assign NextHInReg[32*5-1  : 32*4] =((a==14'h0a4) &ce &we[0]) ? d[31:0] : HInReg[32*5-1  : 32*4]; 
assign NextHInReg[32*6-1  : 32*5] =((a==14'h0a5) &ce &we[0]) ? d[31:0] : HInReg[32*6-1  : 32*5]; 
assign NextHInReg[32*7-1  : 32*6] =((a==14'h0a6) &ce &we[0]) ? d[31:0] : HInReg[32*7-1  : 32*6]; 
assign NextHInReg[32*8-1  : 32*7] =((a==14'h0a7) &ce &we[0]) ? d[31:0] : HInReg[32*8-1  : 32*7]; 
assign NextHInReg[32*9-1  : 32*8] =((a==14'h0a8) &ce &we[0]) ? d[31:0] : HInReg[32*9-1  : 32*8]; 
assign NextHInReg[32*10-1 : 32*9] =((a==14'h0a9) &ce &we[0]) ? d[31:0] : HInReg[32*10-1 : 32*9]; 
assign NextHInReg[32*11-1 :32*10] =((a==14'h0aa) &ce &we[0]) ? d[31:0] : HInReg[32*11-1 :32*10]; 
assign NextHInReg[32*12-1 :32*11] =((a==14'h0ab) &ce &we[0]) ? d[31:0] : HInReg[32*12-1 :32*11]; 
assign NextHInReg[32*13-1 :32*12] =((a==14'h0ac) &ce &we[0]) ? d[31:0] : HInReg[32*13-1 :32*12]; 
assign NextHInReg[32*14-1 :32*13] =((a==14'h0ad) &ce &we[0]) ? d[31:0] : HInReg[32*14-1 :32*13]; 
assign NextHInReg[32*15-1 :32*14] =((a==14'h0ae) &ce &we[0]) ? d[31:0] : HInReg[32*15-1 :32*14]; 
assign NextHInReg[32*16-1 :32*15] =((a==14'h0af) &ce &we[0]) ? d[31:0] : HInReg[32*16-1 :32*15]; 


assign NextHOutReg[255:0] = 
                           Sha256En & EndSha256 ? HOut_256 	  :
                           Sha512En & EndSha512 ? HOut_512[255:0] :
                                                  HOutReg[255:0]  ;

assign NextHOutReg[511:256] =
                           Sha512En & EndSha512 ? HOut_512[511:256] :
                                                  HOutReg[511:256]  ;

assign Sha256En = PkeIR[26] == 1'b0; 
assign Sha512En = PkeIR[26] == 1'b1;

assign ShaStart_256 = ~PkeStatus[0] & NextPkeStatus[0] & PkeIR[7:0]==8'h90;
assign ShaStart_512 = ~PkeStatus[0] & NextPkeStatus[0] & PkeIR[7:0]==8'h91;

wire   ClearStatus;
assign ClearStatus = (a ==14'h047) & ce & we[0] & d[0] ;

assign NextPkeDone 	= PkeStart | AesStart | GHashStart | ShaStart_256 | ShaStart_512 ? 1'b0 	: 
                          PkeInt   ? 1'b1 		:
                          AesDone  ? 1'b1 		: 
                          GHashDone ? 1'b1 		:
                          EndSha256 ? 1'b1              :
                          EndSha512 ? 1'b1              :
                          ClearStatus ? 1'b0            : 
			             PkeDone		;

assign ModInvRdy  = ModInvPkeRdy;

assign PkeJ0 = {PkeJ0High, PkeJ0Low};

// Pke Registers
//
assign aRam00HEn_tmp	= (~a[11] & a[10] & ~a[9] & ~a[0]) & ce;
assign aRam00HWr_tmp[0] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[1] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[2] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[3] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[4] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[5] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[6] = aRam00HEn_tmp & we[0];
assign aRam00HWr_tmp[7] = aRam00HEn_tmp & we[0];

assign aRam00HWr_tmp[8]  = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[9]  = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[10] = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[11] = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[12] = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[13] = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[14] = aRam00HEn_tmp & we[1];
assign aRam00HWr_tmp[15] = aRam00HEn_tmp & we[1];

assign aRam00HWr_tmp[16] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[17] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[18] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[19] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[20] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[21] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[22] = aRam00HEn_tmp & we[2];
assign aRam00HWr_tmp[23] = aRam00HEn_tmp & we[2];

assign aRam00HWr_tmp[24] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[25] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[26] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[27] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[28] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[29] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[30] = aRam00HEn_tmp & we[3];
assign aRam00HWr_tmp[31] = aRam00HEn_tmp & we[3];


assign aRam00HAdr_tmp 	= a[8:1];
assign aRam00HDatIn_tmp	= d;

assign aRam01HEn_tmp	= (~a[11] & a[10] & ~a[9] & a[0] ) & ce;
assign aRam01HWr_tmp[0] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[1] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[2] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[3] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[4] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[5] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[6] = aRam01HEn_tmp & we[0];
assign aRam01HWr_tmp[7] = aRam01HEn_tmp & we[0];

assign aRam01HWr_tmp[8]  = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[9]  = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[10] = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[11] = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[12] = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[13] = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[14] = aRam01HEn_tmp & we[1];
assign aRam01HWr_tmp[15] = aRam01HEn_tmp & we[1];

assign aRam01HWr_tmp[16] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[17] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[18] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[19] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[20] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[21] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[22] = aRam01HEn_tmp & we[2];
assign aRam01HWr_tmp[23] = aRam01HEn_tmp & we[2];

assign aRam01HWr_tmp[24] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[25] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[26] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[27] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[28] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[29] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[30] = aRam01HEn_tmp & we[3];
assign aRam01HWr_tmp[31] = aRam01HEn_tmp & we[3];

assign aRam01HAdr_tmp 	= a[8:1];
assign aRam01HDatIn_tmp	= d;

assign aRam0PEn_tmp	= PkeRamRd0 | PkeRamWr0;
assign aRam0PWr_tmp	= PkeRamWr0;
assign aRam0PAdr_tmp	= PkeRamAddr0;
assign aRam0PDatIn_tmp	= PkeRamDat0;

assign aRam10HEn_tmp	= (~a[11] &a[10] & a[9] & ~a[0]) & ce;
assign aRam10HWr_tmp[0] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[1] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[2] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[3] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[4] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[5] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[6] = aRam10HEn_tmp & we[0];
assign aRam10HWr_tmp[7] = aRam10HEn_tmp & we[0];

assign aRam10HWr_tmp[8]  = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[9]  = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[10] = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[11] = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[12] = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[13] = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[14] = aRam10HEn_tmp & we[1];
assign aRam10HWr_tmp[15] = aRam10HEn_tmp & we[1];

assign aRam10HWr_tmp[16] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[17] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[18] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[19] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[20] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[21] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[22] = aRam10HEn_tmp & we[2];
assign aRam10HWr_tmp[23] = aRam10HEn_tmp & we[2];

assign aRam10HWr_tmp[24] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[25] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[26] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[27] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[28] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[29] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[30] = aRam10HEn_tmp & we[3];
assign aRam10HWr_tmp[31] = aRam10HEn_tmp & we[3];

assign aRam10HAdr_tmp 	= a[8:1];
assign aRam10HDatIn_tmp	= d;

assign aRam11HEn_tmp	= (~a[11] &a[10] & a[9] & a[0]) & ce;
assign aRam11HWr_tmp[0] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[1] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[2] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[3] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[4] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[5] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[6] = aRam11HEn_tmp & we[0];
assign aRam11HWr_tmp[7] = aRam11HEn_tmp & we[0];

assign aRam11HWr_tmp[8]  = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[9]  = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[10] = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[11] = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[12] = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[13] = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[14] = aRam11HEn_tmp & we[1];
assign aRam11HWr_tmp[15] = aRam11HEn_tmp & we[1];

assign aRam11HWr_tmp[16] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[17] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[18] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[19] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[20] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[21] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[22] = aRam11HEn_tmp & we[2];
assign aRam11HWr_tmp[23] = aRam11HEn_tmp & we[2];

assign aRam11HWr_tmp[24] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[25] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[26] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[27] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[28] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[29] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[30] = aRam11HEn_tmp & we[3];
assign aRam11HWr_tmp[31] = aRam11HEn_tmp & we[3];

assign aRam11HAdr_tmp 	= a[8:1];
assign aRam11HDatIn_tmp	= d;

assign aRam1PEn_tmp	= PkeRamRd1 | PkeRamWr1;
assign aRam1PWr_tmp	= PkeRamWr1;
assign aRam1PAdr_tmp	= PkeRamAddr1;
assign aRam1PDatIn_tmp	= PkeRamDat1;

always @(posedge aclk or negedge hresetn)
if(~hresetn)
    PkeEn <= 1'b0;
else
    PkeEn <= NextPkeEn;

assign NextPkeEn = PkeStart ? 1'b1 :
                   PkeInt   ? 1'b0 :
                              PkeEn;

always @(posedge aclk or negedge hresetn)
if(~hresetn)
    AesEn <= 1'b0;
else
    AesEn <= NextAesEn;

assign NextAesEn = AesStart ? 1'b1 :
                   AesDone  ? 1'b0 :
                              AesEn;

assign Ram00CeN = PkeEn ? ~aRam0PEn_tmp    	:
                  AesEn ? ~AesRam00ce	        :
                          ~aRam00HEn_tmp        ;
assign Ram00WeN = PkeEn ? {32{~aRam0PWr_tmp}}   :
                  AesEn ? {32{~AesRam00we}}	:
                           ~aRam00HWr_tmp	;
assign Ram00Adr = PkeEn ? aRam0PAdr_tmp    	:
                  AesEn ? AesRam00Adr		: 
                          aRam00HAdr_tmp	;
assign Ram00Din = PkeEn ? aRam0PDatIn_tmp[31:0] :
                  AesEn & AesOFB? AesRamDat1	:
                  AesEn & ~AesOFB ? AesRamDat   :
                          aRam00HDatIn_tmp	;

assign Ram01CeN = PkeEn ? ~aRam0PEn_tmp    :
                  AesEn ? ~AesRam01ce	        :
                          ~aRam01HEn_tmp;
assign Ram01WeN = PkeEn ? {32{~aRam0PWr_tmp}}    :
                  AesEn ? {32{~AesRam01we}}	:
                          ~aRam01HWr_tmp;
assign Ram01Adr = PkeEn ? aRam0PAdr_tmp    :  
                  AesEn ? AesRam01Adr		: 
                          aRam01HAdr_tmp;
assign Ram01Din = PkeEn ? aRam0PDatIn_tmp[63:32]  :  
                  AesEn & AesOFB? AesRamDat1	:
                  AesEn & ~AesOFB ? AesRamDat   :
                          aRam01HDatIn_tmp;
assign aRam00HDatOut = Ram00Dout;
assign aRam01HDatOut = Ram01Dout;
assign aRam0PDatOut = {Ram01Dout,Ram00Dout};

assign RamAesDat   = 
                    AesRam00ce_reg ? Ram00Dout  :
                    AesRam01ce_reg ? Ram01Dout  :
                    AesRam10ce_reg ? Ram10Dout  :
                    AesRam11ce_reg ? Ram11Dout  :
                                     32'h0000   ;

assign Ram10CeN = PkeEn ? ~aRam1PEn_tmp         :
                  AesEn ? ~AesRam10ce	        :
                          ~aRam10HEn_tmp        ;
assign Ram10WeN = PkeEn ? {32{~aRam1PWr_tmp}}   : 
                  AesEn ? {32{~AesRam10we}}	:
                          ~aRam10HWr_tmp        ;
assign Ram10Adr = PkeEn ? aRam1PAdr_tmp         :  
                  AesEn ? AesRam10Adr		: 
                          aRam10HAdr_tmp        ;
assign Ram10Din = PkeEn ? aRam1PDatIn_tmp[31:0] :  
                  AesEn ? AesRamDat		:
                          aRam10HDatIn_tmp      ;
assign Ram11CeN = PkeEn ? ~aRam1PEn_tmp         : 
                  AesEn ? ~AesRam11ce	        :
                          ~aRam11HEn_tmp        ;
assign Ram11WeN = PkeEn ? {32{~aRam1PWr_tmp}}   : 
                  AesEn ? {32{~AesRam11we}}	:
                          ~aRam11HWr_tmp        ;
assign Ram11Adr = PkeEn ? aRam1PAdr_tmp         : 
                  AesEn ? AesRam11Adr		: 
                          aRam11HAdr_tmp;
assign Ram11Din = PkeEn ? aRam1PDatIn_tmp[63:32]: 
                  AesEn ? AesRamDat		:
                          aRam11HDatIn_tmp;


assign aRam10HDatOut = Ram10Dout;
assign aRam11HDatOut = Ram11Dout;
assign aRam1PDatOut = {Ram11Dout,Ram10Dout};

sram128X32C2V4_wrp PkeRam00(
   .Q		(Ram00Dout	),
   .CLK		(r0clk		),
   .CEN		(Ram00CeN	),
   .WEB		(Ram00WeN	),
   .A		(Ram00Adr	),
   .D		(Ram00Din	),
   .OEN		(1'b0		)
);

sram128X32C2V4_wrp PkeRam01(
   .Q		(Ram01Dout       ),
   .CLK		(r1clk		),
   .CEN		(Ram01CeN	),
   .WEB		(Ram01WeN	),
   .A		(Ram01Adr	),
   .D		(Ram01Din      	),
   .OEN		(1'b0		)
);


sram128X32C2V4_wrp PkeRam10(
   .Q		(Ram10Dout	),
   .CLK		(r2clk		),
   .CEN		(Ram10CeN	),
   .WEB		(Ram10WeN	),
   .A		(Ram10Adr	),
   .D		(Ram10Din	),
   .OEN		(1'b0		)
);

sram128X32C2V4_wrp PkeRam11(
   .Q		(Ram11Dout       ),
   .CLK		(r3clk		),
   .CEN		(Ram11CeN	),
   .WEB		(Ram11WeN	),
   .A		(Ram11Adr	),
   .D		(Ram11Din	),
   .OEN		(1'b0		)
);


assign RamPkeDat0 = aRam0PDatOut;
assign RamPkeDat1 = aRam1PDatOut;

reg [31:0] RegOut;
wire  [31:0] NextRegOut;
always @(posedge hclk or negedge hresetn)
if(~hresetn)
    RegOut <= 32'h0;
else  
    RegOut <= NextRegOut;   

assign NextRegOut = (a==14'h047 ) ? {ModInvRdy,PkeDone} :  //zxjian,20220317
                    (a==14'h040)  ? PkeStatus      	:
                    (a==14'h041)  ? PkeIR          	:			  
                    (a==14'h042)  ? PkeJ0Low       	:
                    (a==14'h043)  ? PkeJ0High      	:
                    (a==14'h044)  ? NLen           	:
                    (a==14'h045)  ? ELen           	:
//zxjian,20220317
                    (a==14'h048)  ? XReg[31:0]          :
                    (a==14'h049)  ? XReg[63:32]         :
                    (a==14'h04a)  ? XReg[95:64]         :
                    (a==14'h04b)  ? XReg[127:96]        :
                    (a==14'h04c)  ? YReg[31:0]          :
                    (a==14'h04d)  ? YReg[63:32]         :
                    (a==14'h04e)  ? YReg[95:64]         :
                    (a==14'h04f)  ? YReg[127:96]        :
                    (a==14'h050)  ? ZReg[31:0]          :
                    (a==14'h051)  ? ZReg[63:32]         :
                    (a==14'h052)  ? ZReg[95:64]         :
                    (a==14'h053)  ? ZReg[127:96]        :
                    (a==14'h054)  ? IVector[31:0]       :
                    (a==14'h055)  ? IVector[63:32]      :
                    (a==14'h056)  ? IVector[95:64]      :
                    (a==14'h057)  ? IVector[127:96]     :
//zxjian,20220608
                    //maybe deleted if timming can't met                
                    (a==14'h080)  ? MInReg[31      :    0]:
                    (a==14'h081)  ? MInReg[32*2-1  :   32]:
                    (a==14'h082)  ? MInReg[32*3-1  : 32*2]:
                    (a==14'h083)  ? MInReg[32*4-1  : 32*3]:
                    (a==14'h084)  ? MInReg[32*5-1  : 32*4]:
                    (a==14'h085)  ? MInReg[32*6-1  : 32*5]:
                    (a==14'h086)  ? MInReg[32*7-1  : 32*6]:
                    (a==14'h087)  ? MInReg[32*8-1  : 32*7]:
                    (a==14'h088)  ? MInReg[32*9-1  : 32*8]:
                    (a==14'h089)  ? MInReg[32*10-1 : 32*9]:
                    (a==14'h08a)  ? MInReg[32*11-1 :32*10]:
                    (a==14'h08b)  ? MInReg[32*12-1 :32*11]:
                    (a==14'h08c)  ? MInReg[32*13-1 :32*12]:
                    (a==14'h08d)  ? MInReg[32*14-1 :32*13]:
                    (a==14'h08e)  ? MInReg[32*15-1 :32*14]:
                    (a==14'h08f)  ? MInReg[32*16-1 :32*15]:
                    //maybe deleted if timming can't met                
                    (a==14'h090)  ? MInReg[32*17-1 :32*16]:                       
                    (a==14'h091)  ? MInReg[32*18-1 :32*17]:
                    (a==14'h092)  ? MInReg[32*19-1 :32*18]:
                    (a==14'h093)  ? MInReg[32*20-1 :32*19]:
                    (a==14'h094)  ? MInReg[32*21-1 :32*20]:
                    (a==14'h095)  ? MInReg[32*22-1 :32*21]:
                    (a==14'h096)  ? MInReg[32*23-1 :32*22]:
                    (a==14'h097)  ? MInReg[32*24-1 :32*23]:
                    (a==14'h098)  ? MInReg[32*25-1 :32*24]:
                    (a==14'h099)  ? MInReg[32*26-1 :32*25]:
                    (a==14'h09a)  ? MInReg[32*27-1 :32*26]:
                    (a==14'h09b)  ? MInReg[32*28-1 :32*27]:
                    (a==14'h09c)  ? MInReg[32*29-1 :32*28]:
                    (a==14'h09d)  ? MInReg[32*30-1 :32*29]:
                    (a==14'h09e)  ? MInReg[32*31-1 :32*30]:
                    (a==14'h09f)  ? MInReg[32*32-1 :32*31]:
                    //maybe deleted if timming can't met                
                    (a==14'h0a0)  ? HInReg[31      :    0]:
                    (a==14'h0a1)  ? HInReg[32*2-1  :   32]:
                    (a==14'h0a2)  ? HInReg[32*3-1  : 32*2]:
                    (a==14'h0a3)  ? HInReg[32*4-1  : 32*3]:
                    (a==14'h0a4)  ? HInReg[32*5-1  : 32*4]:
                    (a==14'h0a5)  ? HInReg[32*6-1  : 32*5]:
                    (a==14'h0a6)  ? HInReg[32*7-1  : 32*6]:
                    (a==14'h0a7)  ? HInReg[32*8-1  : 32*7]:
                    (a==14'h0a8)  ? HInReg[32*9-1  : 32*8]:
                    (a==14'h0a9)  ? HInReg[32*10-1 : 32*9]:
                    (a==14'h0aa)  ? HInReg[32*11-1 :32*10]:
                    (a==14'h0ab)  ? HInReg[32*12-1 :32*11]:
                    (a==14'h0ac)  ? HInReg[32*13-1 :32*12]:
                    (a==14'h0ad)  ? HInReg[32*14-1 :32*13]:
                    (a==14'h0ae)  ? HInReg[32*15-1 :32*14]:
                    (a==14'h0af)  ? HInReg[32*16-1 :32*15]:
                                  
                    (a==14'h0b0)  ? HOutReg[31      :    0]:
                    (a==14'h0b1)  ? HOutReg[32*2-1  :   32]:
                    (a==14'h0b2)  ? HOutReg[32*3-1  : 32*2]:
                    (a==14'h0b3)  ? HOutReg[32*4-1  : 32*3]:
                    (a==14'h0b4)  ? HOutReg[32*5-1  : 32*4]:
                    (a==14'h0b5)  ? HOutReg[32*6-1  : 32*5]:
                    (a==14'h0b6)  ? HOutReg[32*7-1  : 32*6]:
                    (a==14'h0b7)  ? HOutReg[32*8-1  : 32*7]:
                    (a==14'h0b8)  ? HOutReg[32*9-1  : 32*8]:
                    (a==14'h0b9)  ? HOutReg[32*10-1 : 32*9]:
                    (a==14'h0ba)  ? HOutReg[32*11-1 :32*10]:
                    (a==14'h0bb)  ? HOutReg[32*12-1 :32*11]:
                    (a==14'h0bc)  ? HOutReg[32*13-1 :32*12]:
                    (a==14'h0bd)  ? HOutReg[32*14-1 :32*13]:
                    (a==14'h0be)  ? HOutReg[32*15-1 :32*14]:
                    (a==14'h0bf)  ? HOutReg[32*16-1 :32*15]:

			            32'h0          	;


//Result from DataOut Reg or from FifoOut
assign q		= 
                  (~aReg[11]& aReg[10]&~aReg[9]&~aReg[0])? aRam00HDatOut:
                  (~aReg[11]& aReg[10]&~aReg[9]&aReg[0] )? aRam01HDatOut:
                  (~aReg[11]&aReg[10]&aReg[9]& ~aReg[0] )? aRam10HDatOut:
                  (~aReg[11]&aReg[10]&aReg[9]& aReg[0]  )? aRam11HDatOut:
				                                 RegOut;

//Pke Core
PkeCore   uPke(
            .Clk              (pkeclk		),
            .Resetn           (hresetn		),
//	    .ModMulJ0         (PkeQ		),
            .PkeIR            (PkeIR[7:0]     	),
            .NLen             (NLen       	),
            .ELen             (ELen       	),
            .PkeStart         (PkeStart   	),
            .RamPkeDat0       (RamPkeDat0  	),
            .RamPkeDat1       (RamPkeDat1  	),
            .PkeInt           (PkeInt		),
            .ModInvRdy        (ModInvPkeRdy	),

            .N0Dat	      (PkeJ0		),
//            .PkeQ	      (PkeQ		),

            .PkeRamRd0        (PkeRamRd0     	),
            .PkeRamWr0        (PkeRamWr0     	),
            .PkeRamAddr0      (PkeRamAddr0   	),
            .PkeRamDat0       (PkeRamDat0    	),
            .PkeRamRd1        (PkeRamRd1     	),
            .PkeRamWr1        (PkeRamWr1     	),
            .PkeRamAddr1      (PkeRamAddr1   	),
            .PkeRamDat1       (PkeRamDat1    	)
          );

// AesCore
//
AesCore uAesCore(
        .Clk		(aesclk		),
	.Resetn		(hresetn	),
	.StartAes	(AesStart 	),
	.AesIR		(AesIR		),
	.AesLen		(AesLen		),
        .AesMode        (AesMode        ),
	.IVector0       (IVector[31:0]	),  
	.IVector1       (IVector[63:32]	),  
	.IVector2       (IVector[95:64]	),  
	.IVector3       (IVector[127:96]),  
		
	.AesRamRd	(AesRamRd 	),
	.AesRamWr	(AesRamWr 	),
	.AesRamAdr	(AesRamAdr	),
	.AesRamDat	(AesRamDat	),
	.AesRamDat1	(AesRamDat1	),
	.RamAesDat	(RamAesDat	),

	.AesDone	(AesDone	)
        );

//assign AesRamRd =0;
//assign AesRamWr =0;
//assign AesRamAdr =0;
//assign AesRamDat =0;
//assign AesDone=0;

// GHashCore
//
GHash uGHash(
        .Clk		(aesclk		),
	.Resetn		(hresetn	),
	.StartGHash     (GHashStart 	),
        .XIn		(XReg		),
	.YIn		(YReg		),

	.ZOut		(ZOut		),
	.EndGHash	(GHashDone	)
        );


//Sha1/224/256

Sha  Sha256(
        .Clk		(aclk		),
        .Resetn		(hresetn	),
        .StartSha	(ShaStart_256	),
        .ShaMode	(ShaMode_256	),
        .MIn		(MIn_256	),
        .HIn		(HIn_256	),
            
     	.HOut		(HOut_256	),
        .EndSha		(EndSha256	)
        );



//Sha384/512

Sha512 Sha512(
        .Clk		(aclk		),
        .Resetn		(hresetn	),
        .StartSha	(ShaStart_512	),
        .ShaMode	(ShaMode_512	),
        .MIn		(MIn_512	),
        .HIn		(HIn_512	),
            
     	.HOut		(HOut_512	),
        .EndSha		(EndSha512	)
            );

endmodule
