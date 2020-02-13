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

from gateware import sram_32

sim_config = {
    # freqs
    "input_clk_freq": 12e6,
#    "sys_clk_freq": 12e6,  # UP5K-side
#    "spi_clk_freq": 24e6,
    "sys_clk_freq": 100e6,  # Artix-side
    "au_mclk" : 768e3,  # 16,000 Hz * 24x2 bits per sample
}


_io = [
    ("clk12", 0, Pins("X")),
    ("rst", 0, Pins("X")),

    ("serial", 0,
     Subsignal("tx", Pins("V6")),
     Subsignal("rx", Pins("V7")),
     IOStandard("LVCMOS18"),
     ),

    # Audio interface
    ("i2s", 0,  # headset & mic
     Subsignal("clk", Pins("D14")),
     Subsignal("tx", Pins("D12")),  # au_sdi1
     Subsignal("rx", Pins("C13")),  # au_sdo1
     Subsignal("sync", Pins("B15")),
     IOStandard("LVCMOS33"),
     Misc("SLEW=SLOW"), Misc("DRIVE=4"),
     ),
    ("i2s", 1,  # speaker
     Subsignal("clk", Pins("F14")),
     Subsignal("tx", Pins("A15")),  # au_sdi2
     Subsignal("sync", Pins("B17")),
     IOStandard("LVCMOS33"),
     Misc("SLEW=SLOW"), Misc("DRIVE=4"),
     ),
     ("au_mclk", 0, Pins("D18"), IOStandard("LVCMOS33"), Misc("SLEW=SLOW"), Misc("DRIVE=8")),

]

class Platform(XilinxPlatform):
    def __init__(self):
        XilinxPlatform.__init__(self, "", _io, toolchain="vivado")


class CRG(Module, AutoCSR):
    def __init__(self, platform, core_config):
        # build a simulated PLL. You can add more pll.create_clkout() lines to add more clock frequencies as necessary
        self.clock_domains.cd_sys = ClockDomain()
        self.clock_domains.cd_spinor = ClockDomain()
        self.clock_domains.cd_idelay_ref = ClockDomain()

        self.submodules.pll = pll = S7MMCM()
        self.comb += pll.reset.eq(platform.request("rst"))
        pll.register_clkin(platform.request("clk12"), sim_config["input_clk_freq"])
        pll.create_clkout(self.cd_sys, sim_config["sys_clk_freq"], margin=0)


class WarmBoot(Module, AutoCSR):
    def __init__(self, parent, reset_vector=0):
        self.ctrl = CSRStorage(size=8)
        self.addr = CSRStorage(size=32, reset=reset_vector)
        self.do_reset = Signal()
        # "Reset Key" is 0xac (0b101011xx)
        self.comb += self.do_reset.eq((self.ctrl.storage & 0xfc) == 0xac)

boot_offset    = 0x0 #0x500000 # enough space to hold 2x FPGA bitstreams before the firmware start

class SimpleSim(SoCCore):
    SoCCore.mem_map = {
        "rom":      0x00000000,
        "sram":     0x10000000,
        "spiflash": 0x20000000,
        "sram_ext": 0x40000000,
        "i2s_duplex":  0xe0000000,
        "i2s_spkr":  0xe0000020,
        "csr":      0xf0000000,
    }

    def __init__(self, platform, **kwargs):
        SoCCore.__init__(self, platform, sim_config["sys_clk_freq"],
                         integrated_rom_size=0x8000,
                         integrated_sram_size=0x20000,
                         ident="betrusted.io LiteX Base SoC",
                         cpu_type="vexriscv", csr_data_width=32,
                         **kwargs)

        self.add_constant("SIMULATION", 1)
        self.add_constant("I2S_SIMULATION", 1)

        self.cpu.use_external_variant("../../gateware/cpu/VexRiscv_BetrustedSoC_Debug.v")

        # instantiate the clock module
        self.submodules.crg = CRG(platform, sim_config)
        self.add_csr("crg")
        # self.platform.add_period_constraint(self.crg.cd_sys.clk, 1e9/sim_config["sys_clk_freq"])

        self.platform.add_platform_command(
            "create_clock -name clk12 -period 83.3333 [get_nets clk12]")

        from litex.soc.cores import i2s
        # shallow fifodepth allows us to work the end points a bit faster in simulation
        self.submodules.i2s_duplex = i2s.S7I2SSlave(platform.request("i2s", 0), fifo_depth=8)
        self.add_wb_slave(self.mem_map["i2s_duplex"], self.i2s_duplex.bus, 0x4)
        self.add_memory_region("i2s_duplex", self.mem_map["i2s_duplex"], 4, type='io')
        self.add_csr("i2s_duplex")
        self.add_interrupt("i2s_duplex")

        self.submodules.i2s_spkr = i2s.S7I2SSlave(platform.request("i2s", 1), fifo_depth=8)
        self.add_wb_slave(self.mem_map["i2s_spkr"], self.i2s_spkr.bus, 0x4)
        self.add_memory_region("i2s_spkr", self.mem_map["i2s_spkr"], 4, type='io')
        self.add_csr("i2s_spkr")
        self.add_interrupt("i2s_spkr")


def generate_top():
    platform = Platform()
    soc = SimpleSim(platform)
    builder = Builder(soc, output_dir="./run", csr_csv="test/csr.csv", compile_software=True, compile_gateware=False)
    builder.software_packages = [
    ("libcompiler_rt", os.path.abspath(os.path.join(os.path.dirname(__file__), "../bios/libcompiler_rt"))),
    ("libbase", os.path.abspath(os.path.join(os.path.dirname(__file__), "../bios/libbase"))),
    ("bios", os.path.abspath(os.path.join(os.path.dirname(__file__), "../bios")))
]
    vns = builder.build()
    soc.do_exit(vns)

#    platform.build(soc, build_dir="./run", run=False)  # run=False prevents synthesis from happening, but a top.v file gets kicked out

# this generates a test bench wrapper verilog file, needed by the xilinx tools
def generate_top_tb():
    f = open("run/top_tb.v", "w")
    f.write("""
`timescale 1ns/1ps

module top_tb();

reg clk12;
initial clk12 = 1'b1;
always #41.66666 clk12 = ~clk12;

wire sclk;
reg fpga_reset;

reg mclk;
initial mclk = 1'b1;
always #651 mclk = ~mclk;  // 768kHz

reg sync;
initial sync = 1'b1;
always #(1302 * 24) sync = ~sync;  // 24x2 bit sync

initial begin
  fpga_reset = 1'b1;  // fpga reset is extra-long to get past init delays of SPINOR; in reality, this is all handled by the config engine
  #1_000;
  fpga_reset = 1'b0;
end   

wire tx0, tx1;

top dut (
    .i2s0_clk(mclk),
    .i2s0_tx(tx0),
    .i2s0_rx(tx0),
    .i2s0_sync(sync),
    
    .i2s1_clk(mclk),
    .i2s1_sync(sync),
    .i2s1_tx(tx1),
    
    .clk12(clk12),
    .rst(fpga_reset)
);

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
