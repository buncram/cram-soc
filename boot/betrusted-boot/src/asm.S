.section .text.init, "ax"
.global _start

_start:
    li          t0, 0xffffffff
    csrw        mideleg, t0
    csrw        medeleg, t0

    // Place the stack pointer at the end of RAM
    li          t0, 0x10000000 // SRAM start
    li          t1, 0x4000 // SRAM length
    add         sp, t0, t1

    // Install a machine mode trap handler
    la          t0, abort
    csrw        mtvec, t0

    // Start Rust
    j   rust_entry

// This is used only in debug mode.
.global abort
abort:
    j           abort

.global start_loader
start_loader:
    // a0 should already have args for xous-kernel.bin
    // a1 should have the loader sig
    // a2 should have the address of the loader itself
	jalr x0, a2, 0          # jump to xous-loader.bin
