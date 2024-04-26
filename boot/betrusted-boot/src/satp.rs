use utralib::generated::*;
use crate::report_api;

pub const PAGE_SIZE: usize = 4096;
const WORD_SIZE: usize = core::mem::size_of::<u32>();

const FLG_VALID: usize = 0x1;
const FLG_X: usize = 0x8;
const FLG_W: usize = 0x4;
const FLG_R: usize = 0x2;
#[allow(dead_code)]
const FLG_U: usize = 0x10;
#[allow(dead_code)]
const FLG_A: usize = 0x40;
#[allow(dead_code)]
const FLG_D: usize = 0x80;

#[repr(C)]
pub struct PageTable {
    entries: [usize; PAGE_SIZE / WORD_SIZE],
}

// locate the page table entries
const ROOT_PT_PA: usize = 0x6100_0000; // 1st level at base of sram
// 2nd level PTs
const SRAM_PT_PA: usize  = 0x6100_1000;
const CODE_PT_PA: usize  = 0x6100_2000;
const CSR_PT_PA: usize   = 0x6100_3000;
const PERI_PT_PA: usize  = 0x6100_4000;
const PIO_PT_PA: usize   = 0x6100_5000;
const XIP_PT_PA: usize   = 0x6100_6000;
const RV_PT_PA: usize    = 0x6100_7000;
// exception handler pages. Mapped 1:1 PA:VA, so no explicit remapping needed as RAM area is already mapped.
pub const SCRATCH_PAGE: usize           = 0x6100_8000; // update this in irq.rs _start_trap() asm! as the scratch are
pub const _EXCEPTION_STACK_LIMIT: usize = 0x6100_9000; // update this in irq.rs as default stack pointer. The start of stack is this + 0x1000 & grows down
const BSS_PAGE: usize                   = 0x6100_A000; // this is manually read out of the link file. Equal to "base of RAM"
pub const PT_LIMIT: usize               = 0x6100_B000; // this is carved out in link.x by setting RAM base at BSS_PAGE start

// VAs
#[cfg(feature="gdb-load")]
const CODE_VA:     usize = 0x6000_0000;
#[cfg(not(feature="gdb-load"))]
const CODE_VA:     usize = 0x0000_0000;
const CSR_VA:      usize = 0x5800_0000;
const PERI_VA:     usize = 0x4000_0000;
const SRAM_VA:     usize = 0x6100_0000;
const PIO_VA:      usize = 0x5000_0000;
pub const XIP_VA:  usize = 0x7000_0000;
const RV_VA:       usize = 0xE000_0000;

// PAs (when different from VAs)
const RERAM_PA: usize = 0x6000_0000;

fn set_l1_pte(from_va: usize, to_pa: usize, root_pt: &mut PageTable) {
    let index = from_va >> 22;
    root_pt.entries[index] =
        ((to_pa & 0xFFFF_FC00) >> 2) // top 2 bits of PA are not used, we don't do 34-bit PA featured by Sv32
        | FLG_VALID;
}

fn set_l2_pte(from_va: usize, to_pa: usize, l2_pt: &mut PageTable, flags: usize) {
    let index = (from_va >> 12) & 0x3_FF;
    l2_pt.entries[index] =
        ((to_pa & 0xFFFF_FC00) >> 2) // top 2 bits of PA are not used, we don't do 34-bit PA featured by Sv32
        | flags
        | FLG_VALID;
}

/// Very simple Sv32 setup that drops into supervisor (kernel) mode, with most
/// mappings being 1:1 between VA->PA, except for code which is remapped to address 0x0 in VA space.
#[inline(never)] // correct behavior depends on RA being set.
pub fn satp_setup() {
    unsafe {
        let pt_clr = ROOT_PT_PA as *mut u32;
        for i in 0..(PT_LIMIT - ROOT_PT_PA) / core::mem::size_of::<u32>() {
            pt_clr.add(i).write_volatile(0x0000_0000);
        }
    }
    // root page table is at p0x6100_0000 == v0x6100_0000
    let mut root_pt = unsafe { &mut *(ROOT_PT_PA as *mut PageTable) };
    let mut sram_pt = unsafe { &mut *(SRAM_PT_PA as *mut PageTable) };
    let mut code_pt = unsafe { &mut *(CODE_PT_PA as *mut PageTable) };
    let mut csr_pt  = unsafe { &mut *(CSR_PT_PA  as *mut PageTable) };
    let mut peri_pt = unsafe { &mut *(PERI_PT_PA as *mut PageTable) };
    let mut pio_pt = unsafe { &mut *(PIO_PT_PA as *mut PageTable) };
    let mut xip_pt = unsafe { &mut *(XIP_PT_PA as *mut PageTable) };
    let mut rv_pt = unsafe { &mut *(RV_PT_PA as *mut PageTable) };

    set_l1_pte(CODE_VA, CODE_PT_PA, &mut root_pt);
    set_l1_pte(CSR_VA, CSR_PT_PA, &mut root_pt);
    set_l1_pte(PERI_VA, PERI_PT_PA, &mut root_pt);
    set_l1_pte(SRAM_VA, SRAM_PT_PA, &mut root_pt); // L1 covers 16MiB, so SP_VA will cover all of SRAM
    set_l1_pte(PIO_VA, PIO_PT_PA, &mut root_pt);
    set_l1_pte(XIP_VA, XIP_PT_PA, &mut root_pt);
    set_l1_pte(RV_VA, RV_PT_PA, &mut root_pt);

    // map code space. This is the only one that has a difference on VA->PA
    const CODE_LEN: usize = 0x40_0000; // 4 megs, a whole superpage.
    for offset in (0..CODE_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CODE_VA + offset, RERAM_PA + offset, &mut code_pt, FLG_X | FLG_R | FLG_U | FLG_W);
    }

    // map sram. Mapping is 1:1, so we use _VA and _PA targets for both args
    const SRAM_LEN: usize = 0x20_0000; // 2 megs
    // make the page tables not writeable
    for offset in (0..SCRATCH_PAGE - utralib::HW_SRAM_MEM).step_by(PAGE_SIZE) {
        set_l2_pte(SRAM_VA + offset, SRAM_VA + offset, &mut sram_pt, FLG_R | FLG_U);
    }
    // rest of RAM is r/w/x
    for offset in ((SCRATCH_PAGE - utralib::HW_SRAM_MEM)..SRAM_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(SRAM_VA + offset, SRAM_VA + offset, &mut sram_pt, FLG_W | FLG_R | FLG_U | FLG_X);
    }
    // map SoC-local peripherals (ticktimer, etc.)
    const CSR_LEN: usize = 0x2_0000;
    for offset in (0..CSR_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CSR_VA + offset, CSR_VA + offset, &mut csr_pt, FLG_W | FLG_R | FLG_U);
    }
    // map APB peripherals (includes a window for the simulation bench)
    const PERI_LEN: usize = 0x10_0000; // 1M window - this will also map all the peripherals, including SCE, except PIO
    for offset in (0..PERI_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(PERI_VA + offset, PERI_VA + offset, &mut peri_pt, FLG_W | FLG_R | FLG_U);
    }
    // map the IF AMBA matrix (includes PIO + UDMA IFRAM)
    const PIO_OFFSET: usize = 0x00_0000;
    const PIO_LEN: usize = 0x21_0000; // this will map all the interfaces up to and including the UDC.
    for offset in (PIO_OFFSET..PIO_OFFSET + PIO_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(PIO_VA + offset, PIO_VA + offset, &mut pio_pt, FLG_W | FLG_R | FLG_U);
    }
    // map the RV peripherals
    const RV_LEN: usize = 0x2_0000; // 128k window
    for offset in (0..RV_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(RV_VA + offset, RV_VA + offset, &mut rv_pt, FLG_W | FLG_R | FLG_U);
    }
    // map the XIP memory, just for testing
    const XIP_LEN: usize = 0x1_0000; // just 64k of it for testing
    for offset in (0..XIP_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(XIP_VA + offset, XIP_VA + offset, &mut xip_pt, FLG_X | FLG_W | FLG_R | FLG_U);
    }
    // clear BSS
    unsafe {
        let bss_region = core::slice::from_raw_parts_mut(BSS_PAGE as *mut u32, PAGE_SIZE / WORD_SIZE);
        for d in bss_region.iter_mut() {
            *d = 0;
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        }
    }

    let asid: u32 = 1;
    let satp: u32 =
        0x8000_0000
        | asid << 22
        | (ROOT_PT_PA as u32 >> 12);

    unsafe {
        core::arch::asm!(
            // Delegate as much as we can supervisor mode
            "li          t0, 0xffffffff",
            "csrw        mideleg, t0",
            "csrw        medeleg, t0",

            // Return to Supervisor mode (1 << 11) when we call `reti`.
            // Disable interrupts (0 << 5), allow supervisor mode to run user mode code (1 << 18)
            "li		    t0, (1 << 11) | (0 << 5) | (1 << 18)",
            "csrw	    mstatus, t0",

            // Enable the MMU (once we issue `mret`) and flush the cache
            "csrw        satp, {satp_val}",
            "sfence.vma",
            satp_val = in(reg) satp,
        );
        #[cfg(not(feature="gdb-load"))]
        core::arch::asm!(
            // Return to the address pointed to by $a4, which should be our return address minus remap offset
            "li          t0, 0x60000000",
        );
        #[cfg(feature="gdb-load")]
        core::arch::asm!(
            // When loading with GDB we don't use a VM offset so GDB is less confused
            "li          t0, 0x0",
        );
        core::arch::asm!(
            "sub         a4, ra, t0",
            "csrw        mepc, a4",

            // sp "shouldn't move" because the mapping will take RAM mapping as 1:1 for VA:PA

            // Issue the return, which will jump to $mepc in Supervisor mode
            "mret",
        );
    }
}

#[inline(never)] // correct behavior depends on RA being set.
pub fn to_user_mode() {
    unsafe {
        core::arch::asm!(
            "csrw   sepc, ra",
            "sret",
        );
    }
}

pub fn satp_test() {
    report_api(0x5a1d_0000);

    let mut coreuser = CSR::new(utra::coreuser::HW_COREUSER_BASE as *mut u32);
    // first, clear the ASID table to 0
    for asid in 0..512 {
        coreuser.wo(utra::coreuser::SET_ASID,
            coreuser.ms(utra::coreuser::SET_ASID_ASID, asid)
            | coreuser.ms(utra::coreuser::SET_ASID_TRUSTED, 0)
        );
    }

    // set some ASIDs to trusted. Values picked to somewhat challenge the decoding
    let trusted_asids = [1, 0x17, 0x18, 0x52, 0x57, 0x5A, 0x5F, 0x60, 0x61, 0x62, 0x116, 0x18F];
    for asid in trusted_asids {
        coreuser.wo(utra::coreuser::SET_ASID,
            coreuser.ms(utra::coreuser::SET_ASID_ASID, asid)
            | coreuser.ms(utra::coreuser::SET_ASID_TRUSTED, 1)
        );
    }
    // readback of table
    /* // this is too slow with the Daric UART model, but uncomment it later when running on real hardware
    for asid in 0..512 {
        coreuser.wfo(utra::coreuser::GET_ASID_ADDR_ASID, asid);
        report_api(
            coreuser.rf(utra::coreuser::GET_ASID_VALUE_VALUE) << 16 | asid
        );
    } */

    // setup window on our root page. Narrowly define it to *just* one page.
    coreuser.wfo(utra::coreuser::WINDOW_AH_PPN, (ROOT_PT_PA >> 12) as u32);
    coreuser.wfo(utra::coreuser::WINDOW_AL_PPN, (ROOT_PT_PA >> 12) as u32);

    // turn on the coreuser computation
    coreuser.wo(utra::coreuser::CONTROL,
        coreuser.ms(utra::coreuser::CONTROL_ASID, 1)
        | coreuser.ms(utra::coreuser::CONTROL_ENABLE, 1)
        | coreuser.ms(utra::coreuser::CONTROL_PPN_A, 1)
    );

    // turn off updates
    coreuser.wo(utra::coreuser::PROTECT, 1);

    // tries to "turn off" protect, but it should do nothing
    coreuser.wo(utra::coreuser::PROTECT, 0);
    // tamper with asid & ppn values, should not change result
    // add `2` to the trusted list (should not work)
    coreuser.wo(utra::coreuser::SET_ASID,
        coreuser.ms(utra::coreuser::SET_ASID_ASID, 2)
        | coreuser.ms(utra::coreuser::SET_ASID_TRUSTED, 1)
    );
    coreuser.wfo(utra::coreuser::WINDOW_AH_PPN, 0xface as u32);
    coreuser.wfo(utra::coreuser::WINDOW_AL_PPN, 0xdead as u32);
    // partial readback of table; `2` should not be trusted
    for asid in 0..4 {
        coreuser.wfo(utra::coreuser::GET_ASID_ADDR_ASID, asid);
        report_api(
    coreuser.rf(utra::coreuser::GET_ASID_VALUE_VALUE) << 16 | asid
        );
    }

    // now try changing the SATP around and see that the coreuser value updates
    // since we are in supervisor mode we can diddle with this at will, normally
    // user processes can't change this
    report_api(0x5a1d_0001);
    for asid in 0..512 {
        let satp: u32 =
        0x8000_0000
        | asid << 22
        | (ROOT_PT_PA as u32 >> 12);
        unsafe {
            core::arch::asm!(
                "csrw        satp, {satp_val}",
                "sfence.vma",
                satp_val = in(reg) satp,
            );
        }
    }
    // restore ASID to 1
    let satp: u32 =
    0x8000_0000
    | 1 << 22
    | (ROOT_PT_PA as u32 >> 12);
    unsafe {
        core::arch::asm!(
            "csrw        satp, {satp_val}",
            "sfence.vma",
            satp_val = in(reg) satp,
        );
    }

    // switch to user mode
    report_api(0x5a1d_0002);
    to_user_mode();

    // attempt to change ASID. This should be ignored or cause a trap, depending on the config of the device!
    // confirmed that without interrupts configured this has no effect; although it causes the following three
    // instructions to be ignored on the error.
    report_api(0x5a1d_0003);
    let satp: u32 =
    0x8000_0000
    | 4 << 22
    | (ROOT_PT_PA as u32 >> 12);
    unsafe {
        core::arch::asm!(
            "csrw        satp, {satp_val}",
            "sfence.vma",
            // this is interesting. any less than 3 `nop`s below cause the 0x5a1d_0002 value to
            // not appear in the final register, to varying degrees. it seems that the pipeline gets a bit
            // imprecise after this sequence...
            "nop",
            "nop",
            "nop",
            satp_val = in(reg) satp,
        );
    }
    report_api(0x5a1d_0004);

    report_api(0x5a1d_600d);
}