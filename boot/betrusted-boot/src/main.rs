#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#![allow(unreachable_code)] // allow debugging of failures to jump out of the bootloader

const VERSION_STR: &'static str = "Betrusted/Precursor Bootloader v0.2.3\n\r";
// v0.2.0 -- initial version
// v0.2.1 -- fix warmboot issue (SHA reset)
// v0.2.2 -- check version & length in header against signed area
// v0.2.3 -- lock out key ROM on signature check failure

#[cfg(feature="hw-sec")]
const LOADER_DATA_OFFSET: u32 = 0x2050_1000;
#[cfg(feature="hw-sec")]
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
#[cfg(feature="sim")]
use core::convert::TryInto;
#[cfg(feature="sim")]
use core::convert::TryFrom;
#[cfg(feature="sim")]
use core::mem::size_of;

#[cfg(any(feature="ahb-test"))]
mod duart;

mod debug;
#[cfg(feature="sim")]
mod satp;
#[cfg(feature="sim")]
mod irqs;

mod asm;
use asm::*;

/*
    Notes about printing:
      - the println! and write! macros are actually quite expensive in the context of a 32kiB ROM (~4k overhead??)
      - we are trying to get away with direct putc() and tiny_write_str() calls. looks weird for Rust, but it saves a few bytes
*/
#[repr(C)]
#[cfg(feature="hw-sec")]
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

static mut REPORT: CSR::<u32> = CSR::<u32>{base: utra::main::HW_MAIN_BASE as *mut u32};

#[cfg(not(feature="daric"))]
pub fn report_api(d: u32) {
    unsafe {
        REPORT.wo(utra::main::REPORT, d);
    }
}
#[cfg(feature="daric")]
pub fn report_api(d: u32) {
    let mut uart = debug::Uart {};
    uart.print_hex_word(d);
}

#[cfg(feature="gfx")]
struct Point {
    x: i16,
    y: i16,
}
#[derive(PartialEq, Eq)]
#[cfg(feature="gfx")]
enum Color {
    Light,
    Dark
}
#[cfg(feature="gfx")]
const FB_WIDTH_WORDS: usize = 11;
#[cfg(feature="gfx")]
#[cfg(feature="gfx")]
const FB_WIDTH_PIXELS: usize = 336;
#[cfg(feature="gfx")]
const FB_LINES: usize = 536;
#[cfg(feature="gfx")]
const FB_SIZE: usize = FB_WIDTH_WORDS * FB_LINES; // 44 bytes by 536 lines
// this font is from the embedded graphics crate https://docs.rs/embedded-graphics/0.7.1/embedded_graphics/
#[cfg(feature="gfx")]
const FONT_IMAGE: &'static [u8] = include_bytes!("font6x12_1bpp.raw");
#[cfg(feature="gfx")]
const CHAR_HEIGHT: u32 = 12;
#[cfg(feature="gfx")]
const CHAR_WIDTH: u32 = 6;
#[cfg(feature="gfx")]
const FONT_IMAGE_WIDTH: u32 = 96;
#[cfg(feature="gfx")]
const LEFT_MARGIN: i16 = 10;

#[cfg(feature="gfx")]
struct Gfx {
    csr: utralib::CSR<u32>,
    fb: &'static mut [u32],
}
#[cfg(feature="gfx")]
impl<'a> Gfx {
    pub fn init(&mut self, clk_mhz: u32) {
        self.csr.wfo(utra::memlcd::PRESCALER_PRESCALER, (clk_mhz / 2_000_000) - 1);
    }
    pub fn update_all(&mut self) {
        self.csr.wfo(utra::memlcd::COMMAND_UPDATEALL, 1);
    }
    pub fn update_dirty(&mut self) {
        self.csr.wfo(utra::memlcd::COMMAND_UPDATEDIRTY, 1);
    }
    pub fn busy(&self) -> bool {
        if self.csr.rf(utra::memlcd::BUSY_BUSY) == 1 {
            true
        } else {
            false
        }
    }
    #[allow(dead_code)]
    pub fn set_devboot(&mut self) {
        self.csr.wfo(utra::memlcd::DEVBOOT_DEVBOOT, 1);
    }

    fn char_offset(&self, c: char) -> u32 {
        let fallback = ' ' as u32 - ' ' as u32;
        if c < ' ' {
            return fallback;
        }
        if c <= '~' {
            return c as u32 - ' ' as u32;
        }
        fallback
    }
    fn put_digit(&mut self, d: u8, pos: &mut Point) {
        let mut buf: [u8; 4] = [0; 4]; // stack buffer for the character encoding
        let nyb = d & 0xF;
        if nyb < 10 {
            self.msg(((nyb + 0x30) as char).encode_utf8(&mut buf), pos);
        } else {
            self.msg(((nyb + 0x61 - 10) as char).encode_utf8(&mut buf), pos);
        }
    }
    fn put_hex(&mut self, c: u8, pos: &mut Point) {
        self.put_digit(c >> 4, pos);
        self.put_digit(c & 0xF, pos);
    }
    pub fn hex_word(&mut self, word: u32, pos: &mut Point) {
        for &byte in word.to_be_bytes().iter() {
            self.put_hex(byte, pos);
        }
    }
    pub fn msg(&mut self, text: &'a str, pos: &mut Point) {
        // this routine is adapted from the embedded graphics crate https://docs.rs/embedded-graphics/0.7.1/embedded_graphics/
        let char_per_row = FONT_IMAGE_WIDTH / CHAR_WIDTH;
        let mut idx = 0;
        let mut x_update: i16 = 0;
        for current_char in text.chars() {
            let mut char_walk_x = 0;
            let mut char_walk_y = 0;

            loop {
                // Char _code_ offset from first char, most often a space
                // E.g. first char = ' ' (32), target char = '!' (33), offset = 33 - 32 = 1
                let char_offset = self.char_offset(current_char);
                let row = char_offset / char_per_row;

                // Top left corner of character, in pixels
                let char_x = (char_offset - (row * char_per_row)) * CHAR_WIDTH;
                let char_y = row * CHAR_HEIGHT;

                // Bit index
                // = X pixel offset for char
                // + Character row offset (row 0 = 0, row 1 = (192 * 8) = 1536)
                // + X offset for the pixel block that comprises this char
                // + Y offset for pixel block
                let bitmap_bit_index = char_x
                    + (FONT_IMAGE_WIDTH * char_y)
                    + char_walk_x
                    + (char_walk_y * FONT_IMAGE_WIDTH);

                let bitmap_byte = bitmap_bit_index / 8;
                let bitmap_bit = 7 - (bitmap_bit_index % 8);

                let color = if FONT_IMAGE[bitmap_byte as usize] & (1 << bitmap_bit) != 0 {
                    Color::Light
                } else {
                    Color::Dark
                };

                let x = pos.x
                    + (CHAR_WIDTH * idx as u32) as i16
                    + char_walk_x as i16;
                let y = pos.y + char_walk_y as i16;

                // draw color at x, y
                if (current_char as u8 != 0xd) && (current_char as u8 != 0xa) { // don't draw CRLF specials
                    self.draw_pixel(Point{x, y}, color);
                }

                char_walk_x += 1;

                if char_walk_x >= CHAR_WIDTH {
                    char_walk_x = 0;
                    char_walk_y += 1;

                    // Done with this char, move on to the next one
                    if char_walk_y >= CHAR_HEIGHT {
                        if current_char as u8 == 0xd { // '\n'
                            pos.y += CHAR_HEIGHT as i16;
                        } else if current_char as u8 == 0xa { // '\r'
                            pos.x = LEFT_MARGIN as i16;
                            x_update = 0;
                        } else {
                            idx += 1;
                            x_update += CHAR_WIDTH as i16;
                        }

                        break;
                    }
                }
            }
        }
        pos.x += x_update;
        self.update_dirty();
        while self.busy() {}
    }
    pub fn draw_pixel(&mut self, pix: Point, color: Color) {
        let mut clip_y: usize = pix.y as usize;
        if clip_y >= FB_LINES {
            clip_y = FB_LINES - 1;
        }
        let clip_x: usize = pix.x as usize;
        if clip_x >= FB_WIDTH_PIXELS {
            clip_y = FB_WIDTH_PIXELS - 1;
        }
        if color == Color::Light {
            self.fb[(clip_x + clip_y * FB_WIDTH_WORDS * 32) / 32] |= 1 << (clip_x % 32)
        } else {
            self.fb[(clip_x + clip_y * FB_WIDTH_WORDS * 32) / 32] &= !(1 << (clip_x % 32))
        }
        // set the dirty bit on the line that contains the pixel
        self.fb[clip_y * FB_WIDTH_WORDS + (FB_WIDTH_WORDS - 1)] |= 0x1_0000;
    }
}

#[cfg(feature="hw-sec")]
struct Keyrom {
    csr: utralib::CSR<u32>,
}
#[derive(Copy, Clone)]
#[cfg(feature="hw-sec")]
enum KeyLoc {
    SelfSignPub = 0x10,
    DevPub = 0x18,
    ThirdPartyPub = 0x20,
}
#[cfg(feature="hw-sec")]
impl Keyrom {
    pub fn new() -> Self {
        Keyrom {
            csr: CSR::new(utra::keyrom::HW_KEYROM_BASE as *mut u32),
        }
    }
    pub fn key_is_zero(&mut self, key_base: KeyLoc) -> bool {
        for offset in key_base as u32..key_base as u32 + 8 {
            self.csr.wfo(utra::keyrom::ADDRESS_ADDRESS, offset as u32);
            if self.csr.rf(utra::keyrom::DATA_DATA) != 0 {
                return false;
            }
        }
        true
    }
    pub fn key_is_dev(&mut self, key_base: KeyLoc) -> bool {
        for offset in 0..8 {
            self.csr.wfo(utra::keyrom::ADDRESS_ADDRESS, offset as u32 + key_base as u32);
            let kval = self.csr.rf(utra::keyrom::DATA_DATA);
            self.csr.wfo(utra::keyrom::ADDRESS_ADDRESS, offset as u32 + KeyLoc::DevPub as u32);
            let dkval = self.csr.rf(utra::keyrom::DATA_DATA);
            if kval != dkval {
                return false;
            }
        }
        true
    }
    pub fn read_ed25519(&mut self, key_base: KeyLoc) -> ed25519_dalek::PublicKey {
        let mut pk_bytes: [u8; 32] = [0; 32];
        for (offset, pk_word) in pk_bytes.chunks_exact_mut(4).enumerate() {
            self.csr.wfo(utra::keyrom::ADDRESS_ADDRESS, key_base as u32 + offset as u32);
            let word = self.csr.rf(utra::keyrom::DATA_DATA);
            for (&src_byte, dst_byte) in word.to_be_bytes().iter().zip(pk_word.iter_mut()) {
                *dst_byte = src_byte;
            }
        }
        ed25519_dalek::PublicKey::from_bytes(&pk_bytes).unwrap()
    }
    /// locks all the keys from future read-out
    pub fn lock(&mut self) {
        for i in 0..256 {
            self.csr.wfo(utra::keyrom::LOCKADDR_LOCKADDR, i);
        }
    }
}

/// chunks through the entire bank of data
#[cfg(feature="sim")]
unsafe fn ramtest_all<T>(test_slice: &mut [T], test_index: u32)
where
    T: TryFrom<usize> + TryInto<u32> + Default + Copy,
{
    let mut sum: u32 = 0;
    for (index, d) in test_slice.iter_mut().enumerate() {
        // Convert the element into a `u32`, failing
        (d as *mut T).write_volatile(
            index
                .try_into()
                .unwrap_or_default()
        );
        sum += TryInto::<u32>::try_into(index).unwrap();
    }
    let mut checksum: u32 = 0;
    for d in test_slice.iter() {
        let a = (d as *const T)
            .read_volatile()
            .try_into()
            .unwrap_or_default();
        checksum += a;
        // report_api(a);
    }

    if sum == checksum {
        report_api(checksum as u32);
        report_api(0x600d_0000 + test_index);
    } else {
        report_api(checksum as u32);
        report_api(sum as u32);
        report_api(0x0bad_0000 + test_index);
    }
}


/// only touches two words on each cache line
/// this one tries to write the same word twice to two consecutive addresses
/// this causes the valid strobe to hit twice in a row. seems to pass.
#[cfg(feature="sim")]
unsafe fn ramtest_fast_specialcase1<T>(test_slice: &mut [T], test_index: u32)
where
    T: TryFrom<usize> + TryInto<u32> + Default + Copy,
{
    const CACHE_LINE_SIZE: usize = 32;
    let mut sum: u32 = 0;
    for (index, d) in test_slice.chunks_mut(CACHE_LINE_SIZE / size_of::<T>()).enumerate() {
        let idxp1 = index + 0;
        // unroll the loop to force b2b writes
        sum += TryInto::<u32>::try_into(index).unwrap();
        sum += TryInto::<u32>::try_into(idxp1).unwrap();
        // Convert the element into a `u32`, failing
        (d.as_mut_ptr() as *mut T).write_volatile(
            index
                .try_into()
                .unwrap_or_default()
        );
        // Convert the element into a `u32`, failing
        (d.as_mut_ptr().add(1) as *mut T).write_volatile(
            idxp1
                .try_into()
                .unwrap_or_default()
        );
    }
    let mut checksum: u32 = 0;
    for d in test_slice.chunks(CACHE_LINE_SIZE / size_of::<T>()) {
        checksum += (d.as_ptr() as *const T)
            .read_volatile()
            .try_into()
            .unwrap_or_default();
        checksum += (d.as_ptr().add(1) as *const T)
            .read_volatile()
            .try_into()
            .unwrap_or_default();
    }

    if sum == checksum {
        report_api(checksum as u32);
        report_api(0x600d_0000 + test_index);
    } else {
        report_api(checksum as u32);
        report_api(sum as u32);
        report_api(0x0bad_0000 + test_index);
    }
}

/// only touches two words on each cache line
#[cfg(feature="sim")]
unsafe fn ramtest_fast<T>(test_slice: &mut [T], test_index: u32)
where
    T: TryFrom<usize> + TryInto<u32> + Default + Copy,
{
    const CACHE_LINE_SIZE: usize = 32;
    let mut sum: u32 = 0;
    for (index, d) in test_slice.chunks_mut(CACHE_LINE_SIZE / size_of::<T>()).enumerate() {
        let idxp1 = index + 1;
        // unroll the loop to force b2b writes
        sum += TryInto::<u32>::try_into(index).unwrap();
        sum += TryInto::<u32>::try_into(idxp1).unwrap();
        // Convert the element into a `u32`, failing
        (d.as_mut_ptr() as *mut T).write_volatile(
            index
                .try_into()
                .unwrap_or_default()
        );
        // Convert the element into a `u32`, failing
        (d.as_mut_ptr().add(1) as *mut T).write_volatile(
            idxp1
                .try_into()
                .unwrap_or_default()
        );
    }
    let mut checksum: u32 = 0;
    for d in test_slice.chunks(CACHE_LINE_SIZE / size_of::<T>()) {
        let a = (d.as_ptr() as *const T)
            .read_volatile()
            .try_into()
            .unwrap_or_default();
        let b = (d.as_ptr().add(1) as *const T)
            .read_volatile()
            .try_into()
            .unwrap_or_default();
        checksum = checksum + a + b;
        // report_api(a);
        // report_api(b);
    }

    if sum == checksum {
        report_api(checksum as u32);
        report_api(0x600d_0000 + test_index);
    } else {
        report_api(checksum as u32);
        report_api(sum as u32);
        report_api(0x0bad_0000 + test_index);
    }
}

/* some LFSR terms
    3 3,2
    4 4,3
    5 5,3
    6 6,5
    7 7,6
    8 8,6,5,4
    9 9,5  <--
    10 10,7
    11 11,9
    12 12,6,4,1
    13 13,4,3,1
    14 14,5,3,1
    15 15,14
    16 16,15,13,4
    17 17,14
    18 18,11
    19 19,6,2,1
    20 20,17

    32 32,22,2,1:
    let bit = ((state >> 31) ^
               (state >> 21) ^
               (state >>  1) ^
               (state >>  0)) & 1;

*/
#[cfg(feature="sim")]
/// our desired test length is 512 entries, so pick an LFSR with a period of 2^9-1...
pub fn lfsr_next(state: u16) -> u16 {
    let bit = ((state >> 8) ^
               (state >>  4)) & 1;

    ((state << 1) + bit) & 0x1_FF
}

#[cfg(feature="sim")]
#[allow(dead_code)]
/// shortened test length is 16 entries, so pick an LFSR with a period of 2^4-1...
pub fn lfsr_next_16(state: u16) -> u16 {
    let bit = ((state >> 3) ^
               (state >>  2)) & 1;

    ((state << 1) + bit) & 0xF
}

/// uses an LFSR to cycle through "random" locations. The slice length
/// should equal the (LFSR period+1), so that we guarantee that each entry
/// is visited once.
#[cfg(feature="sim")]
unsafe fn ramtest_lfsr<T>(test_slice: &mut [T], test_index: u32)
where
    T: TryFrom<usize> + TryInto<u32> + Default + Copy,
{

    if test_slice.len() != 512 {
        report_api(0x0bad_000 + test_index + 0x0F00); // indicate a failure due to configuration
        return;
    }
    let mut state: u16 = 1;
    let mut sum: u32 = 0;
    const MAX_STATES: usize = 511;
    (&mut test_slice[0] as *mut T).write_volatile(
        0.try_into().unwrap_or_default()
    ); // the 0 index is never written to by this, initialize it to 0
    for i in 0..MAX_STATES {
        let wr_val = i * 3;
        (&mut test_slice[state as usize] as *mut T).write_volatile(wr_val.try_into().unwrap_or_default());
        sum += wr_val as u32;
        state = lfsr_next(state);
    }

    // flush cache
    report_api(0xff00_ff00);
    core::arch::asm!(
        ".word 0x500F",
    );
    report_api(0x0f0f_0f0f);

    // we should be able to just iterate in-order and sum all the values, and get the same thing back as above
    let mut checksum: u32 = 0;
    for d in test_slice.iter() {
        let a = (d as *const T)
            .read_volatile()
            .try_into()
            .unwrap_or_default();
        checksum += a;
        // report_api(a);
    }

    if sum == checksum {
        report_api(checksum as u32);
        report_api(0x600d_0000 + test_index);
    } else {
        report_api(checksum as u32);
        report_api(sum as u32);
        report_api(0x0bad_0000 + test_index);
    }
}

#[cfg(feature="ahb-test")]
fn ahb_tests() {
    let mut duart = duart::Duart::new();
    loop {
        duart.puts("DUART up!\n");
    }
}

#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(_unused1: *const usize, _unused2: u32) -> ! {
    #[cfg(feature="sim")]
    {
        let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
        report_api(0x600dc0de);

        // report the measured reset value
        let resetvalue = CSR::new(utra::resetvalue::HW_RESETVALUE_BASE as *mut u32);
        report_api(resetvalue.r(utra::resetvalue::PC));

        // ---------- vm setup -------------------------
        satp::satp_setup(); // at the conclusion of this, we are running in "supervisor" (kernel) mode, with Sv32 semantics
        report_api(0x5a1d_6060);

        #[cfg(feature="daric")]
        {
            let mut uart = debug::Uart {};
            uart.tiny_write_str("hello world!\n\r");
        }

        // TODO: make an XIP test
        //  - copy code to that location
        //  - jump to it
        //  - return

        // ---------- ahb test option -------------
        #[cfg(feature="ahb-test")]
        ahb_tests();
        #[cfg(feature="pio-test")]
        xous_pio::pio_tests::setup_reporting((utra::main::REPORT.offset() + utra::main::HW_MAIN_BASE) as *mut u32);
        #[cfg(feature="pio-test")]
        xous_pio::pio_tests::pio_tests();

        // ---------- exception setup ------------------
        irqs::irq_setup();
        // ---------- coreuser test --------------------
        satp::satp_test();

        // ---------- exception test -------------------
        irqs::irq_test();

        // ---------- CPU CSR tests --------------
        report_api(0xc520_0000);
        let mut csrtest = CSR::new(utra::csrtest::HW_CSRTEST_BASE as *mut u32);
        let mut passing = true;
        for i in 0..4 {
            csrtest.wfo(utra::csrtest::WTEST_WTEST, i);
            let val = csrtest.rf(utra::csrtest::RTEST_RTEST);
            report_api(
                val
            );
            if val != i + 0x1000_0000 {
                passing = false;
            }
        }
        if passing {
            report_api(0xc520_600d);
        } else {
            report_api(0xc520_dead);
        }

        // ---------- wfi test -------------------------
        irqs::wfi_test();

        // ----------- caching tests -------------
        // test of the 0x500F cache flush instruction - this requires manual inspection of the report values
        report_api(0x000c_ac7e);
        const CACHE_WAYS: usize = 4;
        const CACHE_SET_SIZE: usize = 4096 / size_of::<u32>();
        let test_slice = core::slice::from_raw_parts_mut(satp::PT_LIMIT as *mut u32, CACHE_SET_SIZE * CACHE_WAYS);
        // bottom of cache
        for set in 0..4 {
            report_api((&mut test_slice[set * CACHE_SET_SIZE] as *mut u32) as u32);
            (&mut test_slice[set * CACHE_SET_SIZE] as *mut u32).write_volatile(0x0011_1111 * (1 + set as u32));
        }
        // top of cache
        for set in 0..4 {
            report_api((&mut test_slice[set * CACHE_SET_SIZE + CACHE_SET_SIZE - 1] as *mut u32) as u32);
            (&mut test_slice[set * CACHE_SET_SIZE + CACHE_SET_SIZE - 1] as *mut u32).write_volatile(0x1100_2222 * (1 + set as u32));
        }
        // read cached values - first iteration populates the cache; second iteration should be cached
        for iter in 0..2 {
            report_api(0xb1d0_0000 + iter + 1);
            for set in 0..4 {
                let a = (&mut test_slice[set * CACHE_SET_SIZE] as *mut u32).read_volatile();
                report_api(a);
                let b = (&mut test_slice[set * CACHE_SET_SIZE + CACHE_SET_SIZE - 1] as *mut u32).read_volatile();
                report_api(b);
            }
        }
        // flush cache
        report_api(0xff00_ff00);
        core::arch::asm!(
            ".word 0x500F",
        );
        report_api(0x0f0f_0f0f);
        // read cached values - first iteration populates the cache; second iteration should be cached
        for iter in 0..2 {
            report_api(0xb2d0_0000 + iter + 1);
            for set in 0..4 {
                let a = (&mut test_slice[set * CACHE_SET_SIZE] as *mut u32).read_volatile();
                report_api(a);
                let b = (&mut test_slice[set * CACHE_SET_SIZE + CACHE_SET_SIZE - 1] as *mut u32).read_volatile();
                report_api(b);
            }
        }
        report_api(0x600c_ac7e);

        // check that caching is disabled for I/O regions
        let mut checkstate = 0x1234_0000;
        report.wfo(utra::main::WDATA_WDATA, 0x1234_0000);
        let mut checkdata = 0;
        for _ in 0..100 {
            checkdata = report.rf(utra::main::RDATA_RDATA); // RDATA = WDATA + 5, computed in hardware
            report.wfo(utra::main::WDATA_WDATA, checkdata);
            // report_api(checkdata);
            checkstate += 5;
        }
        if checkdata == checkstate {
            report_api(checkstate);
            report_api(0x600d_0001);
        } else {
            report_api(checkstate);
            report_api(checkdata);
            report_api(0x0bad_0001);
        }

        // check that repeated reads of a register fetch new contents
        let mut checkdata = 0; // tracked value via simulation
        let mut computed = 0; // computed value by reading the hardware block
        let mut devstate = 0; // what the state should be
        for _ in 0..20 {
            let readout = report.rf(utra::main::RINC_RINC);
            computed += readout;
            // report_api(readout);
            checkdata += devstate;
            devstate += 3;
        }
        if checkdata == computed {
            report_api(checkdata);
            report_api(0x600d_0002);
        } else {
            report_api(checkdata);
            report_api(computed);
            report_api(0x0bad_0002);
        }

        // ----------- bus tests -------------
        const BASE_ADDR: u32 = satp::PT_LIMIT as u32; // don't overwrite our PT data
        // 'random' access test
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u32, 512);
        ramtest_lfsr(&mut test_slice, 3);

        // now some basic memory read/write tests
        // entirely within cache access test
        // 256-entry by 32-bit slice at start of RAM
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u32, 256);
        ramtest_all(&mut test_slice, 4);
        // byte access test
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u8, 256);
        ramtest_fast(&mut test_slice, 5);
        // word access test
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u16, 512);
        ramtest_fast(&mut test_slice, 6); // 1ff00

        // outside cache test
        // 6144-entry by 32-bit slice at start of RAM - should cross outside cache boundary
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u32, 0x1800);
        ramtest_fast(&mut test_slice, 7);  // c7f600

        // this passed, now that the AXI state machine is fixed.
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u32, 0x1800);
        ramtest_fast_specialcase1(&mut test_slice, 8);  // c7f600

        // u64 access test
        let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u64, 0xC00);
        ramtest_fast(&mut test_slice, 9);

        // random size/access test
        // let mut test_slice = core::slice::from_raw_parts_mut(BASE_ADDR as *mut u8, 0x6000);

        report.wfo(utra::main::DONE_DONE, 1);
    }

    #[cfg(feature="hw-sec")]
    {
        /////// hardware resets
        let mut engine = utralib::CSR::new(utra::engine::HW_ENGINE_BASE as *mut u32);
        engine.wfo(utra::engine::POWER_ON, 0); // power off so as to force a re-sync of the clock domains, in case we entered with power on

        // reset the SHA block, in case we're coming out of a warm reset
        let mut sha = CSR::new(utra::sha512::HW_SHA512_BASE as *mut u32);
        sha.wfo(utra::sha512::POWER_ON, 1);
        sha.wfo(utra::sha512::CONFIG_RESET, 1); // this reset takes ~32 CPU cycles but we do plenty of other stuff
        ///////// end hardware resets

        // conjure the signature struct directly out of memory. super unsafe.
        let sig_ptr = LOADER_SIG_OFFSET as *const SignatureInFlash;
        let sig: &SignatureInFlash = sig_ptr.as_ref().unwrap();
    }
    #[cfg(feature="gfx")]
    let mut cursor = Point {x: LEFT_MARGIN, y: 10};

    // initial banner
    let mut uart = debug::Uart {};
    uart.tiny_write_str("  ");

    // clear screen to all black
    #[cfg(feature="gfx")]
    let mut gfx = Gfx {
        csr: CSR::new(utra::memlcd::HW_MEMLCD_BASE as *mut u32),
        fb: core::slice::from_raw_parts_mut(utralib::HW_MEMLCD_MEM as *mut u32, FB_SIZE), // unsafe but inside an unsafe already
    };
    #[cfg(feature="gfx")]
    gfx.init(100_000_000);

    #[cfg(feature="gfx")]
    for word in gfx.fb.iter_mut() {
        *word = 0x0; // set to all black
    }
    #[cfg(feature="gfx")]
    gfx.update_all();
    #[cfg(feature="gfx")]
    while gfx.busy() { }

    #[cfg(feature="hw-sec")]
    // power on the curve engine -- give it >16 cycles to sync up
    engine.wfo(utra::engine::POWER_ON, 1);

    // now characters should actually be able to print
    uart.tiny_write_str(VERSION_STR);
    #[cfg(feature="gfx")]
    gfx.msg(VERSION_STR, &mut cursor);

    #[cfg(feature="hw-sec")]
    {
        // init the curve25519 engine
        engine.wfo(utra::engine::WINDOW_WINDOW, 0);
        engine.wfo(utra::engine::MPSTART_MPSTART, 0);

        // select the public key
        let mut keyrom = Keyrom::new();
        let mut keyloc = KeyLoc::SelfSignPub; // start from the self-sign key first, then work your way to less secure options
        loop {
            match keyloc {
                KeyLoc::SelfSignPub => {
                    if !keyrom.key_is_zero(KeyLoc::SelfSignPub) { // self-signing key takes priority
                        if keyrom.key_is_dev(KeyLoc::SelfSignPub) {
                            // mainly to protect against devs who were debugging and just stuck a dev key in the secure slot, and forgot to remove it.
                            gfx.msg("DEVELOPER KEY DETECTED\n\r", &mut cursor);
                            gfx.set_devboot();
                        }
                    } else {
                        keyloc = KeyLoc::ThirdPartyPub;
                        continue;
                    }
                },
                KeyLoc::ThirdPartyPub => {
                    // policy note: set the devboot flag also if we're doing a thirdparty pubkey boot
                    // reasoning: the purpose of the hash mark is to indicate if someone could have tampered
                    // with the device. Once an update is installed, it should always be self-signed, as it
                    // protects against the third party pubkey from being compromised and an alternate firmware
                    // being installed with no visible warning. Hence, even tho thirdparty pubkey boots could
                    // be more trusted, let's still flag it.
                    gfx.set_devboot();
                    if !keyrom.key_is_zero(KeyLoc::ThirdPartyPub) { // third party key is second in line
                        if keyrom.key_is_dev(KeyLoc::ThirdPartyPub) {
                            gfx.msg("DEVELOPER KEY DETECTED\n\r", &mut cursor);
                        }
                    } else {
                        keyloc = KeyLoc::DevPub;
                        continue;
                    }
                },
                KeyLoc::DevPub => {
                    if keyrom.key_is_zero(KeyLoc::DevPub) {
                        gfx.msg("Can't boot: No valid keys!", &mut cursor);
                        loop {}
                    }
                    gfx.msg("DEVELOPER KEY DETECTED\n\r", &mut cursor);
                    gfx.set_devboot();
                }
            }
            let pubkey = keyrom.read_ed25519(keyloc);

            uart.tiny_write_str("Using public key: ");
            for &b in pubkey.as_bytes().iter() {
                uart.put_hex(b);
            }
            uart.newline();

            let signed_len = sig.signed_len;
            let image: &[u8] = core::slice::from_raw_parts(LOADER_DATA_OFFSET as *const u8, signed_len as usize);
            let ed25519_signature = ed25519_dalek::Signature::from(sig.signature);

            // extract the version and length from the signed region
            use core::convert::TryInto;
            let protected_version = u32::from_le_bytes(image[signed_len as usize - 8 .. signed_len as usize - 4].try_into().unwrap());
            let protected_len = u32::from_le_bytes(image[signed_len as usize - 4 ..].try_into().unwrap());
            // check that the signed versions match the version reported in the header
            if sig.version != 1 || (sig.version != protected_version) {
                gfx.msg("Sig version mismatch\n\r", &mut cursor);
                uart.tiny_write_str("Sig version mismatch\n\r");
                die();
            }
            if protected_len != signed_len - 4 {
                gfx.msg("Sig length mismatch\n\r", &mut cursor);
                uart.tiny_write_str("Sig length mismatch\n\r");
                die();
            }

            /* // some debug remnants that could be handy in the future
            println!("pubkey: {:?}", pubkey);
            println!("signature: {:?}", ed25519_signature);
            println!("image bytes:");
            for b in image[0..32].iter() {
                print!("{:02x} ", b);
            }
            println!("");
            println!("sha fifo status: 0x{:08x}", sha.r(utra::sha512::FIFO));
            println!("sha config     : 0x{:08x}", sha.r(utra::sha512::CONFIG));
            println!("sha command    : 0x{:08x}", sha.r(utra::sha512::COMMAND));
            println!("sha msglen     : 0x{:08x}", sha.r(utra::sha512::MSG_LENGTH0));
            println!("sha evstatus   : 0x{:08x}", sha.r(utra::sha512::EV_STATUS));
            println!("sha evenable   : 0x{:08x}", sha.r(utra::sha512::EV_ENABLE));
            */

            use ed25519_dalek::Verifier;
            if pubkey.verify(image, &ed25519_signature).is_ok() {
                gfx.msg("Signature check passed\n\r", &mut cursor);
                uart.tiny_write_str("Signature check passed\n\r");
                break;
            } else {
                // signature didn't work out, setup the next key and try it
                match keyloc {
                    KeyLoc::SelfSignPub => {
                        keyloc = KeyLoc::ThirdPartyPub;
                        continue;
                    }
                    KeyLoc::ThirdPartyPub => {
                        // try another key and move on
                        keyloc = KeyLoc::DevPub;
                        continue;
                    }
                    KeyLoc::DevPub => {
                        // we're out of keys...display message, then try to power down
                        gfx.msg("Signature check failed; powering down\n\r", &mut cursor);
                        uart.tiny_write_str("Signature check failed; powering down\n\r");
                        die();
                    }
                }
            }
        }
    }

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
    #[cfg(feature="gfx")]
    gfx.msg("Free stack: 0x", &mut cursor);
    #[cfg(feature="gfx")]
    gfx.hex_word(unused_stack_words * 4, &mut cursor);
    uart.newline();

    let wait_kbhit = false;
    if wait_kbhit {
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
    }
    #[cfg(feature="gfx")]
    gfx.msg("\n\r\n\rJumping to loader...\n\r", &mut cursor);
    uart.tiny_write_str("\n\r\n\rJumping to loader...\n\r");

    #[cfg(feature="hw-sec")]
    {
        let mut sha_csr = CSR::new(utra::sha512::HW_SHA512_BASE as *mut u32);
        sha_csr.wfo(utra::sha512::POWER_ON, 0); // cut power to the SHA block; this is the expected default state after the bootloader is done.
        let mut engine_csr = CSR::new(utra::engine::HW_ENGINE_BASE as *mut u32);
        engine_csr.wfo(utra::engine::POWER_ON, 0); // cut power to the engine block; this is the expected default state after the bootloader is done.
        // note that removing power does *not* clear the RF or microcode state -- data can leak from the bootloader
        // into other areas because of this! (but I think it's OK because we just mess around with public keys here)
    }

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

#[cfg(feature="hw-sec")]
fn die() {
    let ticktimer = CSR::new(utra::ticktimer::HW_TICKTIMER_BASE as *mut u32);
    let mut power = CSR::new(utra::power::HW_POWER_BASE as *mut u32);
    let mut com = CSR::new(utra::com::HW_COM_BASE as *mut u32);
    let mut keyrom = Keyrom::new();
    keyrom.lock();
    let mut start = ticktimer.rf(utra::ticktimer::TIME0_TIME);
    loop {
        // every 15 seconds, attempt to send a power down command
        // any attempt to re-flash the system must halt the CPU before we time-out to this point!
        if ticktimer.rf(utra::ticktimer::TIME0_TIME) - start > 15_000 {
            power.rmwf(utra::power::POWER_STATE, 0);
            power.rmwf(utra::power::POWER_SELF, 0);

            // ship mode is the safest mode -- suitable for long-term storage (~years)
            com.wfo(utra::com::TX_TX, com_rs::ComState::POWER_SHIPMODE.verb as u32);
            while com.rf(utra::com::STATUS_TIP) == 1 {}
            let _ = com.rf(utra::com::RX_RX); // discard the RX result

            start = ticktimer.rf(utra::ticktimer::TIME0_TIME);
            keyrom.lock();
        }
    }
}

