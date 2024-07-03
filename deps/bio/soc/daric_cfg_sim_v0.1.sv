import axi_pkg::*;

package daric_cfg;
// AXIM ID
// ==

    localparam bit [3:0] AMBAID4_CM7A = 4'h2;
    localparam bit [3:0] AMBAID4_VEXI = 4'h3;
    localparam bit [3:0] AMBAID4_VEXD = 4'h4;
    localparam bit [3:0] AMBAID4_SCEA = 4'h5;
    localparam bit [3:0] AMBAID4_SCES = 4'h6;
    localparam bit [3:0] AMBAID4_MDMA = 4'h7;

    localparam bit [3:0] AMBAID4_CM7P = 4'h8;
    localparam bit [3:0] AMBAID4_CM7D = 4'h9;
    localparam bit [3:0] AMBAID4_VEXP = 4'hD;
    localparam bit [3:0] AMBAID4_UDMA = 4'hA;
    localparam bit [3:0] AMBAID4_UDCA = 4'hB;
    localparam bit [3:0] AMBAID4_SDDC = 4'hC;

    localparam IRQCNT = 256;
endpackage : daric_cfg
