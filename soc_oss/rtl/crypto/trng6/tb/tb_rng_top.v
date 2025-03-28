module tb_rng_top();
reg                   clk;
reg                   rstn;
reg  [ANA_NUM-1:0]    clk_ana;
reg  [ANA_NUM-1:0]    ana_en;
reg  [ANA_NUM-1:0]    ana_data;
reg  [ANA_NUM-1:0]    ana_vld;
reg                   partityfilter_en;
reg                   rngcore_en;
reg                   rngcore_rddone;
reg                   trng_drng_sel;
reg  [1:0]            generate_interval;
reg  [1:0]            reseed_interval;
reg  [1:0]            postprocess_opt;
reg                   additional_input_gen_en;
reg  [255:0]          additional_input_generate;
reg  [255:0]          additional_input_reseed;
reg  [255:0]          personalization_string;
reg                   buf_read;
reg                   buf_write;
reg  [2:0]            buf_addr;
reg  [31:0]           buf_datain;
reg                   healthtest_en;
reg  [5:0]            healthtest_length;
reg  [31:0]           buf_datain_matrix[7:0];

wire                  healthtest_err;
wire [31:0]           buf_dataout;
wire                  drng_reseed_req;
wire [127:0]          rngcore_dataout;
wire                  rngcore_dataout_vld;
wire                  buf_ready; 
integer               i,j,k;


parameter GENERATE_0=4'd1;
parameter GENERATE_1=4'd2;
parameter GENERATE_2=4'd4;
parameter GENERATE_3=4'd8;
parameter RESEED_1  =11'd1;
parameter RESEED_2  =11'd128;
parameter RESEED_3  =11'd1024;
parameter ANA_NUM   =4'd8;


initial begin
clk=0;
rstn=1;
clk_ana={ANA_NUM{1'b0}};
ana_en={ANA_NUM{1'b1}};
ana_data={ANA_NUM{1'b1}};
ana_vld={ANA_NUM{1'b1}};
partityfilter_en=1'b0;
rngcore_en=1'b0;
rngcore_rddone=1'b0;
trng_drng_sel=1'b0;
generate_interval=2'd2;
reseed_interval=2'd1;
postprocess_opt=2'd0;
additional_input_gen_en=1'h1;
additional_input_generate=256'h0;
additional_input_reseed=256'h0;
personalization_string=256'h0;
buf_read=1'h0;
buf_write=1'h0;
buf_addr=3'h0;
buf_datain=32'h0;
healthtest_en=1'h0;
healthtest_length=6'h20;

for(i=0;i<8;i=i+1)begin
    buf_datain_matrix[i]=32'h0;
end

#100
rstn=0;
#100
rstn=1;


$display("~~~~~~~~~1.GET analog raw Data~~~~~~~~~");
$display("analog data go through digilization and save in buf");
$display("read buf_dataout out when 256 bits buf is full");
//select trng mode
trng_drng_sel=1'b0;
//disable rngcore_en
rngcore_en=1'b0;
//enable analog
ana_en={ANA_NUM{1'b1}};
ana_vld={ANA_NUM{1'b1}};
//configure parityfilter
partityfilter_en=1'h0;

for(j=0;j<10;j=j+1)begin
$display("partity=0");
$display("---j=%d---",j);
//wait for buf_ready
    @(posedge buf_ready)
//read out data saved in buf
    for(i=0;i<8;i=i+1)begin
        @(posedge clk)
        buf_read=1'b1;
        buf_addr=i;
        @(posedge clk)
        buf_read=1'b0;
        $display("addr=%d,dataout=%h",i,buf_dataout);
    end
end

//configure parityfilter
partityfilter_en=1'h1;
for(j=0;j<10;j=j+1)begin
$display("partity=1");
$display("---j=%d---",j);
//wait for buf_ready
    @(posedge buf_ready)
//read out data saved in buf
    for(i=0;i<8;i=i+1)begin
        @(posedge clk)
        buf_read=1'b1;
        buf_addr=i;
        @(posedge clk)
        buf_read=1'b0;
        $display("addr=%d,dataout=%h",i,buf_dataout);
    end
end

$display("~~~~~~~~~2.TRNG+LFSR~~~~~~~~~");
$display("analog data go through digilization then output to LFSR process");
$display("read rngcore_dataout when process is done");
#1003
//select trng mode
trng_drng_sel=1'b0;
//select process option= LFSR129
postprocess_opt=2'd0;
//configure interval parameter
generate_interval=2'd2;
reseed_interval=2'd1;
//enable rngcore_en
rngcore_en=1'b1;

for(i=0;i<100;i=i+1)begin
//wait for rngcore_dataout_vld
    @(posedge rngcore_dataout_vld)
//read rngcore_dataout
    @(posedge clk)
    rngcore_rddone=1'b1;
    @(posedge clk)
    rngcore_rddone=1'b0;
    $display("i=%d,dataout=%h",i,rngcore_dataout);
end
//disable rngcore_en
rngcore_en=1'b0;

$display("~~~~~~~~~3.TRNG+AES128-AUTO~~~~~~~~~");
$display("analog data go through digilization then output to AES-128 auto process");
$display("read rngcore_dataout when process is done");
#1003
//select trng mode
trng_drng_sel=1'b0;
//select process option= DRBG_AES128 auto
postprocess_opt=2'd2;
//configure interval parameter
generate_interval=2'd2;
reseed_interval=2'd1;
//configure additional_input and personalization_string
additional_input_gen_en=1'h1;
additional_input_generate=256'h0;
additional_input_reseed=256'h0;
personalization_string=256'h0;
//enable rngcore_en
rngcore_en=1'b1;

for(i=0;i<100;i=i+1)begin
//wait for rngcore_dataout_vld
    @(posedge rngcore_dataout_vld)
//read rngcore_dataout
    @(posedge clk)
    rngcore_rddone=1'b1;
    @(posedge clk)
    rngcore_rddone=1'b0;
    $display("i=%d,dataout=%h",i,rngcore_dataout);
    @(posedge clk)
    ;
end
//disable rngcore_en
rngcore_en=1'b0;

$display("~~~~~~~~~4.DRNG seed confiugre~~~~~~~~~");
$display("In DRNG mode, 256 bits seed configure into buf");
$display("prepare 32*8 bit random bits");
for(i=0;i<8;i=i+1)begin
    buf_datain_matrix[i]=$random%32'hffffffff;
end

$display("----DRNG+Write buf----");
//select drng mode
trng_drng_sel=1'b1;
//disable rngcore_en
rngcore_en=1'b0;
//write in seed
for(i=0;i<8;i=i+1)begin
    @(posedge clk)
    buf_write=1'b1;
    buf_addr=i;
    buf_datain=buf_datain_matrix[i];
    @(posedge clk)
    buf_write=1'b0;
end

$display("In DRNG mode,256 bits seed read out from buf");
$display("----DRNG+Read buf----");
//read out seed
for(i=0;i<8;i=i+1)begin
    @(posedge clk)
    buf_read=1'b1;
    buf_addr=i;
    @(posedge clk)
    buf_read=1'b0;
    $display("i=%d,buf dataout=%h",i,buf_dataout);
//compare read out with write in 
    if(buf_dataout==buf_datain_matrix[i])
        $display("compare pass");
    else
        $display("compare fail");
end
//clear matrix
for(i=0;i<8;i=i+1)begin
    buf_datain_matrix[i]=32'h0;
end
#1003



$display("~~~~~~~~~5.DRNG+LFSR~~~~~~~~~");
$display("In DRNG mode, 256 bits seed configure into buf then output to LFSR process");
$display("read rngcore_dataout when process is done");
//select drng mode
trng_drng_sel=1'b1;
//select process option= LFSR129
postprocess_opt=2'd0;
//configure interval parameter
reseed_interval=2'd2;
//enable rngcore_en
rngcore_en=1'b1;
//write in DRNG seed
$display("----DRNG write seed----");
for(i=0;i<8;i=i+1)begin
    @(posedge clk)
    buf_write=1'b1;
    buf_addr=i;
    buf_datain=$random%32'hffffffff;
    @(posedge clk)
    buf_write=1'b0;
end
//wirte done
@(posedge buf_ready)
$display("----BUF ready----");

for(i=0;i<100;i=i+1)begin
//wait for rngcore_dataout_vld
    @(posedge rngcore_dataout_vld)
//read rngcore_dataout
    @(posedge clk)
    rngcore_rddone=1'b1;
    @(posedge clk)
    rngcore_rddone=1'b0;
    $display("i=%d,dataout=%h",i,rngcore_dataout);
end
//disable rngcore_en
rngcore_en=1'b0;


$display("~~~~~~~~~6.DRNG+AES128-MANUAL~~~~~~~~~");
$display("In DRNG mode, 256 bits seed configure into buf then output to AES128 manual process");
$display("AES128 manual process include AES-128 encrytion calculation twice");
$display("read rngcore_dataout when process is done");
//select drng mode
trng_drng_sel=1'b1;
//select process option= AES128-MANUAL
postprocess_opt=2'd1;
//configure interval parameter
reseed_interval=2'd2;
//disable rngcore_en
rngcore_en=1'b0;

for(j=0;j<10;j=j+1)begin
    $display("---j=%d---",j);
//write in DRNG seed	
    for(i=0;i<8;i=i+1)begin
        @(posedge clk)
        buf_write=1'b1;
        buf_addr=i;
        buf_datain=j;
        buf_datain_matrix[i]=buf_datain;
        @(posedge clk)
        buf_write=1'b0;
    end
    $display("datain=%h%h%h%h%h%h%h%h",buf_datain_matrix[0],buf_datain_matrix[1],buf_datain_matrix[2],buf_datain_matrix[3],
                                       buf_datain_matrix[4],buf_datain_matrix[5],buf_datain_matrix[6],buf_datain_matrix[7]);
    #1003
//enable rngcore_en
rngcore_en=1'b1;
//wait for rngcore_dataout_vld
    @(posedge rngcore_dataout_vld)
//read rngcore_dataout twice
    @(posedge clk)   //1st
    rngcore_rddone=1'b1;
    @(posedge clk)
    rngcore_rddone=1'b0;
    $display("dataout1=%h",rngcore_dataout);
    @(posedge clk)   //2nd
    rngcore_rddone=1'b1;
    @(posedge clk)
    rngcore_rddone=1'b0;
    $display("dataout2=%h",rngcore_dataout);
//disable rngcore_en
    rngcore_en=1'b0;
end

$display("~~~~~~~~~7.HealthTest~~~~~~~~~");
$display("Healthtest check data output from digilization ");
$display("Once continuously 1 or 0 bits over healthtest_length, healthtest_err is set");
//configure parityfilten
partityfilter_en=1'h0;
//enable healthtest_en
healthtest_en=1'h1;
//enable analog
ana_en={ANA_NUM{1'b1}};
//force ana_data=serial 1
force ana_data={ANA_NUM{1'b1}};
ana_vld={ANA_NUM{1'b1}};
//wait for healthtest_err
wait(healthtest_err==1'b1)
$display("HealthErr alarm");
//release ana_data
release ana_data;

#10000
$finish;
end

always begin
#500
ana_data=$random%{ANA_NUM{1'b1}};
end

always begin
    #250
    for(k=0;k<ANA_NUM;k=k+1)begin    
        clk_ana[k]=~clk_ana[k];
    end
end

always begin
clk=~clk;
#10;
end

//initial begin
//$fsdbDumpfile("rng.fsdb");
//$fsdbDumpvars;
//end


rng_top #(
	      .GENERATE_0(GENERATE_0),
 	      .GENERATE_1(GENERATE_1),
    	      .GENERATE_2(GENERATE_2),
   	      .GENERATE_3(GENERATE_3),
              .RESEED_1  (RESEED_1  ),
              .RESEED_2  (RESEED_2  ),
              .RESEED_3  (RESEED_3  ),
	      .ANA_NUM   (ANA_NUM   ))
u_rng_top(
    .clk                           (clk                        ),
    .rstn                          (rstn                       ),
    .clk_ana                       (clk_ana                    ),
    .ana_en                        (ana_en                     ),
    .ana_data                      (ana_data                   ),
    .ana_vld                       (ana_vld                    ),
    .partityfilter_en              (partityfilter_en           ),
    .rngcore_en                    (rngcore_en                 ),
    .rngcore_rddone                (rngcore_rddone             ),
    .trng_drng_sel                 (trng_drng_sel              ),
    .generate_interval             (generate_interval          ),
    .reseed_interval               (reseed_interval            ),
    .postprocess_opt               (postprocess_opt            ),
    .additional_input_gen_en       (additional_input_gen_en    ),
    .additional_input_generate     (additional_input_generate  ),
    .additional_input_reseed       (additional_input_reseed    ),
    .personalization_string        (personalization_string     ),
    .buf_read                      (buf_read                   ),
    .buf_write                     (buf_write                  ),
    .buf_addr                      (buf_addr                   ),
    .buf_datain                    (buf_datain                 ),
    .buf_ready                     (buf_ready                  ),
    .healthtest_en                 (healthtest_en              ),
    .healthtest_length             (healthtest_length          ),
    .healthtest_err                (healthtest_err             ),
    .buf_dataout                   (buf_dataout                ),
    .drng_reseed_req               (drng_reseed_req            ),
    .rngcore_dataout               (rngcore_dataout            ),
    .rngcore_dataout_vld           (rngcore_dataout_vld        )
);
endmodule     
