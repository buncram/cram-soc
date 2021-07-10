#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#![allow(unreachable_code)] // allow debugging of failures to jump out of the bootloader

const VERSION_STR: &'static str = "Bootloader v0.1.1e\n\r";
const LOADER_DATA_OFFSET: u32 = 0x2050_1000;
const LOADER_SIG_OFFSET: u32 = 0x2050_0000;
// changing the bootloader stack is very tricky. here's some places where it needs to be updated:
// - here
// - inside asm.S for stack guard
// - loader - reserved pages (near bottom of file)
// - loader - a second place for reserved placed (around line 1407)
// - loader - clean suspend marker (around line 1318)
// - susres - clean suspend marker location (around line 144)
// - loader - backup args  (line 1250)
// - loader - backup args  (line 1280)
// should probably fix this to make it easier, except it's splattered across so many moving parts...
const STACK_LEN: u32 = 8192 - (7 * 4); // 7 words for backup kernel args
const STACK_TOP: u32 = 0x4100_0000 - STACK_LEN;

use utralib::generated::*;

mod debug;

/*
    Notes about printing:
      - the println! and write! macros are actually quite expensive in the context of a 32kiB ROM (~4k overhead??)
      - we are trying to get away with direct putc() and tiny_write_str() calls. looks weird for Rust, but it saves a few bytes
*/
#[repr(C)]
struct SignatureInFlash {
    pub version: u32,
    pub signed_len: u32,
    pub signature: [u8; 64],
}

mod panic_handler {
    use core::panic::PanicInfo;
    use crate::debug;
    #[panic_handler]
    fn handle_panic(arg: &PanicInfo) -> ! {
        //crate::println!("{}", _arg);
        let mut uart = debug::Uart {};
        if let Some(s) = arg.payload().downcast_ref::<&str>() {
            uart.tiny_write_str(s);
        } else {
            uart.tiny_write_str("unspecified panic!\n\r");
        }
        loop {}
    }
}

const FB_WIDTH_WORDS: usize = 11;
const FB_LINES: usize = 536;
const FB_SIZE: usize = FB_WIDTH_WORDS * FB_LINES; // 44 bytes by 536 lines
struct Gfx {
    csr: utralib::CSR<u32>,
    fb: &'static mut [u32],
}
impl Gfx {
    pub fn init(&mut self, clk_mhz: u32) {
        self.csr.wfo(utra::memlcd::PRESCALER_PRESCALER, (clk_mhz / 2_000_000) - 1);
    }
    pub fn update_all(&mut self) {
        self.csr.wfo(utra::memlcd::COMMAND_UPDATEALL, 1);
    }
    pub fn busy(&self) -> bool {
        if self.csr.rf(utra::memlcd::BUSY_BUSY) == 1 {
            true
        } else {
            false
        }
    }
}

struct Keyrom {
    csr: utralib::CSR<u32>,
}
impl Keyrom {
    pub fn new() -> Self {
        Keyrom {
            csr: CSR::new(utra::keyrom::HW_KEYROM_BASE as *mut u32),
        }
    }
    pub fn read_ed25519(&mut self, base: u8) -> ed25519_dalek::PublicKey {
        let mut pk_bytes: [u8; 32] = [0; 32];
        for (offset, pk_word) in pk_bytes.chunks_exact_mut(4).enumerate() {
            self.csr.wfo(utra::keyrom::ADDRESS_ADDRESS, base as u32 + offset as u32);
            let word = self.csr.rf(utra::keyrom::DATA_DATA);
            for (&src_byte, dst_byte) in word.to_be_bytes().iter().zip(pk_word.iter_mut()) {
                *dst_byte = src_byte;
            }
        }
        println!("pk_bytes: {:?}", pk_bytes);
        ed25519_dalek::PublicKey::from_bytes(&pk_bytes).unwrap()
    }
}

/*
Bootloader v0.1.1e
Y: [28, 155, 234, 227, 42, 234, 200, 117, 7, 193, 128, 148, 56, 126, 255, 28, 116, 97, 66, 130, 175, 253, 129, 82, 216, 113, 53, 46, 223, 63, 88, 187]
Z: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
YY: [154, 180, 250, 64, 35, 64, 104, 144, 32, 104, 151, 198, 217, 243, 39, 92, 185, 19, 97, 174, 19, 28, 33, 38, 133, 33, 204, 118, 61, 247, 167, 88]
u: [64, 64, 64, 64, 144, 144, 144, 144, 198, 198, 198, 198, 92, 92, 92, 92, 174, 174, 174, 174, 38, 38, 38, 38, 118, 118, 118, 118, 88, 88, 88, 88]
v: [93, 93, 93, 93, 189, 189, 189, 189, 50, 50, 50, 50, 179, 179, 179, 179, 179, 179, 179, 179, 222, 222, 222, 222, 26, 26, 26, 26, 52, 52, 52, 52]
isvalid: Choice(0)
unspecified panic!

Bootloader v0.1.1e
Y: [28, 155, 234, 227, 42, 234, 200, 117, 7, 193, 128, 148, 56, 126, 255, 28, 116, 97, 66, 130, 175, 253, 129, 82, 216, 113, 53, 46, 223, 63, 88, 59]
Z: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
YY: [101, 137, 91, 248, 108, 169, 16, 111, 156, 124, 237, 118, 114, 34, 91, 239, 182, 179, 14, 79, 136, 33, 206, 80, 52, 227, 152, 34, 98, 136, 137, 109]
u: [100, 137, 91, 248, 108, 169, 16, 111, 156, 124, 237, 118, 114, 34, 91, 239, 182, 179, 14, 79, 136, 33, 206, 80, 52, 227, 152, 34, 98, 136, 137, 109]
v: [35, 9, 216, 132, 178, 48, 0, 68, 27, 126, 176, 192, 134, 154, 105, 74, 5, 175, 50, 103, 175, 248, 162, 91, 181, 103, 74, 52, 232, 90, 1, 62]
isvalid: Choice(1)
valid
negate
*/

#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(_unused1: *const usize, _unused2: u32) -> ! {
    // conjure the signature struct directly out of memory. super unsafe.
    let sig_ptr = LOADER_SIG_OFFSET as *const SignatureInFlash;
    let sig: &SignatureInFlash = sig_ptr.as_ref().unwrap();

    // initial banner
    let mut uart = debug::Uart {};
    uart.tiny_write_str("  ");

    // clear screen to all black
    let mut gfx = Gfx {
        csr: CSR::new(utra::memlcd::HW_MEMLCD_BASE as *mut u32),
        fb: core::slice::from_raw_parts_mut(utralib::HW_MEMLCD_MEM as *mut u32, FB_SIZE), // unsafe but inside an unsafe already
    };
    gfx.init(100_000_000);

    for word in gfx.fb.iter_mut() {
        *word = 0x0; // set to all black
    }
    gfx.update_all();
    while gfx.busy() { }

    // now characters should actually be able to print
    uart.tiny_write_str(VERSION_STR);

    // init the curve25519 engine
    let mut engine = utralib::CSR::new(utra::engine::HW_ENGINE_BASE as *mut u32);
    engine.wfo(utra::engine::POWER_ON, 1);
    engine.wfo(utra::engine::WINDOW_WINDOW, 0);
    engine.wfo(utra::engine::MPSTART_MPSTART, 0);

    // get the public key
    let mut keyrom = Keyrom::new();
    let devkey = keyrom.read_ed25519(0x18);
    println!("key: {:?}", devkey);

    uart.tiny_write_str("Dev key: ");
    for &b in devkey.as_bytes().iter() {
        uart.put_hex(b);
    }
    uart.newline();

    if sig.version != 1 {
        uart.tiny_write_str("Warning: signature version mismatch!");
        uart.newline();
    }
    let signed_len = sig.signed_len;
    let image: &[u8] = core::slice::from_raw_parts(LOADER_DATA_OFFSET as *const u8, signed_len as usize);
    let ed25519_signature = ed25519_dalek::Signature::from(sig.signature);

    use ed25519_dalek::Verifier;
    if devkey.verify(image, &ed25519_signature).is_ok() {
        uart.tiny_write_str("Signature check passed");
    } else {
        uart.tiny_write_str("Signature check failed");
    }
    uart.newline();

    // check the stack usage
    let stack: &[u32] = core::slice::from_raw_parts(STACK_TOP as *const u32, (STACK_LEN as usize / core::mem::size_of::<u32>()) as usize);
    let mut unused_stack_words = 0;
    for &word in stack.iter() {
        if word != 0xDEAD_C0DE {
            break;
        }
        unused_stack_words += 1;
    }
    uart.tiny_write_str("Free stack: 0x");
    uart.print_hex_word(unused_stack_words * 4);
    uart.newline();

    let mut last_char: u8 = 0;
    loop {
        if let Some(c) = uart.getc() {
            uart.putc(c);
            if c == 0xd { // add an LF to a CR
                uart.putc(0xa);
            }
            if c == 0xd && last_char == 0x21 { // '!'
                break;
            }
            last_char = c;
        }
    }
    uart.tiny_write_str("Jumping to loader...");
    uart.newline();

    let mut sha_csr = CSR::new(utra::sha512::HW_SHA512_BASE as *mut u32);
    sha_csr.wfo(utra::sha512::POWER_ON, 0); // cut power to the SHA block; this is the expected default state after the bootloader is done.

    // now jump to the loader once everything checks out.
    start_loader(
        0x2098_0000,  // start of kernel arguments
        0x0,           // this is unused
        0x2050_1000,  // jump address of the loader itself
    );
    uart.tiny_write_str("Should have jumped to loader!");
    uart.newline();
    loop {
    }
}

extern "C" {
    fn start_loader(
        arg_buffer: usize,
        signature: usize,
        loader_addr: usize,
    ) -> !;
}
