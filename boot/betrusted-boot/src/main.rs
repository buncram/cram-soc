#![no_std]
#![no_main]

#![allow(unreachable_code)] // allow debugging of failures to jump out of the bootloader

use utralib::generated::*;
use core::convert::TryInto;
use core::convert::TryFrom;
use core::mem::size_of;

mod debug;
mod satp;
mod irqs;

mod asm;

#[cfg(feature="full-chip")]
mod daric_generated;
// you know what's irritating? if this file is named apb_test, clippy complains because
// it's not a #test. wtf yo. not all tests are just for you, clippy!
#[cfg(feature="apb-test")]
mod apb_check;
#[cfg(feature="apb-test")]
use apb_check::apb_test;

/*
    Notes about printing:
      - the println! and write! macros are actually quite expensive in the context of a 32kiB ROM (~4k overhead??)
      - we are trying to get away with direct putc() and tiny_write_str() calls. looks weird for Rust, but it saves a few bytes
*/

#[cfg(target_os="none")]
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

#[cfg(not(feature="daric"))]
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
    uart.putc(0xdu8); // add a CR character
}

/// chunks through the entire bank of data
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
/// our desired test length is 512 entries, so pick an LFSR with a period of 2^9-1...
pub fn lfsr_next(state: u16) -> u16 {
    let bit = ((state >> 8) ^
               (state >>  4)) & 1;

    ((state << 1) + bit) & 0x1_FF
}

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

pub fn xip_test() {
    report_api(0x61D0_0000);
    // a code snippet that adds 0x400 to the argument and returns
    let code = [0x4005_0513u32, 0x0000_8082u32];

    // shove it into the XIP region
    let xip_dest = unsafe{core::slice::from_raw_parts_mut(satp::XIP_VA as *mut u32, 2)};
    xip_dest.copy_from_slice(&code);

    // run the code
    let mut test_val: usize = 0x5555_0000;
    let mut expected: usize = test_val;
    for _ in 0..8 {
        test_val = crate::asm::jmp_remote(test_val, satp::XIP_VA);
        report_api(test_val as u32);
        expected += 0x0400;
        assert!(expected == test_val);
    }

    // prep a second region, a little bit further away to trigger a second access
    // self-modifying code is *not* supported on Vex
    const XIP_OFFSET: usize = 0;
    let xip_dest2 = unsafe{core::slice::from_raw_parts_mut((satp::XIP_VA + XIP_OFFSET) as *mut u32, 2)};
    let code2 = [0x0015_0513u32, 0x0000_8082u32];
    xip_dest2.copy_from_slice(&code2);
    // this forces a reload of the i-cache
    unsafe {
    core::arch::asm!(
        "fence.i",
    );}

    // run the new code and see that it was updated?
    for _ in 0..8 {
        test_val = crate::asm::jmp_remote(test_val, satp::XIP_VA + XIP_OFFSET);
        report_api(test_val as u32);
        expected += 1;
        assert!(expected == test_val);
    }
    report_api(0x61D0_600D);
}

#[cfg(feature="full-chip")]
pub fn reset_ticktimer() {
    let mut  tt = CSR::new(utra::ticktimer::HW_TICKTIMER_BASE as *mut u32);
    // tt.wo(utra::ticktimer::CLOCKS_PER_TICK, 160);
    tt.wo(utra::ticktimer::CLOCKS_PER_TICK, 369560); // based on 369.56MHz default clock
    tt.wfo(utra::ticktimer::CONTROL_RESET, 1);
    tt.wo(utra::ticktimer::CONTROL, 0);
}
#[cfg(feature="full-chip")]
pub fn snap_ticks(title: &str) {
    let tt = CSR::new(utra::ticktimer::HW_TICKTIMER_BASE as *mut u32);
    let mut uart = debug::Uart {};
    uart.tiny_write_str(title);
    uart.tiny_write_str(" time: ");
    uart.print_hex_word(tt.rf(utra::ticktimer::TIME0_TIME));
    // write!(uart, "{} time: {} ticks\n", title, elapsed).ok();
    uart.tiny_write_str(" ticks\n");
}


#[cfg(feature="full-chip")]
pub fn early_init() {
    let mut uart = debug::Uart {};

    #[cfg(not(feature="cram-fpga"))]
    unsafe {
        (0x400400a0 as *mut u32).write_volatile(0x1F598); // F
        uart.print_hex_word((0x400400a0 as *const u32).read_volatile());
        uart.putc('\n' as u32 as u8);
        let poke_array: [(u32, u32, bool); 12] = [
            // commented out because the FPGA does not take kindly to this being set twice
            (0x400400a4, 0x2812, false),   //  MN
            (0x400400a8, 0x3301, false),   //  Q
            (0x40040090, 0x0032, true),  // setpll
            (0x40040014, 0x7f7f, false),  // fclk
            (0x40040018, 0x7f7f, false),  // aclk
            (0x4004001c, 0x3f3f, false),  // hclk
            (0x40040020, 0x1f1f, false),  // iclk
            (0x40040024, 0x0f0f, false),  // pclk
            (0x40040010, 0x0001, false),  // sel0
            (0x4004002c, 0x0032, true),  // setcgu
            (0x40040060, 0x0003, false),  // aclk gates
            (0x40040064, 0x0003, false),  // hclk gates
        ];
        for &(addr, dat, is_u32) in poke_array.iter() {
            let rbk = if is_u32 {
                (addr as *mut u32).write_volatile(dat);
                (addr as *const u32).read_volatile()
            } else {
                (addr as *mut u16).write_volatile(dat as u16);
                (addr as *const u16).read_volatile() as u32
            };
            uart.print_hex_word(rbk);
            if dat != rbk {
                uart.putc('*' as u32 as u8);
            }
            uart.putc('\n' as u32 as u8);
        }
    }
    #[cfg(feature="cram-fpga")]
    unsafe {
        let poke_array: [(u32, u32, bool); 9] = [
            (0x40040030, 0x0001, true),  // cgusel1
            (0x40040010, 0x0001, true),  // cgusel0
            (0x40040010, 0x0001, true),  // cgusel0
            (0x40040014, 0x007f, true),  // fdfclk
            (0x40040018, 0x007f, true),  // fdaclk
            (0x4004001c, 0x007f, true),  // fdhclk
            (0x40040020, 0x007f, true),  // fdiclk
            (0x40040024, 0x007f, true),  // fdpclk
            (0x400400a0, 0x4040, false),  // pllmn FPGA
        ];
        for &(addr, dat, is_u32) in poke_array.iter() {
            let rbk = if is_u32 {
                (addr as *mut u32).write_volatile(dat);
                (addr as *const u32).read_volatile()
            } else {
                (addr as *mut u16).write_volatile(dat as u16);
                (addr as *const u16).read_volatile() as u32
            };
            uart.print_hex_word(rbk);
            if dat != rbk {
                uart.putc('*' as u32 as u8);
            }
            uart.putc('\n' as u32 as u8);
        }
    }

}

// these register do not exist in our local simulation model
//#[cfg(feature="full-chip")]
pub fn setup_uart1() {
    let mut uart = debug::Uart {};
    let sysctrl = CSR::new(utra::sysctrl::HW_SYSCTRL_BASE as *mut u32);
    uart.tiny_write_str("FREQ0: ");
    uart.print_hex_word(sysctrl.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ0_FSFREQ0));
    uart.tiny_write_str("\n\r");
    uart.tiny_write_str("FREQ1: ");
    uart.print_hex_word(sysctrl.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ1_FSFREQ1));
    uart.tiny_write_str("\n\r");
    uart.tiny_write_str("FREQ2: ");
    uart.print_hex_word(sysctrl.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ2_FSFREQ2));
    uart.tiny_write_str("\n\r");
    uart.tiny_write_str("FREQ3: ");
    uart.print_hex_word(sysctrl.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ3_FSFREQ3));
    uart.tiny_write_str("\n\r");

    let mut udma_ctrl = CSR::new(utra::udma_ctrl::HW_UDMA_CTRL_BASE as *mut u32);
    // peripheral ID 1 is UART0
    // setup iomux
    // TODO: fix parameter resolution to trace back to the top level file. there's actually 6*16 IO banks, not the 4 specified in the leaf design file :P
    let iox_csr = utra::iox::HW_IOX_BASE as *mut u32;
    unsafe {
        iox_csr.add(0).write_volatile(0b00_00_00_01_01_00_00_00);  // PAL AF1 on PA3/PA4
        iox_csr.add(0x1c / core::mem::size_of::<u32>()).write_volatile(0x1400); // PDH
        iox_csr.add(0x148 / core::mem::size_of::<u32>()).write_volatile(0x10); // PA4 output
        iox_csr.add(0x148 / core::mem::size_of::<u32>() + 3).write_volatile(0xffff); // PD
        iox_csr.add(0x160 / core::mem::size_of::<u32>()).write_volatile(0x8); // PA3 pullup
    }

    // TODO: fix register generator, input is incorrect and pins peripheral count at 6
    uart.tiny_write_str("udma\r");
    udma_ctrl.wo(utra::udma_ctrl::REG_CG, 1);

    let baudrate: u32 = 115200;
    let freq: u32 = 100_000_000;
    let clk_counter: u32 = (freq + baudrate / 2) / baudrate;
    let mut udma_uart = CSR::new(utra::udma_uart_0::HW_UDMA_UART_0_BASE as *mut u32);
    udma_uart.wo(utra::udma_uart_0::REG_UART_SETUP,
        0x0306 | (clk_counter << 16));

    let tx_buf = utralib::HW_IFRAM0_MEM as *mut u8;
    uart.print_hex_word(
        udma_uart.r(utra::udma_uart_0::REG_UART_SETUP)
    );
    // let mut tx_buf = [0u8; 256];
    for i in 0..16 {
        unsafe { tx_buf.add(i).write_volatile('0' as u32 as u8 + i as u8) };
    }
    udma_uart.wo(utra::udma_uart_0::REG_TX_SADDR, tx_buf as u32);
    udma_uart.wo(utra::udma_uart_0::REG_TX_SIZE, 4); // abridged so simulation run faster
    // send it
    udma_uart.wo(utra::udma_uart_0::REG_TX_CFG, 0x10); // EN
    // wait for it all to be done
    while udma_uart.rf(utra::udma_uart_0::REG_TX_CFG_R_TX_EN) != 0 {   }
    while (udma_uart.r(utra::udma_uart_0::REG_STATUS) & 1) != 0 {  }
    uart.tiny_write_str("udma done\r");

}

/// used to generate some test vectors
pub fn lfsr_next_u32(state: u32) -> u32 {
    let bit = ((state >> 31) ^
               (state >> 21) ^
               (state >>  1) ^
               (state >>  0)) & 1;

    (state << 1) + bit
}

pub fn sce_dma_tests() -> bool {
    let mut uart = debug::Uart {};
    let mut sce_ctl_csr = CSR::new(utra::sce_glbsfr::HW_SCE_GLBSFR_BASE as *mut u32);
    sce_ctl_csr.wfo(utra::sce_glbsfr::SFR_SUBEN_CR_SUBEN, 0x1F);
    let mut sdma_csr = CSR::new(utra::scedma::HW_SCEDMA_BASE as *mut u32);
    const BLOCKLEN: usize = 16; // blocks must be pre-padded or of exactly this length
    const DMA_LEN: usize = BLOCKLEN; // FIFO buffers
    let sk: [u32; 72] = [
        0x6a09e667,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
        0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
        0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
        0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
        0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
        0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
        0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
        0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
        0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
    ];
    uart.tiny_write_str("init hash\r");
    // setup the sk region
    let sk_mem = unsafe{
        core::slice::from_raw_parts_mut(
            utralib::HW_SEG_LKEY_MEM as *mut u32,
            sk.len())
    };
    // zeroize
    for d in sk_mem.iter_mut() {
        *d = 0;
    }
    // then init hash value
    sk_mem[..sk.len()].copy_from_slice(&sk);

    // setup the SCEDMA to do a simple transfer between two memory regions
    let mut region_a = [0u32; DMA_LEN];
    let region_b = [0u32; DMA_LEN];
    let region_c = [0u32; DMA_LEN];
    if false {
        let mut state = 0xF0F0_A0A0;
        for d in region_a.iter_mut() {
            *d = state;
            state = lfsr_next_u32(state);
        }
    } else {
        for d in region_a.iter_mut() {
            *d = 0x9999_9999; // palindromic value just to rule out endianness in first testing
        }
    }

    uart.tiny_write_str("init done\r");
    // enable the hash FIFO (bit 1) -- this must happen first
    sce_ctl_csr.wfo(utra::sce_glbsfr::SFR_FFEN_CR_FFEN, 0b00010);

    // -------- combohash tests --------
    let mut hash_csr = CSR::new(utra::combohash::HW_COMBOHASH_BASE as *mut u32);
    hash_csr.wfo(utra::combohash::SFR_CRFUNC_CR_FUNC, 0); // HF_SHA256
    hash_csr.wfo(utra::combohash::SFR_OPT1_CR_OPT_HASHCNT, 0); // run the hash on two DMA blocks
    hash_csr.wfo(utra::combohash::SFR_OPT2_CR_OPT_IFSTART, 1); // start from 1st block
    hash_csr.rmwf(utra::combohash::SFR_OPT2_CR_OPT_IFSOB, 1); // write data to seg-sob when done
    hash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_MSG_SEGID_MSG, 0); // message goes from location 0
    hash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT, 0); // message goes to location in HOUT area
    // trigger start hash, but it should wait until the DMA runs
    hash_csr.wfo(utra::combohash::SFR_AR_SFR_AR, 0x5A);

    // dma the data in region_a to the hash engine; device should automatically ensure no buffers are overfilled
    sdma_csr.wfo(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, region_a.as_ptr() as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_OPT_SFR_XCH_OPT, 0b1_0000); // endian swap
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, 4); // HASH_MSG region
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, 0);
    sdma_csr.wfo(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, DMA_LEN as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, 0); // 0 == AXI read, 1 == AXI write
    sdma_csr.wfo(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, 0xA5); // 0x5a ich start, 0xa5 xch start, 0xaa sch start

    // observe the hash done output
    for _ in 0..2 {
        uart.print_hex_word(sce_ctl_csr.r(utra::combohash::SFR_FR));
        uart.tiny_write_str(" <- hash FR\r")
    }

    // print the hash output
    let hout_mem = unsafe{
        core::slice::from_raw_parts(
            utralib::HW_SEG_HOUT_MEM as *mut u32,
            utralib::HW_SEG_HOUT_MEM_LEN / core::mem::size_of::<u32>())
    };
    sce_ctl_csr.wfo(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT, 0b1_0000); // endian swap AHB read
    uart.tiny_write_str("HOUT (BE): ");
    for i in 0..8 {
        // should be big-endian
        uart.print_hex_word(hout_mem[i]);
    }
    uart.tiny_write_str("\r");
    sce_ctl_csr.wfo(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT, 0b0_0000);
    uart.tiny_write_str("HOUT (LE): ");
    for i in 0..8 {
        // should be big-endian
        uart.print_hex_word(hout_mem[i]);
    }
    uart.tiny_write_str("\r");

    uart.tiny_write_str("HIN ");
    for d in region_a {
        // big-endian, so make it one big string
        uart.print_hex_word(d);
    }
    uart.tiny_write_str("\r");

    // -------- AES tests ---------
    // fifo 2 = AES in, fifo 3 = AES out -- this must happen first
    sce_ctl_csr.wfo(utra::sce_glbsfr::SFR_FFEN_CR_FFEN, 0b00100);

    // make sure that the destination is empty
    let mut errs = 0;
    for (src, dst) in region_a.iter().zip(region_b.iter()) {
        if *src != *dst {
            errs += 1;
        }
    }
    uart.tiny_write_str("dest mismatch count (should not be 0): ");
    uart.print_hex_word(errs);
    uart.tiny_write_str("\r");

    let mut aes_csr = CSR::new(utra::aes::HW_AES_BASE as *mut u32);
    // schedule the 0-key
    aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_AKEY, 0);
    aes_csr.rmwf(utra::aes::SFR_OPT_OPT_KLEN0, 0b10); // 256 bit key
    aes_csr.rmwf(utra::aes::SFR_OPT_OPT_MODE0, 0b000); // ECB
    aes_csr.wfo(utra::aes::SFR_CRFUNC_SFR_CRFUNC, 0x0); // AES-KS
    aes_csr.wo(utra::aes::SFR_AR, 0x5a);
    uart.tiny_write_str("AES KS\r");

    // setup the encryption
    aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_AIB, 0);
    aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_AOB, 0);
    aes_csr.rmwf(utra::aes::SFR_OPT_OPT_KLEN0, 0b10); // 256 bit key
    aes_csr.rmwf(utra::aes::SFR_OPT_OPT_MODE0, 0b000); // ECB
    aes_csr.wfo(utra::aes::SFR_CRFUNC_SFR_CRFUNC, 0x1); // AES-ENC

    // start the AES op, should not run until FIFO fills data...
    uart.tiny_write_str("start AES op\r");
    aes_csr.wfo(utra::aes::SFR_OPT1_SFR_OPT1, DMA_LEN as u32 / (128 / 32));
    aes_csr.wo(utra::aes::SFR_AR, 0x5a);

    // dma the data in region_a to the AES engine
    sdma_csr.wfo(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, region_a.as_ptr() as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, 14); // 13 AKEY, 14 AIB, 15, AOB
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, 0);
    sdma_csr.wfo(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, DMA_LEN as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, 0); // 0 == AXI read, 1 == AXI write
    sdma_csr.wfo(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, 0xA5); // 0x5a ich start, 0xa5 xch start, 0xaa sch start

    uart.tiny_write_str("scdma op 1 in progress\r"); // waste some time while the DMA runs...
    // while sce_ctl_csr.rf(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY) != 0 {
        uart.print_hex_word(sce_ctl_csr.rf(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY));
        uart.tiny_write_str(" ");
        uart.print_hex_word(sce_ctl_csr.rf(utra::sce_glbsfr::SFR_FRDONE_FR_DONE));
        uart.tiny_write_str(" waiting\r");
    // }

    // wait for aes op to be done
    // while aes_csr.rf(utra::sce_glbsfr::SFR_FRDONE_FR_DONE) != 0 {
        uart.print_hex_word(aes_csr.rf(utra::aes::SFR_SEGPTR_PTRID_AOB_PTRID_AOB));
        uart.tiny_write_str(" aes waiting\r");
    // }

    // dma the data in region_b from the segment
    sdma_csr.wfo(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, region_b.as_ptr() as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, 15);
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, 0);
    sdma_csr.wfo(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, DMA_LEN as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, 1); // 0 == AXI read, 1 == AXI write
    sdma_csr.wfo(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, 0xA5); // 0x5a ich start, 0xa5 xch start, 0xaa sch start
    uart.tiny_write_str("scdma op 2 in progress\r"); // waste some time while the DMA runs...

    // flush the cache, otherwise we won't see the updated values in region_b
    unsafe {core::arch::asm!(
        ".word 0x500F",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
    ); }

    for (i, (src, dst)) in region_a.iter().zip(region_b.iter()).enumerate() {
        if *src != *dst {
            uart.tiny_write_str("error in iter ");
            uart.print_hex_word(i as u32);
            uart.tiny_write_str(": ");
            uart.print_hex_word(*src);
            uart.tiny_write_str(" s<->d ");
            uart.print_hex_word(*dst);
            uart.tiny_write_str("\r");
            break; // just print something so we can know the intermediate is "ok"
        }
    }

    // decode the data to see if it's at least symmetric
    aes_csr.wfo(utra::aes::SFR_CRFUNC_SFR_CRFUNC, 0x2); // AES-DEC

    // dma the data in region_a to the AES engine
    sdma_csr.wfo(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, region_b.as_ptr() as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, 14); // 13 AKEY, 14 AIB, 15, AOB
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, 0);
    sdma_csr.wfo(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, DMA_LEN as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, 0); // 0 == AXI read, 1 == AXI write
    sdma_csr.wfo(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, 0xA5); // 0x5a ich start, 0xa5 xch start, 0xaa sch start

    // start the AES op
    uart.tiny_write_str("start AES op\r");
    aes_csr.wfo(utra::aes::SFR_OPT1_SFR_OPT1, DMA_LEN as u32 / (128 / 32));
    aes_csr.wo(utra::aes::SFR_AR, 0x5a);
    uart.tiny_write_str("scdma op 3 in progress\r"); // waste some time while the DMA runs...

    // dma the data in region_b from the segment
    sdma_csr.wfo(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, region_c.as_ptr() as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, 15);
    sdma_csr.wfo(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, 0);
    sdma_csr.wfo(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, DMA_LEN as u32);
    sdma_csr.wfo(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, 1); // 0 == AXI read, 1 == AXI write
    sdma_csr.wfo(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, 0xA5); // 0x5a ich start, 0xa5 xch start, 0xaa sch start
    uart.tiny_write_str("scdma op 4 in progress\r"); // waste some time while the DMA runs...

    // flush the cache, otherwise we won't see the updated values in region_b
    unsafe {core::arch::asm!(
        ".word 0x500F",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
    ); }

    let mut passing = true;
    errs = 0;
    // compare a to c: these should now be identical, with enc->dec
    for (i, (src, dst)) in region_a.iter().zip(region_c.iter()).enumerate() {
        if *src != *dst {
            uart.tiny_write_str("error in iter ");
            uart.print_hex_word(i as u32);
            uart.tiny_write_str(": ");
            uart.print_hex_word(*src);
            uart.tiny_write_str(" s<->d ");
            uart.print_hex_word(*dst);
            uart.tiny_write_str("\r");
            passing = false;
            errs += 1;
        }
    }
    uart.tiny_write_str("errs: ");
    uart.print_hex_word(errs);
    uart.tiny_write_str("\r");

    passing
}

#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(_unused1: *const usize, _unused2: u32) -> ! {
    let mut uart = debug::Uart {};
    uart.tiny_write_str("hello world!\r");

    #[cfg(feature="full-chip")]
    {
        let u8_test =  crate::debug::duart::HW_DUART_BASE as *mut u8;
        let u16_test = crate::debug::duart::HW_DUART_BASE as *mut u16;

        // quick test to check byte and word write strobes on the
        unsafe {
            u8_test.write_volatile(0x31);
            u8_test.add(1).write_volatile(32);
            u8_test.add(2).write_volatile(33);
            u8_test.add(3).write_volatile(34);

            u16_test.write_volatile(0x44);
            u16_test.add(1).write_volatile(0x55);
        }
        reset_ticktimer();
        snap_ticks("sysctrl: ipen ");

        early_init();
    }

    uart.tiny_write_str("booting... 006\r");

    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report_api(0x600dc0de);

    // report the measured reset value
    let resetvalue = CSR::new(utra::resetvalue::HW_RESETVALUE_BASE as *mut u32);
    report_api(resetvalue.r(utra::resetvalue::PC));

    #[cfg(feature="full-chip")]
    // sce_dma_tests();

    #[cfg(feature="full-chip")]
    setup_uart1();

    // ---------- if activated, run the APB test. This is based off of Philip's "touch all the registers" test.
    #[cfg(feature="apb-test")]
    apb_test();

    // ---------- vm setup -------------------------
    satp::satp_setup(); // at the conclusion of this, we are running in "supervisor" (kernel) mode, with Sv32 semantics
    report_api(0x5a1d_6060);

    #[cfg(feature="daric")]
    {
        let mut uart = debug::Uart {};
        uart.tiny_write_str("hello world!\r");
    }
    #[cfg(feature="pio-test")]
    xous_pio::pio_tests::setup_reporting((utra::main::REPORT.offset() + utra::main::HW_MAIN_BASE) as *mut u32);

    // ---------- PIO hack-test ----------------
    //#[cfg(feature="pio-test")]
    //{
    //    uart.tiny_write_str("spi test\r");
    //    pio_hack_test();
    //    uart.tiny_write_str("spi test done\r");
    //}

    // ---------- pio test option -------------
    #[cfg(feature="pio-test")]
    xous_pio::pio_tests::pio_tests();

    // ---------- bio test option -------------
    #[cfg(feature="bio-test")]
    uart.tiny_write_str("bio start\r");
    #[cfg(feature="bio-test")]
    xous_bio::bio_tests::bio_tests();
    #[cfg(feature="bio-test")]
    uart.tiny_write_str("bio end\r");

    // ---------- exception setup ------------------
    irqs::irq_setup();

    // ---------- PL230 test option ----------------
    #[cfg(feature="pl230-test")] {
        let iox_csr = utra::iox::HW_IOX_BASE as *mut u32;
        unsafe {
            iox_csr.add(0x8 / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PBL
            iox_csr.add(0xC / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PBH
            iox_csr.add(0x10 / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PCL
            iox_csr.add(0x14 / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PCH
            iox_csr.add(0x200 / core::mem::size_of::<u32>()).write_volatile(0xffffffff); // PIO sel port D31-0
        }
        xous_pl230::pl230_tests::pl230_tests();
    }

    uart.tiny_write_str("done\r");

    // ---------- coreuser test --------------------
    satp::satp_test();
    uart.tiny_write_str("satp done\r");

    // ---------- exception test -------------------
    irqs::irq_test();

    // ---------- xip region test ------------------
    #[cfg(feature="xip")]
    xip_test();

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
    #[cfg(not(feature="full-chip"))] // these register do not exist on the full chip, it's only in the local validation framework
    {
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

    uart.tiny_write_str("test finished\r");
    loop {
        #[cfg(feature="daric")]
        {
            // uart.tiny_write_str("test finished\r");
        }
    }
}

use xous_pio::*;
use xous_pio::pio_tests::spi::*;

pub fn spi_test_core_boot(pio_sm: &mut PioSm) -> bool {
    report_api(0x0D10_05D1);

    const BUF_SIZE: usize = 20;
    let mut state: u16 = 0xAF;
    let mut tx_buf = [0u8; BUF_SIZE];
    let mut rx_buf = [0u8; BUF_SIZE];
    // init the TX buf
    for d in tx_buf.iter_mut() {
        state = crate::lfsr_next(state);
        *d = state as u8;
        report_api(*d as u32);
    }
    pio_spi_write8_read8_blocking(pio_sm, &tx_buf, &mut rx_buf);
    let mut pass = true;
    for (&s, &d) in tx_buf.iter().zip(rx_buf.iter()) {
        if s != d {
            report_api(0xDEAD_0000 | (s as u32) << 8 | ((d as u32) << 0));
            pass = false;
        }
    }
    report_api(0x600D_05D1);
    pass
}

pub fn pio_hack_test() -> bool {
    let iox_csr = utra::iox::HW_IOX_BASE as *mut u32;
    unsafe {
        iox_csr.add(0x8 / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PBL
        iox_csr.add(0xC / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PBH
        iox_csr.add(0x10 / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PCL
        iox_csr.add(0x14 / core::mem::size_of::<u32>()).write_volatile(0b0101_0101_0101_0101);  // PCH
        iox_csr.add(0x200 / core::mem::size_of::<u32>()).write_volatile(0xffffffff); // PIO sel port D31-0
    }

    const PIN_SCK: usize = 16;  // PC00
    const PIN_MOSI: usize = 17; // PC01
    const PIN_MISO: usize = 17; // loopback    18; // PC02

    let mut pio_csr = CSR::new(utra::rp_pio::HW_RP_PIO_BASE as *mut u32);

    report_api(0x0D10_05D1);

    let mut pio_ss = PioSharedState::new();
    let mut pio_sm = pio_ss.alloc_sm().unwrap();

    // spi_cpha0 example
    let spi_cpha0_prog = pio_proc::pio_asm!(
        ".side_set 1",
        "out pins, 1 side 0 [1]",
        "in pins, 1  side 1 [1]",
    );
    // spi_cpha1 example
    let spi_cpha1_prog = pio_proc::pio_asm!(
        ".side_set 1",
        "out x, 1    side 0", // Stall here on empty (keep SCK deasserted)
        "mov pins, x side 1 [1]", // Output data, assert SCK (mov pins uses OUT mapping)
        "in pins, 1  side 0" // Input data, deassert SCK
    );
    let prog_cpha0 = LoadedProg::load(spi_cpha0_prog.program, &mut pio_ss).unwrap();
    report_api(0x05D1_0000);
    let prog_cpha1 = LoadedProg::load(spi_cpha1_prog.program, &mut pio_ss).unwrap();
    report_api(0x05D1_0001);

    let clkdiv: f32 = 137.25;
    let mut passing = true;
    let mut cpol = false;
    pio_csr.wo(utra::rp_pio::SFR_IRQ0_INTE, pio_sm.sm_bitmask());
    pio_csr.wo(utra::rp_pio::SFR_IRQ1_INTE, (pio_sm.sm_bitmask()) << 4);
    loop {
        // pha = 1
        report_api(0x05D1_0002);
        pio_spi_init(
            &mut pio_sm,
            &prog_cpha0, // cpha set here
            8,
            clkdiv,
            cpol,
            PIN_SCK,
            PIN_MOSI,
            PIN_MISO
        );
        report_api(0x05D1_0003);
        if spi_test_core_boot(&mut pio_sm) == false {
            passing = false;
        };

        // pha = 0
        report_api(0x05D1_0004);
        pio_spi_init(
            &mut pio_sm,
            &prog_cpha1, // cpha set here
            8,
            clkdiv,
            cpol,
            PIN_SCK,
            PIN_MOSI,
            PIN_MISO
        );
        report_api(0x05D1_0005);
        if spi_test_core_boot(&mut pio_sm) == false {
            passing = false;
        };
        if cpol {
            break;
        }
        // switch to next cpol value for test
        cpol = true;
    }
    // cleanup external side effects for next test
    pio_sm.gpio_reset_overrides();
    pio_csr.wo(utra::rp_pio::SFR_IRQ0_INTE, 0);
    pio_csr.wo(utra::rp_pio::SFR_IRQ1_INTE, 0);
    pio_csr.wo(utra::rp_pio::SFR_SYNC_BYPASS, 0);

    if passing {
        report_api(0x05D1_600D);
    } else {
        report_api(0x05D1_DEAD);
    }
    assert!(passing);
    passing
}