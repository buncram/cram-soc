use utralib::generated::*;

pub const PAGE_SIZE: usize = 4096;
const WORD_SIZE: usize = core::mem::size_of::<usize>();

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
const SRAM_PT_PA: usize = 0x6100_1000;
const CODE_PT_PA: usize = 0x6100_2000;
const CODE2_PT_PA: usize = 0x6100_3000;
const CSR_PT_PA: usize  = 0x6100_4000;
const PERI_PT_PA: usize = 0x6100_5000;
// exception handler pages. Mapped 1:1 PA:VA, so no explicit remapping needed as RAM area is already mapped.
const _SCRATCH_PAGE: usize = 0x6100_6000;
const _EXCEPTION_STACK_LIMIT: usize = 0x6100_7000; // the start of stack is this + 0x1000 & grows down
pub const PT_LIMIT: usize = 0x6100_8000;

// VAs
const CODE_VA: usize = 0x0000_0000;
const CSR_VA:  usize = 0x5800_0000;
const PERI_VA: usize = 0x4010_0000;
const SRAM_VA: usize = 0x6100_0000;

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
    // re-layout memory in virtual space
    // map ReRAM to v0x0000_0000
    // map SRAM  to v0x6100_0000 (1:1 map)
    // map CSR   to v0x5800_0000 (1:1 map)
    // map peri  to v0x4010_0000 (1:1 map)
    //
    // root page table is at p0x6100_0000 == v0x6100_0000
    let mut root_pt = unsafe { &mut *(ROOT_PT_PA as *mut PageTable) };
    let mut sram_pt = unsafe { &mut *(SRAM_PT_PA as *mut PageTable) };
    let mut code_pt = unsafe { &mut *(CODE_PT_PA as *mut PageTable) };
    let mut code2_pt = unsafe { &mut *(CODE2_PT_PA as *mut PageTable) };
    let mut csr_pt  = unsafe { &mut *(CSR_PT_PA  as *mut PageTable) };
    let mut peri_pt = unsafe { &mut *(PERI_PT_PA as *mut PageTable) };

    set_l1_pte(CODE_VA, CODE_PT_PA, &mut root_pt);
    set_l1_pte(CODE_VA + 0x40_0000, CODE2_PT_PA, &mut root_pt);
    set_l1_pte(CSR_VA, CSR_PT_PA, &mut root_pt);
    set_l1_pte(PERI_VA, PERI_PT_PA, &mut root_pt);
    set_l1_pte(SRAM_VA, SRAM_PT_PA, &mut root_pt); // L1 covers 16MiB, so SP_VA will cover all of SRAM

    // map code space. This is the only one that has a difference on VA->PA
    const CODE_LEN: usize = 0x65536;
    for offset in (0..CODE_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CODE_VA + offset, RERAM_PA + offset, &mut code_pt, FLG_X | FLG_R | FLG_U);
    }
    const SPI_OFFSET: usize = 0x50_0000;
    for offset in (SPI_OFFSET..SPI_OFFSET + CODE_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CODE_VA + offset, RERAM_PA + offset, &mut code2_pt, FLG_X | FLG_R | FLG_U);
    }

    // map sram. Mapping is 1:1, so we use _VA and _PA targets for both args
    const SRAM_LEN: usize = 65536;
    for offset in (0..SRAM_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(SRAM_VA + offset, SRAM_VA + offset, &mut sram_pt, FLG_W | FLG_R | FLG_U);
    }
    // map peripherals
    const CSR_LEN: usize = 0x2_0000;
    const PERI_LEN: usize = 0xA000;
    for offset in (0..CSR_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CSR_VA + offset, CSR_VA + offset, &mut csr_pt, FLG_W | FLG_R | FLG_U);
    }
    for offset in (0..PERI_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(PERI_VA + offset, PERI_VA + offset, &mut peri_pt, FLG_W | FLG_R | FLG_U);
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

            // Return to the address pointed to by $a4, which should be our return address minus remap offset
            "li          t0, 0x60000000",
            "sub         a4, ra, t0",
            "csrw        mepc, a4",

            // sp "shouldn't move" because the mapping will take RAM mapping as 1:1 for vA:PA

            // Issue the return, which will jump to $mepc in Supervisor mode
            "mret",
            satp_val = in(reg) satp,
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
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x5a1d_0000);

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
    for asid in 0..512 {
        coreuser.wfo(utra::coreuser::GET_ASID_ADDR_ASID, asid);
        report.wfo(utra::main::REPORT_REPORT,
            coreuser.rf(utra::coreuser::GET_ASID_VALUE_VALUE) << 16 | asid
        );
    }

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
        report.wfo(utra::main::REPORT_REPORT,
    coreuser.rf(utra::coreuser::GET_ASID_VALUE_VALUE) << 16 | asid
        );
    }

    // now try changing the SATP around and see that the coreuser value updates
    // since we are in supervisor mode we can diddle with this at will, normally
    // user processes can't change this
    report.wfo(utra::main::REPORT_REPORT, 0x5a1d_0001);
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
    report.wfo(utra::main::REPORT_REPORT, 0x5a1d_0002);
    to_user_mode();

    // attempt to change ASID. This should be ignored or cause a trap, depending on the config of the device!
    // confirmed that without interrupts configured this has no effect; although it causes the following three
    // instructions to be ignored on the error.
    report.wfo(utra::main::REPORT_REPORT, 0x5a1d_0003);
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
    report.wfo(utra::main::REPORT_REPORT, 0x5a1d_0004);

    report.wfo(utra::main::REPORT_REPORT, 0x5a1d_600d);
}