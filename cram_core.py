#!/usr/bin/env python3
#
# Copyright (c) 2022 Cramium Labs, Inc.
# Derived from litex_soc_gen.py:
# Copyright (c) 2022 Florent Kermarrec <florent@enjoy-digital.fr>
# SPDX-License-Identifier: BSD-2-Clause

import argparse
import litex.soc.doc as lxsocdoc

from migen import *

from litex.build.generic_platform import *

from litex.soc.integration.soc_core import *
from litex.soc.integration.soc import SoCRegion, SoCIORegion
from litex.soc.integration.builder import *
from litex.soc.interconnect import wishbone
from litex.soc.interconnect import axi
from litex.soc.interconnect import ahb
from litex.soc.interconnect.csr import *
from litex.soc.integration.soc import SoCBusHandler
from litex.soc.integration.doc import AutoDoc,ModuleDoc

from litex.soc.interconnect.csr_eventmanager import *
from litex.soc.interconnect.csr_eventmanager import _EventSource

from deps.gateware.gateware import ticktimer

# Interrupt emulator -------------------------------------------------------------------------------

class InterruptBank(Module, AutoCSR):
    def __init__(self):
        self.submodules.ev = EventManager()

# IOs/Interfaces -----------------------------------------------------------------------------------
IRQ_BANKS=20
IRQS_PER_BANK=20

def get_common_ios():
    ios = [
        # Clk/Rst.
        ("aclk", 0, Pins(1)),
        ("rst", 0, Pins(1)),
        # `always_on` is an `aclk` replica that is running even when the core `aclk` is stopped.
        # if power management is not supported, tie this directly to `aclk`
        ("always_on", 0, Pins(1)),
        # trimming_reset is a reset vector, specified by the trimming bits. Only loaded if trimming_reset_ena is set.
        ("trimming_reset", 0, Pins(32)),
        ("trimming_reset_ena", 0, Pins(1)),
        # coreuser signal
        ("coreuser", 0, Pins(1)),
    ]
    irqs = ["irqarray", 0]
    for bank in range(IRQ_BANKS):
        irqs += [Subsignal("bank{}".format(bank), Pins(IRQS_PER_BANK))]
    ios += [tuple(irqs)]
    return ios

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

# Interrupts ------------------------------------------------------------------------------------
class EventSourceFlex(Module, _EventSource):
    def __init__(self, trigger, soft_trigger, name=None, description=None):
        _EventSource.__init__(self, name, description)
        self.trigger = trigger
        self.soft_trigger = soft_trigger
        self.comb += [
            self.status.eq(self.trigger | self.soft_trigger),
        ]
        self.sync += [
            If(self.trigger | self.soft_trigger,
                self.pending.eq(1)
            ).Elif(self.clear,
                self.pending.eq(0)
            ).Else(
                self.pending.eq(self.pending)
            ),
        ]

class IrqArray(Module, AutoCSR, AutoDoc):
    """Interrupt Array Handler"""
    def __init__(self, bank, pins):
        self.intro = ModuleDoc("""
`IrqArray` provides a large bank of interrupts for SoC integration. It is different from e.g. the NVIC
or CLINT in that the register bank is structured along page boundaries, so that the interrupt handler CSRs
can be owned by a specific virtual memory process, instead of bouncing through a common handler
and forcing an inter-process message to be generated to route interrupts to their final destination.

The incoming interrupt signals are assumed to be synchronized to `aclk`.

Priorities are enforced entirely through software; the handler must read the `pending` bits and
decide which ones should be handled first.

The `EventSource` is an `EventSourceFlex` which can handle pulses and levels, as well as software triggers.

The interrupt pending bit is latched when the trigger goes high, and stays high
until software clears the event. The trigger takes precedence over clearing, so
if the interrupt source is not cleared prior to clearing the interrupt pending bit,
the interrupt will trigger again.

`status` reflects the instantaneous value of the trigger.

A separate input line is provided so that software can induce an interrupt by
writing to a soft-trigger bit.
        """)
        ints_per_bank = len(pins)
        self.submodules.ev = ev = EventManager()
        self.interrupts = interrupts = Signal(ints_per_bank)
        self.comb += self.interrupts.eq(pins)
        setattr(self, 'bank{}_ints'.format(bank), interrupts)
        soft = CSRStorage(
            size=ints_per_bank,
            description="""Software interrupt trigger register.

Bits set to `1` will trigger an interrupt. Interrupts trigger on write, but the
value will persist in the register, allowing software to determine if a software
interrupt was triggered by reading back the register.

Software is responsible for clearing the register to 0.

Repeated `1` writes without clearing will still trigger an interrupt.""",
            fields=[
                CSRField("trigger", size=ints_per_bank, pulse=True)
            ])
        for i in range(ints_per_bank):
            bit_int = EventSourceFlex(
                trigger=interrupts[i],
                soft_trigger=soft.fields.trigger[i],
                name='source{}'.format(i),
                description='`1` when a source{} event occurs. This event uses an `EventSourceFlex` form of triggering'.format(i)
            )
            setattr(ev, 'source{}'.format(i), bit_int)

        ev.soft = soft
        ev.finalize()
        # setattr(self, 'evm{}'.format(bank), ev)

# ResetValue ----------------------------------------------------------------------------------

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

# CoreUser ------------------------------------------------------------------------------------

class CoreUser(Module, AutoCSR, AutoDoc):
    """Core User computation logic"""
    def __init__(self, cpu, coreuser):
        self.intro = ModuleDoc("""
`CoreUser` is a hardware signal that indicates that the code executing is in a highly trusted
piece of code. This is determined by examining a configurable combination of the SATP's ASID and
PPN values, allowing the OS to target certain virtual memory spaces as more trusted than
others. `CoreUser` can only be computed when the RISC-V core is in Sv32 mode (that is, virtual
memory has been enabled).

When specifying PPN values, two windows are provided, `a` and `b`. The windows are
computed independently, and then OR'd together. The `a` and `b` windows should be non-overlapping.
If they overlap, or the windows are poorly-specified, the behavior is not guaranteed. The intention
of having two windows is not so that the OS can specify only two processes as `CoreUser`. Rather,
the OS should design to allocate all CoreUser processes within a single range that is protected
by a single window. The alternate window is provided only so that the OS can have a scratch space to
re-organize or shuffle around process spaces at a higher level.

The `CoreUser` signal is not cycle-precise; it will assert roughly 2 cycles after the current
instruction being executed. Thus the signal is meant to be indicative and best used in conjunction
with a virtual memory enabled OS where a kernel context swap (which takes several cycles) is considered to
be non-trusted code. This ensures a break-before-make behavior in the kernel when changing contexts, thus removing
the opportunity for malicious processes to use their first few instructions to intervene and access
sensitive hardware.
        """)
        self.set_asid = CSRStorage(fields=[
            CSRField("asid", size=9, description="ASID to set. Writing to this register commits the value in `trusted` to the specified `asid` value"),
            CSRField("trusted", size=1, description="Set to `1` if the ASID is trusted"),
        ])
        self.get_asid_addr = CSRStorage(fields=[
            CSRField("asid", size=9, description="ASID to read back.")
        ])
        self.get_asid_value = CSRStorage(fields=[
            CSRField("value", size=1, description="Value corresponding to the ASID specified it `get_asid_addr`. `1` means trusted"),
        ])
        self.control = CSRStorage(fields=[
            CSRField("enable", size=1, description="Enable `CoreUser` computation. When set to `1`, the settings are applied; when cleared to `0`, the `CoreUser` signal is always valid. Defaults to `0`."),
            CSRField("asid", size=1, description="When `1`, requires the ASID mapping to be trusted to assert `CoreUser`"),
            CSRField("ppn_a", size=1, description="When set to `1`, requires the `a` `ppn` window to be trusted to assert `CoreUser`"),
            CSRField("ppn_b", size=1, description="When set to `1`, requires the `b` `ppn` window to be trusted to assert `CoreUser`")
        ])
        self.protect = CSRStorage(size=1, description="Writing `1` to this bit prevents any further updates to CoreUser configuration status. Can only be reversed with a system reset.");
        self.window_al = CSRStorage(fields=[
            CSRField("ppn", size=22, description="PPN match value, `a` window lower bound. Matches if ppn is greater than or equal to this value"),
        ])
        self.window_ah = CSRStorage(fields=[
            CSRField("ppn", size=22, description="PPN match value, `a` window upper bound. Matches if ppn is less than or equal to this value (so a value of 255 would match everything from 0 to 255; resulting in 256 total locations"),
        ])
        self.window_bl = CSRStorage(fields=[
            CSRField("ppn", size=22, description="PPN match value, `b` window lower bound. Matches if ppn is greater than or equal to this value"),
        ])
        self.window_bh = CSRStorage(fields=[
            CSRField("ppn", size=22, description="PPN match value, `b` window upper bound. Matches if ppn is less than or equal to this value (so a value of 255 would match everything from 0 to 255; resulting in 256 total locations"),
        ])
        # one-way door for protecting block from updates
        protect = Signal()
        self.sync += [
            If(self.protect.storage,
                protect.eq(1)
            ).Else(
                protect.eq(protect)
            )
        ]
        enable = Signal()
        require_asid = Signal()
        require_ppn_a = Signal()
        require_ppn_b = Signal()
        self.sync += [
            If(protect,
                enable.eq(enable),
                require_asid.eq(require_asid),
                require_ppn_a.eq(require_ppn_a),
                require_ppn_b.eq(require_ppn_b),
            ).Else(
                enable.eq(self.control.fields.enable),
                require_asid.eq(self.control.fields.asid),
                require_ppn_a.eq(self.control.fields.ppn_a),
                require_ppn_b.eq(self.control.fields.ppn_b),
            )
        ]

        asid_lut = Memory(1, 512, init=None, name="asid_lut_nomap")
        self.specials += asid_lut
        asid_rd = asid_lut.get_port(write_capable=False)
        asid_wr = asid_lut.get_port(write_capable=True)
        self.specials += asid_rd
        self.specials += asid_wr

        coreuser_asid = Signal()

        self.comb += [
            asid_rd.adr.eq(cpu.satp_asid),
            coreuser_asid.eq(asid_rd.dat_r),
            asid_wr.adr.eq(self.set_asid.fields.asid),
            asid_wr.dat_w.eq(self.set_asid.fields.trusted),
            asid_wr.we.eq(~protect & self.set_asid.re),
            self.get_asid_value.fields.value.eq(asid_wr.dat_r),
        ]
        window_al = Signal(22)
        window_ah = Signal(22)
        window_bl = Signal(22)
        window_bh = Signal(22)

        self.sync += [
            If(protect,
                window_al.eq(window_al),
                window_ah.eq(window_ah),
                window_bl.eq(window_bh),
                window_bh.eq(window_bh)
            ).Else(
                window_al.eq(self.window_al.fields.ppn),
                window_ah.eq(self.window_ah.fields.ppn),
                window_bl.eq(self.window_bl.fields.ppn),
                window_bh.eq(self.window_bh.fields.ppn),
            ),
            coreuser.eq(
                # always trusted if we're not in Sv32 mode
                ~cpu.satp_mode |
                # always trusted if this check is disabled
                ~enable |
                # ASID-based check
                (coreuser_asid | ~require_asid) &
                # PPN window A check
                (~require_ppn_a | (
                    (cpu.satp_ppn >= window_al) &
                    (cpu.satp_ppn <= window_ah)
                )) &
                # PPN window B check
                (~require_ppn_b | (
                    (cpu.satp_ppn >= window_bl) &
                    (cpu.satp_ppn <= window_bh)
                ))
            )
        ]

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
        self.clock_domains.cd_always_on = ClockDomain()
        self.comb += self.cd_always_on.clk.eq(platform.request("always_on"))

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

        self.cpu.use_external_variant("deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_CramSoC.v")
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

        # CoreUser computation ---------------------------------------------------------------------
        self.submodules.coreuser = CoreUser(self.cpu, platform.request("coreuser"))

        # Interrupt Array --------------------------------------------------------------------------
        irqpins = platform.request("irqarray")
        for bank in range(IRQ_BANKS):
            pins = getattr(irqpins, 'bank{}'.format(bank))
            setattr(self.submodules, 'irqarray{}'.format(bank), IrqArray(bank, pins))
            self.irq.add("irqarray{}".format(bank))

        # Ticktimer --------------------------------------------------------------------------------
        self.submodules.ticktimer = ticktimer.TickTimer(2000, 800e6)
        self.irq.add("ticktimer")

        # CSR bus test loopback register -----------------------------------------------------------
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

    # TODO: add SBT run to generate core whenever this is invoked, to ensure that docs are
    # consistent with the source code.

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
    vns = builder.build(build_name=args.name, run=False, regular_comb=rc)

    soc.do_exit(vns)
    lxsocdoc.generate_docs(
        soc, "build/documentation", note_pulses=True,
        sphinx_extensions=['sphinx_math_dollar', 'sphinx.ext.mathjax'],
        project_name="Cramium SoC (RISC-V Core Complex)",
        author="Cramium, Inc.",
            sphinx_extra_config=r"""
mathjax_config = {
   'tex2jax': {
       'inlineMath': [ ["\\(","\\)"] ],
       'displayMath': [["\\[","\\]"] ],
   },
}""")
    print("LIES! The command is `sphinx-build -M html build/gateware/build/documentation/ build/gateware/build/documentation/_build`")

if __name__ == "__main__":
    main()
