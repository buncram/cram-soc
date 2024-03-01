# Better IO

The Better IO (BIO) is a microcoded I/O peripheral which consists of four of Claire Xenia Wolfe's
PicoRV RV32E+ cores running out of a private memory space, with the top 16 registers wired to
peripheral and synchronization functions.

The PicoRV cores are configured as follows:
  - R[31:16] disabled (RV32E mode)
  - Custom `picorv32_regs` module
  - Barrel shifter enabled
  - Dual port register file enabled
  - Compressed instructions on

All four PicoRV cores fetch instructions out of a shared 1kx32, 4-read, 1-write RAM. The RAM is accessible from the host via the 1-write port, and is memory mapped into the host memory space. Reads from the instruction space only succeed if core #3 is stopped, as it shares a read port with the host.

The BIO is managed via a host interface, which is memory-mapped into the host system. To be clear, the BIO
PicoRV cores have no access to the host address space; the BIO CPU cores live entirely within their private
memory space.

`ld` instructions read data out of the shared instruction memory through the same port as the instruction fetch.

`st` instructions on all cores are effectively NOPs, except for core #0 which can have its `st` result wired into the instruction memory while the core is running, at the cost of the host being unable to write to the instruction memory as execution is active. This capability is set by a host register.

Each core has a `quantum` signal, configured by a host register, which is derived by creating one `quantum` pulse every `quantum_count` cycles of `aclk`.

All four cores run at `aclk` (800 MHz) speed, but each core has an independent fetch-stall signal. Each core also has an independent enable/run signal, which is automatically synchronized to the quantum signal.

Each core has a reset vector that can be independently set.

Each core can be independently configured to wrap the PC back to the reset value when a fetch happens to a prescribed address. If none is set, the PC will wrap around to 0 if it increments off the end of instruction memory.

## Writing Code for the RV32E+C

### Only R0-R15 Have Guaranteed Arithmetic Behaviors
Note that the PicoRV is wired to only correctly execute code out of R0-R15. Thus, the upper
registers can only safely be accessed with an explicit `mv` instruction to or from the register;
for example, immediate opcodes won't always decode correctly when used in combination with upper
registers for arithmetic.

For example, it might be tempting to use

`li x26, 0xFF00`

to setup the GPIO mask. This won't work, because the immediate does not decode correctly
(you end up getting 0xFFFFFF00 in x26, because the immediate is 0 in the pipe). Instead, use
the two-instruction sequence

`li x2, 0xFF00`
`mv x26, x2`

to load these registers. Same goes for the FIFO registers, etc. Two-operand arithmetic, however,
seems to work correctly when accessing the upper registers, but this has not been thoroughly characterized.

### Compressed Instructions
The core will execute "C" instructions (which is not part of the RV32E spec). Beware when laying
out the initial jump vector table, that most assemblers will emit a compressed jump if your code
starts in the bottom 2k of instruction memory, but will emit an uncompressed instruction if it's
farther out. This can cause some troubles laying out the vector table if your code extends
beyond the 2k limit.

## Inter-core FIFO bank R16-R19

- R[16:19] are depth-K FIFOs (K=8 by default), such that any core can read from the head of the FIFO, and any core can write to the tail of the FIFO with a `mov` instruction.
- Furthermore, the host can read from or write to any of these FIFOs, by accessing a single memory-mapped register per FIFO read or write (8 total memory-mapped host registers: 4 FIFOs x (1r + 1w))
- The semantics of the FIFO are such that on any given cycle, any CPU or the host reading the FIFO will remove exactly one item, and all CPUs or host get the same item
- The intention is that generally, a programmer will have exactly one producer and one consumer per FIFO.
However, the following contention rules are provided because someone can and will try to do something outside
of the intention:

- If multiple CPUs write to the FIFO on the same cycle, the core with the lower number takes priority on the write; the host has a higher priority than any CPU. Only one piece of data can be written per cycle.
- If a core tries to move data from an empty FIFO, it will stall until data is available.
- If a core tries to write data to a full FIFO, the write will stall until there is space for the write. Note that data can still be lost if multiple cores are contending for the same write slot.

Remember, just because the rules allow it, doesn't mean it's a good idea. It just means it's defined behavior.

FIFO events:

The host can configure a watermark flags on every FIFO to trigger an event when the value is less than, equal to, and/or greater than the specified value. lt, eq, gt are a bit mask of statuses that are OR'd together to create the event. Each FIFO has two configurable event flags.

These events are combinable into four IRQ lines that go to the host system. The intention is that the IRQ lines should be routed to the MDMA block for automated refilling of FIFOs.

## Halt to Quantum R20

R20 is a dummy register that discards any data written. However, when any CPU writes to R20, its clock is stalled until the next quantum pulse.

Normally, the code loop run by one core should finish before the quantum is up, so that every CPU runs its loop in sync. However, if a CPU does not end its code with a `mov r20, r0`, it will free-run.

When the `quanta` value is identical across all cores, the cores will all run in lock-step with each other. However, the user is free to configure the per-core `quanta` however they see fit.

Reads to this register return an undefined value and have no effect on the clocking of the block.

## GPIO R21-26

GPIOs are wired to the cores as follows:

- All cores can read R21 at any time to get the state of a pin. A per-core host register configures if R21 updates only at the rising edge of every quantum, or if the values are directly piped in from the I/O pin at `aclk` rate. R21 is not masked by R26.

For the following registers, the result only reflects to the GPIO bank on the rising edge of every quantum. Only the last update to a given register will have any effect. This also allows a core to both set and clear bits on a GPIO simultaneously on a given quantum, even though the instructions are executed separately.

- Writes to R21 will be masked by R26 and "clobber" all unmasked values on the GPIO block
- Bits set on a write to R22 will set the corresponding GPIO pin
- Bits cleared on a write to R23 will clear the corresponding GPIO pin
- Bits set on a write to R24 will drive the corresponding GPIO pin
- Bits cleared on a write to R25 will tristate the corresponding GPIO pin
- Bits set in R26 will mask operations to R22-25. It is all 1's on reset. Reads from R26 return the mask state.
- Reads from R22-25 are undefined, but do not block execution.

In the case of a conflict (set and clear simultaneously), the command is ignored, and the previous state is kept.

If the goal is to have a constant bit-pattern appear on a set of GPIO pins, the code
to do that would be a `mov r22, const` followed by a `mov r23, const`. This works because
of the inverted sense of the set/clear on r23 and r23, allowing the same value to be re-used
to compose a single bit pattern.

## Inter-core Events R27, R28, R29, R30

A core indicates which events it is sensitive to by writing a `1` into a bit in R27.

A core can set an event bit by writing a `1` to R28. This write does not regard the R27 mask.

A core can clear an event bit by writing a `1` to R29. This write does not regard the R27 mask.

A core can wait until an event happens by reading R30. It will stall until all of the bits marked as sensitive in R27 are set. The stall is computed at `aclk` rates, e.g. if one needs synchronization to the quantum timer, the code sequence should be `mov r20, r0` followed by `mov ra, r29`.

Bits 0-7 on R30 are wired to the FIFO level event flags; these bits cannot be set or cleared by R28 and R29.

The host can read the contents of the aggregated events in real-time, and an interrupt can be generated based on an enable mask AND'd with the contents of R30.

The host also similarly has bit-wise set/clear write-only registers that can manipulate the aggregated events. The host's set/clear commands have priority over all of the cores. In case of simultaneous set/clear, the conflicting bits are ignored and the previous state is kept.

# R31 core ID and cycle count

Reading R31 returns the ID number of the core (0-3), and number of aclk cycles elapsed since reset:

- r31[30:31] contains the ID of the core (0-3)
- r31[0:29] contains the elapsed aclk count

The count will wrap around on overflow.

## Missed Quantum Register

This is a host register, one per core, that counts the number of quanta that were missed by a given core (e.g., a quanta pulse has passed without the core stalling on the quanta pulse). This is primarily for debugging code loops.

