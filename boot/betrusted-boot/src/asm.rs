use core::arch::asm;

const RAM_BASE: usize = 0x61000000;
const RAM_SIZE: usize = 0x00010000;
// Assembly stubs for entering into the loader, and exiting it.

// Note: inline constants are not yet stable in Rust: https://github.com/rust-lang/rust/pull/104087
#[link_section = ".text.init"]
#[export_name = "_start"]
pub extern "C" fn _start() {
    unsafe {
        asm! (
            // cause default reset to fail; we can only boot if trimming_reset worked. Requires test bench
            // to set trimming_reset_ena to 1 and trimming_reset to 0x6000_0002
            // "j           _start",

            // decorate our stack area with a canary pattern
            "li          t1, 0xDEADC0DE",
            "mv          t0, {stack_limit}",
            "mv          t2, {ram_top}",
        "100:", // fillstack
            "sw          t1, 0(t0)",
            "addi        t0, t0, 4",
            "bltu        t0, t2, 100b",

            // Place the stack pointer at the end of RAM
            "mv          sp, {ram_top}",

            // Install a machine mode trap handler
            "la          t0, _start",
            "csrw        mtvec, t0",

            // Start Rust
            "j   rust_entry",

            ram_top = in(reg) (RAM_BASE + RAM_SIZE),
            // On Precursor - 0x40FFE01C: currently allowed stack extent - 8k - (7 words). 7 words are for kernel backup args - see bootloader in betrusted-soc
            stack_limit = in(reg) (RAM_BASE + RAM_SIZE - 8192 + 7 * core::mem::size_of::<usize>()),
            options(noreturn)
        );
    }
}

#[link_section = ".text.init"]
#[export_name = "abort"]
/// This is only used in debug mode
pub extern "C" fn abort() {
    unsafe {
        asm! (
            "300:", // abort
                "j 300b",
            options(noreturn)
        );
    }
}

#[inline(never)]
#[export_name = "start_loader"]
pub extern "C" fn start_loader(
    _arg_buffer: usize,
    _signature: usize,
    _loader_addr: usize,
) -> ! {
    unsafe {
        asm! (
            "jalr x0, a2, 0",
            options(noreturn)
        );
    }
}

