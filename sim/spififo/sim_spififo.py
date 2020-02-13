#!/usr/bin/env python3

import sys
sys.path.append("../")    # FIXME
sys.path.append("../../") # FIXME

import lxbuildenv

# This variable defines all the external programs that this module
# relies on.  lxbuildenv reads this variable in order to ensure
# the build will finish without exiting due to missing third-party
# programs.
LX_DEPENDENCIES = ["riscv", "vivado"]

# print('\n'.join(sys.path))  # help with debugging PYTHONPATH issues

from migen import *

from litex.build.generic_platform import *
from litex.build.xilinx import XilinxPlatform

from litex.soc.integration.soc_core import *
from litex.soc.integration.builder import *
from litex.soc.cores.clock import *

from gateware import spi
from gateware import spi_ec

sim_config = {
    # freqs
    "input_clk_freq": 12e6,
    "sys_clk_freq": 12e6,  # UP5K-side
    "spi_clk_freq": 24e6,
#    "sys_clk_freq": 100e6,  # Artix-side
#    "spi_clk_freq": 25e6,
}


_io = [
    ("clk12", 0, Pins("X")),
    ("rst", 0, Pins("X")),

    ("serial", 0,
     Subsignal("tx", Pins("V6")),
     Subsignal("rx", Pins("V7")),
     IOStandard("LVCMOS18"),
     ),

    # COM to UP5K (maste0)
    ("com", 0,
     Subsignal("csn", Pins("T15"), IOStandard("LVCMOS18")),
     Subsignal("miso", Pins("P16"), IOStandard("LVCMOS18")),
     Subsignal("mosi", Pins("N18"), IOStandard("LVCMOS18")),
     Subsignal("sclk", Pins("R16"), IOStandard("LVCMOS18")),
     ),

    # slave interface for testing UP5K side
    ("slave", 0,
     Subsignal("csn", Pins("dummy0")),
     Subsignal("miso", Pins("dummy1")),
     Subsignal("mosi", Pins("dummy2")),
     Subsignal("sclk", Pins("dummy3")),
     Subsignal("irq", Pins("dummy4")),
     ),
]


class Platform(XilinxPlatform):
    def __init__(self):
        XilinxPlatform.__init__(self, "", _io, toolchain="vivado")


class CRG(Module):
    def __init__(self, platform, core_config):
        # build a simulated PLL. You can add more pll.create_clkout() lines to add more clock frequencies as necessary
        self.clock_domains.cd_sys = ClockDomain()
        self.clock_domains.cd_spi = ClockDomain()

        self.submodules.pll = pll = S7MMCM()
        self.comb += pll.reset.eq(platform.request("rst"))
        pll.register_clkin(platform.request("clk12"), sim_config["input_clk_freq"])
        pll.create_clkout(self.cd_sys, sim_config["sys_clk_freq"])
        pll.create_clkout(self.cd_spi, sim_config["spi_clk_freq"])

class WarmBoot(Module, AutoCSR):
    def __init__(self, parent, reset_vector=0):
        self.ctrl = CSRStorage(size=8)
        self.addr = CSRStorage(size=32, reset=reset_vector)
        self.do_reset = Signal()
        # "Reset Key" is 0xac (0b101011xx)
        self.comb += self.do_reset.eq((self.ctrl.storage & 0xfc) == 0xac)

boot_offset    = 0x0 #0x500000 # enough space to hold 2x FPGA bitstreams before the firmware start


class SimpleSim(SoCCore):
    mem_map = {
        "wifi": 0xd0000000,
    }
    mem_map.update(SoCCore.mem_map)

    def __init__(self, platform, **kwargs):
        SoCCore.__init__(self, platform, sim_config["sys_clk_freq"],
                         integrated_rom_size=0x8000,
                         integrated_sram_size=0x20000,
                         ident="betrusted.io LiteX Base SoC",
                         cpu_type="vexriscv", csr_data_width=32,
                         **kwargs)

        self.add_constant("SIMULATION", 1)
        self.add_constant("SPIFIFO_SIMULATION", 1)

        # instantiate the clock module
        self.submodules.crg = CRG(platform, sim_config)
        self.platform.add_period_constraint(self.crg.cd_sys.clk, 1e9/sim_config["sys_clk_freq"])

        self.platform.add_platform_command(
            "create_clock -name clk12 -period 83.3333 [get_nets clk12]")

        # SPI interface
        self.submodules.spimaster = spi.SPIMaster(platform.request("com"))
        self.add_csr("spimaster")

        self.submodules.com = ClockDomainsRenamer({"spislave":"spi"})(spi_ec.SpiFifoSlave(platform.request("slave")))
        self.add_wb_slave(self.mem_map["wifi"], self.com.bus, 4)
        self.add_memory_region("wifi", self.mem_map["wifi"], 4, type='io')
        self.add_csr("com")
        self.add_interrupt("com")



def generate_top():
    platform = Platform()
    soc = SimpleSim(platform)
    builder = Builder(soc, output_dir="./run", csr_csv="test/csr.csv", compile_software=True, compile_gateware=False)
    builder.software_packages = [
#        ("libcompiler_rt", os.path.abspath(os.path.join(os.path.dirname(__file__), "../bios/libcompiler_rt"))),
        ("libbase", os.path.abspath(os.path.join(os.path.dirname(__file__), "../bios/libbase"))),
        ("bios", os.path.abspath(os.path.join(os.path.dirname(__file__), "../bios")))
    ]
    vns = builder.build()
    soc.do_exit(vns)

# this generates a test bench wrapper verilog file, needed by the xilinx tools
def generate_top_tb():
    f = open("run/top_tb.v", "w")
    f.write("""
`timescale 1ns/1ps

module top_tb();

reg clk12;
initial clk12 = 1'b1;
always #41.16666 clk12 = ~clk12;

wire miso;
wire sclk;
wire csn;
wire mosi;

top dut (
    .clk12(clk12),
    .rst(0),
    .com_sclk(sclk),
    .com_mosi(mosi),
    .com_miso(miso),
    .com_csn(csn),

    .slave_sclk(sclk),
    .slave_mosi(mosi),
    .slave_miso(miso),
    .slave_csn(csn)
);

// reg [15:0] value;
// initial miso = 1'b0;
// initial value = 16'ha503;
// always @(posedge sclk) begin
//    miso <= value[15];
//    value <= {value[14:0],value[15]};
// end

endmodule""")
    f.close()


# this ties it all together
def run_sim(gui=False):
    os.system("mkdir -p run")
    os.system("rm -rf run/xsim.dir")
    if sys.platform == "win32":
        call_cmd = "call "
    else:
        call_cmd = ""
    os.system(call_cmd + "cd run && cp gateware/*.init .")
    os.system(call_cmd + "cd run && cp gateware/*.v .")
    os.system(call_cmd + "cd run && xvlog ../../glbl.v")
    os.system(call_cmd + "cd run && xvlog top.v -sv")
    os.system(call_cmd + "cd run && xvlog top_tb.v -sv ")
    #os.system(call_cmd + "cd run && xvlog ../../../deps/litex/litex/soc/cores/cpu/vexriscv/verilog/VexRiscv.v")
    os.system(call_cmd + "cd run && xvlog ../../../gateware/cpu/VexRiscv_BetrustedSoC_Debug.v")
    os.system(call_cmd + "cd run && xelab -debug typical top_tb glbl -s top_tb_sim -L unisims_ver -L unimacro_ver -L SIMPRIM_VER -L secureip -L $xsimdir/xil_defaultlib -timescale 1ns/1ps")
    if gui:
        os.system(call_cmd + "cd run && xsim top_tb_sim -gui")
    else:
        os.system(call_cmd + "cd run && xsim top_tb_sim -runall")


def main():
    import subprocess

    subprocess.Popen(['cp', '../bios/linker_rom.ld', '../bios/linker.ld'])

    generate_top()
    generate_top_tb()
    run_sim(gui=True)


if __name__ == "__main__":
    main()
