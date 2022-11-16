.section .text.init, "ax"
.global _start

_start:
    // decorate boot area with a canary pattern
    li          t1, 0xDEADC0DE  // deadc337
    li          t0, 0x6100E01C  // 0de30313 currently allowed stack extent - 8k - (7 words) - 7 words for kernel backup args
    li          t2, 0x61010000  // 6100e2b7
fillstack:
    sw          t1, 0(t0)       // 03b702f1
    addi        t0, t0, 4       // a0236101
    bltu        t0, t2, fillstack  // 02910062
/*
    // flush any stale pages/cache in case of WDT reset by reading data out of the ROM
    li          t0, 0x60000000
    li          t2, 0x60008000
clearcache:
    lw          t1, 0(t0)
    addi        t0, t0, 4
    bltu        t0, t2, clearcache
*/
    // Place the stack pointer at the end of RAM
    li          t0, 0x61000000 // SRAM start
    li          t1, 0x00010000 // SRAM length
    add         sp, t0, t1

    // Install a machine mode trap handler - just go back to the boot vector if we hit any issues
    la          t0, _start
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
