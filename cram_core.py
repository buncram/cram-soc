#!/usr/bin/env python3
#
# Copyright (c) 2022 Cramium Labs, Inc.
# Derived from litex_soc_gen.py:
# Copyright (c) 2022 Florent Kermarrec <florent@enjoy-digital.fr>
# SPDX-License-Identifier: BSD-2-Clause

import argparse

from migen import *

from litex.build.generic_platform import *

from litex.soc.integration.soc_core import *
from litex.soc.integration.soc import SoCRegion
from litex.soc.integration.builder import *
from litex.soc.interconnect import wishbone
from litex.soc.interconnect import axi
from litex.soc.interconnect import ahb

# IOs/Interfaces -----------------------------------------------------------------------------------

def get_common_ios():
    return [
        # Clk/Rst.
        ("aclk", 0, Pins(1)),
        ("rst", 0, Pins(1)),
        ("hclk", 0, Pins(1)),
        ("hrst", 0, Pins(1)),
    ]

def get_debug_ios():
    return [
        ("jtag", 0,
            Subsignal("tdi",Pins(1)),
            Subsignal("tdo",Pins(1)),
            Subsignal("tms",Pins(1)),
            Subsignal("tck",Pins(1)),
            Subsignal("trst",Pins(1)),
        )
    ]

# Platform -----------------------------------------------------------------------------------------

class Platform(GenericPlatform):
    def build(self, fragment, build_dir, build_name, **kwargs):
        os.makedirs(build_dir, exist_ok=True)
        os.chdir(build_dir)
        conv_output = self.get_verilog(fragment, name=build_name)
        conv_output.write(f"{build_name}.v")

# cramSoC -------------------------------------------------------------------------------------

class cramSoC(SoCCore):
    # I/O range: 0x80000000-0xfffffffff (not cacheable)
    SoCCore.mem_map = {
        "rom":             0x60000000,
        "sram"     : 0x2000_0000,
        "main_ram" : 0x6100_0000,
        "vexriscv_debug":  0xefff0000,
        "csr":             0xa0000000,
        "p_axi":           0x40000000,
    }

    def __init__(self, sys_clk_freq=int(100e6),
                 bios_path='boot/boot.bin',
                 **kwargs):
        global bios_size

        # Platform ---------------------------------------------------------------------------------
        platform = Platform(device="", io=get_common_ios())
        platform.name = "litex_soc"

        # CRG --------------------------------------------------------------------------------------
        self.submodules.crg = CRG(
            clk = platform.request("aclk"),
            rst = platform.request("rst"),
        )

        hclk = platform.request("hclk")
        hrst = platform.request("hrst")
        self.clock_domains.cd_hclk = ClockDomain()
        self.comb += [
            self.cd_hclk.clk.eq(hclk),
            self.cd_hclk.rst.eq(hrst),
        ]

        # CPU
        reset_address = self.mem_map["rom"]

        # SoCCore ----------------------------------------------------------------------------------
        SoCCore.__init__(self, platform, sys_clk_freq,
            integrated_rom_size  = 0,    # don't use default ROM
            integrated_rom_init  = None, # bios_path,
            integrated_sram_size = 0,    # Use external SRAM for boot code
            cpu_type             = "vexriscv_axi",
            csr_paging           = 4096,  # increase paging to 1 page size
            csr_address_width    = 16,    # increase to accommodate larger page size
            with_uart            = False, # implemented manually to allow for UART mux
            cpu_reset_address    = reset_address,
            with_ctrl            = False,
            with_timer           = True, # override default timer with a timer that operates in a low-power clock domain
            bus_standard         = "axi",
            **kwargs)
        # Litex will always try to move the ROM back to 0.
        # Move ROM and RAM to uncached regions - we only use these at boot, and they are already quite fast
        # this helps remove their contribution from the cache tag critical path
        if self.mem_map["rom"] == 0:
            self.mem_map["rom"] += 0x60000000

        # CPU --------------------------------------------------------------------------------------
        self.cpu.use_external_variant("deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_cramSoC.v")
        self.cpu.add_debug()
        self.cpu.set_reset_address(0x6000_0000)

        # MMAP Master Interface --------------------------------------------------------------------
        # p_bus = wishbone.Interface(data_width=32)
        # self.bus.add_slave("csr", p_bus, SoCRegion(origin=self.mem_map["csr"], size=0x1000_0000, cached=False))
        #p_bus = ahb.Interface()
        #self.bus.add_slave("csr", p_bus, SoCRegion(origin=self.mem_map["csr"], size=0x1000_0000, cached=False))
        # ahb_bus = ahb.Interface()
        # self.submodules += ahb.AHB2Wishbone(ahb_bus, p_bus)
        # platform.add_extension(p_bus.get_ios("ahb_bus"))
        # ahb_pads = platform.request("ahb_bus")
        # left off: write some custom connector code here
        # self.comb += ahb_bus.connect_to_pads(ahb_pads, mode="master")

        p_bus = axi.AXILiteInterface()
        p_region = SoCRegion(origin=self.mem_map["p_axi"], size=0x1000_0000, cached=False)
        self.bus.add_slave(name="p_axi", slave=p_bus, region=p_region)
        platform.add_extension(p_bus.get_ios("p_axi"))
        p_pads = platform.request("p_axi")
        self.comb += p_bus.connect_to_pads(p_pads, mode="master")

        for mem_bus in self.cpu.memory_buses:
            if 'ibus' in mem_bus:
                ibus = mem_bus[1]
                if True:
                    ibus_ios = ibus.get_ios("ibus_axi")
                else: # aborted attempt to filter what gets connected out
                    suppress = [
                        'awvalid',
                        'awready',
                        'awaddr',
                        'awburst',
                        'awcache',
                        'awlen',
                        'awlock',
                        'awprot',
                        'awsize',
                        'awqos',
                        'awid',
                        'awregion',
                        'wvalid',
                        'wready',
                        'wlast',
                        'wstrb',
                        'wdata',
                        'wid',
                        'bvalid',
                        'bready',
                        'bresp',
                        'bid',
                    ]
                    ibus_ios_all = ibus.get_ios("ibus_axi")
                    subsignals = []
                    for s in ibus_ios_all[0]:
                        if type(s) is Subsignal:
                            if s.name not in suppress:
                                subsignals += [s]
                        else:
                            subsignals += [s]
                    subsignals = tuple(subsignals)
                    ibus_ios = [subsignals]

                ibus_region =  SoCRegion(origin=self.mem_map["rom"], size=0x1000_0000, cached=True)
                self.bus.add_slave(name="ibus", slave=ibus, region=ibus_region)
                platform.add_extension(ibus_ios)
                ibus_pads = platform.request("ibus_axi")
                print(ibus_pads)
                self.comb += ibus.connect_to_pads(ibus_pads, mode="master")
            elif 'dbus' in mem_bus:
                dbus = mem_bus[1]
                dbus_region =  SoCRegion(origin=self.mem_map["sram"], size=0x1000_0000, cached=True)
                self.bus.add_slave(name="dbus", slave=dbus, region=dbus_region)
                platform.add_extension(dbus.get_ios("dbus_axi"))
                dbus_pads = platform.request("dbus_axi")
                self.comb += dbus.connect_to_pads(dbus_pads, mode="master")
            else:
                print("Unhandled AXI bus from CPU core: {}".format(mem_bus))

        # Debug ------------------------------------------------------------------------------------
        platform.add_extension(get_debug_ios())
        jtag_pads = platform.request("jtag")
        self.cpu.add_jtag(jtag_pads)
        #self.comb += [
            # Export Signal(s) for debug.
        #    jtag_pads.tdi.eq(1),
            # Etc...
        #]

# Build --------------------------------------------------------------------------------------------
def main():
    # Arguments.
    from litex.soc.integration.soc import LiteXSoCArgumentParser
    parser = LiteXSoCArgumentParser(description="LiteX standalone SoC generator")
    target_group = parser.add_argument_group(title="Generator options")
    target_group.add_argument("--name",          default="cram_axi", help="SoC Name.")
    target_group.add_argument("--build",         action="store_true", help="Build SoC.")
    target_group.add_argument("--sys-clk-freq",  default=int(50e6),   help="System clock frequency.")
    builder_args(parser)
    soc_core_args(parser)
    args = parser.parse_args()

    # SoC. Pass 1: make the bios
    soc = cramSoC(
        name         = args.name,
        sys_clk_freq = int(float(args.sys_clk_freq)),
#        **soc_core_argdict(args)
    )
    builder = Builder(soc, output_dir="build", csr_csv="build/csr.csv", csr_svd="build/software/soc.svd",
        compile_software=False, compile_gateware=False)
    builder.software_packages=[] # necessary to bypass Meson dependency checks required by Litex libc
    builder.build(build_name=args.name, run=False)

"""
    # SoC. Pass 2: make the soc
    soc = cramSoC(
        name         = args.name,
        sys_clk_freq = int(float(args.sys_clk_freq)),
        **soc_core_argdict(args)
    )
    builder = Builder(soc, output_dir="build",
        csr_csv="build/csr.csv", csr_svd="build/software/soc.svd",
        compile_software=False, compile_gateware=False)
    builder.software_packages=[] # necessary to bypass Meson dependency checks required by Litex libc

    vns = builder.build()
"""

if __name__ == "__main__":
    main()
