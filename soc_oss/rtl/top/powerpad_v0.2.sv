module powerpad ();

`ifndef SIM

localparam CNT_PBRKB2B_H = 1; //zmj modify 20230909
localparam CNT_PBRKB2B_V = 5;
localparam CNT_PBRK = 0;
localparam CNT_PVDD09_H = 6;
localparam CNT_PVSS09_H = 6;
localparam CNT_PVDD33_H = 12;
localparam CNT_PVSS33_H = 12;
localparam CNT_PVDD09_V = 11;
localparam CNT_PVSS09_V = 11;
localparam CNT_PVDD33_V = 12;
localparam CNT_PVSS33_V = 15;
localparam CNT_PFILL10 = 2;
localparam CNT_PVSENSE = 2;
localparam CNT_PVDDTIE = 3;
localparam CNT_PVDDI09 = 4;
localparam CNT_PBRKANA_H = 3; //zmj modify 20230917
localparam CNT_PBRKANA_V = 2;

genvar i;
generate
//CNT_PBRKB2B_H = 1
                PBRKB2B_33_33_NT_DR gbrkb2b_h_0__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));

//CNT_PBRKB2B_V = 5
                PBRKB2B_33_33_NT_DR gbrkb2b_v_0__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));
                PBRKB2B_33_33_NT_DR gbrkb2b_v_1__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));
                PBRKB2B_33_33_NT_DR gbrkb2b_v_2__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));
                PBRKB2B_33_33_NT_DR gbrkb2b_v_3__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));
                PBRKB2B_33_33_NT_DR gbrkb2b_v_4__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));

//CNT_PBRK =0
//                PBRK_33_33_NT_DR gbrk_xxxx__u (.RTO( rto_ ),.SNS( sns_ ),.RTOBRK('0),.SNSBRK('0));
//CNT_PVDD09_H = 6
                PVDD_09_09_NT_DR_H gvdd09_h_0__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_H gvdd09_h_1__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_H gvdd09_h_2__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_H gvdd09_h_3__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_H gvdd09_h_4__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_H gvdd09_h_5__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVSS09_H = 6
                PVSS_09_09_NT_DR_H gvss09_h_0__u (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_H gvss09_h_1__u (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_H gvss09_h_2__u (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_H gvss09_h_3__u (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_H gvss09_h_4__u (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_H gvss09_h_5__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVDD33_H = 12
                PDVDD_33_33_NT_DR_H gvdd33_h_0__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_1__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_2__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_3__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_4__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_5__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_6__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_7__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_8__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_9__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_10__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_H gvdd33_h_11__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVSS33_H = 12
                PDVSS_33_33_NT_DR_H gvss33_h_0__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_1__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_2__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_3__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_4__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_5__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_6__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_7__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_8__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_9__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_10__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_H gvss33_h_11__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVDD09_V = 11
                PVDD_09_09_NT_DR_V gvdd09_v_0__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_1__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_2__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_3__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_4__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_5__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_6__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_7__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_8__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_9__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVDD_09_09_NT_DR_V gvdd09_v_10__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVSS09_V = 12
                PVSS_09_09_NT_DR_V gvss09_v_0__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_1__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_2__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_3__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_4__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_5__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_6__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_7__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_8__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_9__u  (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_10__u (.RTO( rto_ ),.SNS( sns_ ));
                PVSS_09_09_NT_DR_V gvss09_v_11__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVDD33_V = 12
                PDVDD_33_33_NT_DR_V gvdd33_v_0__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_1__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_2__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_3__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_4__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_5__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_6__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_7__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_8__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_9__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_10__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVDD_33_33_NT_DR_V gvdd33_v_11__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVSS33_V = 15
                PDVSS_33_33_NT_DR_V gvss33_v_0__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_1__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_2__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_3__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_4__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_5__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_6__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_7__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_8__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_9__u  (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_10__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_11__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_12__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_13__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVSS_33_33_NT_DR_V gvss33_v_14__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVSENSE = 2
                PVSENSE_33_33_NT_DR_H gvsense_0__u (.RTO( rto_ ),.SNS( sns_ ),.RETOFF('0),.RETON('0));
                PVSENSE_33_33_NT_DR_H gvsense_1__u (.RTO( rto_ ),.SNS( sns_ ),.RETOFF('0),.RETON('0));

//CNT_PVDDTIE = 3
                PDVDDTIE_33_33_NT_DR_V gvddtie_0__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVDDTIE_33_33_NT_DR_V gvddtie_1__u (.RTO( rto_ ),.SNS( sns_ ));
                PDVDDTIE_33_33_NT_DR_V gvddtie_2__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PVDDI09 = 4
                PVDDI_09_09_NT_DR_V gvddi_0__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDDI_09_09_NT_DR_V gvddi_1__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDDI_09_09_NT_DR_V gvddi_2__u (.RTO( rto_ ),.SNS( sns_ ));
                PVDDI_09_09_NT_DR_V gvddi_3__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PBRKANA_H = 3
                PBRKANALOGB2B_33_33_NT_DR brkana_h_0__u (.RTO( rto_ ),.SNS( sns_ ));
                PBRKANALOGB2B_33_33_NT_DR brkana_h_1__u (.RTO( rto_ ),.SNS( sns_ ));
                PBRKANALOGB2B_33_33_NT_DR brkana_h_2__u (.RTO( rto_ ),.SNS( sns_ ));

//CNT_PBRKANA_V = 2
                PBRKANALOGB2B_33_33_NT_DR brkana_v_0__u (.RTO( rto_ ),.SNS( sns_ ));
                PBRKANALOGB2B_33_33_NT_DR brkana_v_1__u (.RTO( rto_ ),.SNS( sns_ ));

         PANALOG_33_33_NT_DR_V upll_vcca(.RTO( rto_ ),.SNS( sns_ )); //zmj modfiy 20230917
         PANALOG_33_33_NT_DR_V upll_vssa(.RTO( rto_ ),.SNS( sns_ ));
         PDVDD_33_33_NT_DR_V upll_vdd33(.RTO( rto_ ),.SNS( sns_ ));
         PDVSS_33_33_NT_DR_V upll_vss33(.RTO( rto_ ),.SNS( sns_ ));
         PVDD_09_09_NT_DR_V upll_vdd09(.RTO( rto_ ),.SNS( sns_ ));
         PVSS_09_09_NT_DR_V upll_vss09(.RTO( rto_ ),.SNS( sns_ ));
         PVDDI_09_09_NT_DR_V upll_vccd(.RTO( rto_ ),.SNS( sns_ ));

endgenerate
`endif
endmodule



`ifdef SIM
module PBRKB2B_33_33_NT_DR (
                RTO,
                RTOBRK,
                SNS,
                SNSBRK
 );
   input SNS;
   input RTO;
   input SNSBRK;
   input RTOBRK;
endmodule // PBRKB2B_33_33_NT_DR

module PBRK_33_33_NT_DR (
                RTO,
                RTOBRK,
                SNS,
                SNSBRK
 );
   input SNS;
   input RTO;
   input SNSBRK;
   input RTOBRK;
endmodule // PBRKB2B_33_33_NT_DR

module PVDD_09_09_NT_DR_H ( input SNS, input RTO ); endmodule
module PVSS_09_09_NT_DR_H ( input SNS, input RTO ); endmodule
module PDVDD_33_33_NT_DR_H ( input SNS, input RTO ); endmodule
module PDVSS_33_33_NT_DR_H ( input SNS, input RTO ); endmodule
module PFILL10_33_33_NT_DR ( input SNS, input RTO ); endmodule
module PVSENSE_33_33_NT_DR_H ( output SNS, output RTO, input RETOFF, input RETON ); endmodule
module PDVDDTIE_33_33_NT_DR_H ( output SNS, output RTO ); endmodule
module PVDDI_09_09_NT_DR_H ( input SNS, input RTO ); endmodule
`endif
