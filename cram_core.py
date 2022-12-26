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
        # wfi active signal
        ("wfi_active", 0, Pins(1)),
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

# Mailbox ------------------------------------------------------------------------------------------
class Mailbox(Module, AutoCSR, AutoDoc):
    def __init__(self, fifo_depth=1024):
        self.intro = ModuleDoc("""Mailbox: An inter-CPU mailbox
The `Mailbox` is a bi-directional, inter-CPU mailbox for delivering messages between CPUs
without requiring shared memory.

A single message consists of a packet up to {} words long, where each word is 32 bits in length.

Both CPUs are considered as "peers"; each can initiate a packet at-will.
""".format(fifo_depth))
        self.data_transfer = ModuleDoc("""Data Transfer Protocol
The protocol has two levels, one at a MAC level, and one at an APP level.

The MAC level protocol controls synchronization of data transfer, and the transfer of single, fully-formed
packets between the devices. The MAC protocol is implemented by this hardware block.

The APP protocol is managed by the operating system, and can be considered advisory as
just one of many ways to use this system to communicate between CPUs. However, it helps to ground
the protocol in an APP framework as some details of the MAC impact the APP framework, especially
around synchronization and conflict avoidance.

Each peer has a channel to write data to the other peer, using 32 bits `dat`, one `valid` to
indicate when data is available, and `ready` to indicate when the data has been latched by
the corresponding peer's hardware FIFO. Generally, `valid`/`ready` is managed exclusively by
hardware state machines and the host CPUs are not aware of these signals; they mainly exist
to avoid overflowing the FIFO in the case that one is pipelining multiple packets through
the interface.

There is an additional `done` signal which is asserted for exactly one cycle, and it indicates
to the other peer that the sender has finished writing all the data for a given packet. The `done`
signal is provided so that the corresponding peer does not need to busy-monitor the FIFO depth.

    .. wavedrom::
        :caption: Sending four words of data, followed by a `done`.

        { "signal" : [
            {"name": "clk",       "wave": "p........" },
            {"name": "dat",       "wave": "x=.x===xx", "data" : ["D0", "D1", "D2", "D3", "D4"]},
            {"name": "valid",     "wave": "01.01..0."},
            {"name": "ready",     "wave": "0.101..0."},
            {"name": "done",      "wave": "0......10"},
        ]}

The above example shows a packet with a length of four words being transmitted. The first word
takes an extra cycle to be acknowledged; the remaining three are immediately accepted. The `done`
signal could come as early as simultaneously with the last `ready`, but in practice it comes a couple
cycles later since it would be triggered by a write from the CPU to the `done` register.

The data transfer protocol is symmetric across the peers.
        """)
        self.abort_doc = ModuleDoc("""Abort Protocol
The abort protocol is used to recover the protocol to a known state: all FIFOs empty, and both hosts
state machines in an idle state. This is accomplished by cross-wiring `w_abort` on the sending
peer to `r_abort` on the corresponding peer. Either peer can assert `w_abort`, and it must stay asserted
until `r_abort` is pulsed to acknowledged the abort condition. At the conclusion of the protocol,
both FIFOs are empty and their protocol state machines are idle.

    .. wavedrom::
        :caption: Normal abort

        { "config": {skin : "default"},
        "signal" : [
            {"name": "clk",         "wave": "p........" },
            {"name": "w_abort",     "wave": "01....0.."},
            {"name": "r_abort",     "wave": "0....10.."},
            {},
            {"name": "w_state",     "wave": "==...==..", "data" : ["XXX", "REQ", "ACK", "IDLE"]},
            {"name": "w_fifo",      "wave": "x=.......", "data" : ["EMPTY"]},
            {},
            {"name": "r_state",     "wave": "=..=.==..", "data" : ["XXX", "PROC", "ACK", "IDLE"]},
            {"name": "r_fifo",      "wave": "x....=...", "data" : ["EMPTY"]},
        ]}

In the diagram above, `w_abort` is asserted; on the rising edge of `w_abort`, we see that the `w_fifo`
is already empty and the state machine is in the `REQ` (requesting abort) state.

The corresponding peer's `r_state` machine may take an arbitrarily long period of time to leave its
current state and enter a `PROC` or processing state, where it is cleaning up from the abort. Once
it can guarantee its FIFOs are empty and is ready to transition to idle, it pulses `r_abort` for exactly
one cycle and both machines transition to an `IDLE`/empty FIFO state.

In order to make the case work where both peers attempt to initiate an abort at the same time, the
initiator guarantees that on asserting `w_abort` it is immediately ready to act on an `r_abort` pulse.
This means the hardware guarantees two things:

- All FIFOs are cleared by the request
- The incoming `abort` response line is prevented from generating an interrupt

    .. wavedrom::
        :caption: Edge case: simultaneous abort

        { "config": {skin : "default"},
        "signal" : [
            {"name": "clk",         "wave": "p........" },
            {"name": "w_abort",     "wave": "0.10....."},
            {"name": "r_abort",     "wave": "0.10....."},
            {},
            {"name": "w_state",     "wave": "=.==.....", "data" : ["XXX", "REQ", "IDLE"]},
            {"name": "w_fifo",      "wave": "x.=......", "data" : ["EMPTY"]},
            {"name": "w_abort_ack", "wave": "xx1......"},
            {},
            {"name": "r_state",     "wave": "=.==.....", "data" : ["XXX", "REQ", "IDLE"]},
            {"name": "r_fifo",      "wave": "x.=......", "data" : ["EMPTY"]},
            {"name": "r_abort_ack", "wave": "xx1......"},
        ]}

Above is the rare edge case of a cycle-perfect simultaneous abort request. It "just works", and
both devices immediately transition from `REQ`->`IDLE`.

    .. wavedrom::
        :caption: Edge case: semi-simultaneous abort

        { "config": {skin : "default"},
        "signal" : [
            {"name": "clk",         "wave": "p........" },
            {"name": "w_abort",     "wave": "0.1.0...."},
          {"name": "r_abort",     "wave": "0..10....", "node": "...a....."},
            {},
            {"name": "w_state",     "wave": "=.===....", "data" : ["XXX", "REQ", "ACK", "IDLE"]},
            {"name": "w_fifo",      "wave": "x.=......", "data" : ["EMPTY"]},
            {"name": "w_abort_ack", "wave": "xx0......"},
            {},
            {"name": "r_state",     "wave": "=..==....", "data" : ["XXX", "REQ", "IDLE"]},
            {"name": "r_fifo",      "wave": "x..=.....", "data" : ["EMPTY"]},
            {"name": "r_abort_ack", "wave": "xxx1....."},
            {"name": "r_abort_int", "wave": "0.....1..", "node": "......b.."},
        ],
          "edge" : ['a~>b race condition']}

Above is the more practical edge case where one peer has initiated an abort, and the other
is preparing to initiate at the same time, but is perhaps a cycle or two later. In this case
the first initiator goes through the `ACK` cycle. However, the second initiator would kick off
an abort-loop if it does not first mask its abort interrupt before attempting to initiate its
abort. A failure to mask the interrupt would mean the receiver may respond to the incoming request
as a new request, even though it should have been an acknowledgement.

A race condition is still unavoidable because it takes time for a CPU to decide to initiate an
abort; during the processing of the "store" instruction, an abort interrupt could come in,
and generate a late "race condition" interrupt. Thus an `abort_acknowledged` bit is generated,
which is set on the peer that is the later of the two asserting the abort signal, or by both if they
happen to be perfectly simultaneously. This bit should be checked
by the peer when handling an interrupt. This signal, when asserted, means an incoming abort signal
was already acknowledged, and the latest interrupt should be disregraded.
        """)

        self.abort_doc = ModuleDoc("""Application Protocol

The application protocol wraps a packet format around each packet. The general format of
a packet is as follows:

* Word 0

  * Bit 31 - set if a response; cleared if initiating
  * Bit 30:16 - sequence number
  * Bit 15:10 - tag
  * Bit 9:0 - length in words of the packet, excluding word 0

The sequence number allows responses to occur out of order with respect to requests.

The tag encodes the operation intended by the packet. Within the tag, further meaning
may be ascribed to later fields in the packet. As an example, a `tag` of 0 could indicate
an RPC, and in this case `word 1` would encode the desired system call, and then
the subsequent words would encode arguments to that system call. After processing the data,
the response to this system call would be returned to the corresponding peer, using the same
`tag` and `sequence number`, but with the `response` bit set.

Further definition of the protocol would extend from here, for example, a `send` of data
could use a tag of `1`, and the response would be with the same tag
and sequence number to acknowledge that the sent data was accepted, with the length
field specifying the number of words that were accepted.
""")
        depth_bits = log2_int(fifo_depth)
        layout = [
            # data going to the peer. `valid` indicates data is ready to be written; `ready` acknowledges the current write
            ("w_dat", 32, DIR_M_TO_S),
            ("w_valid", 1, DIR_M_TO_S),
            ("w_ready", 1, DIR_S_TO_M),
            # Interrupt signal to peer. A single pulse used to indicate when the full packet is in the FIFO.
            ("w_done", 1, DIR_M_TO_S),
            # data coming from the peer
            ("r_dat", 32, DIR_S_TO_M),
            ("r_valid", 1, DIR_S_TO_M),
            ("r_ready", 1, DIR_M_TO_S),
            # Interrupt signal from peer. A single pulse used to indicate when the full packet is in the FIFO.
            ("r_done", 1, DIR_S_TO_M),
            # bi-directional sync signal. This can be used at any time to recover the protocol to a known state.
            # The signal is cross-wired, e.g. `w_abort` on one peer connects to `r_abort` on the other.
            # Either peer can assert `w_abort`, and it must stay asserted until `r_abort` is pulsed to acknowledge the abort.
            # Asserting `w_abort` immediately clears the sender's FIFO, and blocks new data from being loaded until `r_abort` is asserted.
            # In the case that both happen to simultaneously assert `w_abort`, the protocol completes in one cycle.
            ("w_abort", 1, DIR_M_TO_S),
            ("r_abort", 1, DIR_S_TO_M),
        ]
        self.wdata = CSRStorage(32, name="wdata", description="Write data to outgoing FIFO.")
        self.rdata = CSRStatus(32, name="rdata", description="Read data from incoming FIFO.")
        self.submodules.ev = ev = EventManager()
        self.ev.int = EventSourcePulse(name="int", description="Triggers when a full packet is available")
        self.ev.abort = EventSourceProcess(name="abort", description="Triggers when abort is asserted by the peer", edge="rising")
        self.ev.finalize()
        self.status = CSRStatus(fields=[
            CSRField(name="rx_words", size=depth_bits, description="Number of words available to read"),
            CSRField(name="tx_avail", size=depth_bits, description="Number of words free to write"),
            CSRField(name="abort_in_progress", size=1, description="This bit is set if an `aborting` event was initiated and is still in progress."),
            CSRField(name="abort_ack", size=1,
            description="""This bit is set by the peer that acknowledged the incoming abort
(the later of the two, in case of an imperfect race condition). The abort response handler should
check this bit; if it is set, no new acknowledgement shall be issued. The bit is cleared
when an initiator initiates a new abort. The initiator shall also ignore the state of this
bit if it is intending to initiate a new abort cycle.""")
        ])
        self.control = CSRStorage(fields=[
            CSRField(name="abort", size=1, description=
            """Write `1` to this field to both initiate and acknowledge an abort.
Empties both FIFOs, asserts `aborting`, and prevents an interrupt from being generated by
an incoming abort request. New reads & writes are ignored until `aborted` is asserted
from the peer.""", pulse=True)
        ])
        self.packet = CSRStorage(1, name="done", description="Writing a `1` to this field indicates that the packet is done. There is no need to clear this register after writing.")

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

The `CoreUser` signal is not cycle-precise; it will assert roughly 2 cycles after the `satp` is updated.
Furthermore, the `satp` ASID field is an advisory field that isn't used by CPU hardware to enforce
page access. You can think of `coreuser` as a signal that the kernel can control to indicate if the
context we are swapping into should be trusted. Fortunately, any update to `satp` in a virtual memory OS
should be followed by an `sfence` instruction (to invalidate TLB mappings etc.), which gives time for
the `coreuser` signal to propagate through the pipeline.

Thus in practice by the time the first instruction of user code runs, `coreuser` should be set properly.
However, from  a security audit perspective, it is important to keep in mind that there is a race condition between
the `satp` setting and user code execution.
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
        # instantiate as an explicitly reset FF since this signal *must* have a defined reset behavior
        self.specials += Instance(
            "fdre_cosim",
            i_C=ClockSignal(),
            i_D=1,
            i_R_n=~ResetSignal(),
            o_Q=protect,
            i_CE=self.protect.storage,
        )

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

        # WFI breakout -----------------------------------------------------------------------------
        wfi_active = platform.request("wfi_active")
        self.comb += wfi_active.eq(self.cpu.wfi_active)

        # Interrupt Array --------------------------------------------------------------------------
        irqpins = platform.request("irqarray")
        for bank in range(IRQ_BANKS):
            pins = getattr(irqpins, 'bank{}'.format(bank))
            setattr(self.submodules, 'irqarray{}'.format(bank), IrqArray(bank, pins))
            self.irq.add("irqarray{}".format(bank))

        # Ticktimer --------------------------------------------------------------------------------
        self.submodules.ticktimer = ticktimer.TickTimer(2000, 800e6)
        self.irq.add("ticktimer")

        # Mailbox ----------------------------------------------------------------------------------
        self.submodules.mailbox = Mailbox(fifo_depth=1024)
        self.irq.add("mailbox")

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
