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
from litex.soc.integration.soc import SoCRegion, SoCIORegion
from litex.soc.integration.builder import *
from litex.soc.interconnect import wishbone
from litex.soc.interconnect import axi
from litex.soc.interconnect import ahb
from litex.soc.interconnect.csr import *
from litex.soc.interconnect.csr_eventmanager import *
from litex.soc.integration.soc import SoCBusHandler
from litex.soc.integration.doc import AutoDoc,ModuleDoc

# Interrupt emulator -------------------------------------------------------------------------------

class InterruptBank(Module, AutoCSR):
    def __init__(self):
        self.submodules.ev = EventManager()

# IOs/Interfaces -----------------------------------------------------------------------------------

def get_common_ios():
    return [
        # Clk/Rst.
        ("aclk", 0, Pins(1)),
        ("rst", 0, Pins(1)),
        ("interrupt", 0, Pins(32)),
        # trimming_reset is a reset vector, specified by the trimming bits. Only loaded if trimming_reset_ena is set.
        ("trimming_reset", 0, Pins(32)),
        ("trimming_reset_ena", 0, Pins(1)),
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

class CsrTest(Module, AutoCSR, AutoDoc):
    def __init__(self):
        self.csr_wtest = CSRStorage(32, name="wtest", description="Write test data here")
        self.csr_rtest = CSRStatus(32, name="rtest", description="Read test data here")
        self.comb += [
            self.csr_rtest.status.eq(self.csr_wtest.storage + 0x1000_0000)
        ]

# Needs to handle: pulsed or level; priorities, from 0-255
class IrqBank(Module, AutoCSR, AutoDoc):
    def __init__(self, ints_per_bank=16):
        self.submodules.ev = EventManager()
        # TODO

class IrqArray(Module, AutoCSR, AutoDoc):
    """Interrupt Array Handler"""
    def __init__(self, banks=16, ints_per_bank=16):
        self.intro = ModuleDoc("""
`IrqArray` provides a large bank of interrupts for SoC integration. It is different from e.g. the NVIC
or CLINT in that the register bank is structured along page boundaries, so that the interrupt handler CSRs
can be owned by a specific virtual memory process, instead of bouncing through a common handler
and forcing an inter-process message to be generated to route interrupts to their final destination.
        """)
        # TODO

class ResetValue(Module, AutoCSR, AutoDoc):
    """Actual reset value"""
    def __init__(self, default_value, trimming_reset, trimming_reset_ena):
        self.intro = ModuleDoc("""
`ResetValue` captures the actual reset value present at a reset event. The reason this is
necessary is because the reset value could either be that built into the silicon, or it could
come from a "trimming value" that is programmed via ReRAM bits. This vector can be read back to
confirm that the reset vector is, in fact, where we expected it to be.

`default_value` specifies what the value would be if the `trimming_reset` ReRAM bits are not
enabled with `trimming_reset_ena`.
        """)
        self.reset_value = CSRStatus(32, name="pc", description="Latched value for PC on reset")
        latched_value = Signal(32, reset_less=True)
        self.sync += [
            If(ResetSignal(),
                If(trimming_reset_ena,
                    latched_value.eq(trimming_reset)
                ).Else(
                    latched_value.eq(default_value)
                )
            ).Else(
                latched_value.eq(latched_value)
            )
        ]
        self.comb += self.reset_value.status.eq(latched_value)

# cramSoC -------------------------------------------------------------------------------------

class cramSoC(SoCCore):
    # I/O range: 0x80000000-0xfffffffff (not cacheable)
    SoCCore.mem_map = {
        "reram"     : 0x6000_0000, # +3M
        "sram"      : 0x6100_0000, # +2M
        "p_axi"     : 0x4000_0000, # +256M  # this is an IO region
        "vexriscv_debug": 0xefff_0000,
        "csr"       : 0x5800_0000,
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

        # SoCMini ----------------------------------------------------------------------------------
        reset_address = self.mem_map["reram"]
        SoCMini.__init__(self, platform, sys_clk_freq,
            cpu_type             = "vexriscv_axi",
            csr_paging           = 4096,  # increase paging to 1 page size
            csr_address_width    = 16,    # increase to accommodate larger page size
            cpu_reset_address    = reset_address,
            cpu_custom_memory    = True,
            bus_standard         = "axi-lite",
            bus_interconnect     = "crossbar",
            # bus_timeout          = None,
            with_ctrl            = False,
            io_regions           = {
                # Origin, Length.
                0x4000_0000 : 0x2000_0000,
                0xa000_0000 : 0x6000_0000,
            },
            **kwargs)

        self.cpu.use_external_variant("deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_cramSoC.v")
        self.cpu.add_debug()
        # self.cpu.set_reset_address(reset_address)
        self.cpu.disable_reset_address_check()
        trimming_reset = Signal(32)
        trimming_reset_ena = Signal()
        self.submodules.resetvalue = ResetValue(reset_address, trimming_reset, trimming_reset_ena)
        self.comb += [
            trimming_reset.eq(platform.request("trimming_reset")),
            trimming_reset_ena.eq(platform.request("trimming_reset_ena")),
            self.cpu.trimming_reset.eq(trimming_reset),
            self.cpu.trimming_reset_ena.eq(trimming_reset_ena),
        ]

        # Break out custom busses to pads ----------------------------------------------------------
        # All appear as "memory", to avoid triggering interference from the bushandler automation
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

                #ibus_region =  SoCRegion(origin=self.mem_map["reram"], size=3 * 1024 * 1024, cached=True)
                #self.bus.add_slave(name="ibus", slave=ibus, region=ibus_region)
                platform.add_extension(ibus_ios)
                ibus_pads = platform.request("ibus_axi")
                self.comb += ibus.connect_to_pads(ibus_pads, mode="master")
            elif 'dbus' in mem_bus:
                dbus = mem_bus[1]
                #dbus_region =  SoCRegion(origin=self.mem_map["sram"], size=2 * 1024 * 1024, cached=True)
                #self.bus.add_slave(name="dbus", slave=dbus, region=dbus_region)
                platform.add_extension(dbus.get_ios("dbus_axi"))
                dbus_pads = platform.request("dbus_axi")
                self.comb += dbus.connect_to_pads(dbus_pads, mode="master")
            elif 'pbus' in mem_bus:
                p_bus = mem_bus[1]
                platform.add_extension(p_bus.get_ios("p_axi"))
                p_pads = platform.request("p_axi")
                self.sync += p_bus.connect_to_pads(p_pads, mode="master") # was comb
            else:
                print("Unhandled AXI bus from CPU core: {}".format(mem_bus))

        # Debug ------------------------------------------------------------------------------------
        platform.add_extension(get_debug_ios())
        jtag_pads = platform.request("jtag")
        self.cpu.add_jtag(jtag_pads)

        int_pads = platform.request("interrupt")
        self.comb += [
            self.cpu.interrupt.eq(int_pads)
        ]

        # Test module
        self.submodules.csrtest = CsrTest()


# Build --------------------------------------------------------------------------------------------
def main():
    # Arguments.
    from litex.soc.integration.soc import LiteXSoCArgumentParser
    parser = LiteXSoCArgumentParser(description="LiteX standalone SoC generator")
    target_group = parser.add_argument_group(title="Generator options")
    target_group.add_argument("--name",          default="cram_axi", help="SoC Name.")
    target_group.add_argument("--build",         action="store_true", help="Build SoC.")
    target_group.add_argument("--sys-clk-freq",  default=int(50e6),   help="System clock frequency.")
    parser.add_argument(
        "-D", "--document-only", default=False, action="store_true", help="dummy arg to be consistent with cram_soc"
    )
    parser.add_argument(
        "-S", "--sim", default=False, action="store_true", help="Run simulation. Changes `comb` description style slightly for improved simulator compatibility."
    )
    args = parser.parse_args()

    # Generate the SoC
    soc = cramSoC(
        name         = args.name,
        sys_clk_freq = int(float(args.sys_clk_freq)),
    )
    builder = Builder(soc, output_dir="build", csr_csv="build/csr.csv", csr_svd="build/software/core.svd",
        compile_software=False, compile_gateware=False)
    builder.software_packages=[] # necessary to bypass Meson dependency checks required by Litex libc
    # turn off regular_comb for simulation. Can't just use ~ because Python.
    if args.sim:
        rc=False
    else:
        rc=True
    builder.build(build_name=args.name, run=False, regular_comb=rc)


if __name__ == "__main__":
    main()
