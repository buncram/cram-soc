#!/usr/bin/env python3

from migen import *
from migen.genlib.cdc import *

from litex.build.generic_platform import *
from litex.build.xilinx import XilinxPlatform
from litex.build.sim import SimPlatform

# IOs ----------------------------------------------------------------------------------------------

_io = [
    ("clk_a",  0, Pins("H17"), IOStandard("LVCMOS33")),

    ("clk_b",  0, Pins("J15"), IOStandard("LVCMOS33")),

    ("in_a", 0, Pins("N17"), IOStandard("LVCMOS33")),
    #("data_a", 0, Pins(16)),

    ("out_b", 0, Pins("E3"), IOStandard("LVCMOS33")),
    #("data_b", 0, Pins(16)),

    ("reset", 0, Pins("C12"), IOStandard("LVCMOS33")),
]

# Platform -----------------------------------------------------------------------------------------

class Platform(SimPlatform):
    default_clk_name   = "clk100"
    default_clk_period = 1e9/100e6

    def __init__(self):
        SimPlatform.__init__(self, "generic", _io)# (self, "xc7a100t-csg324-1", _io, toolchain="vivado")

# Design -------------------------------------------------------------------------------------------

# Create our platform (fpga interface)
platform = Platform()

# Create our module (fpga description)
class Cdc(Module):
    def __init__(self, platform):
        self.clock_domains.cd_a = ClockDomain()
        self.clock_domains.cd_b = ClockDomain()
        reset = platform.request("reset")
        self.comb += [
            self.cd_a.clk.eq(platform.request("clk_a")),
            self.cd_a.rst.eq(reset),
            self.cd_b.clk.eq(platform.request("clk_b")),
            self.cd_b.rst.eq(reset),
        ]
        in_a = platform.request("in_a")
        out_b = platform.request("out_b")
        self.submodules.xfer = xfer = BlindTransfer("a", "b")
        self.comb += [
            xfer.i.eq(in_a),
            out_b.eq(xfer.o),
            #xfer.data_i.eq(platform.request("data_a")),
            #platform.request("data_b").eq(xfer.data_o),
        ]

cdc = Cdc(platform)

# Build --------------------------------------------------------------------------------------------

platform.build(cdc)