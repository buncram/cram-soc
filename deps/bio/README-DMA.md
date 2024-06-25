# Better IO + Better DMA

The Better IO (BIO) + Better DMA (BDMA) is a microcoded I/O peripheral which consists of four of Claire Xenia Wolfe's
PicoRV RV32E+ cores running out of a private instruction memory space, with the top 16 registers wired to
peripheral and synchronization functions.

The PicoRV cores are configured as follows:
  - R[31:16] disabled (RV32E mode)
  - Custom `picorv32_regs` module
  - Barrel shifter enabled
  - Dual port register file enabled
  - Compressed instructions on

Each PicoRV core fetches instructions out of a private, 512x32, single-ported synchronous RAM. When the core is disabled, the RAM is mapped into the host memory space, where it can be updated with new instructions.

In addition to the memory-mapped code space, there is a register space used by the host to manage parameters such as the clock divider, and other options for synchronization and interrupts.

Addresses below 0x1000_0000 just return aliased copies of the private memory space. Thus, each core resets at address 0x0, but the instructions fetched at 0x0 will be different for each core because each core has a different physical memory.

Addresses 0x1000_0000 and above will attempt to access the system memory through a bus master interface. There is only a single bus master interface, and all cores get the same view of the system above 0x1000_0000.

A core will get a lock on the interface for the entire duration of a load or store transaction, and will continue to keep the lock as long as data is in flight. There is no mechanism to guarantee fairness between the cores: in the case that two cores contend simultaneously, the lower-numbered core has priority.

In general, it's assumed that only one core will use external system resources as a DMA master; if a system programmer wants to try to use multiple cores to access system resources, they can, but there is no guarantee of coherence, locking or fairness.

Each core has a `quantum` signal, configured by a host register, which is derived by creating one `quantum` pulse every `quantum_count` cycles of `fclk`.

All four cores run at `fclk` (800 MHz) speed, but each core has an independent fetch-stall signal. Each core also has an independent enable/run signal, which is automatically synchronized to the quantum signal.

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

## Extended Registers

## Summary

FIFO - 8-deep fifo head/tail access. Cores halt on overflow/underflow.
- x16 r/w  fifo[0]
- x17 r/w  fifo[1]
- x18 r/w  fifo[2]
- x19 r/w  fifo[3]

Quantum - core will halt until host-configured clock divider pules occurs,
or an external event comes in on a host-specified GPIO pin.
- x20 z/w  halt to quantum

GPIO - note clear-on-0 semantics for bit-clear for data pins!
  This is done so we can do a shift-and-move without an invert to
  bitbang a data pin. Direction retains a more "conventional" meaning
  where a write of `1` to either clear or set will cause the action,
  as pin direction toggling is less likely to be in a tight inner loop.
- x21 r/w  write: (x26 & x21) -> gpio pins; read: gpio pins -> x21
- x22 -/w  (x26 & x22) -> `1` will set corresponding pin on gpio
- x23 -/w  (x26 & x23) -> `0` will clear corresponding pin on gpio
- x24 -/w  (x26 & x24) -> `1` will make corresponding gpio pin an output
- x25 -/w  (x26 & x25) -> `1` will make corresponding gpio pin an input
- x26 r/w  mask GPIO action outputs

Events - operate on a shared event register. Bits [7:0] are hard-wired to FIFO
level flags, configured by the host; writes to bits [7:0] are ignored.
- x27 -/w  mask event sensitivity bits
- x28 -/w  `1` will set the corresponding event bit. Only [31:8] are wired up.
- x29 -/w  `1` will clear the corresponding event bit Only [31:8] are wired up.
- x30 r/-  halt until ((x27 & events) == x27), and return unmasked `events` value

Core ID & debug:
- x31 r/-  [31:30] -> core ID; [29:0] -> cpu clocks since reset

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

The host can configure a watermark flags on every FIFO to trigger an event when the value is less than, equal to, and/or greater than the specified value. lt, eq, gt are a bit mask of statuses that are OR'd together to create the event. Each FIFO has two configurable event flag channels, A and B, mapped in a bit vector like [3b, 3a, 2b, 2a, 1b, 1a, 0b, 0a] to the highest bits of the event aggregator.

These events are combinable into four IRQ lines that go to the host system. The intention is that the IRQ lines should be routed to the MDMA block for automated refilling of FIFOs.

## Halt to Quantum R20

R20 is a dummy register that discards any data written. However, when any CPU accesses R20, the accessing CPU's clock is stalled until the next quantum pulse.

The quantum pulse can originate from two sources:
- Internal fractional clock divider, dividing down from `fclk` (one per core)
- External clock pin, selected by flipping `use_extclk` and configuring `extclk_gpio` (one pin per core)

The `extclk` pin will unstall a core waiting on an R20 write on its rising edge. If a falling edge
unstall is desired, use the `io_i_inv` register to invert the input bit. Note that the input signal used to derive `extclk` is always before any quantum snapping. If sampling on a quantum is desired, simply read the GPIO register immediately after a resume from quantum.

Normally, the code loop run by one core should finish before the quantum is up, so that every CPU runs its loop in sync. However, if a CPU does not end its code with a `mov r20, r0`, it will free-run.

When the `quanta` value is identical across all cores, the cores will all run in lock-step with each other. However, the user is free to configure the per-core `quanta` however they see fit.

Reads from this register return `0`.

## GPIO R21-26

GPIOs are wired to the cores as follows:

- All cores can read R21 at any time to get the state of a pin. R21 is not masked by R26.
- Writes to R21 will be masked by R26 and "clobber" all unmasked values on the GPIO block
- Bits set on a write to R22 will set the corresponding GPIO pin
- Bits cleared on a write to R23 will clear the corresponding GPIO pin
- Bits set on a write to R24 will drive the corresponding GPIO pin
- Bits cleared on a write to R25 will tristate the corresponding GPIO pin
- Bits set in R26 will mask operations to R22-25. It is all 1's on reset. Reads from R26 return the mask state.
- Reads from R22-25 are undefined, but do not block execution.

In the case of a conflict (set and clear simultaneously), the command is ignored, and the previous state is kept.

A host register configures if the external GPIO values update only at the rising edge of every quantum, or if the values update directly at `fclk` rate. Setting external update at quantum edges allows users to compose GPIO patterns with multiple accesses to the GPIO registers, without the partially finished intermediate values appearing on the output.

- When snap-to-quantum is active, it applies to all GPIO pins, and only one core's clock may be used to snap all the pins at once.
- Input and output directions may independently specify snap-to-quantum, as well as their snap-to-quantum core clock.

## Inter-core Events R27, R28, R29, R30

A core indicates which events it is sensitive to by writing a `1` into a bit in R27.

A core can set an event bit by writing a `1` to R28. This write does not regard the R27 mask.

A core can clear an event bit by writing a `1` to R29. This write does not regard the R27 mask.

A core can wait until an event happens by reading R30. It will stall until all of the bits marked as sensitive in R27 are set. The stall is computed at `fclk` rates, e.g. if one needs synchronization to the quantum timer, the code sequence should be `mov r20, r0` followed by `mov ra, r29`.

Bits 31:24 on R30 are wired to the FIFO level event flags; these bits cannot be set or cleared by R28 and R29.

The host can read the contents of the aggregated events in real-time, and an interrupt can be generated based on an enable mask AND'd with the contents of R30.

The host also similarly has bit-wise set/clear write-only registers that can manipulate the aggregated events. The host's set/clear commands have priority over all of the cores. In case of simultaneous set/clear, the conflicting bits are ignored and the previous state is kept.

# R31 core ID and cycle count

Reading R31 returns the ID number of the core (0-3), and number of fclk cycles elapsed since reset:

- r31[30:31] contains the ID of the core (0-3)
- r31[0:29] contains the elapsed fclk count

The count will wrap around on overflow.

## Missed Quantum Register

This is a host register, one per core, that counts the number of quanta that were missed by a given core (e.g., a quanta pulse has passed without the core stalling on the quanta pulse). This is primarily for debugging code loops.

# Use as a DMA Engine

Every core can fetch and store data to system memory, allowing them to operate as a DMA controller. However, it is strongly recommended that the system programmer only ever use one core at a time for this, because there is no mechanism for locking or fairness if multiple cores contend for the system address space.

Here is a simple example of a copy DMA loop using just one core. This will wait until it receives the source address, destination address, and the number of bytes to copy, before executing the DMA and then returning to the wait state.

```
wait:
  mv x3, x18   // src address
  mv x2, x17   // dst address
  mv x1, x16   // wait for # of bytes to move

  add x4, x1, x3  // x4 <- end condition based on source address increment

loop:
  ld  x5, 0(x3)    // blocks until load responds
  st  x5, 0(x2)    // blocks until store completes
  addi x3, x3, 4   // 3 cycles
  addi x2, x2, 4   // 3 cycles
  bne x3, x4, loop // 5 cycles
  j wait
```

Better performance can be achieved if the loop counters are updated by another core. Here is an example that uses three cores simultaneously (note that the labels are symbolic for readability, the actual assembler requires labels to be numeric codes):

Core 0:

```
// core 0 just waits for addresses to appear on FIFOs x16, x17
core0:
  ld x5, 0(x16)
  st x5, 0(x17)
  ld x5, 0(x16)  // optionally unroll the loop to amortize jump cost
  st x5, 0(x17)
  ld x5, 0(x16)
  st x5, 0(x17)
  ld x5, 0(x16)
  st x5, 0(x17)
  j core0
```

Core 1:
```
core1:
  mv x1, x18  // src address on FIFO x18
  mv x2, x18  // # bytes to move on FIFO x18
  add x3, x2, x2
core1_loop:
  mv x16, x1
  addi x1, x1, 4
  bne x1, x3, core1_loop
  j core1
```

Core 2:

```
core2:
  mv x1, x19  // dst address on FIFO x19
  mv x2, x19  // # bytes to move on FIFO x19
  add x3, x2, x2
core2_loop:
  mv x17, x1
  addi x1, x1, 4
  bne x1, x3, core2_loop
  j core2

```

Here, core 0 just waits for addresses to appear on FIFOs x16 and x17, performing loads and stores at the maximum possible rate.

Core 1 waits for two words to appear, the source address and bytes to move to appear on FIFO 18; core 2 waits for two words to appear, the destination address and bytes to move. Each core computes the addresses and feeds them to core 0 via the respective FIFOs for the source and destination addresses.
