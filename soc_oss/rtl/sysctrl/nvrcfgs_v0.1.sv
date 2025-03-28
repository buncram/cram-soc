`include "template.sv"

import cms_pkg::*;

package nvrcfg_pkg;

    localparam RRAW = 22;  //4k
    localparam RRBAW = 8;  //256 blks, 16KB/blk

    typedef bit [RRBAW-1:0] rrblk_adr_t;

    typedef bit [255:0] nvrdat_t;
    localparam nvrdat_t defnvrdat256 = 256'hcf9defc0de;

    typedef struct packed {
        bit [7:0] reram_start    ; //0
        bit [7:0] reram_end      ; //1
        bit [7:0] m7_init        ; //2
        bit [7:0] m7_boot0_start ; //3
        bit [7:0] m7_boot1_start ; //4
        bit [7:0] m7_fw0_start   ; //5
        bit [7:0] m7_fw1_start   ; //6
        bit [7:0] m7_fw1_end     ; //7
        bit [7:0] rv_init        ; //8
        bit [7:0] rv_boot0_start ; //9
        bit [7:0] rv_boot1_start ; //10
        bit [7:0] rv_fw0_start   ; //11
        bit [7:0] rv_fw1_start   ; //12
        bit [7:0] rv_fw1_end     ; //13
        bit [14:31][7:0] rev ;
    }cfgrrsub_t;
    localparam cfgrrsub_t defcfgrrsub = '{
        reram_start    : '0,  //0
        reram_end      : '1,  //1
        m7_init        : '0,  //2
        m7_boot0_start : '0,  //3
        m7_boot1_start : '1,  //4
        m7_fw0_start   : '1,  //5
        m7_fw1_start   : '1,  //6
        m7_fw1_end     : '1,  //7
        rv_init        : '0,  //8
        rv_boot0_start : '0,  //9
        rv_boot1_start : '1,  //10
        rv_fw0_start   : '1,  //11
        rv_fw1_start   : '1,  //12
        rv_fw1_end     : '1,  //13
        rev : '0
        };

    typedef struct packed {
        bit [31:0] devena ;
        bit [31:0] coreselcm7 ;
        bit [31:0] coreselvex ;
        bit [3:7][31:0] rev ;
    }cfgcore_t;
    localparam bit [31:0] cpudevmode = 32'h298ca435;
    localparam bit [31:0] coreselcm7_code = 32'h7e20a453;
    localparam bit [31:0] coreselvex_code = 32'h6a428c82;
    localparam cfgcore_t defcfgcore = '{
        devena      : cpudevmode,
        coreselcm7  : coreselcm7_code,
        coreselvex  : 31'h00,
        rev     : '0
    };




    typedef struct packed {
        cms_pkg::cmsdata_e cmsdata1;
        cms_pkg::cmsdata_e cmsdata0;
    }nvrcms_t;
    localparam nvrcms_t defnvrcms = '{
        cmsdata1: cms_pkg::CMSDAT_USERMODE,
        cmsdata0: cms_pkg::CMSDAT_USERMODE
    };

    typedef struct packed{
        nvrdat_t            ipm0;          //1
        nvrdat_t            ipm1;          //2
        nvrdat_t            ipm2;          //3
    }nvripm_t;
    localparam nvripm_t defnvripm = '{
        ipm0      : defnvrdat256 ,
        ipm1      : defnvrdat256 ,
        ipm2      : defnvrdat256 
    };

    typedef struct packed{
        nvrdat_t            cfginfo;       //4
        nvrdat_t            nvrrev05;      //5
        cfgrrsub_t          cfgrrsub;      //6
        nvrdat_t            nvrrev07;      //7
        nvrdat_t            nvrrev08;      //8
        nvrdat_t            nvrrev09;      //9
        nvrdat_t            cfgsce;        //10
        nvrdat_t            nvrrev11;      //11
        cfgcore_t           cfgcore;       //12
        nvrdat_t            nvrrev13;      //13
        nvrdat_t            nvrrev14;      //14
        nvrdat_t            nvrrev15;      //15
    }nvrcfg_t;
    localparam nvrcfg_t defnvrcfg = '{
        cfginfo   : 256'hda11ccf9c0de ,
        nvrrev05  : defnvrdat256 ,
        cfgrrsub  : defcfgrrsub  ,
        nvrrev07  : defnvrdat256 ,
        nvrrev08  : defnvrdat256 ,
        nvrrev09  : defnvrdat256 ,
        cfgsce    : defnvrdat256 ,
        nvrrev11  : defnvrdat256 ,
        cfgcore   : defcfgcore   ,
        nvrrev13  : defnvrdat256 ,
        nvrrev14  : defnvrdat256 ,
        nvrrev15  : defnvrdat256 
    };


endpackage

`define ambarrb(theblx) { 10'b0110_0000_00, theblx, 14'h0 }

