`default_nettype wire

`include "rtl/include/common_cell_inc_v0.1.sv"

`include "ips/ahb_bmx33/ahb_bmx33_intf.sv"
`include "ips/ahb_bmx33/ahb_bmx33.v"
`include "ips/ahb_bmx33/ahb_bmx33_default_slave.v"
`include "ips/ahb_bmx33/abm0.v"
`include "ips/ahb_bmx33/abm1.v"
`include "ips/ahb_bmx33/abm2.v"
`include "ips/ahb_bmx33/ib.v"
`include "ips/ahb_bmx33/mbs0.v"
`include "ips/ahb_bmx33/mbs1.v"
`include "ips/ahb_bmx33/mbs2.v"
`include "ips/ahb_bmx33/obm0.v"
`include "ips/ahb_bmx33/obm1.v"
`include "ips/ahb_bmx33/obm2.v"

`include "rtl/include/nic400_inc_v0.3.sv"

`include "rtl/bmxcore/bmxcore_v0.2.sv"
`include "rtl/bmxcore/nic1_intf_v0.2.sv"
