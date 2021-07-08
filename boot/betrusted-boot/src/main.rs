#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

const VERSION_STR: &'static str = "Bootloader v0.1.0\n\r";
const LOADER_DATA_OFFSET: u32 = 0x2050_1000;
const LOADER_SIG_OFFSET: u32 = 0x2050_0000;

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
        ed25519_dalek::PublicKey::from_bytes(&pk_bytes).unwrap()
    }
}

#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(_unused1: *const usize, _unused2: u32) -> ! {
    // conjure the signature struct directly out of memory. super unsafe.
    let sig_ptr = LOADER_SIG_OFFSET as *const SignatureInFlash;
    let sig: &SignatureInFlash = sig_ptr.as_ref().unwrap();

    // initial banner
    let mut uart = debug::Uart {};
    uart.tiny_write_str("fix LiteX UART startup issue.");

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

    let mut keyrom = Keyrom::new();
    let devkey = keyrom.read_ed25519(0x18);

    uart.tiny_write_str("Dev key: ");
    for &b in devkey.as_bytes().iter() {
        uart.put_hex(b);
    }
    uart.newline();

    if sig.version != 1 {
        uart.tiny_write_str("Warning: signature version mismatch!");
    }
    let signed_len = sig.signed_len;
    let image: &[u8] = core::slice::from_raw_parts(LOADER_DATA_OFFSET as *const u8, signed_len as usize);
    let ed25519_signature = ed25519_dalek::Signature::from(sig.signature);
    /*
    use ed25519_dalek::Verifier;
    if devkey.verify(image, &ed25519_signature).is_ok() {
        uart.tiny_write_str("Signature check passed");
    } else {
        uart.tiny_write_str("Signature check failed");
    }*/

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

    // now jump to the loader if everything checks out.
    start_loader(
        0x2098_0000,  // start of kernel arguments
        0x0,          // this is unsused
        0x2051_0000,  // jump address of the loader itself
    );
}

extern "C" {
    fn start_loader(
        arg_buffer: usize,
        signature: usize,
        loader_addr: usize,
    ) -> !;
}
