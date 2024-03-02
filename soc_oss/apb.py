from migen import *
from litex.soc.interconnect.ahb import *

# AHB Definition -----------------------------------------------------------------------------------

def apb_description(data_width, address_width):
    return [
        ("paddr",     address_width, DIR_M_TO_S),
        ("presetn",               1, DIR_M_TO_S),
        ("pprot",                 3, DIR_M_TO_S),
        ("pnse",                  1, DIR_M_TO_S),
        ("psel",                  1, DIR_M_TO_S),
        ("penable",               1, DIR_M_TO_S),
        ("pwdata",       data_width, DIR_M_TO_S),
        ("pwrite",                1, DIR_M_TO_S),
        ("pstrb",     data_width//8, DIR_M_TO_S),
        ("prdata",       data_width, DIR_S_TO_M),
        ("pready",                1, DIR_S_TO_M),
        ("pslverr",               1, DIR_S_TO_M),
        ("pwakeup",               1, DIR_S_TO_M),
        ("pactive",               1, DIR_M_TO_S), # not a standard signal, but provided for clock gating
]

class APBInterface(Record):
    def __init__(self, data_width=32, address_width=12):
        Record.__init__(self, apb_description(data_width, address_width))
        self.data_width    = data_width
        self.address_width = address_width
        self.addressing    = "byte"

class AHB2APB(Module):
    def __init__(self, ahb, apb, base = 0x0000):
        assert ahb.data_width == apb.data_width

        apb_strb = Signal(apb.data_width//8)
        data_phase = Signal()
        self.comb += [
            Case(
                ahb.size, {
                    0: apb_strb.eq(1),
                    1: apb_strb.eq(3),
                    "default": apb_strb.eq(0xF),
                }
            ),
            apb.pactive.eq((ahb.sel & (ahb.trans != 0)) | data_phase | apb.psel)
        ]
        # FSM.
        self.submodules.ahb_apb_fsm = fsm = FSM()
        fsm.act("ADDRESS-PHASE",
            ahb.readyout.eq(1),
            data_phase.eq(0),
            NextValue(apb.penable, 0),
            If(
              (ahb.addr >= base) &
              (ahb.addr < (base + (1 << apb.address_width))) &
              (ahb.size  <= log2_int(ahb.data_width//8)) &
              (ahb.trans == AHBTransferType.NONSEQUENTIAL),
                NextValue(apb.paddr, ahb.addr[:apb.address_width]),
                NextValue(apb.pwrite, ahb.write),
                NextValue(apb.psel, 1),
                NextValue(apb.pwdata, ahb.wdata),
                NextValue(apb.pstrb, apb_strb),

                NextState("DATA-PHASE"),
            ).Else(
                NextValue(apb.psel, 0)
            )
        )
        fsm.act("DATA-PHASE",
            data_phase.eq(1),
            ahb.resp.eq(apb.pslverr),
            NextValue(apb.penable, 1),
            If(apb.pready & apb.penable, # funky hack that adds an extra beat to data-phase, because penable is a NextValue
                NextValue(ahb.rdata, apb.prdata),
                NextState("ADDRESS-PHASE")
            ).Else(
            )
        )