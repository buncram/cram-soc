//------------------------------------------------------------------
//-- File generated by RobustVerilog parser
//-- RobustVerilog version 1.2 (limited free version)
//-- Invoked Mon Feb 06 01:29:27 2023
//-- Source file: axi2ahb_cmd.v
//-- Parent file: axi2ahb.v
//-- Run directory: F:/largework/rust-win/code/robust_axi2ahb/
//-- Target directory: out/
//-- Command flags: .\src\base\axi2ahb.v -od out -I .\src\gen\ -list list.txt -listpath -header 
//-- www.provartec.com/edatools ... info@provartec.com
//------------------------------------------------------------------




  

module  axi2ahb_cmd (clk,reset,AWID,AWADDR,AWLEN,AWSIZE,AWVALID,AWREADY,ARID,ARADDR,ARLEN,ARSIZE,ARVALID,ARREADY,HADDR,HBURST,HSIZE,HTRANS,HWRITE,HWDATA,HRDATA,HREADY,HRESP,ahb_finish,cmd_empty,cmd_read,cmd_id,cmd_addr,cmd_len,cmd_size,cmd_err);

   input           clk;
   input                  reset;

   input [3:0]            AWID;
   input [23:0]           AWADDR;
   input [3:0]            AWLEN;
   input [1:0]            AWSIZE;
   input                  AWVALID;
   output                 AWREADY;
   input [3:0]            ARID;
   input [23:0]           ARADDR;
   input [3:0]            ARLEN;
   input [1:0]            ARSIZE;
   input                  ARVALID;
   output                 ARREADY;
   input [23:0]           HADDR;
   input [2:0]            HBURST;
   input [1:0]            HSIZE;
   input [1:0]            HTRANS;
   input                  HWRITE;
   input [31:0]           HWDATA;
   input [31:0]           HRDATA;
   input                  HREADY;
   input                  HRESP;
         
   input                  ahb_finish;
   output                 cmd_empty;
   output                 cmd_read;
   output [4-1:0]   cmd_id;
   output [24-1:0] cmd_addr;
   output [3:0]           cmd_len;
   output [1:0]           cmd_size;
   output                 cmd_err;
    
   
   wire [3:0]             AID;
   wire [23:0]            AADDR;
   wire [3:0]             ALEN;
   wire [1:0]             ASIZE;
   wire                   AVALID;
   wire                   AREADY;
   
   wire                   cmd_push;
   wire                   cmd_pop;
   wire                   cmd_empty;
   wire                   cmd_full;
   reg                    read;
   wire                   err;

   
   wire                   wreq, rreq;
   wire                   wack, rack;
   wire                   AERR;
   
   assign                 wreq = AWVALID;
   assign                 rreq = ARVALID;
   assign                 wack = AWVALID & AWREADY;
   assign                 rack = ARVALID & ARREADY;
        
   always @(posedge clk or posedge reset)
     if (reset)
       read <=  1'b1;
     else if (wreq & (rack | (~rreq)))
       read <=  1'b0;
     else if (rreq & (wack | (~wreq)))
       read <=  1'b1;

    //command mux
    assign AID = read ? ARID : AWID;
    assign AADDR = read ? ARADDR : AWADDR;
    assign ALEN = read ? ARLEN : AWLEN;
    assign ASIZE = read ? ARSIZE : AWSIZE;
    assign AVALID = read ? ARVALID : AWVALID;
    assign AREADY = read ? ARREADY : AWREADY;
   
   assign ARREADY = (~cmd_full) & read;
   assign AWREADY = (~cmd_full) & (~read);

   assign err = 
          ((ALEN != 4'd0) & 
           (ALEN != 4'd3) & 
           (ALEN != 4'd7) & 
           (ALEN != 4'd15)) |
          (((ASIZE == 2'b01) & (AADDR[0] != 1'b0)) |
           ((ASIZE == 2'b10) & (AADDR[1:0] != 2'b00)) |
           ((ASIZE == 2'b11) & (AADDR[2:0] != 3'b000)));
   
   
   
    assign               cmd_push  = AVALID & AREADY;
    assign               cmd_pop   = ahb_finish;
   
   prgen_fifo #(4+24+4+2+1+1, 1) 
   cmd_fifo(
        .clk(clk),
        .reset(reset),
        .push(cmd_push),
        .pop(cmd_pop),
        .din({
          AID,
          AADDR,
                  ALEN,
                  ASIZE,
          read,
                  err
          }
         ),
        .dout({
           cmd_id,
           cmd_addr,
                   cmd_len,
                   cmd_size,
           cmd_read,
                   cmd_err
           }
          ),
        .empty(cmd_empty),
        .full(cmd_full)
        );

        
   
endmodule




