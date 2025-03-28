`ifndef _AMBA_INTERFACE_DEFINE

interface ahbif #(
    parameter AW=32,
    parameter DW=32
)();

    wire            hsel;           // Slave Select
    wire  [AW-1:0]  haddr;          // Address bus
    wire  [1:0]     htrans;         // Transfer type
    wire            hwrite;         // Transfer direction
    wire  [2:0]     hsize;          // Transfer size
    wire  [2:0]     hburst;         // Burst type
    wire  [3:0]     hprot;          // Protection control
    wire  [3:0]     hmaster;        //Master select
    wire  [DW-1:0]  hwdata;         // Write data
    wire            hmasterlock;    // Locked Sequence
    wire            hreadyin;       // Transfer done
    wire  [DW-1:0]  hrdata;         // Read data bus
    wire            hready;         // HREADY feedback
    wire            hresp;          // Transfer response

  modport slave ( 
    input  hsel,         
    input  haddr,        
    input  htrans,       
    input  hwrite,       
    input  hsize,        
    input  hburst,       
    input  hprot,         
    input  hmaster,      
    input  hwdata,       
    input  hmasterlock,  
    input  hreadyin,     

    output  hrdata,      
    output  hready,      
    output  hresp       
    );
 
  modport master ( 
    output  hsel,         
    output  haddr,        
    output  htrans,       
    output  hwrite,       
    output  hsize,        
    output  hburst,       
    output  hprot,         
    output  hmaster,      
    output  hwdata,       
    output  hmasterlock,  
    output  hreadyin,     

    input  hrdata,      
    input  hready,      
    input  hresp       
    );

endinterface

interface apbif #(
    parameter PAW=16,
    parameter DW=32
)();

    wire               psel;
    wire   [PAW-1:0]    paddr;
    wire               penable;
    wire               pwrite;
    wire   [3:0]       pstrb;
    wire   [2:0]       pprot;
    wire   [31:0]      pwdata;
    wire               apbactive;
    wire   [DW-1:0]    prdata;
    wire               pready;
    wire               pslverr;

  modport slave ( 
    input   psel,
    input   paddr,
    input   penable,
    input   pwrite,
    input   pstrb,
    input   pprot,
    input   pwdata,
    input   apbactive,

    output  prdata,
    output  pready,
    output  pslverr
    );
 
  modport master ( 
    output  psel,
    output  paddr,
    output  penable,
    output  pwrite,
    output  pstrb,
    output  pprot,
    output  pwdata,
    output  apbactive,

    input   prdata,
    input   pready,
    input   pslverr
    );

endinterface
`endif //`ifndef _INTERFACE_DEFINE

`define _AMBA_INTERFACE_DEFINE

