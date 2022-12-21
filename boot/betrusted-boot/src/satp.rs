pub const PAGE_SIZE: usize = 4096;
const WORD_SIZE: usize = core::mem::size_of::<usize>();

const FLG_VALID: usize = 0x1;
const FLG_X: usize = 0x8;
const FLG_W: usize = 0x4;
const FLG_R: usize = 0x2;
const FLG_U: usize = 0x10;
const FLG_A: usize = 0x40;
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
const CSR_PT_PA: usize  = 0x6100_3000;
const PERI_PT_PA: usize = 0x6100_4000;

// VAs
const CODE_VA: usize = 0x0000_0000;
const CSR_VA:  usize = 0x5800_0000;
const PERI_VA: usize = 0x4000_0000;
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

#[inline(never)] // correct behavior depends on RA being set.
pub fn satp_setup() {
    // re-layout memory in virtual space
    // map ReRAM to v0x0000_0000
    // map SRAM  to v0x6100_0000 (1:1 map)
    // map CSR   to v0x5800_0000 (1:1 map)
    // map peri  to v0x4000_0000 (1:1 map)
    //
    // root page table is at p0x6100_0000 == v0x6100_0000
    let mut root_pt = unsafe { &mut *(ROOT_PT_PA as *mut PageTable) };
    let mut sram_pt = unsafe { &mut *(SRAM_PT_PA as *mut PageTable) };
    let mut code_pt = unsafe { &mut *(CODE_PT_PA as *mut PageTable) };
    let mut csr_pt  = unsafe { &mut *(CSR_PT_PA  as *mut PageTable) };
    let mut peri_pt = unsafe { &mut *(PERI_PT_PA as *mut PageTable) };

    set_l1_pte(CODE_VA, CODE_PT_PA, &mut root_pt);
    set_l1_pte(CSR_VA, CSR_PT_PA, &mut root_pt);
    set_l1_pte(PERI_VA, PERI_PT_PA, &mut root_pt);
    set_l1_pte(SRAM_VA, SRAM_PT_PA, &mut root_pt); // L1 covers 16MiB, so SP_VA will cover all of SRAM

    // map code space. This is the only one that has a difference on VA->PA
    const CODE_LEN: usize = 65536;
    for offset in (0..CODE_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CODE_VA + offset, RERAM_PA + offset, &mut code_pt, FLG_X | FLG_R);
    }
    // map sram. Mapping is 1:1, so we use _VA and _PA targets for both args
    const SRAM_LEN: usize = 65536;
    for offset in (0..SRAM_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(SRAM_VA + offset, SRAM_VA + offset, &mut sram_pt, FLG_W | FLG_R);
    }
    // map peripherals
    const CSR_LEN: usize = 0xA000;
    const PERI_LEN: usize = 0xA000;
    for offset in (0..CSR_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(CSR_VA + offset, CSR_VA + offset, &mut csr_pt, FLG_W | FLG_R);
    }
    for offset in (0..PERI_LEN).step_by(PAGE_SIZE) {
        set_l2_pte(PERI_VA + offset, PERI_VA + offset, &mut peri_pt, FLG_W | FLG_R);
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
            // Disable interrupts (0 << 5)
            "li		    t0, (1 << 11) | (0 << 5)",
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
