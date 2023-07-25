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

    unsafe {
        (0x400400a0 as *mut u32).write_volatile(0x1F598); // F
        uart.print_hex_word((0x400400a0 as *const u32).read_volatile());
        uart.putc('\n' as u32 as u8);
        let poke_array: [(u32, u32, bool); 12] = [
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
}

#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(_unused1: *const usize, _unused2: u32) -> ! {
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

    let mut uart = debug::Uart {};
    uart.tiny_write_str("booting... 006\r");

    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report_api(0x600dc0de);

    // report the measured reset value
    let resetvalue = CSR::new(utra::resetvalue::HW_RESETVALUE_BASE as *mut u32);
    report_api(resetvalue.r(utra::resetvalue::PC));

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

    // ---------- pio test option -------------
    #[cfg(feature="pio-test")]
    xous_pio::pio_tests::setup_reporting((utra::main::REPORT.offset() + utra::main::HW_MAIN_BASE) as *mut u32);
    #[cfg(feature="pio-test")]
    xous_pio::pio_tests::pio_tests();

    // ---------- exception setup ------------------
    irqs::irq_setup();

    // ---------- PL230 test option ----------------
    #[cfg(feature="pl230-test")]
    xous_pl230::pl230_tests::pl230_tests();

    // ---------- coreuser test --------------------
    satp::satp_test();

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

    loop {
        #[cfg(feature="daric")]
        {
            let mut uart = debug::Uart {};
            uart.tiny_write_str("test finished\r");
        }
    }
}

