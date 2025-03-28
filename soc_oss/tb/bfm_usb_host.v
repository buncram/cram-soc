`timescale 1ps/1ps

module bfm_usb_host (
  inout    DP,
  inout    DM
  );

reg        OE;
reg        DP_OUT;
reg        DM_OUT;

static reg cur_state=1;
static int frame_num=0;
static  reg [2:0] one_cnt=0;

parameter  EOP_PERIOD = 2.5*1000*100;
parameter  RESET_PERIOD = 15*1000*1000*1000;
parameter  PERIOD = 83333;

initial begin
  OE <= 1;  
end

assign   DP   =  OE ? DP_OUT : 1'bz;
assign   DM   =  OE ? DM_OUT : 1'bz;

task usb_idle;
   DP_OUT   =  1;
   DM_OUT   =  0;
   cur_state =1;
   #166666;
   //#PERIOD*2;
endtask

task usb_j;
   DP_OUT   =  1;
   DM_OUT   =  0;
   cur_state =1;
   #PERIOD;
endtask

task usb_k;
   DP_OUT   =  0;
   DM_OUT   =  1;
   cur_state =0;
   #PERIOD;
endtask

task usb_reset;
   DP_OUT   =  0;
   DM_OUT   =  0;
   #RESET_PERIOD;
endtask

task usb_se0;
   DP_OUT   =  0;
   DM_OUT   =  0;
   #166666;
endtask


task usb_se1;
   DP_OUT   =  1;
   DM_OUT   =  1;
   #PERIOD;
endtask

task usb_EOP;
  usb_se0;

endtask

task usb_sof(
  input [10:0] frame_num,
  input [4:0]  crc
  );
  reg   cur_bit; 
  reg   next_bit; 
  reg[4:0] crc;
  reg[4:0] temp;
  integer i;
//8'h80
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_k;
//8'h5A
  usb_k;
  usb_j;
  usb_j;
  usb_k;
  usb_j;
  usb_j;
  usb_k;
  usb_k;
  
  cur_state= 0;
  next_bit = 0;
  //crc = crc5(frame_num,5'H1F);
  
  for (i=0;i<11;i++) begin
    next_bit = (frame_num>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
  
  for (i=0;i<5;i++) begin
      temp     =crc>>i;
      next_bit = temp[0];
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
    
  usb_EOP;

  usb_idle;

endtask

task usb_setup(
  input [6:0] addr,
  input [3:0] endp,
  input [4:0] crc
  );
  reg   cur_bit; 
  reg   next_bit; 
  reg[4:0] crc;
  reg[4:0] temp;
  integer i;
//8'h80
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_k;
  cur_state=0;
//8'h2d
  for (i=0;i<8;i++) begin
    next_bit = (8'h2d>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
// send addr
  for (i=0;i<7;i++) begin
    next_bit = (addr>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
// send endp
  for (i=0;i<4;i++) begin
    next_bit = (endp>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
//send crc
  //crc = crc5({endp,addr},5'h1F);
  for (i=0;i<5;i++) begin
      temp     =crc>>i;
      next_bit = temp[0];
    //next_bit = (crc>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
    
  usb_EOP;

  usb_idle;

endtask


task usb_token_in(
  input [6:0] addr,
  input [3:0] endp,
  input [4:0] crc
  );
  reg   cur_bit; 
  reg   next_bit; 
  reg[4:0] crc;
  reg[4:0] temp;
  integer i;
//8'h80
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_k;
  cur_state=0;
//8'h69
  for (i=0;i<8;i++) begin
    next_bit = (8'h69>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
// send addr
  for (i=0;i<7;i++) begin
    next_bit = (addr>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
// send endp
  for (i=0;i<4;i++) begin
    next_bit = (endp>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
//send crc
  //crc = crc5({endp,addr},5'h1F);
  for (i=0;i<5;i++) begin
      temp     =crc>>i;
      next_bit = temp[0];
    //next_bit = (crc>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
    end
    else begin
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
    
  usb_EOP;

  usb_idle;

endtask

task usb_setup_data0(
  input [7:0] data0[],
  input [15:0] crc
  );
  reg         next_bit; 
  reg [15:0]  temp; 
  
  integer     i;
  integer     j;
  one_cnt = 0;
//8'h80
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_j;
  usb_k;
  usb_k;
  one_cnt=1;
  cur_state=0;
//8'hC3 setup data0
  for (i=0;i<8;i++) begin
    next_bit = (8'hC3>>i);
    if (next_bit==1) begin
      if(cur_state==0) begin
        usb_k;
        cur_state = 0;
      end
      else begin
        usb_j;
        cur_state = 1;
      end
      one_cnt=one_cnt +1;
    end
    else begin
      one_cnt=0;
      if(cur_state==0) begin
        usb_j;
        cur_state = 1;
      end
      else begin
        usb_k;
        cur_state = 0;
      end
    end
  end
  one_cnt=2;
  //next_crc = 16'hFFFF;
// data0
//for (j=0;j<data0.size;j++) begin
  foreach(data0[j]) begin
    for (i=0;i<8;i++) begin
      temp[7:0]=data0[j]>>i;
      next_bit = temp[0];
      if (next_bit==1) begin
        if(one_cnt<6) begin
          if(cur_state==0) begin
            usb_k;
            cur_state = 0;
          end
          else begin
            usb_j;
            cur_state = 1;
          end
          one_cnt= one_cnt +1;
        end
        else begin
          if(cur_state==0) begin
            usb_j;
            usb_j;
            cur_state = 1;
            one_cnt= 1;
          end
          else begin
            usb_k;
            usb_k;
            cur_state = 0;
          end
          one_cnt= 1;
        end
      end
      else begin
        if(cur_state==0) begin
          usb_j;
          cur_state = 1;
        end
        else begin
          usb_k;
          cur_state = 0;
        end
        one_cnt= 0;
      end
    end
  end
//send crc
  usb_j;//0
  usb_k;//0
  usb_k;//1
  usb_k;//1
  usb_j;//0
  usb_j;//1
  usb_j;//1
  usb_j;//1

  usb_j;//1
  usb_j;//1
  usb_j;//1
  usb_k;//0
  usb_k;//1
  usb_j;//0
  usb_j;//1
  usb_j;//1
  usb_j;//1

  //for(i=0;i<16;i++) begin
  //    //temp     = (crc>>i);
  //    //next_bit = temp[0];
  //    next_bit = crc[i];
  //    $display("Cur bit=%h",next_bit);
  //    if (next_bit==1) begin
  //      if(one_cnt<6) begin
  //        if(cur_state==0) begin
  //          usb_k;
  //          cur_state = 0;
  //        end
  //        else begin
  //          usb_j;
  //          cur_state = 1;
  //        end
  //        one_cnt= one_cnt +1;
  //      end
  //      else begin
  //        if(cur_state==0) begin
  //          usb_j;
  //          usb_j;
  //          cur_state = 1;
  //        end
  //        else begin
  //          usb_k;
  //          usb_k;
  //          cur_state = 0;
  //        end
  //        one_cnt= 1;
  //      end
  //    end
  //end
    
  usb_EOP;

  usb_idle;

endtask

  task usb_ack;
    reg   cur_bit; 
    reg   next_bit; 
    reg[4:0] crc;
    integer i;
  //8'h80
    usb_j;
    usb_k;
    usb_j;
    usb_k;
    usb_j;
    usb_k;
    usb_j;
    usb_k;
    usb_k;
    cur_state=0;

    //OE=0;
  //8'h4b
    for (i=0;i<8;i++) begin
      next_bit = (8'h4b>>i);
      if (next_bit==1) begin
        if(cur_state==0) begin
          usb_k;
          cur_state = 0;
        end
        else begin
          usb_j;
          cur_state = 1;
        end
      end
      else begin
        if(cur_state==0) begin
          usb_j;
          cur_state = 1;
        end
        else begin
          usb_k;
          cur_state = 0;
        end
      end
    end
  
  
endtask


task usb_emu;
  
  usb_idle;
  usb_reset;
  usb_idle;
  usb_sof(11'h0,5'h2);
  usb_reset;
  # 1ms;
  //a5
  usb_sof(11'h1,5'h1D);
  //2D
  usb_setup(0,0,5'h2);
  //C3
  //usb_setup_data0({0,8'h40,0,0,1,0,6,8'h80});
  //usb_setup_data0({8'h80,6,0,1,0,0,8'h40,0});
  usb_setup_data0({8'h0,8'h5,8'h2b,8'h0,8'h0,8'h0,8'h0,8'h0},16'hEFEC);

  OE=0;
  repeat(5) #83333;
  //OE=0;

  //usb_token_in(0,0,5'h2);
  //usb_ack;
   

endtask

function [4:0] CRC5;
    input [10:0] Data;
    input [4:0] crc;
    reg [10:0] d;
    reg [4:0] c;
    reg [4:0] newcrc;
    begin
      d = Data;
      c = crc;
      newcrc[0] = d[10] ^ d[9] ^ d[6] ^ d[5] ^ d[3] ^ d[0] ^ c[0] ^ c[3] ^ c[4];
      newcrc[1] = d[10] ^ d[7] ^ d[6] ^ d[4] ^ d[1] ^ c[0] ^ c[1] ^ c[4];
      newcrc[2] = d[10] ^ d[9] ^ d[8] ^ d[7] ^ d[6] ^ d[3] ^ d[2] ^ d[0] ^ c[0] ^ c[1] ^ c[2] ^ c[3] ^ c[4];
      newcrc[3] = d[10] ^ d[9] ^ d[8] ^ d[7] ^ d[4] ^ d[3] ^ d[1] ^ c[1] ^ c[2] ^ c[3] ^ c[4];
      newcrc[4] = d[10] ^ d[9] ^ d[8] ^ d[5] ^ d[4] ^ d[2] ^ c[2] ^ c[3] ^ c[4];
      CRC5 = newcrc;
    end
endfunction

function [4:0] crc5;
  input [10:0] Data;
  input [4:0] crc;

  reg [4:0] lfsr_q,lfsr_c;

  reg [10:0]  data_in;

  begin

     data_in = Data;
     lfsr_q  = crc;

     lfsr_c[0] = lfsr_q[0] ^ lfsr_q[3] ^ lfsr_q[4] ^ data_in[0] ^ data_in[3] ^ data_in[5] ^ data_in[6] ^ data_in[9] ^ data_in[10];
     lfsr_c[1] = lfsr_q[0] ^ lfsr_q[1] ^ lfsr_q[4] ^ data_in[1] ^ data_in[4] ^ data_in[6] ^ data_in[7] ^ data_in[10];
     lfsr_c[2] = lfsr_q[0] ^ lfsr_q[1] ^ lfsr_q[2] ^ lfsr_q[3] ^ lfsr_q[4] ^ data_in[0] ^ data_in[2] ^ data_in[3] ^ data_in[6] ^ data_in[7] ^ data_in[8] ^ data_in[9] ^ data_in[10];
     lfsr_c[3] = lfsr_q[1] ^ lfsr_q[2] ^ lfsr_q[3] ^ lfsr_q[4] ^ data_in[1] ^ data_in[3] ^ data_in[4] ^ data_in[7] ^ data_in[8] ^ data_in[9] ^ data_in[10];
     lfsr_c[4] = lfsr_q[2] ^ lfsr_q[3] ^ lfsr_q[4] ^ data_in[2] ^ data_in[4] ^ data_in[5] ^ data_in[8] ^ data_in[9] ^ data_in[10];
     crc5 = lfsr_c;

  end

endfunction


function [15:0] CRC16;
    input [7:0] Data;
    input [15:0] crc;
    reg [7:0] d;
    reg [15:0] c;
    reg [15:0] newcrc;
    begin
      d = Data;
      c = crc;

      newcrc[0] = d[7] ^ d[6] ^ d[5] ^ d[4] ^ d[3] ^ d[2] ^ d[1] ^ d[0] ^ c[8] ^ c[9] ^ c[10] ^ c[11] ^ c[12] ^ c[13] ^ c[14] ^ c[15];
      newcrc[1] = d[7] ^ d[6] ^ d[5] ^ d[4] ^ d[3] ^ d[2] ^ d[1] ^ c[9] ^ c[10] ^ c[11] ^ c[12] ^ c[13] ^ c[14] ^ c[15];
      newcrc[2] = d[1] ^ d[0] ^ c[8] ^ c[9];
      newcrc[3] = d[2] ^ d[1] ^ c[9] ^ c[10];
      newcrc[4] = d[3] ^ d[2] ^ c[10] ^ c[11];
      newcrc[5] = d[4] ^ d[3] ^ c[11] ^ c[12];
      newcrc[6] = d[5] ^ d[4] ^ c[12] ^ c[13];
      newcrc[7] = d[6] ^ d[5] ^ c[13] ^ c[14];
      newcrc[8] = d[7] ^ d[6] ^ c[0] ^ c[14] ^ c[15];
      newcrc[9] = d[7] ^ c[1] ^ c[15];
      newcrc[10] = c[2];
      newcrc[11] = c[3];
      newcrc[12] = c[4];
      newcrc[13] = c[5];
      newcrc[14] = c[6];
      newcrc[15] = d[7] ^ d[6] ^ d[5] ^ d[4] ^ d[3] ^ d[2] ^ d[1] ^ d[0] ^ c[7] ^ c[8] ^ c[9] ^ c[10] ^ c[11] ^ c[12] ^ c[13] ^ c[14] ^ c[15];
      CRC16 = newcrc;
    end
endfunction

function [15:0] crc16;
  input [7:0]   Data;
  input [15:0] crc;

  reg [15:0] lfsr_q,lfsr_c;
  reg [15:0] data_in;


  begin
  
  lfsr_q = crc;

  data_in = Data;

  $display("input:datain=%h",data_in);

  lfsr_c[0] = lfsr_q[8] ^ lfsr_q[9] ^ lfsr_q[10] ^ lfsr_q[11] ^ lfsr_q[12] ^ lfsr_q[13] ^ lfsr_q[14] ^ lfsr_q[15] ^ data_in[0] ^ data_in[1] ^ data_in[2] ^ data_in[3] ^ data_in[4] ^ data_in[5] ^ data_in[6] ^ data_in[7];
  lfsr_c[1]  = lfsr_q[9] ^ lfsr_q[10] ^ lfsr_q[11] ^ lfsr_q[12] ^ lfsr_q[13] ^ lfsr_q[14] ^ lfsr_q[15] ^ data_in[1] ^ data_in[2] ^ data_in[3] ^ data_in[4] ^ data_in[5] ^ data_in[6] ^ data_in[7];
  lfsr_c[2]  = lfsr_q[8] ^ lfsr_q[9] ^ data_in[0] ^ data_in[1];
  lfsr_c[3]  = lfsr_q[9] ^ lfsr_q[10] ^ data_in[1] ^ data_in[2];
  lfsr_c[4]  = lfsr_q[10] ^ lfsr_q[11] ^ data_in[2] ^ data_in[3];
  lfsr_c[5]  = lfsr_q[11] ^ lfsr_q[12] ^ data_in[3] ^ data_in[4];
  lfsr_c[6]  = lfsr_q[12] ^ lfsr_q[13] ^ data_in[4] ^ data_in[5];
  lfsr_c[7]  = lfsr_q[13] ^ lfsr_q[14] ^ data_in[5] ^ data_in[6];
  lfsr_c[8]  = lfsr_q[0] ^ lfsr_q[14] ^ lfsr_q[15] ^ data_in[6] ^ data_in[7];
  lfsr_c[9]  = lfsr_q[1] ^ lfsr_q[15] ^ data_in[7];
  lfsr_c[10] = lfsr_q[2];
  lfsr_c[11] = lfsr_q[3];
  lfsr_c[12] = lfsr_q[4];
  lfsr_c[13] = lfsr_q[5];
  lfsr_c[14] = lfsr_q[6];
  lfsr_c[15] = lfsr_q[7] ^ lfsr_q[8] ^ lfsr_q[9] ^ lfsr_q[10] ^ lfsr_q[11] ^ lfsr_q[12] ^ lfsr_q[13] ^ lfsr_q[14] ^ lfsr_q[15] ^ data_in[0] ^ data_in[1] ^ data_in[2] ^ data_in[3] ^ data_in[4] ^ data_in[5] ^ data_in[6] ^ data_in[7];

  crc16 = lfsr_c;
  $display("one time crc16=%h",crc16);

  end

endfunction


endmodule
