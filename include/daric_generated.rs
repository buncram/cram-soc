
#![allow(dead_code)]
use core::convert::TryInto;
#[cfg(feature="std")]
use core::sync::atomic::AtomicPtr;
#[cfg(feature="std")]
use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub struct Register {
    /// Offset of this register within this CSR
    offset: usize,
    /// Mask of SVD-specified bits for the register
    mask: usize,
}
impl Register {
    pub const fn new(offset: usize, mask: usize) -> Register {
        Register { offset, mask }
    }
    pub const fn offset(&self) -> usize { self.offset }
    pub const fn mask(&self) -> usize { self.mask }
}
#[derive(Debug, Copy, Clone)]
pub struct Field {
    /// A bitmask we use to AND to the value, unshifted.
    /// E.g. for a width of `3` bits, this mask would be 0b111.
    mask: usize,
    /// Offset of the first bit in this field
    offset: usize,
    /// A copy of the register address that this field
    /// is a member of. Ideally this is optimized out by the
    /// compiler.
    register: Register,
}
impl Field {
    /// Define a new CSR field with the given width at a specified
    /// offset from the start of the register.
    pub const fn new(width: usize, offset: usize, register: Register) -> Field {
        let mask = if width < 32 { (1 << width) - 1 } else {0xFFFF_FFFF};
        Field {
            mask,
            offset,
            register,
        }
    }
    pub const fn offset(&self) -> usize { self.offset }
    pub const fn mask(&self) -> usize { self.mask }
}
#[derive(Debug, Copy, Clone)]
pub struct CSR<T> {
    base: *mut T,
}
impl<T> CSR<T>
where
    T: core::convert::TryFrom<usize> + core::convert::TryInto<usize> + core::default::Default,
{
    pub fn new(base: *mut T) -> Self {
        CSR { base }
    }
    /// Retrieve the raw pointer used as the base of the CSR. This is unsafe because the copied
    /// value can be used to do all kinds of awful shared mutable operations (like creating
    /// another CSR accessor owned by another thread). However, sometimes this is unavoidable
    /// because hardware is in fact shared mutable state.
    pub unsafe fn base(&self) -> *mut T {
        self.base
    }
    /// Read the contents of this register
    pub fn r(&self, reg: Register) -> T {
        // prevent re-ordering
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base) };
        unsafe { usize_base.add(reg.offset).read_volatile() }
            .try_into()
            .unwrap_or_default()
    }
    /// Read a field from this CSR
    pub fn rf(&self, field: Field) -> T {
        // prevent re-ordering
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base) };
        ((unsafe { usize_base.add(field.register.offset).read_volatile() } >> field.offset)
            & field.mask)
            .try_into()
            .unwrap_or_default()
    }
    /// Read-modify-write a given field in this CSR
    pub fn rmwf(&mut self, field: Field, value: T) {
        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base) };
        let value_as_usize: usize = value.try_into().unwrap_or_default() << field.offset;
        let previous =
            unsafe { usize_base.add(field.register.offset).read_volatile() } & !(field.mask << field.offset);
        unsafe {
            usize_base
                .add(field.register.offset)
                .write_volatile(previous | value_as_usize)
        };
        // prevent re-ordering
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    /// Write a given field without reading it first
    pub fn wfo(&mut self, field: Field, value: T) {
        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base) };
        let value_as_usize: usize = (value.try_into().unwrap_or_default() & field.mask) << field.offset;
        unsafe {
            usize_base
                .add(field.register.offset)
                .write_volatile(value_as_usize)
        };
        // Ensure the compiler doesn't re-order the write.
        // We use `SeqCst`, because `Acquire` only prevents later accesses from being reordered before
        // *reads*, but this method only *writes* to the locations.
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    /// Write the entire contents of a register without reading it first
    pub fn wo(&mut self, reg: Register, value: T) {
        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base) };
        let value_as_usize: usize = value.try_into().unwrap_or_default();
        unsafe { usize_base.add(reg.offset).write_volatile(value_as_usize) };
        // Ensure the compiler doesn't re-order the write.
        // We use `SeqCst`, because `Acquire` only prevents later accesses from being reordered before
        // *reads*, but this method only *writes* to the locations.
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    /// Zero a field from a provided value
    pub fn zf(&self, field: Field, value: T) -> T {
        let value_as_usize: usize = value.try_into().unwrap_or_default();
        (value_as_usize & !(field.mask << field.offset))
            .try_into()
            .unwrap_or_default()
    }
    /// Shift & mask a value to its final field position
    pub fn ms(&self, field: Field, value: T) -> T {
        let value_as_usize: usize = value.try_into().unwrap_or_default();
        ((value_as_usize & field.mask) << field.offset)
            .try_into()
            .unwrap_or_default()
    }
}

#[derive(Debug)]
#[cfg(feature="std")]
pub struct AtomicCsr<T> {
    base: Arc::<AtomicPtr<T>>,
}
#[cfg(feature="std")]
impl<T> AtomicCsr<T>
where
    T: core::convert::TryFrom<usize> + core::convert::TryInto<usize> + core::default::Default,
{
    /// AtomicCsr wraps the CSR in an Arc + AtomicPtr, so that write operations don't require
    /// a mutable reference. This allows us to stick CSR accesses into APIs that require
    /// non-mutable references to hardware state (such as certain "standardized" USB APIs).
    /// Hiding the fact that you're tweaking hardware registers behind Arc/AtomicPtr seems a little
    /// scary, but, it does make for nicer Rust semantics.
    pub fn new(base: *mut T) -> Self {
        AtomicCsr {
            base: Arc::new(AtomicPtr::new(base))
        }
    }
    pub fn clone(&self) -> Self {
        AtomicCsr {
            base: self.base.clone()
        }
    }
    /// Read the contents of this register
    pub fn r(&self, reg: Register) -> T {
        // prevent re-ordering
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base.load(core::sync::atomic::Ordering::SeqCst)) };
        unsafe { usize_base.add(reg.offset).read_volatile() }
            .try_into()
            .unwrap_or_default()
    }
    /// Read a field from this CSR
    pub fn rf(&self, field: Field) -> T {
        // prevent re-ordering
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base.load(core::sync::atomic::Ordering::SeqCst)) };
        ((unsafe { usize_base.add(field.register.offset).read_volatile() } >> field.offset)
            & field.mask)
            .try_into()
            .unwrap_or_default()
    }
    /// Read-modify-write a given field in this CSR
    pub fn rmwf(&self, field: Field, value: T) {
        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base.load(core::sync::atomic::Ordering::SeqCst)) };
        let value_as_usize: usize = value.try_into().unwrap_or_default() << field.offset;
        let previous =
            unsafe { usize_base.add(field.register.offset).read_volatile() } & !(field.mask << field.offset);
        unsafe {
            usize_base
                .add(field.register.offset)
                .write_volatile(previous | value_as_usize)
        };
        // prevent re-ordering
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    /// Write a given field without reading it first
    pub fn wfo(&self, field: Field, value: T) {
        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base.load(core::sync::atomic::Ordering::SeqCst)) };
        let value_as_usize: usize = (value.try_into().unwrap_or_default() & field.mask) << field.offset;
        unsafe {
            usize_base
                .add(field.register.offset)
                .write_volatile(value_as_usize)
        };
        // Ensure the compiler doesn't re-order the write.
        // We use `SeqCst`, because `Acquire` only prevents later accesses from being reordered before
        // *reads*, but this method only *writes* to the locations.
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    /// Write the entire contents of a register without reading it first
    pub fn wo(&self, reg: Register, value: T) {
        let usize_base: *mut usize = unsafe { core::mem::transmute(self.base.load(core::sync::atomic::Ordering::SeqCst)) };
        let value_as_usize: usize = value.try_into().unwrap_or_default();
        unsafe { usize_base.add(reg.offset).write_volatile(value_as_usize) };
        // Ensure the compiler doesn't re-order the write.
        // We use `SeqCst`, because `Acquire` only prevents later accesses from being reordered before
        // *reads*, but this method only *writes* to the locations.
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    /// Zero a field from a provided value
    pub fn zf(&self, field: Field, value: T) -> T {
        let value_as_usize: usize = value.try_into().unwrap_or_default();
        (value_as_usize & !(field.mask << field.offset))
            .try_into()
            .unwrap_or_default()
    }
    /// Shift & mask a value to its final field position
    pub fn ms(&self, field: Field, value: T) -> T {
        let value_as_usize: usize = value.try_into().unwrap_or_default();
        ((value_as_usize & field.mask) << field.offset)
            .try_into()
            .unwrap_or_default()
    }
}
// Physical base addresses of memory regions
pub const HW_SCE_MEM:     usize = 0x40028000;
pub const HW_SCE_MEM_LEN: usize = 32768;
pub const HW_SYSCTRL_MEM:     usize = 0x40040000;
pub const HW_SYSCTRL_MEM_LEN: usize = 65536;
pub const HW_IFSUB_MEM:     usize = 0x50120000;
pub const HW_IFSUB_MEM_LEN: usize = 65536;
pub const HW_CORESUB_MEM:     usize = 0x40000000;
pub const HW_CORESUB_MEM_LEN: usize = 65536;
pub const HW_SECSUB_MEM:     usize = 0x40050000;
pub const HW_SECSUB_MEM_LEN: usize = 65536;
pub const HW_SEG_LKEY_MEM:     usize = 0x40020000;
pub const HW_SEG_LKEY_MEM_LEN: usize = 256;
pub const HW_SEG_KEY_MEM:     usize = 0x40020100;
pub const HW_SEG_KEY_MEM_LEN: usize = 256;
pub const HW_SEG_SKEY_MEM:     usize = 0x40020200;
pub const HW_SEG_SKEY_MEM_LEN: usize = 256;
pub const HW_SEG_SCRT_MEM:     usize = 0x40020300;
pub const HW_SEG_SCRT_MEM_LEN: usize = 256;
pub const HW_SEG_MSG_MEM:     usize = 0x40020400;
pub const HW_SEG_MSG_MEM_LEN: usize = 512;
pub const HW_SEG_HOUT_MEM:     usize = 0x40020600;
pub const HW_SEG_HOUT_MEM_LEN: usize = 256;
pub const HW_SEG_SOB_MEM:     usize = 0x40020700;
pub const HW_SEG_SOB_MEM_LEN: usize = 256;
pub const HW_SEG_PCON_MEM:     usize = 0x40020800;
pub const HW_SEG_PCON_MEM_LEN: usize = 256;
pub const HW_SEG_PKB_MEM:     usize = 0x40020900;
pub const HW_SEG_PKB_MEM_LEN: usize = 256;
pub const HW_SEG_PIB_MEM:     usize = 0x40020a00;
pub const HW_SEG_PIB_MEM_LEN: usize = 1024;
pub const HW_SEG_PSIB_MEM:     usize = 0x40020e00;
pub const HW_SEG_PSIB_MEM_LEN: usize = 1024;
pub const HW_SEG_POB_MEM:     usize = 0x40021200;
pub const HW_SEG_POB_MEM_LEN: usize = 1024;
pub const HW_SEG_PSOB_MEM:     usize = 0x40021600;
pub const HW_SEG_PSOB_MEM_LEN: usize = 1024;
pub const HW_SEG_AKEY_MEM:     usize = 0x40021a00;
pub const HW_SEG_AKEY_MEM_LEN: usize = 256;
pub const HW_SEG_AIB_MEM:     usize = 0x40021b00;
pub const HW_SEG_AIB_MEM_LEN: usize = 256;
pub const HW_SEG_AOB_MEM:     usize = 0x40021c00;
pub const HW_SEG_AOB_MEM_LEN: usize = 256;
pub const HW_SEG_RNGA_MEM:     usize = 0x40021d00;
pub const HW_SEG_RNGA_MEM_LEN: usize = 1024;
pub const HW_SEG_RNGB_MEM:     usize = 0x40022100;
pub const HW_SEG_RNGB_MEM_LEN: usize = 1024;

// Physical base addresses of registers
pub const HW_AES_BASE :   usize = 0x4002d000;
pub const HW_COMBOHASH_BASE :   usize = 0x4002b000;
pub const HW_PKE_BASE :   usize = 0x4002c000;
pub const HW_SCEDMA_BASE :   usize = 0x40029000;
pub const HW_SCE_GLBSFR_BASE :   usize = 0x40028000;
pub const HW_TRNG_BASE :   usize = 0x4002e000;
pub const HW_ALU_BASE :   usize = 0x4002f000;
pub const HW_DUART_BASE :   usize = 0x40042000;
pub const HW_WDG_INTF_BASE :   usize = 0x40041000;
pub const HW_TIMER_INTF_BASE :   usize = 0x40043000;
pub const HW_EVC_BASE :   usize = 0x40044000;
pub const HW_SYSCTRL_BASE :   usize = 0x40040000;
pub const HW_APB_THRU_BASE :   usize = 0x50122000;
pub const HW_IOX_BASE :   usize = 0x5012f000;
pub const HW_PWM_BASE :   usize = 0x50120000;
pub const HW_SDDC_BASE :   usize = 0x50121000;
pub const HW_MDMA_BASE :   usize = 0x40002000;
pub const HW_QFC_BASE :   usize = 0x40000000;
pub const HW_PL230_BASE :   usize = 0x40001000;
pub const HW_GLUECHAIN_BASE :   usize = 0x40054000;
pub const HW_MESH_BASE :   usize = 0x40052000;
pub const HW_SENSORC_BASE :   usize = 0x40053000;


pub mod utra {

    pub mod aes {
        pub const AES_NUMREGS: usize = 16;

        pub const SFR_CRFUNC: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_CRFUNC_SFR_CRFUNC: crate::Field = crate::Field::new(8, 0, SFR_CRFUNC);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const SFR_SRMFSM: crate::Register = crate::Register::new(2, 0xff);
        pub const SFR_SRMFSM_SFR_SRMFSM: crate::Field = crate::Field::new(8, 0, SFR_SRMFSM);

        pub const SFR_FR: crate::Register = crate::Register::new(3, 0xf);
        pub const SFR_FR_MFSM_DONE: crate::Field = crate::Field::new(1, 0, SFR_FR);
        pub const SFR_FR_ACORE_DONE: crate::Field = crate::Field::new(1, 1, SFR_FR);
        pub const SFR_FR_CHNLO_DONE: crate::Field = crate::Field::new(1, 2, SFR_FR);
        pub const SFR_FR_CHNLI_DONE: crate::Field = crate::Field::new(1, 3, SFR_FR);

        pub const SFR_OPT: crate::Register = crate::Register::new(4, 0x1ff);
        pub const SFR_OPT_OPT_KLEN0: crate::Field = crate::Field::new(4, 0, SFR_OPT);
        pub const SFR_OPT_OPT_MODE0: crate::Field = crate::Field::new(4, 4, SFR_OPT);
        pub const SFR_OPT_OPT_IFSTART0: crate::Field = crate::Field::new(1, 8, SFR_OPT);

        pub const SFR_OPT1: crate::Register = crate::Register::new(5, 0xffff);
        pub const SFR_OPT1_SFR_OPT1: crate::Field = crate::Field::new(16, 0, SFR_OPT1);

        pub const RESERVED6: crate::Register = crate::Register::new(6, 0x1);
        pub const RESERVED6_RESERVED6: crate::Field = crate::Field::new(1, 0, RESERVED6);

        pub const RESERVED7: crate::Register = crate::Register::new(7, 0x1);
        pub const RESERVED7_RESERVED7: crate::Field = crate::Field::new(1, 0, RESERVED7);

        pub const RESERVED8: crate::Register = crate::Register::new(8, 0x1);
        pub const RESERVED8_RESERVED8: crate::Field = crate::Field::new(1, 0, RESERVED8);

        pub const RESERVED9: crate::Register = crate::Register::new(9, 0x1);
        pub const RESERVED9_RESERVED9: crate::Field = crate::Field::new(1, 0, RESERVED9);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const RESERVED11: crate::Register = crate::Register::new(11, 0x1);
        pub const RESERVED11_RESERVED11: crate::Field = crate::Field::new(1, 0, RESERVED11);

        pub const SFR_SEGPTR_PTRID_IV: crate::Register = crate::Register::new(12, 0xfff);
        pub const SFR_SEGPTR_PTRID_IV_PTRID_IV: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_IV);

        pub const SFR_SEGPTR_PTRID_AKEY: crate::Register = crate::Register::new(13, 0xfff);
        pub const SFR_SEGPTR_PTRID_AKEY_PTRID_AKEY: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_AKEY);

        pub const SFR_SEGPTR_PTRID_AIB: crate::Register = crate::Register::new(14, 0xfff);
        pub const SFR_SEGPTR_PTRID_AIB_PTRID_AIB: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_AIB);

        pub const SFR_SEGPTR_PTRID_AOB: crate::Register = crate::Register::new(15, 0xfff);
        pub const SFR_SEGPTR_PTRID_AOB_PTRID_AOB: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_AOB);

        pub const HW_AES_BASE: usize = 0x4002d000;
    }

    pub mod combohash {
        pub const COMBOHASH_NUMREGS: usize = 15;

        pub const SFR_CRFUNC: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_CRFUNC_CR_FUNC: crate::Field = crate::Field::new(8, 0, SFR_CRFUNC);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const SFR_SRMFSM: crate::Register = crate::Register::new(2, 0xff);
        pub const SFR_SRMFSM_MFSM: crate::Field = crate::Field::new(8, 0, SFR_SRMFSM);

        pub const SFR_FR: crate::Register = crate::Register::new(3, 0xf);
        pub const SFR_FR_MFSM_DONE: crate::Field = crate::Field::new(1, 0, SFR_FR);
        pub const SFR_FR_HASH_DONE: crate::Field = crate::Field::new(1, 1, SFR_FR);
        pub const SFR_FR_CHNLO_DONE: crate::Field = crate::Field::new(1, 2, SFR_FR);
        pub const SFR_FR_CHNLI_DONE: crate::Field = crate::Field::new(1, 3, SFR_FR);

        pub const SFR_OPT1: crate::Register = crate::Register::new(4, 0xffff);
        pub const SFR_OPT1_CR_OPT_HASHCNT: crate::Field = crate::Field::new(16, 0, SFR_OPT1);

        pub const SFR_OPT2: crate::Register = crate::Register::new(5, 0x7);
        pub const SFR_OPT2_CR_OPT_SCRTCHK: crate::Field = crate::Field::new(1, 0, SFR_OPT2);
        pub const SFR_OPT2_CR_OPT_IFSOB: crate::Field = crate::Field::new(1, 1, SFR_OPT2);
        pub const SFR_OPT2_CR_OPT_IFSTART: crate::Field = crate::Field::new(1, 2, SFR_OPT2);

        pub const RESERVED6: crate::Register = crate::Register::new(6, 0x1);
        pub const RESERVED6_RESERVED6: crate::Field = crate::Field::new(1, 0, RESERVED6);

        pub const RESERVED7: crate::Register = crate::Register::new(7, 0x1);
        pub const RESERVED7_RESERVED7: crate::Field = crate::Field::new(1, 0, RESERVED7);

        pub const SFR_SEGPTR_SEGID_LKEY: crate::Register = crate::Register::new(8, 0xfff);
        pub const SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_LKEY);

        pub const SFR_SEGPTR_SEGID_KEY: crate::Register = crate::Register::new(9, 0xfff);
        pub const SFR_SEGPTR_SEGID_KEY_SEGID_KEY: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_KEY);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const SFR_SEGPTR_SEGID_SCRT: crate::Register = crate::Register::new(11, 0xfff);
        pub const SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_SCRT);

        pub const SFR_SEGPTR_SEGID_MSG: crate::Register = crate::Register::new(12, 0xfff);
        pub const SFR_SEGPTR_SEGID_MSG_SEGID_MSG: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_MSG);

        pub const SFR_SEGPTR_SEGID_HOUT: crate::Register = crate::Register::new(13, 0xfff);
        pub const SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_HOUT);

        pub const SFR_SEGPTR_SEGID_SOB: crate::Register = crate::Register::new(14, 0xfff);
        pub const SFR_SEGPTR_SEGID_SOB_SEGID_SOB: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_SOB);

        pub const HW_COMBOHASH_BASE: usize = 0x4002b000;
    }

    pub mod pke {
        pub const PKE_NUMREGS: usize = 17;

        pub const SFR_CRFUNC: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_CRFUNC_SFR_CRFUNC: crate::Field = crate::Field::new(8, 0, SFR_CRFUNC);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const SFR_SRMFSM: crate::Register = crate::Register::new(2, 0x1ff);
        pub const SFR_SRMFSM_MFSM: crate::Field = crate::Field::new(8, 0, SFR_SRMFSM);
        pub const SFR_SRMFSM_MODINVREADY: crate::Field = crate::Field::new(1, 8, SFR_SRMFSM);

        pub const SFR_FR: crate::Register = crate::Register::new(3, 0x1f);
        pub const SFR_FR_MFSM_DONE: crate::Field = crate::Field::new(1, 0, SFR_FR);
        pub const SFR_FR_PCORE_DONE: crate::Field = crate::Field::new(1, 1, SFR_FR);
        pub const SFR_FR_CHNLO_DONE: crate::Field = crate::Field::new(1, 2, SFR_FR);
        pub const SFR_FR_CHNLI_DONE: crate::Field = crate::Field::new(1, 3, SFR_FR);
        pub const SFR_FR_CHNLX_DONE: crate::Field = crate::Field::new(1, 4, SFR_FR);

        pub const SFR_OPTNW: crate::Register = crate::Register::new(4, 0x1fff);
        pub const SFR_OPTNW_SFR_OPTNW: crate::Field = crate::Field::new(13, 0, SFR_OPTNW);

        pub const SFR_OPTEW: crate::Register = crate::Register::new(5, 0x1fff);
        pub const SFR_OPTEW_SFR_OPTEW: crate::Field = crate::Field::new(13, 0, SFR_OPTEW);

        pub const RESERVED6: crate::Register = crate::Register::new(6, 0x1);
        pub const RESERVED6_RESERVED6: crate::Field = crate::Field::new(1, 0, RESERVED6);

        pub const RESERVED7: crate::Register = crate::Register::new(7, 0x1);
        pub const RESERVED7_RESERVED7: crate::Field = crate::Field::new(1, 0, RESERVED7);

        pub const SFR_OPTMASK: crate::Register = crate::Register::new(8, 0xffff);
        pub const SFR_OPTMASK_SFR_OPTMASK: crate::Field = crate::Field::new(16, 0, SFR_OPTMASK);

        pub const RESERVED9: crate::Register = crate::Register::new(9, 0x1);
        pub const RESERVED9_RESERVED9: crate::Field = crate::Field::new(1, 0, RESERVED9);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const RESERVED11: crate::Register = crate::Register::new(11, 0x1);
        pub const RESERVED11_RESERVED11: crate::Field = crate::Field::new(1, 0, RESERVED11);

        pub const SFR_SEGPTR_PTRID_PCON: crate::Register = crate::Register::new(12, 0xfff);
        pub const SFR_SEGPTR_PTRID_PCON_PTRID_PCON: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_PCON);

        pub const SFR_SEGPTR_PTRID_PIB0: crate::Register = crate::Register::new(13, 0xfff);
        pub const SFR_SEGPTR_PTRID_PIB0_PTRID_PIB0: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_PIB0);

        pub const SFR_SEGPTR_PTRID_PIB1: crate::Register = crate::Register::new(14, 0xfff);
        pub const SFR_SEGPTR_PTRID_PIB1_PTRID_PIB1: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_PIB1);

        pub const SFR_SEGPTR_PTRID_PKB: crate::Register = crate::Register::new(15, 0xfff);
        pub const SFR_SEGPTR_PTRID_PKB_PTRID_PKB: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_PKB);

        pub const SFR_SEGPTR_PTRID_POB: crate::Register = crate::Register::new(16, 0xfff);
        pub const SFR_SEGPTR_PTRID_POB_PTRID_POB: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_PTRID_POB);

        pub const HW_PKE_BASE: usize = 0x4002c000;
    }

    pub mod scedma {
        pub const SCEDMA_NUMREGS: usize = 25;

        pub const SFR_SCHSTART_AR: crate::Register = crate::Register::new(0, 0xffffffff);
        pub const SFR_SCHSTART_AR_SFR_SCHSTART_AR: crate::Field = crate::Field::new(32, 0, SFR_SCHSTART_AR);

        pub const RESERVED1: crate::Register = crate::Register::new(1, 0x1);
        pub const RESERVED1_RESERVED1: crate::Field = crate::Field::new(1, 0, RESERVED1);

        pub const RESERVED2: crate::Register = crate::Register::new(2, 0x1);
        pub const RESERVED2_RESERVED2: crate::Field = crate::Field::new(1, 0, RESERVED2);

        pub const RESERVED3: crate::Register = crate::Register::new(3, 0x1);
        pub const RESERVED3_RESERVED3: crate::Field = crate::Field::new(1, 0, RESERVED3);

        pub const SFR_XCH_FUNC: crate::Register = crate::Register::new(4, 0x1);
        pub const SFR_XCH_FUNC_SFR_XCH_FUNC: crate::Field = crate::Field::new(1, 0, SFR_XCH_FUNC);

        pub const SFR_XCH_OPT: crate::Register = crate::Register::new(5, 0xff);
        pub const SFR_XCH_OPT_SFR_XCH_OPT: crate::Field = crate::Field::new(8, 0, SFR_XCH_OPT);

        pub const SFR_XCH_AXSTART: crate::Register = crate::Register::new(6, 0xffffffff);
        pub const SFR_XCH_AXSTART_SFR_XCH_AXSTART: crate::Field = crate::Field::new(32, 0, SFR_XCH_AXSTART);

        pub const SFR_XCH_SEGID: crate::Register = crate::Register::new(7, 0xff);
        pub const SFR_XCH_SEGID_SFR_XCH_SEGID: crate::Field = crate::Field::new(8, 0, SFR_XCH_SEGID);

        pub const SFR_XCH_SEGSTART: crate::Register = crate::Register::new(8, 0xfff);
        pub const SFR_XCH_SEGSTART_XCHCR_SEGSTART: crate::Field = crate::Field::new(12, 0, SFR_XCH_SEGSTART);

        pub const SFR_XCH_TRANSIZE: crate::Register = crate::Register::new(9, 0x3fffffff);
        pub const SFR_XCH_TRANSIZE_XCHCR_TRANSIZE: crate::Field = crate::Field::new(30, 0, SFR_XCH_TRANSIZE);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const RESERVED11: crate::Register = crate::Register::new(11, 0x1);
        pub const RESERVED11_RESERVED11: crate::Field = crate::Field::new(1, 0, RESERVED11);

        pub const SFR_SCH_FUNC: crate::Register = crate::Register::new(12, 0x1);
        pub const SFR_SCH_FUNC_SFR_SCH_FUNC: crate::Field = crate::Field::new(1, 0, SFR_SCH_FUNC);

        pub const SFR_SCH_OPT: crate::Register = crate::Register::new(13, 0xff);
        pub const SFR_SCH_OPT_SFR_SCH_OPT: crate::Field = crate::Field::new(8, 0, SFR_SCH_OPT);

        pub const SFR_SCH_AXSTART: crate::Register = crate::Register::new(14, 0xffffffff);
        pub const SFR_SCH_AXSTART_SFR_SCH_AXSTART: crate::Field = crate::Field::new(32, 0, SFR_SCH_AXSTART);

        pub const SFR_SCH_SEGID: crate::Register = crate::Register::new(15, 0xff);
        pub const SFR_SCH_SEGID_SFR_SCH_SEGID: crate::Field = crate::Field::new(8, 0, SFR_SCH_SEGID);

        pub const SFR_SCH_SEGSTART: crate::Register = crate::Register::new(16, 0xfff);
        pub const SFR_SCH_SEGSTART_SCHCR_SEGSTART: crate::Field = crate::Field::new(12, 0, SFR_SCH_SEGSTART);

        pub const SFR_SCH_TRANSIZE: crate::Register = crate::Register::new(17, 0x3fffffff);
        pub const SFR_SCH_TRANSIZE_SCHCR_TRANSIZE: crate::Field = crate::Field::new(30, 0, SFR_SCH_TRANSIZE);

        pub const RESERVED18: crate::Register = crate::Register::new(18, 0x1);
        pub const RESERVED18_RESERVED18: crate::Field = crate::Field::new(1, 0, RESERVED18);

        pub const RESERVED19: crate::Register = crate::Register::new(19, 0x1);
        pub const RESERVED19_RESERVED19: crate::Field = crate::Field::new(1, 0, RESERVED19);

        pub const SFR_ICH_OPT: crate::Register = crate::Register::new(20, 0xf);
        pub const SFR_ICH_OPT_SFR_ICH_OPT: crate::Field = crate::Field::new(4, 0, SFR_ICH_OPT);

        pub const SFR_ICH_SEGID: crate::Register = crate::Register::new(21, 0xffff);
        pub const SFR_ICH_SEGID_SFR_ICH_SEGID: crate::Field = crate::Field::new(16, 0, SFR_ICH_SEGID);

        pub const SFR_ICH_RPSTART: crate::Register = crate::Register::new(22, 0xfff);
        pub const SFR_ICH_RPSTART_ICHCR_RPSTART: crate::Field = crate::Field::new(12, 0, SFR_ICH_RPSTART);

        pub const SFR_ICH_WPSTART: crate::Register = crate::Register::new(23, 0xfff);
        pub const SFR_ICH_WPSTART_ICHCR_WPSTART: crate::Field = crate::Field::new(12, 0, SFR_ICH_WPSTART);

        pub const SFR_ICH_TRANSIZE: crate::Register = crate::Register::new(24, 0xfff);
        pub const SFR_ICH_TRANSIZE_ICHCR_TRANSIZE: crate::Field = crate::Field::new(12, 0, SFR_ICH_TRANSIZE);

        pub const HW_SCEDMA_BASE: usize = 0x40029000;
    }

    pub mod sce_glbsfr {
        pub const SCE_GLBSFR_NUMREGS: usize = 22;

        pub const SFR_SCEMODE: crate::Register = crate::Register::new(0, 0x3);
        pub const SFR_SCEMODE_CR_SCEMODE: crate::Field = crate::Field::new(2, 0, SFR_SCEMODE);

        pub const SFR_SUBEN: crate::Register = crate::Register::new(1, 0xffff);
        pub const SFR_SUBEN_CR_SUBEN: crate::Field = crate::Field::new(16, 0, SFR_SUBEN);

        pub const SFR_AHBS: crate::Register = crate::Register::new(2, 0x1f);
        pub const SFR_AHBS_CR_AHBSOPT: crate::Field = crate::Field::new(5, 0, SFR_AHBS);

        pub const RESERVED3: crate::Register = crate::Register::new(3, 0x1);
        pub const RESERVED3_RESERVED3: crate::Field = crate::Field::new(1, 0, RESERVED3);

        pub const SFR_SRBUSY: crate::Register = crate::Register::new(4, 0xffff);
        pub const SFR_SRBUSY_SR_BUSY: crate::Field = crate::Field::new(16, 0, SFR_SRBUSY);

        pub const SFR_FRDONE: crate::Register = crate::Register::new(5, 0xffff);
        pub const SFR_FRDONE_FR_DONE: crate::Field = crate::Field::new(16, 0, SFR_FRDONE);

        pub const SFR_FRERR: crate::Register = crate::Register::new(6, 0xffff);
        pub const SFR_FRERR_FR_ERR: crate::Field = crate::Field::new(16, 0, SFR_FRERR);

        pub const SFR_ARCLR: crate::Register = crate::Register::new(7, 0xffffffff);
        pub const SFR_ARCLR_AR_CLRRAM: crate::Field = crate::Field::new(32, 0, SFR_ARCLR);

        pub const SFR_TICKCYC: crate::Register = crate::Register::new(8, 0xff);
        pub const SFR_TICKCYC_SFR_TICKCYC: crate::Field = crate::Field::new(8, 0, SFR_TICKCYC);

        pub const SFR_TICKCNT: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_TICKCNT_SFR_TICKCNT: crate::Field = crate::Field::new(32, 0, SFR_TICKCNT);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const RESERVED11: crate::Register = crate::Register::new(11, 0x1);
        pub const RESERVED11_RESERVED11: crate::Field = crate::Field::new(1, 0, RESERVED11);

        pub const SFR_FFEN: crate::Register = crate::Register::new(12, 0x3f);
        pub const SFR_FFEN_CR_FFEN: crate::Field = crate::Field::new(6, 0, SFR_FFEN);

        pub const SFR_FFCLR: crate::Register = crate::Register::new(13, 0xffffffff);
        pub const SFR_FFCLR_AR_FFCLR: crate::Field = crate::Field::new(32, 0, SFR_FFCLR);

        pub const RESERVED14: crate::Register = crate::Register::new(14, 0x1);
        pub const RESERVED14_RESERVED14: crate::Field = crate::Field::new(1, 0, RESERVED14);

        pub const RESERVED15: crate::Register = crate::Register::new(15, 0x1);
        pub const RESERVED15_RESERVED15: crate::Field = crate::Field::new(1, 0, RESERVED15);

        pub const SFR_FFCNT_SR_FF0: crate::Register = crate::Register::new(16, 0xffff);
        pub const SFR_FFCNT_SR_FF0_SR_FF0: crate::Field = crate::Field::new(16, 0, SFR_FFCNT_SR_FF0);

        pub const SFR_FFCNT_SR_FF1: crate::Register = crate::Register::new(17, 0xffff);
        pub const SFR_FFCNT_SR_FF1_SR_FF1: crate::Field = crate::Field::new(16, 0, SFR_FFCNT_SR_FF1);

        pub const SFR_FFCNT_SR_FF2: crate::Register = crate::Register::new(18, 0xffff);
        pub const SFR_FFCNT_SR_FF2_SR_FF2: crate::Field = crate::Field::new(16, 0, SFR_FFCNT_SR_FF2);

        pub const SFR_FFCNT_SR_FF3: crate::Register = crate::Register::new(19, 0xffff);
        pub const SFR_FFCNT_SR_FF3_SR_FF3: crate::Field = crate::Field::new(16, 0, SFR_FFCNT_SR_FF3);

        pub const SFR_FFCNT_SR_FF4: crate::Register = crate::Register::new(20, 0xffff);
        pub const SFR_FFCNT_SR_FF4_SR_FF4: crate::Field = crate::Field::new(16, 0, SFR_FFCNT_SR_FF4);

        pub const SFR_FFCNT_SR_FF5: crate::Register = crate::Register::new(21, 0xffff);
        pub const SFR_FFCNT_SR_FF5_SR_FF5: crate::Field = crate::Field::new(16, 0, SFR_FFCNT_SR_FF5);

        pub const HW_SCE_GLBSFR_BASE: usize = 0x40028000;
    }

    pub mod trng {
        pub const TRNG_NUMREGS: usize = 1;

        pub const RESERVED0: crate::Register = crate::Register::new(0, 0x1);
        pub const RESERVED0_RESERVED0: crate::Field = crate::Field::new(1, 0, RESERVED0);

        pub const HW_TRNG_BASE: usize = 0x4002e000;
    }

    pub mod alu {
        pub const ALU_NUMREGS: usize = 1;

        pub const RESERVED0: crate::Register = crate::Register::new(0, 0x1);
        pub const RESERVED0_RESERVED0: crate::Field = crate::Field::new(1, 0, RESERVED0);

        pub const HW_ALU_BASE: usize = 0x4002f000;
    }

    pub mod duart {
        pub const DUART_NUMREGS: usize = 4;

        pub const SFR_TXD: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_TXD_SFR_TXD: crate::Field = crate::Field::new(8, 0, SFR_TXD);

        pub const SFR_CR: crate::Register = crate::Register::new(1, 0x1);
        pub const SFR_CR_SFR_CR: crate::Field = crate::Field::new(1, 0, SFR_CR);

        pub const SFR_SR: crate::Register = crate::Register::new(2, 0x1);
        pub const SFR_SR_SFR_SR: crate::Field = crate::Field::new(1, 0, SFR_SR);

        pub const SFR_ETUC: crate::Register = crate::Register::new(3, 0xffff);
        pub const SFR_ETUC_SFR_ETUC: crate::Field = crate::Field::new(16, 0, SFR_ETUC);

        pub const HW_DUART_BASE: usize = 0x40042000;
    }

    pub mod wdg_intf {
        pub const WDG_INTF_NUMREGS: usize = 1;

        pub const RESERVED0: crate::Register = crate::Register::new(0, 0x1);
        pub const RESERVED0_RESERVED0: crate::Field = crate::Field::new(1, 0, RESERVED0);

        pub const HW_WDG_INTF_BASE: usize = 0x40041000;
    }

    pub mod timer_intf {
        pub const TIMER_INTF_NUMREGS: usize = 1;

        pub const RESERVED0: crate::Register = crate::Register::new(0, 0x1);
        pub const RESERVED0_RESERVED0: crate::Field = crate::Field::new(1, 0, RESERVED0);

        pub const HW_TIMER_INTF_BASE: usize = 0x40043000;
    }

    pub mod evc {
        pub const EVC_NUMREGS: usize = 33;

        pub const SFR_CM7EVSEL_CM7EVSEL0: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL0_CM7EVSEL0: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL0);

        pub const SFR_CM7EVSEL_CM7EVSEL1: crate::Register = crate::Register::new(1, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL1_CM7EVSEL1: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL1);

        pub const SFR_CM7EVSEL_CM7EVSEL2: crate::Register = crate::Register::new(2, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL2_CM7EVSEL2: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL2);

        pub const SFR_CM7EVSEL_CM7EVSEL3: crate::Register = crate::Register::new(3, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL3_CM7EVSEL3: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL3);

        pub const SFR_CM7EVSEL_CM7EVSEL4: crate::Register = crate::Register::new(4, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL4_CM7EVSEL4: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL4);

        pub const SFR_CM7EVSEL_CM7EVSEL5: crate::Register = crate::Register::new(5, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL5_CM7EVSEL5: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL5);

        pub const SFR_CM7EVSEL_CM7EVSEL6: crate::Register = crate::Register::new(6, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL6_CM7EVSEL6: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL6);

        pub const SFR_CM7EVSEL_CM7EVSEL7: crate::Register = crate::Register::new(7, 0xff);
        pub const SFR_CM7EVSEL_CM7EVSEL7_CM7EVSEL7: crate::Field = crate::Field::new(8, 0, SFR_CM7EVSEL_CM7EVSEL7);

        pub const SFR_CM7EVEN: crate::Register = crate::Register::new(8, 0xff);
        pub const SFR_CM7EVEN_CM7EVEN: crate::Field = crate::Field::new(8, 0, SFR_CM7EVEN);

        pub const SFR_CM7EVFR: crate::Register = crate::Register::new(9, 0xff);
        pub const SFR_CM7EVFR_CM7EVS: crate::Field = crate::Field::new(8, 0, SFR_CM7EVFR);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const RESERVED11: crate::Register = crate::Register::new(11, 0x1);
        pub const RESERVED11_RESERVED11: crate::Field = crate::Field::new(1, 0, RESERVED11);

        pub const SFR_TMREVSEL: crate::Register = crate::Register::new(12, 0xffff);
        pub const SFR_TMREVSEL_TMR_EVSEL: crate::Field = crate::Field::new(16, 0, SFR_TMREVSEL);

        pub const SFR_PWMEVSEL: crate::Register = crate::Register::new(13, 0xffffffff);
        pub const SFR_PWMEVSEL_PWM_EVSEL: crate::Field = crate::Field::new(32, 0, SFR_PWMEVSEL);

        pub const RESERVED14: crate::Register = crate::Register::new(14, 0x1);
        pub const RESERVED14_RESERVED14: crate::Field = crate::Field::new(1, 0, RESERVED14);

        pub const RESERVED15: crate::Register = crate::Register::new(15, 0x1);
        pub const RESERVED15_RESERVED15: crate::Field = crate::Field::new(1, 0, RESERVED15);

        pub const SFR_IFEVEN_IFEVEN0: crate::Register = crate::Register::new(16, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN0_IFEVEN0: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN0);

        pub const SFR_IFEVEN_IFEVEN1: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN1_IFEVEN1: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN1);

        pub const SFR_IFEVEN_IFEVEN2: crate::Register = crate::Register::new(18, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN2_IFEVEN2: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN2);

        pub const SFR_IFEVEN_IFEVEN3: crate::Register = crate::Register::new(19, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN3_IFEVEN3: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN3);

        pub const SFR_IFEVEN_IFEVEN4: crate::Register = crate::Register::new(20, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN4_IFEVEN4: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN4);

        pub const SFR_IFEVEN_IFEVEN5: crate::Register = crate::Register::new(21, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN5_IFEVEN5: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN5);

        pub const SFR_IFEVEN_IFEVEN6: crate::Register = crate::Register::new(22, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN6_IFEVEN6: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN6);

        pub const SFR_IFEVEN_IFEVEN7: crate::Register = crate::Register::new(23, 0xffffffff);
        pub const SFR_IFEVEN_IFEVEN7_IFEVEN7: crate::Field = crate::Field::new(32, 0, SFR_IFEVEN_IFEVEN7);

        pub const SFR_IFEVERRFR: crate::Register = crate::Register::new(24, 0xffffffff);
        pub const SFR_IFEVERRFR_IFEV_ERRS: crate::Field = crate::Field::new(32, 0, SFR_IFEVERRFR);

        pub const RESERVED25: crate::Register = crate::Register::new(25, 0x1);
        pub const RESERVED25_RESERVED25: crate::Field = crate::Field::new(1, 0, RESERVED25);

        pub const RESERVED26: crate::Register = crate::Register::new(26, 0x1);
        pub const RESERVED26_RESERVED26: crate::Field = crate::Field::new(1, 0, RESERVED26);

        pub const RESERVED27: crate::Register = crate::Register::new(27, 0x1);
        pub const RESERVED27_RESERVED27: crate::Field = crate::Field::new(1, 0, RESERVED27);

        pub const RESERVED28: crate::Register = crate::Register::new(28, 0x1);
        pub const RESERVED28_RESERVED28: crate::Field = crate::Field::new(1, 0, RESERVED28);

        pub const RESERVED29: crate::Register = crate::Register::new(29, 0x1);
        pub const RESERVED29_RESERVED29: crate::Field = crate::Field::new(1, 0, RESERVED29);

        pub const RESERVED30: crate::Register = crate::Register::new(30, 0x1);
        pub const RESERVED30_RESERVED30: crate::Field = crate::Field::new(1, 0, RESERVED30);

        pub const RESERVED31: crate::Register = crate::Register::new(31, 0x1);
        pub const RESERVED31_RESERVED31: crate::Field = crate::Field::new(1, 0, RESERVED31);

        pub const SFR_CM7ERRFR: crate::Register = crate::Register::new(32, 0xffffffff);
        pub const SFR_CM7ERRFR_ERRIN: crate::Field = crate::Field::new(32, 0, SFR_CM7ERRFR);

        pub const HW_EVC_BASE: usize = 0x40044000;
    }

    pub mod sysctrl {
        pub const SYSCTRL_NUMREGS: usize = 44;

        pub const SFR_CGUSEC: crate::Register = crate::Register::new(0, 0xffff);
        pub const SFR_CGUSEC_SFR_CGUSEC: crate::Field = crate::Field::new(16, 0, SFR_CGUSEC);

        pub const SFR_CGULP: crate::Register = crate::Register::new(1, 0xffff);
        pub const SFR_CGULP_SFR_CGULP: crate::Field = crate::Field::new(16, 0, SFR_CGULP);

        pub const RESERVED2: crate::Register = crate::Register::new(2, 0x1);
        pub const RESERVED2_RESERVED2: crate::Field = crate::Field::new(1, 0, RESERVED2);

        pub const RESERVED3: crate::Register = crate::Register::new(3, 0x1);
        pub const RESERVED3_RESERVED3: crate::Field = crate::Field::new(1, 0, RESERVED3);

        pub const SFR_CGUSEL0: crate::Register = crate::Register::new(4, 0x3);
        pub const SFR_CGUSEL0_SFR_CGUSEL0: crate::Field = crate::Field::new(2, 0, SFR_CGUSEL0);

        pub const SFR_CGUFD_CFGFDCR0: crate::Register = crate::Register::new(5, 0xffff);
        pub const SFR_CGUFD_CFGFDCR0_CFGFDCR0: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR0);

        pub const SFR_CGUFD_CFGFDCR1: crate::Register = crate::Register::new(6, 0xffff);
        pub const SFR_CGUFD_CFGFDCR1_CFGFDCR1: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR1);

        pub const SFR_CGUFD_CFGFDCR2: crate::Register = crate::Register::new(7, 0xffff);
        pub const SFR_CGUFD_CFGFDCR2_CFGFDCR2: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR2);

        pub const SFR_CGUFD_CFGFDCR3: crate::Register = crate::Register::new(8, 0xffff);
        pub const SFR_CGUFD_CFGFDCR3_CFGFDCR3: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR3);

        pub const SFR_CGUFD_CFGFDCR4: crate::Register = crate::Register::new(9, 0xffff);
        pub const SFR_CGUFD_CFGFDCR4_CFGFDCR4: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR4);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const SFR_CGUSET: crate::Register = crate::Register::new(11, 0xffffffff);
        pub const SFR_CGUSET_SFR_CGUSET: crate::Field = crate::Field::new(32, 0, SFR_CGUSET);

        pub const SFR_CGUSEL1: crate::Register = crate::Register::new(12, 0x1);
        pub const SFR_CGUSEL1_SFR_CGUSEL1: crate::Field = crate::Field::new(1, 0, SFR_CGUSEL1);

        pub const RESERVED13: crate::Register = crate::Register::new(13, 0x1);
        pub const RESERVED13_RESERVED13: crate::Field = crate::Field::new(1, 0, RESERVED13);

        pub const RESERVED14: crate::Register = crate::Register::new(14, 0x1);
        pub const RESERVED14_RESERVED14: crate::Field = crate::Field::new(1, 0, RESERVED14);

        pub const RESERVED15: crate::Register = crate::Register::new(15, 0x1);
        pub const RESERVED15_RESERVED15: crate::Field = crate::Field::new(1, 0, RESERVED15);

        pub const SFR_CGUFSSR_FSFREQ0: crate::Register = crate::Register::new(16, 0xffff);
        pub const SFR_CGUFSSR_FSFREQ0_FSFREQ0: crate::Field = crate::Field::new(16, 0, SFR_CGUFSSR_FSFREQ0);

        pub const SFR_CGUFSSR_FSFREQ1: crate::Register = crate::Register::new(17, 0xffff);
        pub const SFR_CGUFSSR_FSFREQ1_FSFREQ1: crate::Field = crate::Field::new(16, 0, SFR_CGUFSSR_FSFREQ1);

        pub const SFR_CGUFSSR_FSFREQ2: crate::Register = crate::Register::new(18, 0xffff);
        pub const SFR_CGUFSSR_FSFREQ2_FSFREQ2: crate::Field = crate::Field::new(16, 0, SFR_CGUFSSR_FSFREQ2);

        pub const SFR_CGUFSSR_FSFREQ3: crate::Register = crate::Register::new(19, 0xffff);
        pub const SFR_CGUFSSR_FSFREQ3_FSFREQ3: crate::Field = crate::Field::new(16, 0, SFR_CGUFSSR_FSFREQ3);

        pub const SFR_CGUFSVLD: crate::Register = crate::Register::new(20, 0xf);
        pub const SFR_CGUFSVLD_SFR_CGUFSVLD: crate::Field = crate::Field::new(4, 0, SFR_CGUFSVLD);

        pub const SFR_CGUFSCR: crate::Register = crate::Register::new(21, 0xffff);
        pub const SFR_CGUFSCR_SFR_CGUFSCR: crate::Field = crate::Field::new(16, 0, SFR_CGUFSCR);

        pub const RESERVED22: crate::Register = crate::Register::new(22, 0x1);
        pub const RESERVED22_RESERVED22: crate::Field = crate::Field::new(1, 0, RESERVED22);

        pub const RESERVED23: crate::Register = crate::Register::new(23, 0x1);
        pub const RESERVED23_RESERVED23: crate::Field = crate::Field::new(1, 0, RESERVED23);

        pub const SFR_ACLKGR: crate::Register = crate::Register::new(24, 0xff);
        pub const SFR_ACLKGR_SFR_ACLKGR: crate::Field = crate::Field::new(8, 0, SFR_ACLKGR);

        pub const SFR_HCLKGR: crate::Register = crate::Register::new(25, 0xff);
        pub const SFR_HCLKGR_SFR_HCLKGR: crate::Field = crate::Field::new(8, 0, SFR_HCLKGR);

        pub const SFR_ICLKGR: crate::Register = crate::Register::new(26, 0xff);
        pub const SFR_ICLKGR_SFR_ICLKGR: crate::Field = crate::Field::new(8, 0, SFR_ICLKGR);

        pub const SFR_PCLKGR: crate::Register = crate::Register::new(27, 0xff);
        pub const SFR_PCLKGR_SFR_PCLKGR: crate::Field = crate::Field::new(8, 0, SFR_PCLKGR);

        pub const RESERVED28: crate::Register = crate::Register::new(28, 0x1);
        pub const RESERVED28_RESERVED28: crate::Field = crate::Field::new(1, 0, RESERVED28);

        pub const RESERVED29: crate::Register = crate::Register::new(29, 0x1);
        pub const RESERVED29_RESERVED29: crate::Field = crate::Field::new(1, 0, RESERVED29);

        pub const RESERVED30: crate::Register = crate::Register::new(30, 0x1);
        pub const RESERVED30_RESERVED30: crate::Field = crate::Field::new(1, 0, RESERVED30);

        pub const RESERVED31: crate::Register = crate::Register::new(31, 0x1);
        pub const RESERVED31_RESERVED31: crate::Field = crate::Field::new(1, 0, RESERVED31);

        pub const SFR_RCURST0: crate::Register = crate::Register::new(32, 0xffffffff);
        pub const SFR_RCURST0_SFR_RCURST0: crate::Field = crate::Field::new(32, 0, SFR_RCURST0);

        pub const SFR_RCURST1: crate::Register = crate::Register::new(33, 0xffffffff);
        pub const SFR_RCURST1_SFR_RCURST1: crate::Field = crate::Field::new(32, 0, SFR_RCURST1);

        pub const SFR_RCUSRCFR: crate::Register = crate::Register::new(34, 0xffff);
        pub const SFR_RCUSRCFR_SFR_RCUSRCFR: crate::Field = crate::Field::new(16, 0, SFR_RCUSRCFR);

        pub const RESERVED35: crate::Register = crate::Register::new(35, 0x1);
        pub const RESERVED35_RESERVED35: crate::Field = crate::Field::new(1, 0, RESERVED35);

        pub const SFR_IPCARIPFLOW: crate::Register = crate::Register::new(36, 0xffffffff);
        pub const SFR_IPCARIPFLOW_SFR_IPCARIPFLOW: crate::Field = crate::Field::new(32, 0, SFR_IPCARIPFLOW);

        pub const SFR_IPCEN: crate::Register = crate::Register::new(37, 0xffff);
        pub const SFR_IPCEN_SFR_IPCEN: crate::Field = crate::Field::new(16, 0, SFR_IPCEN);

        pub const SFR_IPCLPEN: crate::Register = crate::Register::new(38, 0xffff);
        pub const SFR_IPCLPEN_SFR_IPCLPEN: crate::Field = crate::Field::new(16, 0, SFR_IPCLPEN);

        pub const SFR_IPCOSC: crate::Register = crate::Register::new(39, 0x7f);
        pub const SFR_IPCOSC_SFR_IPCOSC: crate::Field = crate::Field::new(7, 0, SFR_IPCOSC);

        pub const SFR_IPCPLLMN: crate::Register = crate::Register::new(40, 0x1ffff);
        pub const SFR_IPCPLLMN_SFR_IPCPLLMN: crate::Field = crate::Field::new(17, 0, SFR_IPCPLLMN);

        pub const SFR_IPCPLLF: crate::Register = crate::Register::new(41, 0x1ffffff);
        pub const SFR_IPCPLLF_SFR_IPCPLLF: crate::Field = crate::Field::new(25, 0, SFR_IPCPLLF);

        pub const SFR_IPCPLLQ: crate::Register = crate::Register::new(42, 0x7fff);
        pub const SFR_IPCPLLQ_SFR_IPCPLLQ: crate::Field = crate::Field::new(15, 0, SFR_IPCPLLQ);

        pub const SFR_IPCCR: crate::Register = crate::Register::new(43, 0xffff);
        pub const SFR_IPCCR_SFR_IPCCR: crate::Field = crate::Field::new(16, 0, SFR_IPCCR);

        pub const HW_SYSCTRL_BASE: usize = 0x40040000;
    }

    pub mod apb_thru {
        pub const APB_THRU_NUMREGS: usize = 1;

        pub const RESERVED0: crate::Register = crate::Register::new(0, 0x1);
        pub const RESERVED0_RESERVED0: crate::Field = crate::Field::new(1, 0, RESERVED0);

        pub const HW_APB_THRU_BASE: usize = 0x50122000;
    }

    pub mod iox {
        pub const IOX_NUMREGS: usize = 152;

        pub const SFR_AFSEL_CRAFSEL0: crate::Register = crate::Register::new(0, 0xffff);
        pub const SFR_AFSEL_CRAFSEL0_CRAFSEL0: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL0);

        pub const SFR_AFSEL_CRAFSEL1: crate::Register = crate::Register::new(1, 0xffff);
        pub const SFR_AFSEL_CRAFSEL1_CRAFSEL1: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL1);

        pub const SFR_AFSEL_CRAFSEL2: crate::Register = crate::Register::new(2, 0xffff);
        pub const SFR_AFSEL_CRAFSEL2_CRAFSEL2: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL2);

        pub const SFR_AFSEL_CRAFSEL3: crate::Register = crate::Register::new(3, 0xffff);
        pub const SFR_AFSEL_CRAFSEL3_CRAFSEL3: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL3);

        pub const SFR_AFSEL_CRAFSEL4: crate::Register = crate::Register::new(4, 0xffff);
        pub const SFR_AFSEL_CRAFSEL4_CRAFSEL4: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL4);

        pub const SFR_AFSEL_CRAFSEL5: crate::Register = crate::Register::new(5, 0xffff);
        pub const SFR_AFSEL_CRAFSEL5_CRAFSEL5: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL5);

        pub const SFR_AFSEL_CRAFSEL6: crate::Register = crate::Register::new(6, 0xffff);
        pub const SFR_AFSEL_CRAFSEL6_CRAFSEL6: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL6);

        pub const SFR_AFSEL_CRAFSEL7: crate::Register = crate::Register::new(7, 0xffff);
        pub const SFR_AFSEL_CRAFSEL7_CRAFSEL7: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL7);

        pub const RESERVED8: crate::Register = crate::Register::new(8, 0x1);
        pub const RESERVED8_RESERVED8: crate::Field = crate::Field::new(1, 0, RESERVED8);

        pub const RESERVED9: crate::Register = crate::Register::new(9, 0x1);
        pub const RESERVED9_RESERVED9: crate::Field = crate::Field::new(1, 0, RESERVED9);

        pub const RESERVED10: crate::Register = crate::Register::new(10, 0x1);
        pub const RESERVED10_RESERVED10: crate::Field = crate::Field::new(1, 0, RESERVED10);

        pub const RESERVED11: crate::Register = crate::Register::new(11, 0x1);
        pub const RESERVED11_RESERVED11: crate::Field = crate::Field::new(1, 0, RESERVED11);

        pub const RESERVED12: crate::Register = crate::Register::new(12, 0x1);
        pub const RESERVED12_RESERVED12: crate::Field = crate::Field::new(1, 0, RESERVED12);

        pub const RESERVED13: crate::Register = crate::Register::new(13, 0x1);
        pub const RESERVED13_RESERVED13: crate::Field = crate::Field::new(1, 0, RESERVED13);

        pub const RESERVED14: crate::Register = crate::Register::new(14, 0x1);
        pub const RESERVED14_RESERVED14: crate::Field = crate::Field::new(1, 0, RESERVED14);

        pub const RESERVED15: crate::Register = crate::Register::new(15, 0x1);
        pub const RESERVED15_RESERVED15: crate::Field = crate::Field::new(1, 0, RESERVED15);

        pub const RESERVED16: crate::Register = crate::Register::new(16, 0x1);
        pub const RESERVED16_RESERVED16: crate::Field = crate::Field::new(1, 0, RESERVED16);

        pub const RESERVED17: crate::Register = crate::Register::new(17, 0x1);
        pub const RESERVED17_RESERVED17: crate::Field = crate::Field::new(1, 0, RESERVED17);

        pub const RESERVED18: crate::Register = crate::Register::new(18, 0x1);
        pub const RESERVED18_RESERVED18: crate::Field = crate::Field::new(1, 0, RESERVED18);

        pub const RESERVED19: crate::Register = crate::Register::new(19, 0x1);
        pub const RESERVED19_RESERVED19: crate::Field = crate::Field::new(1, 0, RESERVED19);

        pub const RESERVED20: crate::Register = crate::Register::new(20, 0x1);
        pub const RESERVED20_RESERVED20: crate::Field = crate::Field::new(1, 0, RESERVED20);

        pub const RESERVED21: crate::Register = crate::Register::new(21, 0x1);
        pub const RESERVED21_RESERVED21: crate::Field = crate::Field::new(1, 0, RESERVED21);

        pub const RESERVED22: crate::Register = crate::Register::new(22, 0x1);
        pub const RESERVED22_RESERVED22: crate::Field = crate::Field::new(1, 0, RESERVED22);

        pub const RESERVED23: crate::Register = crate::Register::new(23, 0x1);
        pub const RESERVED23_RESERVED23: crate::Field = crate::Field::new(1, 0, RESERVED23);

        pub const RESERVED24: crate::Register = crate::Register::new(24, 0x1);
        pub const RESERVED24_RESERVED24: crate::Field = crate::Field::new(1, 0, RESERVED24);

        pub const RESERVED25: crate::Register = crate::Register::new(25, 0x1);
        pub const RESERVED25_RESERVED25: crate::Field = crate::Field::new(1, 0, RESERVED25);

        pub const RESERVED26: crate::Register = crate::Register::new(26, 0x1);
        pub const RESERVED26_RESERVED26: crate::Field = crate::Field::new(1, 0, RESERVED26);

        pub const RESERVED27: crate::Register = crate::Register::new(27, 0x1);
        pub const RESERVED27_RESERVED27: crate::Field = crate::Field::new(1, 0, RESERVED27);

        pub const RESERVED28: crate::Register = crate::Register::new(28, 0x1);
        pub const RESERVED28_RESERVED28: crate::Field = crate::Field::new(1, 0, RESERVED28);

        pub const RESERVED29: crate::Register = crate::Register::new(29, 0x1);
        pub const RESERVED29_RESERVED29: crate::Field = crate::Field::new(1, 0, RESERVED29);

        pub const RESERVED30: crate::Register = crate::Register::new(30, 0x1);
        pub const RESERVED30_RESERVED30: crate::Field = crate::Field::new(1, 0, RESERVED30);

        pub const RESERVED31: crate::Register = crate::Register::new(31, 0x1);
        pub const RESERVED31_RESERVED31: crate::Field = crate::Field::new(1, 0, RESERVED31);

        pub const RESERVED32: crate::Register = crate::Register::new(32, 0x1);
        pub const RESERVED32_RESERVED32: crate::Field = crate::Field::new(1, 0, RESERVED32);

        pub const RESERVED33: crate::Register = crate::Register::new(33, 0x1);
        pub const RESERVED33_RESERVED33: crate::Field = crate::Field::new(1, 0, RESERVED33);

        pub const RESERVED34: crate::Register = crate::Register::new(34, 0x1);
        pub const RESERVED34_RESERVED34: crate::Field = crate::Field::new(1, 0, RESERVED34);

        pub const RESERVED35: crate::Register = crate::Register::new(35, 0x1);
        pub const RESERVED35_RESERVED35: crate::Field = crate::Field::new(1, 0, RESERVED35);

        pub const RESERVED36: crate::Register = crate::Register::new(36, 0x1);
        pub const RESERVED36_RESERVED36: crate::Field = crate::Field::new(1, 0, RESERVED36);

        pub const RESERVED37: crate::Register = crate::Register::new(37, 0x1);
        pub const RESERVED37_RESERVED37: crate::Field = crate::Field::new(1, 0, RESERVED37);

        pub const RESERVED38: crate::Register = crate::Register::new(38, 0x1);
        pub const RESERVED38_RESERVED38: crate::Field = crate::Field::new(1, 0, RESERVED38);

        pub const RESERVED39: crate::Register = crate::Register::new(39, 0x1);
        pub const RESERVED39_RESERVED39: crate::Field = crate::Field::new(1, 0, RESERVED39);

        pub const RESERVED40: crate::Register = crate::Register::new(40, 0x1);
        pub const RESERVED40_RESERVED40: crate::Field = crate::Field::new(1, 0, RESERVED40);

        pub const RESERVED41: crate::Register = crate::Register::new(41, 0x1);
        pub const RESERVED41_RESERVED41: crate::Field = crate::Field::new(1, 0, RESERVED41);

        pub const RESERVED42: crate::Register = crate::Register::new(42, 0x1);
        pub const RESERVED42_RESERVED42: crate::Field = crate::Field::new(1, 0, RESERVED42);

        pub const RESERVED43: crate::Register = crate::Register::new(43, 0x1);
        pub const RESERVED43_RESERVED43: crate::Field = crate::Field::new(1, 0, RESERVED43);

        pub const RESERVED44: crate::Register = crate::Register::new(44, 0x1);
        pub const RESERVED44_RESERVED44: crate::Field = crate::Field::new(1, 0, RESERVED44);

        pub const RESERVED45: crate::Register = crate::Register::new(45, 0x1);
        pub const RESERVED45_RESERVED45: crate::Field = crate::Field::new(1, 0, RESERVED45);

        pub const RESERVED46: crate::Register = crate::Register::new(46, 0x1);
        pub const RESERVED46_RESERVED46: crate::Field = crate::Field::new(1, 0, RESERVED46);

        pub const RESERVED47: crate::Register = crate::Register::new(47, 0x1);
        pub const RESERVED47_RESERVED47: crate::Field = crate::Field::new(1, 0, RESERVED47);

        pub const RESERVED48: crate::Register = crate::Register::new(48, 0x1);
        pub const RESERVED48_RESERVED48: crate::Field = crate::Field::new(1, 0, RESERVED48);

        pub const RESERVED49: crate::Register = crate::Register::new(49, 0x1);
        pub const RESERVED49_RESERVED49: crate::Field = crate::Field::new(1, 0, RESERVED49);

        pub const RESERVED50: crate::Register = crate::Register::new(50, 0x1);
        pub const RESERVED50_RESERVED50: crate::Field = crate::Field::new(1, 0, RESERVED50);

        pub const RESERVED51: crate::Register = crate::Register::new(51, 0x1);
        pub const RESERVED51_RESERVED51: crate::Field = crate::Field::new(1, 0, RESERVED51);

        pub const RESERVED52: crate::Register = crate::Register::new(52, 0x1);
        pub const RESERVED52_RESERVED52: crate::Field = crate::Field::new(1, 0, RESERVED52);

        pub const RESERVED53: crate::Register = crate::Register::new(53, 0x1);
        pub const RESERVED53_RESERVED53: crate::Field = crate::Field::new(1, 0, RESERVED53);

        pub const RESERVED54: crate::Register = crate::Register::new(54, 0x1);
        pub const RESERVED54_RESERVED54: crate::Field = crate::Field::new(1, 0, RESERVED54);

        pub const RESERVED55: crate::Register = crate::Register::new(55, 0x1);
        pub const RESERVED55_RESERVED55: crate::Field = crate::Field::new(1, 0, RESERVED55);

        pub const RESERVED56: crate::Register = crate::Register::new(56, 0x1);
        pub const RESERVED56_RESERVED56: crate::Field = crate::Field::new(1, 0, RESERVED56);

        pub const RESERVED57: crate::Register = crate::Register::new(57, 0x1);
        pub const RESERVED57_RESERVED57: crate::Field = crate::Field::new(1, 0, RESERVED57);

        pub const RESERVED58: crate::Register = crate::Register::new(58, 0x1);
        pub const RESERVED58_RESERVED58: crate::Field = crate::Field::new(1, 0, RESERVED58);

        pub const RESERVED59: crate::Register = crate::Register::new(59, 0x1);
        pub const RESERVED59_RESERVED59: crate::Field = crate::Field::new(1, 0, RESERVED59);

        pub const RESERVED60: crate::Register = crate::Register::new(60, 0x1);
        pub const RESERVED60_RESERVED60: crate::Field = crate::Field::new(1, 0, RESERVED60);

        pub const RESERVED61: crate::Register = crate::Register::new(61, 0x1);
        pub const RESERVED61_RESERVED61: crate::Field = crate::Field::new(1, 0, RESERVED61);

        pub const RESERVED62: crate::Register = crate::Register::new(62, 0x1);
        pub const RESERVED62_RESERVED62: crate::Field = crate::Field::new(1, 0, RESERVED62);

        pub const RESERVED63: crate::Register = crate::Register::new(63, 0x1);
        pub const RESERVED63_RESERVED63: crate::Field = crate::Field::new(1, 0, RESERVED63);

        pub const SFR_INTCR_CRINT0: crate::Register = crate::Register::new(64, 0x3ff);
        pub const SFR_INTCR_CRINT0_CRINT0: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT0);

        pub const SFR_INTCR_CRINT1: crate::Register = crate::Register::new(65, 0x3ff);
        pub const SFR_INTCR_CRINT1_CRINT1: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT1);

        pub const SFR_INTCR_CRINT2: crate::Register = crate::Register::new(66, 0x3ff);
        pub const SFR_INTCR_CRINT2_CRINT2: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT2);

        pub const SFR_INTCR_CRINT3: crate::Register = crate::Register::new(67, 0x3ff);
        pub const SFR_INTCR_CRINT3_CRINT3: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT3);

        pub const SFR_INTCR_CRINT4: crate::Register = crate::Register::new(68, 0x3ff);
        pub const SFR_INTCR_CRINT4_CRINT4: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT4);

        pub const SFR_INTCR_CRINT5: crate::Register = crate::Register::new(69, 0x3ff);
        pub const SFR_INTCR_CRINT5_CRINT5: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT5);

        pub const SFR_INTCR_CRINT6: crate::Register = crate::Register::new(70, 0x3ff);
        pub const SFR_INTCR_CRINT6_CRINT6: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT6);

        pub const SFR_INTCR_CRINT7: crate::Register = crate::Register::new(71, 0x3ff);
        pub const SFR_INTCR_CRINT7_CRINT7: crate::Field = crate::Field::new(10, 0, SFR_INTCR_CRINT7);

        pub const SFR_INTFR: crate::Register = crate::Register::new(72, 0xff);
        pub const SFR_INTFR_FRINT: crate::Field = crate::Field::new(8, 0, SFR_INTFR);

        pub const RESERVED73: crate::Register = crate::Register::new(73, 0x1);
        pub const RESERVED73_RESERVED73: crate::Field = crate::Field::new(1, 0, RESERVED73);

        pub const RESERVED74: crate::Register = crate::Register::new(74, 0x1);
        pub const RESERVED74_RESERVED74: crate::Field = crate::Field::new(1, 0, RESERVED74);

        pub const RESERVED75: crate::Register = crate::Register::new(75, 0x1);
        pub const RESERVED75_RESERVED75: crate::Field = crate::Field::new(1, 0, RESERVED75);

        pub const SFR_GPIOOUT_CRGO0: crate::Register = crate::Register::new(76, 0xffff);
        pub const SFR_GPIOOUT_CRGO0_CRGO0: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO0);

        pub const SFR_GPIOOUT_CRGO1: crate::Register = crate::Register::new(77, 0xffff);
        pub const SFR_GPIOOUT_CRGO1_CRGO1: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO1);

        pub const SFR_GPIOOUT_CRGO2: crate::Register = crate::Register::new(78, 0xffff);
        pub const SFR_GPIOOUT_CRGO2_CRGO2: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO2);

        pub const SFR_GPIOOUT_CRGO3: crate::Register = crate::Register::new(79, 0xffff);
        pub const SFR_GPIOOUT_CRGO3_CRGO3: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO3);

        pub const SFR_GPIOOE_CRGOE0: crate::Register = crate::Register::new(80, 0xffff);
        pub const SFR_GPIOOE_CRGOE0_CRGOE0: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE0);

        pub const SFR_GPIOOE_CRGOE1: crate::Register = crate::Register::new(81, 0xffff);
        pub const SFR_GPIOOE_CRGOE1_CRGOE1: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE1);

        pub const SFR_GPIOOE_CRGOE2: crate::Register = crate::Register::new(82, 0xffff);
        pub const SFR_GPIOOE_CRGOE2_CRGOE2: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE2);

        pub const SFR_GPIOOE_CRGOE3: crate::Register = crate::Register::new(83, 0xffff);
        pub const SFR_GPIOOE_CRGOE3_CRGOE3: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE3);

        pub const SFR_GPIOPU_CRGPU0: crate::Register = crate::Register::new(84, 0xffff);
        pub const SFR_GPIOPU_CRGPU0_CRGPU0: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU0);

        pub const SFR_GPIOPU_CRGPU1: crate::Register = crate::Register::new(85, 0xffff);
        pub const SFR_GPIOPU_CRGPU1_CRGPU1: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU1);

        pub const SFR_GPIOPU_CRGPU2: crate::Register = crate::Register::new(86, 0xffff);
        pub const SFR_GPIOPU_CRGPU2_CRGPU2: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU2);

        pub const SFR_GPIOPU_CRGPU3: crate::Register = crate::Register::new(87, 0xffff);
        pub const SFR_GPIOPU_CRGPU3_CRGPU3: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU3);

        pub const SFR_GPIOIN_SRGI0: crate::Register = crate::Register::new(88, 0xffff);
        pub const SFR_GPIOIN_SRGI0_SRGI0: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI0);

        pub const SFR_GPIOIN_SRGI1: crate::Register = crate::Register::new(89, 0xffff);
        pub const SFR_GPIOIN_SRGI1_SRGI1: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI1);

        pub const SFR_GPIOIN_SRGI2: crate::Register = crate::Register::new(90, 0xffff);
        pub const SFR_GPIOIN_SRGI2_SRGI2: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI2);

        pub const SFR_GPIOIN_SRGI3: crate::Register = crate::Register::new(91, 0xffff);
        pub const SFR_GPIOIN_SRGI3_SRGI3: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI3);

        pub const RESERVED92: crate::Register = crate::Register::new(92, 0x1);
        pub const RESERVED92_RESERVED92: crate::Field = crate::Field::new(1, 0, RESERVED92);

        pub const RESERVED93: crate::Register = crate::Register::new(93, 0x1);
        pub const RESERVED93_RESERVED93: crate::Field = crate::Field::new(1, 0, RESERVED93);

        pub const RESERVED94: crate::Register = crate::Register::new(94, 0x1);
        pub const RESERVED94_RESERVED94: crate::Field = crate::Field::new(1, 0, RESERVED94);

        pub const RESERVED95: crate::Register = crate::Register::new(95, 0x1);
        pub const RESERVED95_RESERVED95: crate::Field = crate::Field::new(1, 0, RESERVED95);

        pub const RESERVED96: crate::Register = crate::Register::new(96, 0x1);
        pub const RESERVED96_RESERVED96: crate::Field = crate::Field::new(1, 0, RESERVED96);

        pub const RESERVED97: crate::Register = crate::Register::new(97, 0x1);
        pub const RESERVED97_RESERVED97: crate::Field = crate::Field::new(1, 0, RESERVED97);

        pub const RESERVED98: crate::Register = crate::Register::new(98, 0x1);
        pub const RESERVED98_RESERVED98: crate::Field = crate::Field::new(1, 0, RESERVED98);

        pub const RESERVED99: crate::Register = crate::Register::new(99, 0x1);
        pub const RESERVED99_RESERVED99: crate::Field = crate::Field::new(1, 0, RESERVED99);

        pub const RESERVED100: crate::Register = crate::Register::new(100, 0x1);
        pub const RESERVED100_RESERVED100: crate::Field = crate::Field::new(1, 0, RESERVED100);

        pub const RESERVED101: crate::Register = crate::Register::new(101, 0x1);
        pub const RESERVED101_RESERVED101: crate::Field = crate::Field::new(1, 0, RESERVED101);

        pub const RESERVED102: crate::Register = crate::Register::new(102, 0x1);
        pub const RESERVED102_RESERVED102: crate::Field = crate::Field::new(1, 0, RESERVED102);

        pub const RESERVED103: crate::Register = crate::Register::new(103, 0x1);
        pub const RESERVED103_RESERVED103: crate::Field = crate::Field::new(1, 0, RESERVED103);

        pub const RESERVED104: crate::Register = crate::Register::new(104, 0x1);
        pub const RESERVED104_RESERVED104: crate::Field = crate::Field::new(1, 0, RESERVED104);

        pub const RESERVED105: crate::Register = crate::Register::new(105, 0x1);
        pub const RESERVED105_RESERVED105: crate::Field = crate::Field::new(1, 0, RESERVED105);

        pub const RESERVED106: crate::Register = crate::Register::new(106, 0x1);
        pub const RESERVED106_RESERVED106: crate::Field = crate::Field::new(1, 0, RESERVED106);

        pub const RESERVED107: crate::Register = crate::Register::new(107, 0x1);
        pub const RESERVED107_RESERVED107: crate::Field = crate::Field::new(1, 0, RESERVED107);

        pub const RESERVED108: crate::Register = crate::Register::new(108, 0x1);
        pub const RESERVED108_RESERVED108: crate::Field = crate::Field::new(1, 0, RESERVED108);

        pub const RESERVED109: crate::Register = crate::Register::new(109, 0x1);
        pub const RESERVED109_RESERVED109: crate::Field = crate::Field::new(1, 0, RESERVED109);

        pub const RESERVED110: crate::Register = crate::Register::new(110, 0x1);
        pub const RESERVED110_RESERVED110: crate::Field = crate::Field::new(1, 0, RESERVED110);

        pub const RESERVED111: crate::Register = crate::Register::new(111, 0x1);
        pub const RESERVED111_RESERVED111: crate::Field = crate::Field::new(1, 0, RESERVED111);

        pub const RESERVED112: crate::Register = crate::Register::new(112, 0x1);
        pub const RESERVED112_RESERVED112: crate::Field = crate::Field::new(1, 0, RESERVED112);

        pub const RESERVED113: crate::Register = crate::Register::new(113, 0x1);
        pub const RESERVED113_RESERVED113: crate::Field = crate::Field::new(1, 0, RESERVED113);

        pub const RESERVED114: crate::Register = crate::Register::new(114, 0x1);
        pub const RESERVED114_RESERVED114: crate::Field = crate::Field::new(1, 0, RESERVED114);

        pub const RESERVED115: crate::Register = crate::Register::new(115, 0x1);
        pub const RESERVED115_RESERVED115: crate::Field = crate::Field::new(1, 0, RESERVED115);

        pub const RESERVED116: crate::Register = crate::Register::new(116, 0x1);
        pub const RESERVED116_RESERVED116: crate::Field = crate::Field::new(1, 0, RESERVED116);

        pub const RESERVED117: crate::Register = crate::Register::new(117, 0x1);
        pub const RESERVED117_RESERVED117: crate::Field = crate::Field::new(1, 0, RESERVED117);

        pub const RESERVED118: crate::Register = crate::Register::new(118, 0x1);
        pub const RESERVED118_RESERVED118: crate::Field = crate::Field::new(1, 0, RESERVED118);

        pub const RESERVED119: crate::Register = crate::Register::new(119, 0x1);
        pub const RESERVED119_RESERVED119: crate::Field = crate::Field::new(1, 0, RESERVED119);

        pub const RESERVED120: crate::Register = crate::Register::new(120, 0x1);
        pub const RESERVED120_RESERVED120: crate::Field = crate::Field::new(1, 0, RESERVED120);

        pub const RESERVED121: crate::Register = crate::Register::new(121, 0x1);
        pub const RESERVED121_RESERVED121: crate::Field = crate::Field::new(1, 0, RESERVED121);

        pub const RESERVED122: crate::Register = crate::Register::new(122, 0x1);
        pub const RESERVED122_RESERVED122: crate::Field = crate::Field::new(1, 0, RESERVED122);

        pub const RESERVED123: crate::Register = crate::Register::new(123, 0x1);
        pub const RESERVED123_RESERVED123: crate::Field = crate::Field::new(1, 0, RESERVED123);

        pub const RESERVED124: crate::Register = crate::Register::new(124, 0x1);
        pub const RESERVED124_RESERVED124: crate::Field = crate::Field::new(1, 0, RESERVED124);

        pub const RESERVED125: crate::Register = crate::Register::new(125, 0x1);
        pub const RESERVED125_RESERVED125: crate::Field = crate::Field::new(1, 0, RESERVED125);

        pub const RESERVED126: crate::Register = crate::Register::new(126, 0x1);
        pub const RESERVED126_RESERVED126: crate::Field = crate::Field::new(1, 0, RESERVED126);

        pub const RESERVED127: crate::Register = crate::Register::new(127, 0x1);
        pub const RESERVED127_RESERVED127: crate::Field = crate::Field::new(1, 0, RESERVED127);

        pub const SFR_PIOSEL: crate::Register = crate::Register::new(128, 0xffffffff);
        pub const SFR_PIOSEL_PIOSEL: crate::Field = crate::Field::new(32, 0, SFR_PIOSEL);

        pub const RESERVED129: crate::Register = crate::Register::new(129, 0x1);
        pub const RESERVED129_RESERVED129: crate::Field = crate::Field::new(1, 0, RESERVED129);

        pub const RESERVED130: crate::Register = crate::Register::new(130, 0x1);
        pub const RESERVED130_RESERVED130: crate::Field = crate::Field::new(1, 0, RESERVED130);

        pub const RESERVED131: crate::Register = crate::Register::new(131, 0x1);
        pub const RESERVED131_RESERVED131: crate::Field = crate::Field::new(1, 0, RESERVED131);

        pub const RESERVED132: crate::Register = crate::Register::new(132, 0x1);
        pub const RESERVED132_RESERVED132: crate::Field = crate::Field::new(1, 0, RESERVED132);

        pub const RESERVED133: crate::Register = crate::Register::new(133, 0x1);
        pub const RESERVED133_RESERVED133: crate::Field = crate::Field::new(1, 0, RESERVED133);

        pub const RESERVED134: crate::Register = crate::Register::new(134, 0x1);
        pub const RESERVED134_RESERVED134: crate::Field = crate::Field::new(1, 0, RESERVED134);

        pub const RESERVED135: crate::Register = crate::Register::new(135, 0x1);
        pub const RESERVED135_RESERVED135: crate::Field = crate::Field::new(1, 0, RESERVED135);

        pub const RESERVED136: crate::Register = crate::Register::new(136, 0x1);
        pub const RESERVED136_RESERVED136: crate::Field = crate::Field::new(1, 0, RESERVED136);

        pub const RESERVED137: crate::Register = crate::Register::new(137, 0x1);
        pub const RESERVED137_RESERVED137: crate::Field = crate::Field::new(1, 0, RESERVED137);

        pub const RESERVED138: crate::Register = crate::Register::new(138, 0x1);
        pub const RESERVED138_RESERVED138: crate::Field = crate::Field::new(1, 0, RESERVED138);

        pub const RESERVED139: crate::Register = crate::Register::new(139, 0x1);
        pub const RESERVED139_RESERVED139: crate::Field = crate::Field::new(1, 0, RESERVED139);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL0: crate::Register = crate::Register::new(140, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL0);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL1: crate::Register = crate::Register::new(141, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL1);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL2: crate::Register = crate::Register::new(142, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL2);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL3: crate::Register = crate::Register::new(143, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL3);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW0: crate::Register = crate::Register::new(144, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW0);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW1: crate::Register = crate::Register::new(145, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW1);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW2: crate::Register = crate::Register::new(146, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW2);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW3: crate::Register = crate::Register::new(147, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW3);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL0: crate::Register = crate::Register::new(148, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL0);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL1: crate::Register = crate::Register::new(149, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL1);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL2: crate::Register = crate::Register::new(150, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL2);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL3: crate::Register = crate::Register::new(151, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL3);

        pub const HW_IOX_BASE: usize = 0x5012f000;
    }

    pub mod pwm {
        pub const PWM_NUMREGS: usize = 1;

        pub const RESERVED0: crate::Register = crate::Register::new(0, 0x1);
        pub const RESERVED0_RESERVED0: crate::Field = crate::Field::new(1, 0, RESERVED0);

        pub const HW_PWM_BASE: usize = 0x50120000;
    }

    pub mod sddc {
        pub const SDDC_NUMREGS: usize = 125;

        pub const SFR_IO: crate::Register = crate::Register::new(0, 0x3);
        pub const SFR_IO_SFR_IO: crate::Field = crate::Field::new(2, 0, SFR_IO);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const RESERVED2: crate::Register = crate::Register::new(2, 0x1);
        pub const RESERVED2_RESERVED2: crate::Field = crate::Field::new(1, 0, RESERVED2);

        pub const RESERVED3: crate::Register = crate::Register::new(3, 0x1);
        pub const RESERVED3_RESERVED3: crate::Field = crate::Field::new(1, 0, RESERVED3);

        pub const CR_OCR: crate::Register = crate::Register::new(4, 0xffffff);
        pub const CR_OCR_CR_OCR: crate::Field = crate::Field::new(24, 0, CR_OCR);

        pub const CR_RDFFTHRES: crate::Register = crate::Register::new(5, 0xff);
        pub const CR_RDFFTHRES_CR_RDFFTHRES: crate::Field = crate::Field::new(8, 0, CR_RDFFTHRES);

        pub const CR_REV: crate::Register = crate::Register::new(6, 0xffff);
        pub const CR_REV_CFG_REG_SD_SPEC_REVISION: crate::Field = crate::Field::new(8, 0, CR_REV);
        pub const CR_REV_CFG_REG_CCCR_SDIO_REVISION: crate::Field = crate::Field::new(8, 8, CR_REV);

        pub const CR_BACSA: crate::Register = crate::Register::new(7, 0x3ffff);
        pub const CR_BACSA_CFG_BASE_ADDR_CSA: crate::Field = crate::Field::new(18, 0, CR_BACSA);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0: crate::Register = crate::Register::new(8, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0_CFG_BASE_ADDR_IO_FUNC0: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1: crate::Register = crate::Register::new(9, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1_CFG_BASE_ADDR_IO_FUNC1: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2: crate::Register = crate::Register::new(10, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2_CFG_BASE_ADDR_IO_FUNC2: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3: crate::Register = crate::Register::new(11, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3_CFG_BASE_ADDR_IO_FUNC3: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4: crate::Register = crate::Register::new(12, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4_CFG_BASE_ADDR_IO_FUNC4: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5: crate::Register = crate::Register::new(13, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5_CFG_BASE_ADDR_IO_FUNC5: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6: crate::Register = crate::Register::new(14, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6_CFG_BASE_ADDR_IO_FUNC6: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6);

        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7: crate::Register = crate::Register::new(15, 0x3ffff);
        pub const CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7_CFG_BASE_ADDR_IO_FUNC7: crate::Field = crate::Field::new(18, 0, CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0: crate::Register = crate::Register::new(16, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0_CFG_REG_FUNC_CIS_PTR0: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1: crate::Register = crate::Register::new(17, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1_CFG_REG_FUNC_CIS_PTR1: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2: crate::Register = crate::Register::new(18, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2_CFG_REG_FUNC_CIS_PTR2: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3: crate::Register = crate::Register::new(19, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3_CFG_REG_FUNC_CIS_PTR3: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4: crate::Register = crate::Register::new(20, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4_CFG_REG_FUNC_CIS_PTR4: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5: crate::Register = crate::Register::new(21, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5_CFG_REG_FUNC_CIS_PTR5: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6: crate::Register = crate::Register::new(22, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6_CFG_REG_FUNC_CIS_PTR6: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6);

        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7: crate::Register = crate::Register::new(23, 0x1ffff);
        pub const CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7_CFG_REG_FUNC_CIS_PTR7: crate::Field = crate::Field::new(17, 0, CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0: crate::Register = crate::Register::new(24, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0_CFG_REG_FUNC_EXT_STD_CODE0: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1: crate::Register = crate::Register::new(25, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1_CFG_REG_FUNC_EXT_STD_CODE1: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2: crate::Register = crate::Register::new(26, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2_CFG_REG_FUNC_EXT_STD_CODE2: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3: crate::Register = crate::Register::new(27, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3_CFG_REG_FUNC_EXT_STD_CODE3: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4: crate::Register = crate::Register::new(28, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4_CFG_REG_FUNC_EXT_STD_CODE4: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5: crate::Register = crate::Register::new(29, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5_CFG_REG_FUNC_EXT_STD_CODE5: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6: crate::Register = crate::Register::new(30, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6_CFG_REG_FUNC_EXT_STD_CODE6: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6);

        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7: crate::Register = crate::Register::new(31, 0xff);
        pub const CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7_CFG_REG_FUNC_EXT_STD_CODE7: crate::Field = crate::Field::new(8, 0, CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7);

        pub const CR_WRITE_PROTECT: crate::Register = crate::Register::new(32, 0x1);
        pub const CR_WRITE_PROTECT_CR_WRITE_PROTECT: crate::Field = crate::Field::new(1, 0, CR_WRITE_PROTECT);

        pub const CR_REG_DSR: crate::Register = crate::Register::new(33, 0xffff);
        pub const CR_REG_DSR_CR_REG_DSR: crate::Field = crate::Field::new(16, 0, CR_REG_DSR);

        pub const CR_REG_CID_CFG_REG_CID0: crate::Register = crate::Register::new(34, 0xffffffff);
        pub const CR_REG_CID_CFG_REG_CID0_CFG_REG_CID0: crate::Field = crate::Field::new(32, 0, CR_REG_CID_CFG_REG_CID0);

        pub const CR_REG_CID_CFG_REG_CID1: crate::Register = crate::Register::new(35, 0xffffffff);
        pub const CR_REG_CID_CFG_REG_CID1_CFG_REG_CID1: crate::Field = crate::Field::new(32, 0, CR_REG_CID_CFG_REG_CID1);

        pub const CR_REG_CID_CFG_REG_CID2: crate::Register = crate::Register::new(36, 0xffffffff);
        pub const CR_REG_CID_CFG_REG_CID2_CFG_REG_CID2: crate::Field = crate::Field::new(32, 0, CR_REG_CID_CFG_REG_CID2);

        pub const CR_REG_CID_CFG_REG_CID3: crate::Register = crate::Register::new(37, 0xffffffff);
        pub const CR_REG_CID_CFG_REG_CID3_CFG_REG_CID3: crate::Field = crate::Field::new(32, 0, CR_REG_CID_CFG_REG_CID3);

        pub const CR_REG_CSD_CFG_REG_CSD0: crate::Register = crate::Register::new(38, 0xffffffff);
        pub const CR_REG_CSD_CFG_REG_CSD0_CFG_REG_CSD0: crate::Field = crate::Field::new(32, 0, CR_REG_CSD_CFG_REG_CSD0);

        pub const CR_REG_CSD_CFG_REG_CSD1: crate::Register = crate::Register::new(39, 0xffffffff);
        pub const CR_REG_CSD_CFG_REG_CSD1_CFG_REG_CSD1: crate::Field = crate::Field::new(32, 0, CR_REG_CSD_CFG_REG_CSD1);

        pub const CR_REG_CSD_CFG_REG_CSD2: crate::Register = crate::Register::new(40, 0xffffffff);
        pub const CR_REG_CSD_CFG_REG_CSD2_CFG_REG_CSD2: crate::Field = crate::Field::new(32, 0, CR_REG_CSD_CFG_REG_CSD2);

        pub const CR_REG_CSD_CFG_REG_CSD3: crate::Register = crate::Register::new(41, 0xffffffff);
        pub const CR_REG_CSD_CFG_REG_CSD3_CFG_REG_CSD3: crate::Field = crate::Field::new(32, 0, CR_REG_CSD_CFG_REG_CSD3);

        pub const CR_REG_SCR_CFG_REG_SCR0: crate::Register = crate::Register::new(42, 0xffffffff);
        pub const CR_REG_SCR_CFG_REG_SCR0_CFG_REG_SCR0: crate::Field = crate::Field::new(32, 0, CR_REG_SCR_CFG_REG_SCR0);

        pub const CR_REG_SCR_CFG_REG_SCR1: crate::Register = crate::Register::new(43, 0xffffffff);
        pub const CR_REG_SCR_CFG_REG_SCR1_CFG_REG_SCR1: crate::Field = crate::Field::new(32, 0, CR_REG_SCR_CFG_REG_SCR1);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS0: crate::Register = crate::Register::new(44, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS0_CFG_REG_SD_STATUS0: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS0);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS1: crate::Register = crate::Register::new(45, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS1_CFG_REG_SD_STATUS1: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS1);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS2: crate::Register = crate::Register::new(46, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS2_CFG_REG_SD_STATUS2: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS2);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS3: crate::Register = crate::Register::new(47, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS3_CFG_REG_SD_STATUS3: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS3);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS4: crate::Register = crate::Register::new(48, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS4_CFG_REG_SD_STATUS4: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS4);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS5: crate::Register = crate::Register::new(49, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS5_CFG_REG_SD_STATUS5: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS5);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS6: crate::Register = crate::Register::new(50, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS6_CFG_REG_SD_STATUS6: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS6);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS7: crate::Register = crate::Register::new(51, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS7_CFG_REG_SD_STATUS7: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS7);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS8: crate::Register = crate::Register::new(52, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS8_CFG_REG_SD_STATUS8: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS8);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS9: crate::Register = crate::Register::new(53, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS9_CFG_REG_SD_STATUS9: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS9);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS10: crate::Register = crate::Register::new(54, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS10_CFG_REG_SD_STATUS10: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS10);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS11: crate::Register = crate::Register::new(55, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS11_CFG_REG_SD_STATUS11: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS11);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS12: crate::Register = crate::Register::new(56, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS12_CFG_REG_SD_STATUS12: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS12);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS13: crate::Register = crate::Register::new(57, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS13_CFG_REG_SD_STATUS13: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS13);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS14: crate::Register = crate::Register::new(58, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS14_CFG_REG_SD_STATUS14: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS14);

        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS15: crate::Register = crate::Register::new(59, 0xffffffff);
        pub const CR_REG_SD_STATUS_CFG_REG_SD_STATUS15_CFG_REG_SD_STATUS15: crate::Field = crate::Field::new(32, 0, CR_REG_SD_STATUS_CFG_REG_SD_STATUS15);

        pub const RESERVED60: crate::Register = crate::Register::new(60, 0x1);
        pub const RESERVED60_RESERVED60: crate::Field = crate::Field::new(1, 0, RESERVED60);

        pub const RESERVED61: crate::Register = crate::Register::new(61, 0x1);
        pub const RESERVED61_RESERVED61: crate::Field = crate::Field::new(1, 0, RESERVED61);

        pub const RESERVED62: crate::Register = crate::Register::new(62, 0x1);
        pub const RESERVED62_RESERVED62: crate::Field = crate::Field::new(1, 0, RESERVED62);

        pub const RESERVED63: crate::Register = crate::Register::new(63, 0x1);
        pub const RESERVED63_RESERVED63: crate::Field = crate::Field::new(1, 0, RESERVED63);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0: crate::Register = crate::Register::new(64, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0_CFG_BASE_ADDR_MEM_FUNC0: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1: crate::Register = crate::Register::new(65, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1_CFG_BASE_ADDR_MEM_FUNC1: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2: crate::Register = crate::Register::new(66, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2_CFG_BASE_ADDR_MEM_FUNC2: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3: crate::Register = crate::Register::new(67, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3_CFG_BASE_ADDR_MEM_FUNC3: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4: crate::Register = crate::Register::new(68, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4_CFG_BASE_ADDR_MEM_FUNC4: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5: crate::Register = crate::Register::new(69, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5_CFG_BASE_ADDR_MEM_FUNC5: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6: crate::Register = crate::Register::new(70, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6_CFG_BASE_ADDR_MEM_FUNC6: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7: crate::Register = crate::Register::new(71, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7_CFG_BASE_ADDR_MEM_FUNC7: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8: crate::Register = crate::Register::new(72, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8_CFG_BASE_ADDR_MEM_FUNC8: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9: crate::Register = crate::Register::new(73, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9_CFG_BASE_ADDR_MEM_FUNC9: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10: crate::Register = crate::Register::new(74, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10_CFG_BASE_ADDR_MEM_FUNC10: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11: crate::Register = crate::Register::new(75, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11_CFG_BASE_ADDR_MEM_FUNC11: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12: crate::Register = crate::Register::new(76, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12_CFG_BASE_ADDR_MEM_FUNC12: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13: crate::Register = crate::Register::new(77, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13_CFG_BASE_ADDR_MEM_FUNC13: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14: crate::Register = crate::Register::new(78, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14_CFG_BASE_ADDR_MEM_FUNC14: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15: crate::Register = crate::Register::new(79, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15_CFG_BASE_ADDR_MEM_FUNC15: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16: crate::Register = crate::Register::new(80, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16_CFG_BASE_ADDR_MEM_FUNC16: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16);

        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17: crate::Register = crate::Register::new(81, 0x3ffff);
        pub const CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17_CFG_BASE_ADDR_MEM_FUNC17: crate::Field = crate::Field::new(18, 0, CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0: crate::Register = crate::Register::new(82, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1: crate::Register = crate::Register::new(83, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2: crate::Register = crate::Register::new(84, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3: crate::Register = crate::Register::new(85, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4: crate::Register = crate::Register::new(86, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5: crate::Register = crate::Register::new(87, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5);

        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6: crate::Register = crate::Register::new(88, 0xff);
        pub const CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6);

        pub const RESERVED89: crate::Register = crate::Register::new(89, 0x1);
        pub const RESERVED89_RESERVED89: crate::Field = crate::Field::new(1, 0, RESERVED89);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0: crate::Register = crate::Register::new(90, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0_CFG_REG_FUNC_MANUFACT_CODE0: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1: crate::Register = crate::Register::new(91, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1_CFG_REG_FUNC_MANUFACT_CODE1: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2: crate::Register = crate::Register::new(92, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2_CFG_REG_FUNC_MANUFACT_CODE2: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3: crate::Register = crate::Register::new(93, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3_CFG_REG_FUNC_MANUFACT_CODE3: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4: crate::Register = crate::Register::new(94, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4_CFG_REG_FUNC_MANUFACT_CODE4: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5: crate::Register = crate::Register::new(95, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5_CFG_REG_FUNC_MANUFACT_CODE5: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5);

        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6: crate::Register = crate::Register::new(96, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6_CFG_REG_FUNC_MANUFACT_CODE6: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6);

        pub const RESERVED97: crate::Register = crate::Register::new(97, 0x1);
        pub const RESERVED97_RESERVED97: crate::Field = crate::Field::new(1, 0, RESERVED97);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0: crate::Register = crate::Register::new(98, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0_CFG_REG_FUNC_MANUFACT_INFO0: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1: crate::Register = crate::Register::new(99, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1_CFG_REG_FUNC_MANUFACT_INFO1: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2: crate::Register = crate::Register::new(100, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2_CFG_REG_FUNC_MANUFACT_INFO2: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3: crate::Register = crate::Register::new(101, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3_CFG_REG_FUNC_MANUFACT_INFO3: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4: crate::Register = crate::Register::new(102, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4_CFG_REG_FUNC_MANUFACT_INFO4: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5: crate::Register = crate::Register::new(103, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5_CFG_REG_FUNC_MANUFACT_INFO5: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5);

        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6: crate::Register = crate::Register::new(104, 0xffff);
        pub const CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6_CFG_REG_FUNC_MANUFACT_INFO6: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6);

        pub const RESERVED105: crate::Register = crate::Register::new(105, 0x1);
        pub const RESERVED105_RESERVED105: crate::Field = crate::Field::new(1, 0, RESERVED105);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0: crate::Register = crate::Register::new(106, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1: crate::Register = crate::Register::new(107, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2: crate::Register = crate::Register::new(108, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3: crate::Register = crate::Register::new(109, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4: crate::Register = crate::Register::new(110, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5: crate::Register = crate::Register::new(111, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5);

        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6: crate::Register = crate::Register::new(112, 0xff);
        pub const CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6: crate::Field = crate::Field::new(8, 0, CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6);

        pub const RESERVED113: crate::Register = crate::Register::new(113, 0x1);
        pub const RESERVED113_RESERVED113: crate::Field = crate::Field::new(1, 0, RESERVED113);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0: crate::Register = crate::Register::new(114, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0_CFG_REG_FUNC_INFO0: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1: crate::Register = crate::Register::new(115, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1_CFG_REG_FUNC_INFO1: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2: crate::Register = crate::Register::new(116, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2_CFG_REG_FUNC_INFO2: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3: crate::Register = crate::Register::new(117, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3_CFG_REG_FUNC_INFO3: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4: crate::Register = crate::Register::new(118, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4_CFG_REG_FUNC_INFO4: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5: crate::Register = crate::Register::new(119, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5_CFG_REG_FUNC_INFO5: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5);

        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6: crate::Register = crate::Register::new(120, 0xffff);
        pub const CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6_CFG_REG_FUNC_INFO6: crate::Field = crate::Field::new(16, 0, CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6);

        pub const RESERVED121: crate::Register = crate::Register::new(121, 0x1);
        pub const RESERVED121_RESERVED121: crate::Field = crate::Field::new(1, 0, RESERVED121);

        pub const RESERVED122: crate::Register = crate::Register::new(122, 0x1);
        pub const RESERVED122_RESERVED122: crate::Field = crate::Field::new(1, 0, RESERVED122);

        pub const RESERVED123: crate::Register = crate::Register::new(123, 0x1);
        pub const RESERVED123_RESERVED123: crate::Field = crate::Field::new(1, 0, RESERVED123);

        pub const CR_REG_UHS_1_SUPPORT: crate::Register = crate::Register::new(124, 0xffffffff);
        pub const CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT: crate::Field = crate::Field::new(16, 0, CR_REG_UHS_1_SUPPORT);
        pub const CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION: crate::Field = crate::Field::new(8, 16, CR_REG_UHS_1_SUPPORT);
        pub const CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT: crate::Field = crate::Field::new(8, 24, CR_REG_UHS_1_SUPPORT);

        pub const HW_SDDC_BASE: usize = 0x50121000;
    }

    pub mod mdma {
        pub const MDMA_NUMREGS: usize = 24;

        pub const SFR_EVSEL_CR_EVSEL0: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_EVSEL_CR_EVSEL0_CR_EVSEL0: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL0);

        pub const SFR_EVSEL_CR_EVSEL1: crate::Register = crate::Register::new(1, 0xff);
        pub const SFR_EVSEL_CR_EVSEL1_CR_EVSEL1: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL1);

        pub const SFR_EVSEL_CR_EVSEL2: crate::Register = crate::Register::new(2, 0xff);
        pub const SFR_EVSEL_CR_EVSEL2_CR_EVSEL2: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL2);

        pub const SFR_EVSEL_CR_EVSEL3: crate::Register = crate::Register::new(3, 0xff);
        pub const SFR_EVSEL_CR_EVSEL3_CR_EVSEL3: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL3);

        pub const SFR_EVSEL_CR_EVSEL4: crate::Register = crate::Register::new(4, 0xff);
        pub const SFR_EVSEL_CR_EVSEL4_CR_EVSEL4: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL4);

        pub const SFR_EVSEL_CR_EVSEL5: crate::Register = crate::Register::new(5, 0xff);
        pub const SFR_EVSEL_CR_EVSEL5_CR_EVSEL5: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL5);

        pub const SFR_EVSEL_CR_EVSEL6: crate::Register = crate::Register::new(6, 0xff);
        pub const SFR_EVSEL_CR_EVSEL6_CR_EVSEL6: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL6);

        pub const SFR_EVSEL_CR_EVSEL7: crate::Register = crate::Register::new(7, 0xff);
        pub const SFR_EVSEL_CR_EVSEL7_CR_EVSEL7: crate::Field = crate::Field::new(8, 0, SFR_EVSEL_CR_EVSEL7);

        pub const SFR_CR_CR_MDMAREQ0: crate::Register = crate::Register::new(8, 0x1f);
        pub const SFR_CR_CR_MDMAREQ0_CR_MDMAREQ0: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ0);

        pub const SFR_CR_CR_MDMAREQ1: crate::Register = crate::Register::new(9, 0x1f);
        pub const SFR_CR_CR_MDMAREQ1_CR_MDMAREQ1: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ1);

        pub const SFR_CR_CR_MDMAREQ2: crate::Register = crate::Register::new(10, 0x1f);
        pub const SFR_CR_CR_MDMAREQ2_CR_MDMAREQ2: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ2);

        pub const SFR_CR_CR_MDMAREQ3: crate::Register = crate::Register::new(11, 0x1f);
        pub const SFR_CR_CR_MDMAREQ3_CR_MDMAREQ3: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ3);

        pub const SFR_CR_CR_MDMAREQ4: crate::Register = crate::Register::new(12, 0x1f);
        pub const SFR_CR_CR_MDMAREQ4_CR_MDMAREQ4: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ4);

        pub const SFR_CR_CR_MDMAREQ5: crate::Register = crate::Register::new(13, 0x1f);
        pub const SFR_CR_CR_MDMAREQ5_CR_MDMAREQ5: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ5);

        pub const SFR_CR_CR_MDMAREQ6: crate::Register = crate::Register::new(14, 0x1f);
        pub const SFR_CR_CR_MDMAREQ6_CR_MDMAREQ6: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ6);

        pub const SFR_CR_CR_MDMAREQ7: crate::Register = crate::Register::new(15, 0x1f);
        pub const SFR_CR_CR_MDMAREQ7_CR_MDMAREQ7: crate::Field = crate::Field::new(5, 0, SFR_CR_CR_MDMAREQ7);

        pub const SFR_SR_SR_MDMAREQ0: crate::Register = crate::Register::new(16, 0x1f);
        pub const SFR_SR_SR_MDMAREQ0_SR_MDMAREQ0: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ0);

        pub const SFR_SR_SR_MDMAREQ1: crate::Register = crate::Register::new(17, 0x1f);
        pub const SFR_SR_SR_MDMAREQ1_SR_MDMAREQ1: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ1);

        pub const SFR_SR_SR_MDMAREQ2: crate::Register = crate::Register::new(18, 0x1f);
        pub const SFR_SR_SR_MDMAREQ2_SR_MDMAREQ2: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ2);

        pub const SFR_SR_SR_MDMAREQ3: crate::Register = crate::Register::new(19, 0x1f);
        pub const SFR_SR_SR_MDMAREQ3_SR_MDMAREQ3: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ3);

        pub const SFR_SR_SR_MDMAREQ4: crate::Register = crate::Register::new(20, 0x1f);
        pub const SFR_SR_SR_MDMAREQ4_SR_MDMAREQ4: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ4);

        pub const SFR_SR_SR_MDMAREQ5: crate::Register = crate::Register::new(21, 0x1f);
        pub const SFR_SR_SR_MDMAREQ5_SR_MDMAREQ5: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ5);

        pub const SFR_SR_SR_MDMAREQ6: crate::Register = crate::Register::new(22, 0x1f);
        pub const SFR_SR_SR_MDMAREQ6_SR_MDMAREQ6: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ6);

        pub const SFR_SR_SR_MDMAREQ7: crate::Register = crate::Register::new(23, 0x1f);
        pub const SFR_SR_SR_MDMAREQ7_SR_MDMAREQ7: crate::Field = crate::Field::new(5, 0, SFR_SR_SR_MDMAREQ7);

        pub const HW_MDMA_BASE: usize = 0x40002000;
    }

    pub mod qfc {
        pub const QFC_NUMREGS: usize = 10;

        pub const SFR_IO: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_IO_SFR_IO: crate::Field = crate::Field::new(8, 0, SFR_IO);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const SFR_IODRV: crate::Register = crate::Register::new(2, 0xfff);
        pub const SFR_IODRV_PADDRVSEL: crate::Field = crate::Field::new(12, 0, SFR_IODRV);

        pub const RESERVED3: crate::Register = crate::Register::new(3, 0x1);
        pub const RESERVED3_RESERVED3: crate::Field = crate::Field::new(1, 0, RESERVED3);

        pub const CR_XIP_ADDRMODE: crate::Register = crate::Register::new(4, 0x3);
        pub const CR_XIP_ADDRMODE_CR_XIP_ADDRMODE: crate::Field = crate::Field::new(2, 0, CR_XIP_ADDRMODE);

        pub const CR_XIP_OPCODE: crate::Register = crate::Register::new(5, 0xffffffff);
        pub const CR_XIP_OPCODE_CR_XIP_OPCODE: crate::Field = crate::Field::new(32, 0, CR_XIP_OPCODE);

        pub const CR_XIP_WIDTH: crate::Register = crate::Register::new(6, 0x3f);
        pub const CR_XIP_WIDTH_CR_XIP_WIDTH: crate::Field = crate::Field::new(6, 0, CR_XIP_WIDTH);

        pub const CR_XIP_SSEL: crate::Register = crate::Register::new(7, 0x7f);
        pub const CR_XIP_SSEL_CR_XIP_SSEL: crate::Field = crate::Field::new(7, 0, CR_XIP_SSEL);

        pub const CR_XIP_DUMCYC: crate::Register = crate::Register::new(8, 0xffff);
        pub const CR_XIP_DUMCYC_CR_XIP_DUMCYC: crate::Field = crate::Field::new(16, 0, CR_XIP_DUMCYC);

        pub const CR_XIP_CFG: crate::Register = crate::Register::new(9, 0x3fff);
        pub const CR_XIP_CFG_CR_XIP_CFG: crate::Field = crate::Field::new(14, 0, CR_XIP_CFG);

        pub const HW_QFC_BASE: usize = 0x40000000;
    }

    pub mod pl230 {
        pub const PL230_NUMREGS: usize = 1;

        pub const PL230: crate::Register = crate::Register::new(0, 0xffffffff);
        pub const PL230_PLACEHOLDER: crate::Field = crate::Field::new(32, 0, PL230);

        pub const HW_PL230_BASE: usize = 0x40001000;
    }

    pub mod gluechain {
        pub const GLUECHAIN_NUMREGS: usize = 4;

        pub const SFR_GCMASK: crate::Register = crate::Register::new(0, 0xffffffff);
        pub const SFR_GCMASK_CR_GCMASK: crate::Field = crate::Field::new(32, 0, SFR_GCMASK);

        pub const SFR_GCSR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_GCSR_GLUEREG: crate::Field = crate::Field::new(32, 0, SFR_GCSR);

        pub const SFR_GCRST: crate::Register = crate::Register::new(2, 0xffffffff);
        pub const SFR_GCRST_GLUERST: crate::Field = crate::Field::new(32, 0, SFR_GCRST);

        pub const SFR_GCTEST: crate::Register = crate::Register::new(3, 0xffffffff);
        pub const SFR_GCTEST_GLUETEST: crate::Field = crate::Field::new(32, 0, SFR_GCTEST);

        pub const HW_GLUECHAIN_BASE: usize = 0x40054000;
    }

    pub mod mesh {
        pub const MESH_NUMREGS: usize = 10;

        pub const SFR_MLDRV_CR_MLDRV0: crate::Register = crate::Register::new(0, 0xffffffff);
        pub const SFR_MLDRV_CR_MLDRV0_CR_MLDRV0: crate::Field = crate::Field::new(32, 0, SFR_MLDRV_CR_MLDRV0);

        pub const SFR_MLIE_CR_MLIE0: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_MLIE_CR_MLIE0_CR_MLIE0: crate::Field = crate::Field::new(32, 0, SFR_MLIE_CR_MLIE0);

        pub const SFR_MLSR_SR_MLSR0: crate::Register = crate::Register::new(2, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR0_SR_MLSR0: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR0);

        pub const SFR_MLSR_SR_MLSR1: crate::Register = crate::Register::new(3, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR1_SR_MLSR1: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR1);

        pub const SFR_MLSR_SR_MLSR2: crate::Register = crate::Register::new(4, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR2_SR_MLSR2: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR2);

        pub const SFR_MLSR_SR_MLSR3: crate::Register = crate::Register::new(5, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR3_SR_MLSR3: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR3);

        pub const SFR_MLSR_SR_MLSR4: crate::Register = crate::Register::new(6, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR4_SR_MLSR4: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR4);

        pub const SFR_MLSR_SR_MLSR5: crate::Register = crate::Register::new(7, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR5_SR_MLSR5: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR5);

        pub const SFR_MLSR_SR_MLSR6: crate::Register = crate::Register::new(8, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR6_SR_MLSR6: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR6);

        pub const SFR_MLSR_SR_MLSR7: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_MLSR_SR_MLSR7_SR_MLSR7: crate::Field = crate::Field::new(32, 0, SFR_MLSR_SR_MLSR7);

        pub const HW_MESH_BASE: usize = 0x40052000;
    }

    pub mod sensorc {
        pub const SENSORC_NUMREGS: usize = 16;

        pub const SFR_VDMASK0: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_VDMASK0_CR_VDMASK0: crate::Field = crate::Field::new(8, 0, SFR_VDMASK0);

        pub const SFR_VDMASK1: crate::Register = crate::Register::new(1, 0xff);
        pub const SFR_VDMASK1_CR_VDMASK1: crate::Field = crate::Field::new(8, 0, SFR_VDMASK1);

        pub const SFR_VDSR: crate::Register = crate::Register::new(2, 0xff);
        pub const SFR_VDSR_SR_VDSR: crate::Field = crate::Field::new(8, 0, SFR_VDSR);

        pub const RESERVED3: crate::Register = crate::Register::new(3, 0x1);
        pub const RESERVED3_RESERVED3: crate::Field = crate::Field::new(1, 0, RESERVED3);

        pub const SFR_LDMASK: crate::Register = crate::Register::new(4, 0xf);
        pub const SFR_LDMASK_CR_LDMASK: crate::Field = crate::Field::new(4, 0, SFR_LDMASK);

        pub const SFR_LDSR: crate::Register = crate::Register::new(5, 0xf);
        pub const SFR_LDSR_SR_LDSR: crate::Field = crate::Field::new(4, 0, SFR_LDSR);

        pub const SFR_LDCFG: crate::Register = crate::Register::new(6, 0xf);
        pub const SFR_LDCFG_SFR_LDCFG: crate::Field = crate::Field::new(4, 0, SFR_LDCFG);

        pub const RESERVED7: crate::Register = crate::Register::new(7, 0x1);
        pub const RESERVED7_RESERVED7: crate::Field = crate::Field::new(1, 0, RESERVED7);

        pub const SFR_VDCFG_CR_VDCFG0: crate::Register = crate::Register::new(8, 0xf);
        pub const SFR_VDCFG_CR_VDCFG0_CR_VDCFG0: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG0);

        pub const SFR_VDCFG_CR_VDCFG1: crate::Register = crate::Register::new(9, 0xf);
        pub const SFR_VDCFG_CR_VDCFG1_CR_VDCFG1: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG1);

        pub const SFR_VDCFG_CR_VDCFG2: crate::Register = crate::Register::new(10, 0xf);
        pub const SFR_VDCFG_CR_VDCFG2_CR_VDCFG2: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG2);

        pub const SFR_VDCFG_CR_VDCFG3: crate::Register = crate::Register::new(11, 0xf);
        pub const SFR_VDCFG_CR_VDCFG3_CR_VDCFG3: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG3);

        pub const SFR_VDCFG_CR_VDCFG4: crate::Register = crate::Register::new(12, 0xf);
        pub const SFR_VDCFG_CR_VDCFG4_CR_VDCFG4: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG4);

        pub const SFR_VDCFG_CR_VDCFG5: crate::Register = crate::Register::new(13, 0xf);
        pub const SFR_VDCFG_CR_VDCFG5_CR_VDCFG5: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG5);

        pub const SFR_VDCFG_CR_VDCFG6: crate::Register = crate::Register::new(14, 0xf);
        pub const SFR_VDCFG_CR_VDCFG6_CR_VDCFG6: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG6);

        pub const SFR_VDCFG_CR_VDCFG7: crate::Register = crate::Register::new(15, 0xf);
        pub const SFR_VDCFG_CR_VDCFG7_CR_VDCFG7: crate::Field = crate::Field::new(4, 0, SFR_VDCFG_CR_VDCFG7);

        pub const HW_SENSORC_BASE: usize = 0x40053000;
    }
}

// Litex auto-generated constants


#[cfg(test)]
mod tests {

    #[test]
    #[ignore]
    fn compile_check_aes_csr() {
        use super::*;
        let mut aes_csr = CSR::new(HW_AES_BASE as *mut u32);

        let foo = aes_csr.r(utra::aes::SFR_CRFUNC);
        aes_csr.wo(utra::aes::SFR_CRFUNC, foo);
        let bar = aes_csr.rf(utra::aes::SFR_CRFUNC_SFR_CRFUNC);
        aes_csr.rmwf(utra::aes::SFR_CRFUNC_SFR_CRFUNC, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_CRFUNC_SFR_CRFUNC, bar);
        baz |= aes_csr.ms(utra::aes::SFR_CRFUNC_SFR_CRFUNC, 1);
        aes_csr.wfo(utra::aes::SFR_CRFUNC_SFR_CRFUNC, baz);

        let foo = aes_csr.r(utra::aes::SFR_AR);
        aes_csr.wo(utra::aes::SFR_AR, foo);
        let bar = aes_csr.rf(utra::aes::SFR_AR_SFR_AR);
        aes_csr.rmwf(utra::aes::SFR_AR_SFR_AR, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_AR_SFR_AR, bar);
        baz |= aes_csr.ms(utra::aes::SFR_AR_SFR_AR, 1);
        aes_csr.wfo(utra::aes::SFR_AR_SFR_AR, baz);

        let foo = aes_csr.r(utra::aes::SFR_SRMFSM);
        aes_csr.wo(utra::aes::SFR_SRMFSM, foo);
        let bar = aes_csr.rf(utra::aes::SFR_SRMFSM_SFR_SRMFSM);
        aes_csr.rmwf(utra::aes::SFR_SRMFSM_SFR_SRMFSM, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_SRMFSM_SFR_SRMFSM, bar);
        baz |= aes_csr.ms(utra::aes::SFR_SRMFSM_SFR_SRMFSM, 1);
        aes_csr.wfo(utra::aes::SFR_SRMFSM_SFR_SRMFSM, baz);

        let foo = aes_csr.r(utra::aes::SFR_FR);
        aes_csr.wo(utra::aes::SFR_FR, foo);
        let bar = aes_csr.rf(utra::aes::SFR_FR_MFSM_DONE);
        aes_csr.rmwf(utra::aes::SFR_FR_MFSM_DONE, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_FR_MFSM_DONE, bar);
        baz |= aes_csr.ms(utra::aes::SFR_FR_MFSM_DONE, 1);
        aes_csr.wfo(utra::aes::SFR_FR_MFSM_DONE, baz);
        let bar = aes_csr.rf(utra::aes::SFR_FR_ACORE_DONE);
        aes_csr.rmwf(utra::aes::SFR_FR_ACORE_DONE, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_FR_ACORE_DONE, bar);
        baz |= aes_csr.ms(utra::aes::SFR_FR_ACORE_DONE, 1);
        aes_csr.wfo(utra::aes::SFR_FR_ACORE_DONE, baz);
        let bar = aes_csr.rf(utra::aes::SFR_FR_CHNLO_DONE);
        aes_csr.rmwf(utra::aes::SFR_FR_CHNLO_DONE, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_FR_CHNLO_DONE, bar);
        baz |= aes_csr.ms(utra::aes::SFR_FR_CHNLO_DONE, 1);
        aes_csr.wfo(utra::aes::SFR_FR_CHNLO_DONE, baz);
        let bar = aes_csr.rf(utra::aes::SFR_FR_CHNLI_DONE);
        aes_csr.rmwf(utra::aes::SFR_FR_CHNLI_DONE, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_FR_CHNLI_DONE, bar);
        baz |= aes_csr.ms(utra::aes::SFR_FR_CHNLI_DONE, 1);
        aes_csr.wfo(utra::aes::SFR_FR_CHNLI_DONE, baz);

        let foo = aes_csr.r(utra::aes::SFR_OPT);
        aes_csr.wo(utra::aes::SFR_OPT, foo);
        let bar = aes_csr.rf(utra::aes::SFR_OPT_OPT_KLEN0);
        aes_csr.rmwf(utra::aes::SFR_OPT_OPT_KLEN0, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_OPT_OPT_KLEN0, bar);
        baz |= aes_csr.ms(utra::aes::SFR_OPT_OPT_KLEN0, 1);
        aes_csr.wfo(utra::aes::SFR_OPT_OPT_KLEN0, baz);
        let bar = aes_csr.rf(utra::aes::SFR_OPT_OPT_MODE0);
        aes_csr.rmwf(utra::aes::SFR_OPT_OPT_MODE0, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_OPT_OPT_MODE0, bar);
        baz |= aes_csr.ms(utra::aes::SFR_OPT_OPT_MODE0, 1);
        aes_csr.wfo(utra::aes::SFR_OPT_OPT_MODE0, baz);
        let bar = aes_csr.rf(utra::aes::SFR_OPT_OPT_IFSTART0);
        aes_csr.rmwf(utra::aes::SFR_OPT_OPT_IFSTART0, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_OPT_OPT_IFSTART0, bar);
        baz |= aes_csr.ms(utra::aes::SFR_OPT_OPT_IFSTART0, 1);
        aes_csr.wfo(utra::aes::SFR_OPT_OPT_IFSTART0, baz);

        let foo = aes_csr.r(utra::aes::SFR_OPT1);
        aes_csr.wo(utra::aes::SFR_OPT1, foo);
        let bar = aes_csr.rf(utra::aes::SFR_OPT1_SFR_OPT1);
        aes_csr.rmwf(utra::aes::SFR_OPT1_SFR_OPT1, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_OPT1_SFR_OPT1, bar);
        baz |= aes_csr.ms(utra::aes::SFR_OPT1_SFR_OPT1, 1);
        aes_csr.wfo(utra::aes::SFR_OPT1_SFR_OPT1, baz);

        let foo = aes_csr.r(utra::aes::RESERVED6);
        aes_csr.wo(utra::aes::RESERVED6, foo);
        let bar = aes_csr.rf(utra::aes::RESERVED6_RESERVED6);
        aes_csr.rmwf(utra::aes::RESERVED6_RESERVED6, bar);
        let mut baz = aes_csr.zf(utra::aes::RESERVED6_RESERVED6, bar);
        baz |= aes_csr.ms(utra::aes::RESERVED6_RESERVED6, 1);
        aes_csr.wfo(utra::aes::RESERVED6_RESERVED6, baz);

        let foo = aes_csr.r(utra::aes::RESERVED7);
        aes_csr.wo(utra::aes::RESERVED7, foo);
        let bar = aes_csr.rf(utra::aes::RESERVED7_RESERVED7);
        aes_csr.rmwf(utra::aes::RESERVED7_RESERVED7, bar);
        let mut baz = aes_csr.zf(utra::aes::RESERVED7_RESERVED7, bar);
        baz |= aes_csr.ms(utra::aes::RESERVED7_RESERVED7, 1);
        aes_csr.wfo(utra::aes::RESERVED7_RESERVED7, baz);

        let foo = aes_csr.r(utra::aes::RESERVED8);
        aes_csr.wo(utra::aes::RESERVED8, foo);
        let bar = aes_csr.rf(utra::aes::RESERVED8_RESERVED8);
        aes_csr.rmwf(utra::aes::RESERVED8_RESERVED8, bar);
        let mut baz = aes_csr.zf(utra::aes::RESERVED8_RESERVED8, bar);
        baz |= aes_csr.ms(utra::aes::RESERVED8_RESERVED8, 1);
        aes_csr.wfo(utra::aes::RESERVED8_RESERVED8, baz);

        let foo = aes_csr.r(utra::aes::RESERVED9);
        aes_csr.wo(utra::aes::RESERVED9, foo);
        let bar = aes_csr.rf(utra::aes::RESERVED9_RESERVED9);
        aes_csr.rmwf(utra::aes::RESERVED9_RESERVED9, bar);
        let mut baz = aes_csr.zf(utra::aes::RESERVED9_RESERVED9, bar);
        baz |= aes_csr.ms(utra::aes::RESERVED9_RESERVED9, 1);
        aes_csr.wfo(utra::aes::RESERVED9_RESERVED9, baz);

        let foo = aes_csr.r(utra::aes::RESERVED10);
        aes_csr.wo(utra::aes::RESERVED10, foo);
        let bar = aes_csr.rf(utra::aes::RESERVED10_RESERVED10);
        aes_csr.rmwf(utra::aes::RESERVED10_RESERVED10, bar);
        let mut baz = aes_csr.zf(utra::aes::RESERVED10_RESERVED10, bar);
        baz |= aes_csr.ms(utra::aes::RESERVED10_RESERVED10, 1);
        aes_csr.wfo(utra::aes::RESERVED10_RESERVED10, baz);

        let foo = aes_csr.r(utra::aes::RESERVED11);
        aes_csr.wo(utra::aes::RESERVED11, foo);
        let bar = aes_csr.rf(utra::aes::RESERVED11_RESERVED11);
        aes_csr.rmwf(utra::aes::RESERVED11_RESERVED11, bar);
        let mut baz = aes_csr.zf(utra::aes::RESERVED11_RESERVED11, bar);
        baz |= aes_csr.ms(utra::aes::RESERVED11_RESERVED11, 1);
        aes_csr.wfo(utra::aes::RESERVED11_RESERVED11, baz);

        let foo = aes_csr.r(utra::aes::SFR_SEGPTR_PTRID_IV);
        aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_IV, foo);
        let bar = aes_csr.rf(utra::aes::SFR_SEGPTR_PTRID_IV_PTRID_IV);
        aes_csr.rmwf(utra::aes::SFR_SEGPTR_PTRID_IV_PTRID_IV, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_SEGPTR_PTRID_IV_PTRID_IV, bar);
        baz |= aes_csr.ms(utra::aes::SFR_SEGPTR_PTRID_IV_PTRID_IV, 1);
        aes_csr.wfo(utra::aes::SFR_SEGPTR_PTRID_IV_PTRID_IV, baz);

        let foo = aes_csr.r(utra::aes::SFR_SEGPTR_PTRID_AKEY);
        aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_AKEY, foo);
        let bar = aes_csr.rf(utra::aes::SFR_SEGPTR_PTRID_AKEY_PTRID_AKEY);
        aes_csr.rmwf(utra::aes::SFR_SEGPTR_PTRID_AKEY_PTRID_AKEY, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_SEGPTR_PTRID_AKEY_PTRID_AKEY, bar);
        baz |= aes_csr.ms(utra::aes::SFR_SEGPTR_PTRID_AKEY_PTRID_AKEY, 1);
        aes_csr.wfo(utra::aes::SFR_SEGPTR_PTRID_AKEY_PTRID_AKEY, baz);

        let foo = aes_csr.r(utra::aes::SFR_SEGPTR_PTRID_AIB);
        aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_AIB, foo);
        let bar = aes_csr.rf(utra::aes::SFR_SEGPTR_PTRID_AIB_PTRID_AIB);
        aes_csr.rmwf(utra::aes::SFR_SEGPTR_PTRID_AIB_PTRID_AIB, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_SEGPTR_PTRID_AIB_PTRID_AIB, bar);
        baz |= aes_csr.ms(utra::aes::SFR_SEGPTR_PTRID_AIB_PTRID_AIB, 1);
        aes_csr.wfo(utra::aes::SFR_SEGPTR_PTRID_AIB_PTRID_AIB, baz);

        let foo = aes_csr.r(utra::aes::SFR_SEGPTR_PTRID_AOB);
        aes_csr.wo(utra::aes::SFR_SEGPTR_PTRID_AOB, foo);
        let bar = aes_csr.rf(utra::aes::SFR_SEGPTR_PTRID_AOB_PTRID_AOB);
        aes_csr.rmwf(utra::aes::SFR_SEGPTR_PTRID_AOB_PTRID_AOB, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_SEGPTR_PTRID_AOB_PTRID_AOB, bar);
        baz |= aes_csr.ms(utra::aes::SFR_SEGPTR_PTRID_AOB_PTRID_AOB, 1);
        aes_csr.wfo(utra::aes::SFR_SEGPTR_PTRID_AOB_PTRID_AOB, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_combohash_csr() {
        use super::*;
        let mut combohash_csr = CSR::new(HW_COMBOHASH_BASE as *mut u32);

        let foo = combohash_csr.r(utra::combohash::SFR_CRFUNC);
        combohash_csr.wo(utra::combohash::SFR_CRFUNC, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_CRFUNC_CR_FUNC);
        combohash_csr.rmwf(utra::combohash::SFR_CRFUNC_CR_FUNC, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_CRFUNC_CR_FUNC, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_CRFUNC_CR_FUNC, 1);
        combohash_csr.wfo(utra::combohash::SFR_CRFUNC_CR_FUNC, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_AR);
        combohash_csr.wo(utra::combohash::SFR_AR, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_AR_SFR_AR);
        combohash_csr.rmwf(utra::combohash::SFR_AR_SFR_AR, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_AR_SFR_AR, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_AR_SFR_AR, 1);
        combohash_csr.wfo(utra::combohash::SFR_AR_SFR_AR, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SRMFSM);
        combohash_csr.wo(utra::combohash::SFR_SRMFSM, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SRMFSM_MFSM);
        combohash_csr.rmwf(utra::combohash::SFR_SRMFSM_MFSM, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SRMFSM_MFSM, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SRMFSM_MFSM, 1);
        combohash_csr.wfo(utra::combohash::SFR_SRMFSM_MFSM, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_FR);
        combohash_csr.wo(utra::combohash::SFR_FR, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_FR_MFSM_DONE);
        combohash_csr.rmwf(utra::combohash::SFR_FR_MFSM_DONE, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_FR_MFSM_DONE, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_FR_MFSM_DONE, 1);
        combohash_csr.wfo(utra::combohash::SFR_FR_MFSM_DONE, baz);
        let bar = combohash_csr.rf(utra::combohash::SFR_FR_HASH_DONE);
        combohash_csr.rmwf(utra::combohash::SFR_FR_HASH_DONE, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_FR_HASH_DONE, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_FR_HASH_DONE, 1);
        combohash_csr.wfo(utra::combohash::SFR_FR_HASH_DONE, baz);
        let bar = combohash_csr.rf(utra::combohash::SFR_FR_CHNLO_DONE);
        combohash_csr.rmwf(utra::combohash::SFR_FR_CHNLO_DONE, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_FR_CHNLO_DONE, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_FR_CHNLO_DONE, 1);
        combohash_csr.wfo(utra::combohash::SFR_FR_CHNLO_DONE, baz);
        let bar = combohash_csr.rf(utra::combohash::SFR_FR_CHNLI_DONE);
        combohash_csr.rmwf(utra::combohash::SFR_FR_CHNLI_DONE, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_FR_CHNLI_DONE, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_FR_CHNLI_DONE, 1);
        combohash_csr.wfo(utra::combohash::SFR_FR_CHNLI_DONE, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_OPT1);
        combohash_csr.wo(utra::combohash::SFR_OPT1, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_OPT1_CR_OPT_HASHCNT);
        combohash_csr.rmwf(utra::combohash::SFR_OPT1_CR_OPT_HASHCNT, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_OPT1_CR_OPT_HASHCNT, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_OPT1_CR_OPT_HASHCNT, 1);
        combohash_csr.wfo(utra::combohash::SFR_OPT1_CR_OPT_HASHCNT, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_OPT2);
        combohash_csr.wo(utra::combohash::SFR_OPT2, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_OPT2_CR_OPT_SCRTCHK);
        combohash_csr.rmwf(utra::combohash::SFR_OPT2_CR_OPT_SCRTCHK, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_OPT2_CR_OPT_SCRTCHK, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_OPT2_CR_OPT_SCRTCHK, 1);
        combohash_csr.wfo(utra::combohash::SFR_OPT2_CR_OPT_SCRTCHK, baz);
        let bar = combohash_csr.rf(utra::combohash::SFR_OPT2_CR_OPT_IFSOB);
        combohash_csr.rmwf(utra::combohash::SFR_OPT2_CR_OPT_IFSOB, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_OPT2_CR_OPT_IFSOB, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_OPT2_CR_OPT_IFSOB, 1);
        combohash_csr.wfo(utra::combohash::SFR_OPT2_CR_OPT_IFSOB, baz);
        let bar = combohash_csr.rf(utra::combohash::SFR_OPT2_CR_OPT_IFSTART);
        combohash_csr.rmwf(utra::combohash::SFR_OPT2_CR_OPT_IFSTART, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_OPT2_CR_OPT_IFSTART, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_OPT2_CR_OPT_IFSTART, 1);
        combohash_csr.wfo(utra::combohash::SFR_OPT2_CR_OPT_IFSTART, baz);

        let foo = combohash_csr.r(utra::combohash::RESERVED6);
        combohash_csr.wo(utra::combohash::RESERVED6, foo);
        let bar = combohash_csr.rf(utra::combohash::RESERVED6_RESERVED6);
        combohash_csr.rmwf(utra::combohash::RESERVED6_RESERVED6, bar);
        let mut baz = combohash_csr.zf(utra::combohash::RESERVED6_RESERVED6, bar);
        baz |= combohash_csr.ms(utra::combohash::RESERVED6_RESERVED6, 1);
        combohash_csr.wfo(utra::combohash::RESERVED6_RESERVED6, baz);

        let foo = combohash_csr.r(utra::combohash::RESERVED7);
        combohash_csr.wo(utra::combohash::RESERVED7, foo);
        let bar = combohash_csr.rf(utra::combohash::RESERVED7_RESERVED7);
        combohash_csr.rmwf(utra::combohash::RESERVED7_RESERVED7, bar);
        let mut baz = combohash_csr.zf(utra::combohash::RESERVED7_RESERVED7, bar);
        baz |= combohash_csr.ms(utra::combohash::RESERVED7_RESERVED7, 1);
        combohash_csr.wfo(utra::combohash::RESERVED7_RESERVED7, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_LKEY);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_LKEY, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_KEY);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_KEY, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_KEY_SEGID_KEY);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_KEY_SEGID_KEY, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_KEY_SEGID_KEY, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_KEY_SEGID_KEY, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_KEY_SEGID_KEY, baz);

        let foo = combohash_csr.r(utra::combohash::RESERVED10);
        combohash_csr.wo(utra::combohash::RESERVED10, foo);
        let bar = combohash_csr.rf(utra::combohash::RESERVED10_RESERVED10);
        combohash_csr.rmwf(utra::combohash::RESERVED10_RESERVED10, bar);
        let mut baz = combohash_csr.zf(utra::combohash::RESERVED10_RESERVED10, bar);
        baz |= combohash_csr.ms(utra::combohash::RESERVED10_RESERVED10, 1);
        combohash_csr.wfo(utra::combohash::RESERVED10_RESERVED10, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_SCRT);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_SCRT, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_MSG);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_MSG, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_MSG_SEGID_MSG);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_MSG_SEGID_MSG, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_MSG_SEGID_MSG, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_MSG_SEGID_MSG, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_MSG_SEGID_MSG, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_HOUT);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_HOUT, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_SOB);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_SOB, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_SOB_SEGID_SOB);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_SOB_SEGID_SOB, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_SOB_SEGID_SOB, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_SOB_SEGID_SOB, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_SOB_SEGID_SOB, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_pke_csr() {
        use super::*;
        let mut pke_csr = CSR::new(HW_PKE_BASE as *mut u32);

        let foo = pke_csr.r(utra::pke::SFR_CRFUNC);
        pke_csr.wo(utra::pke::SFR_CRFUNC, foo);
        let bar = pke_csr.rf(utra::pke::SFR_CRFUNC_SFR_CRFUNC);
        pke_csr.rmwf(utra::pke::SFR_CRFUNC_SFR_CRFUNC, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_CRFUNC_SFR_CRFUNC, bar);
        baz |= pke_csr.ms(utra::pke::SFR_CRFUNC_SFR_CRFUNC, 1);
        pke_csr.wfo(utra::pke::SFR_CRFUNC_SFR_CRFUNC, baz);

        let foo = pke_csr.r(utra::pke::SFR_AR);
        pke_csr.wo(utra::pke::SFR_AR, foo);
        let bar = pke_csr.rf(utra::pke::SFR_AR_SFR_AR);
        pke_csr.rmwf(utra::pke::SFR_AR_SFR_AR, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_AR_SFR_AR, bar);
        baz |= pke_csr.ms(utra::pke::SFR_AR_SFR_AR, 1);
        pke_csr.wfo(utra::pke::SFR_AR_SFR_AR, baz);

        let foo = pke_csr.r(utra::pke::SFR_SRMFSM);
        pke_csr.wo(utra::pke::SFR_SRMFSM, foo);
        let bar = pke_csr.rf(utra::pke::SFR_SRMFSM_MFSM);
        pke_csr.rmwf(utra::pke::SFR_SRMFSM_MFSM, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SRMFSM_MFSM, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SRMFSM_MFSM, 1);
        pke_csr.wfo(utra::pke::SFR_SRMFSM_MFSM, baz);
        let bar = pke_csr.rf(utra::pke::SFR_SRMFSM_MODINVREADY);
        pke_csr.rmwf(utra::pke::SFR_SRMFSM_MODINVREADY, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SRMFSM_MODINVREADY, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SRMFSM_MODINVREADY, 1);
        pke_csr.wfo(utra::pke::SFR_SRMFSM_MODINVREADY, baz);

        let foo = pke_csr.r(utra::pke::SFR_FR);
        pke_csr.wo(utra::pke::SFR_FR, foo);
        let bar = pke_csr.rf(utra::pke::SFR_FR_MFSM_DONE);
        pke_csr.rmwf(utra::pke::SFR_FR_MFSM_DONE, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_FR_MFSM_DONE, bar);
        baz |= pke_csr.ms(utra::pke::SFR_FR_MFSM_DONE, 1);
        pke_csr.wfo(utra::pke::SFR_FR_MFSM_DONE, baz);
        let bar = pke_csr.rf(utra::pke::SFR_FR_PCORE_DONE);
        pke_csr.rmwf(utra::pke::SFR_FR_PCORE_DONE, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_FR_PCORE_DONE, bar);
        baz |= pke_csr.ms(utra::pke::SFR_FR_PCORE_DONE, 1);
        pke_csr.wfo(utra::pke::SFR_FR_PCORE_DONE, baz);
        let bar = pke_csr.rf(utra::pke::SFR_FR_CHNLO_DONE);
        pke_csr.rmwf(utra::pke::SFR_FR_CHNLO_DONE, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_FR_CHNLO_DONE, bar);
        baz |= pke_csr.ms(utra::pke::SFR_FR_CHNLO_DONE, 1);
        pke_csr.wfo(utra::pke::SFR_FR_CHNLO_DONE, baz);
        let bar = pke_csr.rf(utra::pke::SFR_FR_CHNLI_DONE);
        pke_csr.rmwf(utra::pke::SFR_FR_CHNLI_DONE, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_FR_CHNLI_DONE, bar);
        baz |= pke_csr.ms(utra::pke::SFR_FR_CHNLI_DONE, 1);
        pke_csr.wfo(utra::pke::SFR_FR_CHNLI_DONE, baz);
        let bar = pke_csr.rf(utra::pke::SFR_FR_CHNLX_DONE);
        pke_csr.rmwf(utra::pke::SFR_FR_CHNLX_DONE, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_FR_CHNLX_DONE, bar);
        baz |= pke_csr.ms(utra::pke::SFR_FR_CHNLX_DONE, 1);
        pke_csr.wfo(utra::pke::SFR_FR_CHNLX_DONE, baz);

        let foo = pke_csr.r(utra::pke::SFR_OPTNW);
        pke_csr.wo(utra::pke::SFR_OPTNW, foo);
        let bar = pke_csr.rf(utra::pke::SFR_OPTNW_SFR_OPTNW);
        pke_csr.rmwf(utra::pke::SFR_OPTNW_SFR_OPTNW, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_OPTNW_SFR_OPTNW, bar);
        baz |= pke_csr.ms(utra::pke::SFR_OPTNW_SFR_OPTNW, 1);
        pke_csr.wfo(utra::pke::SFR_OPTNW_SFR_OPTNW, baz);

        let foo = pke_csr.r(utra::pke::SFR_OPTEW);
        pke_csr.wo(utra::pke::SFR_OPTEW, foo);
        let bar = pke_csr.rf(utra::pke::SFR_OPTEW_SFR_OPTEW);
        pke_csr.rmwf(utra::pke::SFR_OPTEW_SFR_OPTEW, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_OPTEW_SFR_OPTEW, bar);
        baz |= pke_csr.ms(utra::pke::SFR_OPTEW_SFR_OPTEW, 1);
        pke_csr.wfo(utra::pke::SFR_OPTEW_SFR_OPTEW, baz);

        let foo = pke_csr.r(utra::pke::RESERVED6);
        pke_csr.wo(utra::pke::RESERVED6, foo);
        let bar = pke_csr.rf(utra::pke::RESERVED6_RESERVED6);
        pke_csr.rmwf(utra::pke::RESERVED6_RESERVED6, bar);
        let mut baz = pke_csr.zf(utra::pke::RESERVED6_RESERVED6, bar);
        baz |= pke_csr.ms(utra::pke::RESERVED6_RESERVED6, 1);
        pke_csr.wfo(utra::pke::RESERVED6_RESERVED6, baz);

        let foo = pke_csr.r(utra::pke::RESERVED7);
        pke_csr.wo(utra::pke::RESERVED7, foo);
        let bar = pke_csr.rf(utra::pke::RESERVED7_RESERVED7);
        pke_csr.rmwf(utra::pke::RESERVED7_RESERVED7, bar);
        let mut baz = pke_csr.zf(utra::pke::RESERVED7_RESERVED7, bar);
        baz |= pke_csr.ms(utra::pke::RESERVED7_RESERVED7, 1);
        pke_csr.wfo(utra::pke::RESERVED7_RESERVED7, baz);

        let foo = pke_csr.r(utra::pke::SFR_OPTMASK);
        pke_csr.wo(utra::pke::SFR_OPTMASK, foo);
        let bar = pke_csr.rf(utra::pke::SFR_OPTMASK_SFR_OPTMASK);
        pke_csr.rmwf(utra::pke::SFR_OPTMASK_SFR_OPTMASK, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_OPTMASK_SFR_OPTMASK, bar);
        baz |= pke_csr.ms(utra::pke::SFR_OPTMASK_SFR_OPTMASK, 1);
        pke_csr.wfo(utra::pke::SFR_OPTMASK_SFR_OPTMASK, baz);

        let foo = pke_csr.r(utra::pke::RESERVED9);
        pke_csr.wo(utra::pke::RESERVED9, foo);
        let bar = pke_csr.rf(utra::pke::RESERVED9_RESERVED9);
        pke_csr.rmwf(utra::pke::RESERVED9_RESERVED9, bar);
        let mut baz = pke_csr.zf(utra::pke::RESERVED9_RESERVED9, bar);
        baz |= pke_csr.ms(utra::pke::RESERVED9_RESERVED9, 1);
        pke_csr.wfo(utra::pke::RESERVED9_RESERVED9, baz);

        let foo = pke_csr.r(utra::pke::RESERVED10);
        pke_csr.wo(utra::pke::RESERVED10, foo);
        let bar = pke_csr.rf(utra::pke::RESERVED10_RESERVED10);
        pke_csr.rmwf(utra::pke::RESERVED10_RESERVED10, bar);
        let mut baz = pke_csr.zf(utra::pke::RESERVED10_RESERVED10, bar);
        baz |= pke_csr.ms(utra::pke::RESERVED10_RESERVED10, 1);
        pke_csr.wfo(utra::pke::RESERVED10_RESERVED10, baz);

        let foo = pke_csr.r(utra::pke::RESERVED11);
        pke_csr.wo(utra::pke::RESERVED11, foo);
        let bar = pke_csr.rf(utra::pke::RESERVED11_RESERVED11);
        pke_csr.rmwf(utra::pke::RESERVED11_RESERVED11, bar);
        let mut baz = pke_csr.zf(utra::pke::RESERVED11_RESERVED11, bar);
        baz |= pke_csr.ms(utra::pke::RESERVED11_RESERVED11, 1);
        pke_csr.wfo(utra::pke::RESERVED11_RESERVED11, baz);

        let foo = pke_csr.r(utra::pke::SFR_SEGPTR_PTRID_PCON);
        pke_csr.wo(utra::pke::SFR_SEGPTR_PTRID_PCON, foo);
        let bar = pke_csr.rf(utra::pke::SFR_SEGPTR_PTRID_PCON_PTRID_PCON);
        pke_csr.rmwf(utra::pke::SFR_SEGPTR_PTRID_PCON_PTRID_PCON, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SEGPTR_PTRID_PCON_PTRID_PCON, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SEGPTR_PTRID_PCON_PTRID_PCON, 1);
        pke_csr.wfo(utra::pke::SFR_SEGPTR_PTRID_PCON_PTRID_PCON, baz);

        let foo = pke_csr.r(utra::pke::SFR_SEGPTR_PTRID_PIB0);
        pke_csr.wo(utra::pke::SFR_SEGPTR_PTRID_PIB0, foo);
        let bar = pke_csr.rf(utra::pke::SFR_SEGPTR_PTRID_PIB0_PTRID_PIB0);
        pke_csr.rmwf(utra::pke::SFR_SEGPTR_PTRID_PIB0_PTRID_PIB0, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SEGPTR_PTRID_PIB0_PTRID_PIB0, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SEGPTR_PTRID_PIB0_PTRID_PIB0, 1);
        pke_csr.wfo(utra::pke::SFR_SEGPTR_PTRID_PIB0_PTRID_PIB0, baz);

        let foo = pke_csr.r(utra::pke::SFR_SEGPTR_PTRID_PIB1);
        pke_csr.wo(utra::pke::SFR_SEGPTR_PTRID_PIB1, foo);
        let bar = pke_csr.rf(utra::pke::SFR_SEGPTR_PTRID_PIB1_PTRID_PIB1);
        pke_csr.rmwf(utra::pke::SFR_SEGPTR_PTRID_PIB1_PTRID_PIB1, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SEGPTR_PTRID_PIB1_PTRID_PIB1, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SEGPTR_PTRID_PIB1_PTRID_PIB1, 1);
        pke_csr.wfo(utra::pke::SFR_SEGPTR_PTRID_PIB1_PTRID_PIB1, baz);

        let foo = pke_csr.r(utra::pke::SFR_SEGPTR_PTRID_PKB);
        pke_csr.wo(utra::pke::SFR_SEGPTR_PTRID_PKB, foo);
        let bar = pke_csr.rf(utra::pke::SFR_SEGPTR_PTRID_PKB_PTRID_PKB);
        pke_csr.rmwf(utra::pke::SFR_SEGPTR_PTRID_PKB_PTRID_PKB, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SEGPTR_PTRID_PKB_PTRID_PKB, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SEGPTR_PTRID_PKB_PTRID_PKB, 1);
        pke_csr.wfo(utra::pke::SFR_SEGPTR_PTRID_PKB_PTRID_PKB, baz);

        let foo = pke_csr.r(utra::pke::SFR_SEGPTR_PTRID_POB);
        pke_csr.wo(utra::pke::SFR_SEGPTR_PTRID_POB, foo);
        let bar = pke_csr.rf(utra::pke::SFR_SEGPTR_PTRID_POB_PTRID_POB);
        pke_csr.rmwf(utra::pke::SFR_SEGPTR_PTRID_POB_PTRID_POB, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_SEGPTR_PTRID_POB_PTRID_POB, bar);
        baz |= pke_csr.ms(utra::pke::SFR_SEGPTR_PTRID_POB_PTRID_POB, 1);
        pke_csr.wfo(utra::pke::SFR_SEGPTR_PTRID_POB_PTRID_POB, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_scedma_csr() {
        use super::*;
        let mut scedma_csr = CSR::new(HW_SCEDMA_BASE as *mut u32);

        let foo = scedma_csr.r(utra::scedma::SFR_SCHSTART_AR);
        scedma_csr.wo(utra::scedma::SFR_SCHSTART_AR, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR);
        scedma_csr.rmwf(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCHSTART_AR_SFR_SCHSTART_AR, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED1);
        scedma_csr.wo(utra::scedma::RESERVED1, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED1_RESERVED1);
        scedma_csr.rmwf(utra::scedma::RESERVED1_RESERVED1, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED1_RESERVED1, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED1_RESERVED1, 1);
        scedma_csr.wfo(utra::scedma::RESERVED1_RESERVED1, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED2);
        scedma_csr.wo(utra::scedma::RESERVED2, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED2_RESERVED2);
        scedma_csr.rmwf(utra::scedma::RESERVED2_RESERVED2, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED2_RESERVED2, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED2_RESERVED2, 1);
        scedma_csr.wfo(utra::scedma::RESERVED2_RESERVED2, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED3);
        scedma_csr.wo(utra::scedma::RESERVED3, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED3_RESERVED3);
        scedma_csr.rmwf(utra::scedma::RESERVED3_RESERVED3, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED3_RESERVED3, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED3_RESERVED3, 1);
        scedma_csr.wfo(utra::scedma::RESERVED3_RESERVED3, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_XCH_FUNC);
        scedma_csr.wo(utra::scedma::SFR_XCH_FUNC, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC);
        scedma_csr.rmwf(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, 1);
        scedma_csr.wfo(utra::scedma::SFR_XCH_FUNC_SFR_XCH_FUNC, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_XCH_OPT);
        scedma_csr.wo(utra::scedma::SFR_XCH_OPT, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_XCH_OPT_SFR_XCH_OPT);
        scedma_csr.rmwf(utra::scedma::SFR_XCH_OPT_SFR_XCH_OPT, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_XCH_OPT_SFR_XCH_OPT, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_XCH_OPT_SFR_XCH_OPT, 1);
        scedma_csr.wfo(utra::scedma::SFR_XCH_OPT_SFR_XCH_OPT, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_XCH_AXSTART);
        scedma_csr.wo(utra::scedma::SFR_XCH_AXSTART, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART);
        scedma_csr.rmwf(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, 1);
        scedma_csr.wfo(utra::scedma::SFR_XCH_AXSTART_SFR_XCH_AXSTART, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_XCH_SEGID);
        scedma_csr.wo(utra::scedma::SFR_XCH_SEGID, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID);
        scedma_csr.rmwf(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, 1);
        scedma_csr.wfo(utra::scedma::SFR_XCH_SEGID_SFR_XCH_SEGID, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_XCH_SEGSTART);
        scedma_csr.wo(utra::scedma::SFR_XCH_SEGSTART, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART);
        scedma_csr.rmwf(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, 1);
        scedma_csr.wfo(utra::scedma::SFR_XCH_SEGSTART_XCHCR_SEGSTART, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_XCH_TRANSIZE);
        scedma_csr.wo(utra::scedma::SFR_XCH_TRANSIZE, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE);
        scedma_csr.rmwf(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, 1);
        scedma_csr.wfo(utra::scedma::SFR_XCH_TRANSIZE_XCHCR_TRANSIZE, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED10);
        scedma_csr.wo(utra::scedma::RESERVED10, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED10_RESERVED10);
        scedma_csr.rmwf(utra::scedma::RESERVED10_RESERVED10, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED10_RESERVED10, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED10_RESERVED10, 1);
        scedma_csr.wfo(utra::scedma::RESERVED10_RESERVED10, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED11);
        scedma_csr.wo(utra::scedma::RESERVED11, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED11_RESERVED11);
        scedma_csr.rmwf(utra::scedma::RESERVED11_RESERVED11, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED11_RESERVED11, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED11_RESERVED11, 1);
        scedma_csr.wfo(utra::scedma::RESERVED11_RESERVED11, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_SCH_FUNC);
        scedma_csr.wo(utra::scedma::SFR_SCH_FUNC, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCH_FUNC_SFR_SCH_FUNC);
        scedma_csr.rmwf(utra::scedma::SFR_SCH_FUNC_SFR_SCH_FUNC, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCH_FUNC_SFR_SCH_FUNC, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCH_FUNC_SFR_SCH_FUNC, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCH_FUNC_SFR_SCH_FUNC, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_SCH_OPT);
        scedma_csr.wo(utra::scedma::SFR_SCH_OPT, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCH_OPT_SFR_SCH_OPT);
        scedma_csr.rmwf(utra::scedma::SFR_SCH_OPT_SFR_SCH_OPT, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCH_OPT_SFR_SCH_OPT, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCH_OPT_SFR_SCH_OPT, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCH_OPT_SFR_SCH_OPT, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_SCH_AXSTART);
        scedma_csr.wo(utra::scedma::SFR_SCH_AXSTART, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCH_AXSTART_SFR_SCH_AXSTART);
        scedma_csr.rmwf(utra::scedma::SFR_SCH_AXSTART_SFR_SCH_AXSTART, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCH_AXSTART_SFR_SCH_AXSTART, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCH_AXSTART_SFR_SCH_AXSTART, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCH_AXSTART_SFR_SCH_AXSTART, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_SCH_SEGID);
        scedma_csr.wo(utra::scedma::SFR_SCH_SEGID, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCH_SEGID_SFR_SCH_SEGID);
        scedma_csr.rmwf(utra::scedma::SFR_SCH_SEGID_SFR_SCH_SEGID, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCH_SEGID_SFR_SCH_SEGID, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCH_SEGID_SFR_SCH_SEGID, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCH_SEGID_SFR_SCH_SEGID, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_SCH_SEGSTART);
        scedma_csr.wo(utra::scedma::SFR_SCH_SEGSTART, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCH_SEGSTART_SCHCR_SEGSTART);
        scedma_csr.rmwf(utra::scedma::SFR_SCH_SEGSTART_SCHCR_SEGSTART, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCH_SEGSTART_SCHCR_SEGSTART, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCH_SEGSTART_SCHCR_SEGSTART, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCH_SEGSTART_SCHCR_SEGSTART, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_SCH_TRANSIZE);
        scedma_csr.wo(utra::scedma::SFR_SCH_TRANSIZE, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_SCH_TRANSIZE_SCHCR_TRANSIZE);
        scedma_csr.rmwf(utra::scedma::SFR_SCH_TRANSIZE_SCHCR_TRANSIZE, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_SCH_TRANSIZE_SCHCR_TRANSIZE, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_SCH_TRANSIZE_SCHCR_TRANSIZE, 1);
        scedma_csr.wfo(utra::scedma::SFR_SCH_TRANSIZE_SCHCR_TRANSIZE, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED18);
        scedma_csr.wo(utra::scedma::RESERVED18, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED18_RESERVED18);
        scedma_csr.rmwf(utra::scedma::RESERVED18_RESERVED18, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED18_RESERVED18, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED18_RESERVED18, 1);
        scedma_csr.wfo(utra::scedma::RESERVED18_RESERVED18, baz);

        let foo = scedma_csr.r(utra::scedma::RESERVED19);
        scedma_csr.wo(utra::scedma::RESERVED19, foo);
        let bar = scedma_csr.rf(utra::scedma::RESERVED19_RESERVED19);
        scedma_csr.rmwf(utra::scedma::RESERVED19_RESERVED19, bar);
        let mut baz = scedma_csr.zf(utra::scedma::RESERVED19_RESERVED19, bar);
        baz |= scedma_csr.ms(utra::scedma::RESERVED19_RESERVED19, 1);
        scedma_csr.wfo(utra::scedma::RESERVED19_RESERVED19, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_ICH_OPT);
        scedma_csr.wo(utra::scedma::SFR_ICH_OPT, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_ICH_OPT_SFR_ICH_OPT);
        scedma_csr.rmwf(utra::scedma::SFR_ICH_OPT_SFR_ICH_OPT, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_ICH_OPT_SFR_ICH_OPT, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_ICH_OPT_SFR_ICH_OPT, 1);
        scedma_csr.wfo(utra::scedma::SFR_ICH_OPT_SFR_ICH_OPT, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_ICH_SEGID);
        scedma_csr.wo(utra::scedma::SFR_ICH_SEGID, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_ICH_SEGID_SFR_ICH_SEGID);
        scedma_csr.rmwf(utra::scedma::SFR_ICH_SEGID_SFR_ICH_SEGID, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_ICH_SEGID_SFR_ICH_SEGID, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_ICH_SEGID_SFR_ICH_SEGID, 1);
        scedma_csr.wfo(utra::scedma::SFR_ICH_SEGID_SFR_ICH_SEGID, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_ICH_RPSTART);
        scedma_csr.wo(utra::scedma::SFR_ICH_RPSTART, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_ICH_RPSTART_ICHCR_RPSTART);
        scedma_csr.rmwf(utra::scedma::SFR_ICH_RPSTART_ICHCR_RPSTART, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_ICH_RPSTART_ICHCR_RPSTART, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_ICH_RPSTART_ICHCR_RPSTART, 1);
        scedma_csr.wfo(utra::scedma::SFR_ICH_RPSTART_ICHCR_RPSTART, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_ICH_WPSTART);
        scedma_csr.wo(utra::scedma::SFR_ICH_WPSTART, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_ICH_WPSTART_ICHCR_WPSTART);
        scedma_csr.rmwf(utra::scedma::SFR_ICH_WPSTART_ICHCR_WPSTART, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_ICH_WPSTART_ICHCR_WPSTART, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_ICH_WPSTART_ICHCR_WPSTART, 1);
        scedma_csr.wfo(utra::scedma::SFR_ICH_WPSTART_ICHCR_WPSTART, baz);

        let foo = scedma_csr.r(utra::scedma::SFR_ICH_TRANSIZE);
        scedma_csr.wo(utra::scedma::SFR_ICH_TRANSIZE, foo);
        let bar = scedma_csr.rf(utra::scedma::SFR_ICH_TRANSIZE_ICHCR_TRANSIZE);
        scedma_csr.rmwf(utra::scedma::SFR_ICH_TRANSIZE_ICHCR_TRANSIZE, bar);
        let mut baz = scedma_csr.zf(utra::scedma::SFR_ICH_TRANSIZE_ICHCR_TRANSIZE, bar);
        baz |= scedma_csr.ms(utra::scedma::SFR_ICH_TRANSIZE_ICHCR_TRANSIZE, 1);
        scedma_csr.wfo(utra::scedma::SFR_ICH_TRANSIZE_ICHCR_TRANSIZE, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_sce_glbsfr_csr() {
        use super::*;
        let mut sce_glbsfr_csr = CSR::new(HW_SCE_GLBSFR_BASE as *mut u32);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_SCEMODE);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_SCEMODE, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_SCEMODE_CR_SCEMODE);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_SCEMODE_CR_SCEMODE, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_SCEMODE_CR_SCEMODE, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_SCEMODE_CR_SCEMODE, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_SCEMODE_CR_SCEMODE, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_SUBEN);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_SUBEN, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_SUBEN_CR_SUBEN);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_SUBEN_CR_SUBEN, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_SUBEN_CR_SUBEN, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_SUBEN_CR_SUBEN, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_SUBEN_CR_SUBEN, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_AHBS);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_AHBS, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_AHBS_CR_AHBSOPT, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::RESERVED3);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::RESERVED3, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::RESERVED3_RESERVED3);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::RESERVED3_RESERVED3, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::RESERVED3_RESERVED3, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::RESERVED3_RESERVED3, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::RESERVED3_RESERVED3, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_SRBUSY);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_SRBUSY, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_SRBUSY_SR_BUSY, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FRDONE);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FRDONE, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FRDONE_FR_DONE);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FRDONE_FR_DONE, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FRDONE_FR_DONE, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FRDONE_FR_DONE, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FRDONE_FR_DONE, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FRERR);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FRERR, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FRERR_FR_ERR);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FRERR_FR_ERR, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FRERR_FR_ERR, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FRERR_FR_ERR, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FRERR_FR_ERR, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_ARCLR);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_ARCLR, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_ARCLR_AR_CLRRAM);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_ARCLR_AR_CLRRAM, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_ARCLR_AR_CLRRAM, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_ARCLR_AR_CLRRAM, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_ARCLR_AR_CLRRAM, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_TICKCYC);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_TICKCYC, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_TICKCYC_SFR_TICKCYC);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_TICKCYC_SFR_TICKCYC, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_TICKCYC_SFR_TICKCYC, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_TICKCYC_SFR_TICKCYC, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_TICKCYC_SFR_TICKCYC, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_TICKCNT);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_TICKCNT, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::RESERVED10);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::RESERVED10, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::RESERVED10_RESERVED10);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::RESERVED10_RESERVED10, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::RESERVED10_RESERVED10, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::RESERVED10_RESERVED10, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::RESERVED10_RESERVED10, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::RESERVED11);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::RESERVED11, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::RESERVED11_RESERVED11);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::RESERVED11_RESERVED11, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::RESERVED11_RESERVED11, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::RESERVED11_RESERVED11, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::RESERVED11_RESERVED11, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFEN);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFEN, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFEN_CR_FFEN);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFEN_CR_FFEN, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFEN_CR_FFEN, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFEN_CR_FFEN, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFEN_CR_FFEN, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCLR);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCLR, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCLR_AR_FFCLR);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCLR_AR_FFCLR, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCLR_AR_FFCLR, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCLR_AR_FFCLR, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCLR_AR_FFCLR, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::RESERVED14);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::RESERVED14, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::RESERVED14_RESERVED14);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::RESERVED14_RESERVED14, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::RESERVED14_RESERVED14, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::RESERVED14_RESERVED14, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::RESERVED14_RESERVED14, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::RESERVED15);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::RESERVED15, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::RESERVED15_RESERVED15);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::RESERVED15_RESERVED15, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::RESERVED15_RESERVED15, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::RESERVED15_RESERVED15, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::RESERVED15_RESERVED15, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCNT_SR_FF0);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCNT_SR_FF0, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCNT_SR_FF0_SR_FF0);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCNT_SR_FF0_SR_FF0, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCNT_SR_FF0_SR_FF0, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCNT_SR_FF0_SR_FF0, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCNT_SR_FF0_SR_FF0, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCNT_SR_FF1);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCNT_SR_FF1, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCNT_SR_FF1_SR_FF1);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCNT_SR_FF1_SR_FF1, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCNT_SR_FF1_SR_FF1, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCNT_SR_FF1_SR_FF1, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCNT_SR_FF1_SR_FF1, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCNT_SR_FF2);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCNT_SR_FF2, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCNT_SR_FF2_SR_FF2);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCNT_SR_FF2_SR_FF2, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCNT_SR_FF2_SR_FF2, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCNT_SR_FF2_SR_FF2, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCNT_SR_FF2_SR_FF2, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCNT_SR_FF3);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCNT_SR_FF3, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCNT_SR_FF3_SR_FF3);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCNT_SR_FF3_SR_FF3, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCNT_SR_FF3_SR_FF3, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCNT_SR_FF3_SR_FF3, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCNT_SR_FF3_SR_FF3, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCNT_SR_FF4);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCNT_SR_FF4, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCNT_SR_FF4_SR_FF4);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCNT_SR_FF4_SR_FF4, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCNT_SR_FF4_SR_FF4, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCNT_SR_FF4_SR_FF4, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCNT_SR_FF4_SR_FF4, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FFCNT_SR_FF5);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FFCNT_SR_FF5, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FFCNT_SR_FF5_SR_FF5);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FFCNT_SR_FF5_SR_FF5, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FFCNT_SR_FF5_SR_FF5, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FFCNT_SR_FF5_SR_FF5, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FFCNT_SR_FF5_SR_FF5, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_trng_csr() {
        use super::*;
        let mut trng_csr = CSR::new(HW_TRNG_BASE as *mut u32);

        let foo = trng_csr.r(utra::trng::RESERVED0);
        trng_csr.wo(utra::trng::RESERVED0, foo);
        let bar = trng_csr.rf(utra::trng::RESERVED0_RESERVED0);
        trng_csr.rmwf(utra::trng::RESERVED0_RESERVED0, bar);
        let mut baz = trng_csr.zf(utra::trng::RESERVED0_RESERVED0, bar);
        baz |= trng_csr.ms(utra::trng::RESERVED0_RESERVED0, 1);
        trng_csr.wfo(utra::trng::RESERVED0_RESERVED0, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_alu_csr() {
        use super::*;
        let mut alu_csr = CSR::new(HW_ALU_BASE as *mut u32);

        let foo = alu_csr.r(utra::alu::RESERVED0);
        alu_csr.wo(utra::alu::RESERVED0, foo);
        let bar = alu_csr.rf(utra::alu::RESERVED0_RESERVED0);
        alu_csr.rmwf(utra::alu::RESERVED0_RESERVED0, bar);
        let mut baz = alu_csr.zf(utra::alu::RESERVED0_RESERVED0, bar);
        baz |= alu_csr.ms(utra::alu::RESERVED0_RESERVED0, 1);
        alu_csr.wfo(utra::alu::RESERVED0_RESERVED0, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_duart_csr() {
        use super::*;
        let mut duart_csr = CSR::new(HW_DUART_BASE as *mut u32);

        let foo = duart_csr.r(utra::duart::SFR_TXD);
        duart_csr.wo(utra::duart::SFR_TXD, foo);
        let bar = duart_csr.rf(utra::duart::SFR_TXD_SFR_TXD);
        duart_csr.rmwf(utra::duart::SFR_TXD_SFR_TXD, bar);
        let mut baz = duart_csr.zf(utra::duart::SFR_TXD_SFR_TXD, bar);
        baz |= duart_csr.ms(utra::duart::SFR_TXD_SFR_TXD, 1);
        duart_csr.wfo(utra::duart::SFR_TXD_SFR_TXD, baz);

        let foo = duart_csr.r(utra::duart::SFR_CR);
        duart_csr.wo(utra::duart::SFR_CR, foo);
        let bar = duart_csr.rf(utra::duart::SFR_CR_SFR_CR);
        duart_csr.rmwf(utra::duart::SFR_CR_SFR_CR, bar);
        let mut baz = duart_csr.zf(utra::duart::SFR_CR_SFR_CR, bar);
        baz |= duart_csr.ms(utra::duart::SFR_CR_SFR_CR, 1);
        duart_csr.wfo(utra::duart::SFR_CR_SFR_CR, baz);

        let foo = duart_csr.r(utra::duart::SFR_SR);
        duart_csr.wo(utra::duart::SFR_SR, foo);
        let bar = duart_csr.rf(utra::duart::SFR_SR_SFR_SR);
        duart_csr.rmwf(utra::duart::SFR_SR_SFR_SR, bar);
        let mut baz = duart_csr.zf(utra::duart::SFR_SR_SFR_SR, bar);
        baz |= duart_csr.ms(utra::duart::SFR_SR_SFR_SR, 1);
        duart_csr.wfo(utra::duart::SFR_SR_SFR_SR, baz);

        let foo = duart_csr.r(utra::duart::SFR_ETUC);
        duart_csr.wo(utra::duart::SFR_ETUC, foo);
        let bar = duart_csr.rf(utra::duart::SFR_ETUC_SFR_ETUC);
        duart_csr.rmwf(utra::duart::SFR_ETUC_SFR_ETUC, bar);
        let mut baz = duart_csr.zf(utra::duart::SFR_ETUC_SFR_ETUC, bar);
        baz |= duart_csr.ms(utra::duart::SFR_ETUC_SFR_ETUC, 1);
        duart_csr.wfo(utra::duart::SFR_ETUC_SFR_ETUC, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_wdg_intf_csr() {
        use super::*;
        let mut wdg_intf_csr = CSR::new(HW_WDG_INTF_BASE as *mut u32);

        let foo = wdg_intf_csr.r(utra::wdg_intf::RESERVED0);
        wdg_intf_csr.wo(utra::wdg_intf::RESERVED0, foo);
        let bar = wdg_intf_csr.rf(utra::wdg_intf::RESERVED0_RESERVED0);
        wdg_intf_csr.rmwf(utra::wdg_intf::RESERVED0_RESERVED0, bar);
        let mut baz = wdg_intf_csr.zf(utra::wdg_intf::RESERVED0_RESERVED0, bar);
        baz |= wdg_intf_csr.ms(utra::wdg_intf::RESERVED0_RESERVED0, 1);
        wdg_intf_csr.wfo(utra::wdg_intf::RESERVED0_RESERVED0, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_timer_intf_csr() {
        use super::*;
        let mut timer_intf_csr = CSR::new(HW_TIMER_INTF_BASE as *mut u32);

        let foo = timer_intf_csr.r(utra::timer_intf::RESERVED0);
        timer_intf_csr.wo(utra::timer_intf::RESERVED0, foo);
        let bar = timer_intf_csr.rf(utra::timer_intf::RESERVED0_RESERVED0);
        timer_intf_csr.rmwf(utra::timer_intf::RESERVED0_RESERVED0, bar);
        let mut baz = timer_intf_csr.zf(utra::timer_intf::RESERVED0_RESERVED0, bar);
        baz |= timer_intf_csr.ms(utra::timer_intf::RESERVED0_RESERVED0, 1);
        timer_intf_csr.wfo(utra::timer_intf::RESERVED0_RESERVED0, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_evc_csr() {
        use super::*;
        let mut evc_csr = CSR::new(HW_EVC_BASE as *mut u32);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL0);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL0, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL0_CM7EVSEL0);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL0_CM7EVSEL0, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL0_CM7EVSEL0, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL0_CM7EVSEL0, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL0_CM7EVSEL0, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL1);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL1, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL1_CM7EVSEL1);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL1_CM7EVSEL1, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL1_CM7EVSEL1, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL1_CM7EVSEL1, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL1_CM7EVSEL1, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL2);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL2, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL2_CM7EVSEL2);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL2_CM7EVSEL2, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL2_CM7EVSEL2, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL2_CM7EVSEL2, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL2_CM7EVSEL2, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL3);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL3, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL3_CM7EVSEL3);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL3_CM7EVSEL3, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL3_CM7EVSEL3, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL3_CM7EVSEL3, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL3_CM7EVSEL3, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL4);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL4, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL4_CM7EVSEL4);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL4_CM7EVSEL4, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL4_CM7EVSEL4, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL4_CM7EVSEL4, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL4_CM7EVSEL4, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL5);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL5, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL5_CM7EVSEL5);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL5_CM7EVSEL5, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL5_CM7EVSEL5, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL5_CM7EVSEL5, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL5_CM7EVSEL5, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL6);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL6, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL6_CM7EVSEL6);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL6_CM7EVSEL6, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL6_CM7EVSEL6, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL6_CM7EVSEL6, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL6_CM7EVSEL6, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVSEL_CM7EVSEL7);
        evc_csr.wo(utra::evc::SFR_CM7EVSEL_CM7EVSEL7, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVSEL_CM7EVSEL7_CM7EVSEL7);
        evc_csr.rmwf(utra::evc::SFR_CM7EVSEL_CM7EVSEL7_CM7EVSEL7, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVSEL_CM7EVSEL7_CM7EVSEL7, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVSEL_CM7EVSEL7_CM7EVSEL7, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVSEL_CM7EVSEL7_CM7EVSEL7, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVEN);
        evc_csr.wo(utra::evc::SFR_CM7EVEN, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVEN_CM7EVEN);
        evc_csr.rmwf(utra::evc::SFR_CM7EVEN_CM7EVEN, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVEN_CM7EVEN, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVEN_CM7EVEN, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVEN_CM7EVEN, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7EVFR);
        evc_csr.wo(utra::evc::SFR_CM7EVFR, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7EVFR_CM7EVS);
        evc_csr.rmwf(utra::evc::SFR_CM7EVFR_CM7EVS, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7EVFR_CM7EVS, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7EVFR_CM7EVS, 1);
        evc_csr.wfo(utra::evc::SFR_CM7EVFR_CM7EVS, baz);

        let foo = evc_csr.r(utra::evc::RESERVED10);
        evc_csr.wo(utra::evc::RESERVED10, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED10_RESERVED10);
        evc_csr.rmwf(utra::evc::RESERVED10_RESERVED10, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED10_RESERVED10, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED10_RESERVED10, 1);
        evc_csr.wfo(utra::evc::RESERVED10_RESERVED10, baz);

        let foo = evc_csr.r(utra::evc::RESERVED11);
        evc_csr.wo(utra::evc::RESERVED11, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED11_RESERVED11);
        evc_csr.rmwf(utra::evc::RESERVED11_RESERVED11, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED11_RESERVED11, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED11_RESERVED11, 1);
        evc_csr.wfo(utra::evc::RESERVED11_RESERVED11, baz);

        let foo = evc_csr.r(utra::evc::SFR_TMREVSEL);
        evc_csr.wo(utra::evc::SFR_TMREVSEL, foo);
        let bar = evc_csr.rf(utra::evc::SFR_TMREVSEL_TMR_EVSEL);
        evc_csr.rmwf(utra::evc::SFR_TMREVSEL_TMR_EVSEL, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_TMREVSEL_TMR_EVSEL, bar);
        baz |= evc_csr.ms(utra::evc::SFR_TMREVSEL_TMR_EVSEL, 1);
        evc_csr.wfo(utra::evc::SFR_TMREVSEL_TMR_EVSEL, baz);

        let foo = evc_csr.r(utra::evc::SFR_PWMEVSEL);
        evc_csr.wo(utra::evc::SFR_PWMEVSEL, foo);
        let bar = evc_csr.rf(utra::evc::SFR_PWMEVSEL_PWM_EVSEL);
        evc_csr.rmwf(utra::evc::SFR_PWMEVSEL_PWM_EVSEL, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_PWMEVSEL_PWM_EVSEL, bar);
        baz |= evc_csr.ms(utra::evc::SFR_PWMEVSEL_PWM_EVSEL, 1);
        evc_csr.wfo(utra::evc::SFR_PWMEVSEL_PWM_EVSEL, baz);

        let foo = evc_csr.r(utra::evc::RESERVED14);
        evc_csr.wo(utra::evc::RESERVED14, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED14_RESERVED14);
        evc_csr.rmwf(utra::evc::RESERVED14_RESERVED14, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED14_RESERVED14, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED14_RESERVED14, 1);
        evc_csr.wfo(utra::evc::RESERVED14_RESERVED14, baz);

        let foo = evc_csr.r(utra::evc::RESERVED15);
        evc_csr.wo(utra::evc::RESERVED15, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED15_RESERVED15);
        evc_csr.rmwf(utra::evc::RESERVED15_RESERVED15, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED15_RESERVED15, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED15_RESERVED15, 1);
        evc_csr.wfo(utra::evc::RESERVED15_RESERVED15, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN0);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN0, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN0_IFEVEN0);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN0_IFEVEN0, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN0_IFEVEN0, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN0_IFEVEN0, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN0_IFEVEN0, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN1);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN1, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN1_IFEVEN1);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN1_IFEVEN1, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN1_IFEVEN1, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN1_IFEVEN1, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN1_IFEVEN1, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN2);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN2, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN2_IFEVEN2);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN2_IFEVEN2, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN2_IFEVEN2, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN2_IFEVEN2, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN2_IFEVEN2, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN3);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN3, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN3_IFEVEN3);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN3_IFEVEN3, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN3_IFEVEN3, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN3_IFEVEN3, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN3_IFEVEN3, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN4);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN4, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN4_IFEVEN4);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN4_IFEVEN4, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN4_IFEVEN4, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN4_IFEVEN4, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN4_IFEVEN4, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN5);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN5, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN5_IFEVEN5);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN5_IFEVEN5, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN5_IFEVEN5, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN5_IFEVEN5, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN5_IFEVEN5, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN6);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN6, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN6_IFEVEN6);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN6_IFEVEN6, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN6_IFEVEN6, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN6_IFEVEN6, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN6_IFEVEN6, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVEN_IFEVEN7);
        evc_csr.wo(utra::evc::SFR_IFEVEN_IFEVEN7, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVEN_IFEVEN7_IFEVEN7);
        evc_csr.rmwf(utra::evc::SFR_IFEVEN_IFEVEN7_IFEVEN7, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVEN_IFEVEN7_IFEVEN7, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVEN_IFEVEN7_IFEVEN7, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVEN_IFEVEN7_IFEVEN7, baz);

        let foo = evc_csr.r(utra::evc::SFR_IFEVERRFR);
        evc_csr.wo(utra::evc::SFR_IFEVERRFR, foo);
        let bar = evc_csr.rf(utra::evc::SFR_IFEVERRFR_IFEV_ERRS);
        evc_csr.rmwf(utra::evc::SFR_IFEVERRFR_IFEV_ERRS, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_IFEVERRFR_IFEV_ERRS, bar);
        baz |= evc_csr.ms(utra::evc::SFR_IFEVERRFR_IFEV_ERRS, 1);
        evc_csr.wfo(utra::evc::SFR_IFEVERRFR_IFEV_ERRS, baz);

        let foo = evc_csr.r(utra::evc::RESERVED25);
        evc_csr.wo(utra::evc::RESERVED25, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED25_RESERVED25);
        evc_csr.rmwf(utra::evc::RESERVED25_RESERVED25, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED25_RESERVED25, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED25_RESERVED25, 1);
        evc_csr.wfo(utra::evc::RESERVED25_RESERVED25, baz);

        let foo = evc_csr.r(utra::evc::RESERVED26);
        evc_csr.wo(utra::evc::RESERVED26, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED26_RESERVED26);
        evc_csr.rmwf(utra::evc::RESERVED26_RESERVED26, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED26_RESERVED26, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED26_RESERVED26, 1);
        evc_csr.wfo(utra::evc::RESERVED26_RESERVED26, baz);

        let foo = evc_csr.r(utra::evc::RESERVED27);
        evc_csr.wo(utra::evc::RESERVED27, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED27_RESERVED27);
        evc_csr.rmwf(utra::evc::RESERVED27_RESERVED27, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED27_RESERVED27, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED27_RESERVED27, 1);
        evc_csr.wfo(utra::evc::RESERVED27_RESERVED27, baz);

        let foo = evc_csr.r(utra::evc::RESERVED28);
        evc_csr.wo(utra::evc::RESERVED28, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED28_RESERVED28);
        evc_csr.rmwf(utra::evc::RESERVED28_RESERVED28, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED28_RESERVED28, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED28_RESERVED28, 1);
        evc_csr.wfo(utra::evc::RESERVED28_RESERVED28, baz);

        let foo = evc_csr.r(utra::evc::RESERVED29);
        evc_csr.wo(utra::evc::RESERVED29, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED29_RESERVED29);
        evc_csr.rmwf(utra::evc::RESERVED29_RESERVED29, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED29_RESERVED29, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED29_RESERVED29, 1);
        evc_csr.wfo(utra::evc::RESERVED29_RESERVED29, baz);

        let foo = evc_csr.r(utra::evc::RESERVED30);
        evc_csr.wo(utra::evc::RESERVED30, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED30_RESERVED30);
        evc_csr.rmwf(utra::evc::RESERVED30_RESERVED30, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED30_RESERVED30, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED30_RESERVED30, 1);
        evc_csr.wfo(utra::evc::RESERVED30_RESERVED30, baz);

        let foo = evc_csr.r(utra::evc::RESERVED31);
        evc_csr.wo(utra::evc::RESERVED31, foo);
        let bar = evc_csr.rf(utra::evc::RESERVED31_RESERVED31);
        evc_csr.rmwf(utra::evc::RESERVED31_RESERVED31, bar);
        let mut baz = evc_csr.zf(utra::evc::RESERVED31_RESERVED31, bar);
        baz |= evc_csr.ms(utra::evc::RESERVED31_RESERVED31, 1);
        evc_csr.wfo(utra::evc::RESERVED31_RESERVED31, baz);

        let foo = evc_csr.r(utra::evc::SFR_CM7ERRFR);
        evc_csr.wo(utra::evc::SFR_CM7ERRFR, foo);
        let bar = evc_csr.rf(utra::evc::SFR_CM7ERRFR_ERRIN);
        evc_csr.rmwf(utra::evc::SFR_CM7ERRFR_ERRIN, bar);
        let mut baz = evc_csr.zf(utra::evc::SFR_CM7ERRFR_ERRIN, bar);
        baz |= evc_csr.ms(utra::evc::SFR_CM7ERRFR_ERRIN, 1);
        evc_csr.wfo(utra::evc::SFR_CM7ERRFR_ERRIN, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_sysctrl_csr() {
        use super::*;
        let mut sysctrl_csr = CSR::new(HW_SYSCTRL_BASE as *mut u32);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUSEC);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUSEC, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUSEC_SFR_CGUSEC);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUSEC_SFR_CGUSEC, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUSEC_SFR_CGUSEC, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUSEC_SFR_CGUSEC, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUSEC_SFR_CGUSEC, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGULP);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGULP, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGULP_SFR_CGULP);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGULP_SFR_CGULP, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGULP_SFR_CGULP, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGULP_SFR_CGULP, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGULP_SFR_CGULP, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED2);
        sysctrl_csr.wo(utra::sysctrl::RESERVED2, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED2_RESERVED2);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED2_RESERVED2, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED2_RESERVED2, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED2_RESERVED2, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED2_RESERVED2, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED3);
        sysctrl_csr.wo(utra::sysctrl::RESERVED3, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED3_RESERVED3);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED3_RESERVED3, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED3_RESERVED3, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED3_RESERVED3, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED3_RESERVED3, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUSEL0);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUSEL0, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR0);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR0, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR0_CFGFDCR0);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR0_CFGFDCR0, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR0_CFGFDCR0, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR0_CFGFDCR0, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR0_CFGFDCR0, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR1);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR1, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR1_CFGFDCR1);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR1_CFGFDCR1, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR1_CFGFDCR1, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR1_CFGFDCR1, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR1_CFGFDCR1, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR2);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR2, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR2_CFGFDCR2);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR2_CFGFDCR2, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR2_CFGFDCR2, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR2_CFGFDCR2, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR2_CFGFDCR2, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR3);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR3, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR3_CFGFDCR3);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR3_CFGFDCR3, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR3_CFGFDCR3, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR3_CFGFDCR3, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR3_CFGFDCR3, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR4);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR4, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR4_CFGFDCR4);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR4_CFGFDCR4, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR4_CFGFDCR4, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR4_CFGFDCR4, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR4_CFGFDCR4, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED10);
        sysctrl_csr.wo(utra::sysctrl::RESERVED10, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED10_RESERVED10);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED10_RESERVED10, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED10_RESERVED10, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED10_RESERVED10, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED10_RESERVED10, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUSET);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUSET, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUSET_SFR_CGUSET);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUSET_SFR_CGUSET, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUSET_SFR_CGUSET, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUSET_SFR_CGUSET, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUSET_SFR_CGUSET, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUSEL1);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUSEL1, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUSEL1_SFR_CGUSEL1);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUSEL1_SFR_CGUSEL1, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUSEL1_SFR_CGUSEL1, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUSEL1_SFR_CGUSEL1, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUSEL1_SFR_CGUSEL1, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED13);
        sysctrl_csr.wo(utra::sysctrl::RESERVED13, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED13_RESERVED13);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED13_RESERVED13, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED13_RESERVED13, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED13_RESERVED13, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED13_RESERVED13, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED14);
        sysctrl_csr.wo(utra::sysctrl::RESERVED14, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED14_RESERVED14);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED14_RESERVED14, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED14_RESERVED14, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED14_RESERVED14, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED14_RESERVED14, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED15);
        sysctrl_csr.wo(utra::sysctrl::RESERVED15, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED15_RESERVED15);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED15_RESERVED15, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED15_RESERVED15, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED15_RESERVED15, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED15_RESERVED15, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFSSR_FSFREQ0);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFSSR_FSFREQ0, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ0_FSFREQ0);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFSSR_FSFREQ0_FSFREQ0, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFSSR_FSFREQ0_FSFREQ0, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFSSR_FSFREQ0_FSFREQ0, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFSSR_FSFREQ0_FSFREQ0, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFSSR_FSFREQ1);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFSSR_FSFREQ1, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ1_FSFREQ1);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFSSR_FSFREQ1_FSFREQ1, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFSSR_FSFREQ1_FSFREQ1, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFSSR_FSFREQ1_FSFREQ1, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFSSR_FSFREQ1_FSFREQ1, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFSSR_FSFREQ2);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFSSR_FSFREQ2, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ2_FSFREQ2);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFSSR_FSFREQ2_FSFREQ2, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFSSR_FSFREQ2_FSFREQ2, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFSSR_FSFREQ2_FSFREQ2, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFSSR_FSFREQ2_FSFREQ2, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFSSR_FSFREQ3);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFSSR_FSFREQ3, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFSSR_FSFREQ3_FSFREQ3);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFSSR_FSFREQ3_FSFREQ3, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFSSR_FSFREQ3_FSFREQ3, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFSSR_FSFREQ3_FSFREQ3, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFSSR_FSFREQ3_FSFREQ3, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFSVLD);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFSVLD, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFSVLD_SFR_CGUFSVLD);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFSVLD_SFR_CGUFSVLD, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFSVLD_SFR_CGUFSVLD, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFSVLD_SFR_CGUFSVLD, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFSVLD_SFR_CGUFSVLD, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFSCR);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFSCR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFSCR_SFR_CGUFSCR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFSCR_SFR_CGUFSCR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFSCR_SFR_CGUFSCR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFSCR_SFR_CGUFSCR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFSCR_SFR_CGUFSCR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED22);
        sysctrl_csr.wo(utra::sysctrl::RESERVED22, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED22_RESERVED22);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED22_RESERVED22, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED22_RESERVED22, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED22_RESERVED22, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED22_RESERVED22, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED23);
        sysctrl_csr.wo(utra::sysctrl::RESERVED23, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED23_RESERVED23);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED23_RESERVED23, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED23_RESERVED23, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED23_RESERVED23, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED23_RESERVED23, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_ACLKGR);
        sysctrl_csr.wo(utra::sysctrl::SFR_ACLKGR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_ACLKGR_SFR_ACLKGR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_ACLKGR_SFR_ACLKGR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_ACLKGR_SFR_ACLKGR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_ACLKGR_SFR_ACLKGR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_ACLKGR_SFR_ACLKGR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_HCLKGR);
        sysctrl_csr.wo(utra::sysctrl::SFR_HCLKGR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_HCLKGR_SFR_HCLKGR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_HCLKGR_SFR_HCLKGR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_HCLKGR_SFR_HCLKGR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_HCLKGR_SFR_HCLKGR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_HCLKGR_SFR_HCLKGR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_ICLKGR);
        sysctrl_csr.wo(utra::sysctrl::SFR_ICLKGR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_ICLKGR_SFR_ICLKGR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_ICLKGR_SFR_ICLKGR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_ICLKGR_SFR_ICLKGR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_ICLKGR_SFR_ICLKGR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_ICLKGR_SFR_ICLKGR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_PCLKGR);
        sysctrl_csr.wo(utra::sysctrl::SFR_PCLKGR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_PCLKGR_SFR_PCLKGR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_PCLKGR_SFR_PCLKGR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_PCLKGR_SFR_PCLKGR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_PCLKGR_SFR_PCLKGR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_PCLKGR_SFR_PCLKGR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED28);
        sysctrl_csr.wo(utra::sysctrl::RESERVED28, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED28_RESERVED28);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED28_RESERVED28, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED28_RESERVED28, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED28_RESERVED28, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED28_RESERVED28, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED29);
        sysctrl_csr.wo(utra::sysctrl::RESERVED29, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED29_RESERVED29);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED29_RESERVED29, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED29_RESERVED29, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED29_RESERVED29, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED29_RESERVED29, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED30);
        sysctrl_csr.wo(utra::sysctrl::RESERVED30, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED30_RESERVED30);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED30_RESERVED30, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED30_RESERVED30, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED30_RESERVED30, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED30_RESERVED30, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED31);
        sysctrl_csr.wo(utra::sysctrl::RESERVED31, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED31_RESERVED31);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED31_RESERVED31, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED31_RESERVED31, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED31_RESERVED31, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED31_RESERVED31, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_RCURST0);
        sysctrl_csr.wo(utra::sysctrl::SFR_RCURST0, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_RCURST0_SFR_RCURST0);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_RCURST0_SFR_RCURST0, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_RCURST0_SFR_RCURST0, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_RCURST0_SFR_RCURST0, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_RCURST0_SFR_RCURST0, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_RCURST1);
        sysctrl_csr.wo(utra::sysctrl::SFR_RCURST1, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_RCURST1_SFR_RCURST1);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_RCURST1_SFR_RCURST1, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_RCURST1_SFR_RCURST1, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_RCURST1_SFR_RCURST1, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_RCURST1_SFR_RCURST1, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_RCUSRCFR);
        sysctrl_csr.wo(utra::sysctrl::SFR_RCUSRCFR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_RCUSRCFR_SFR_RCUSRCFR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_RCUSRCFR_SFR_RCUSRCFR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_RCUSRCFR_SFR_RCUSRCFR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_RCUSRCFR_SFR_RCUSRCFR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_RCUSRCFR_SFR_RCUSRCFR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::RESERVED35);
        sysctrl_csr.wo(utra::sysctrl::RESERVED35, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::RESERVED35_RESERVED35);
        sysctrl_csr.rmwf(utra::sysctrl::RESERVED35_RESERVED35, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::RESERVED35_RESERVED35, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::RESERVED35_RESERVED35, 1);
        sysctrl_csr.wfo(utra::sysctrl::RESERVED35_RESERVED35, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCARIPFLOW);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCARIPFLOW, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCARIPFLOW_SFR_IPCARIPFLOW);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCARIPFLOW_SFR_IPCARIPFLOW, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCARIPFLOW_SFR_IPCARIPFLOW, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCARIPFLOW_SFR_IPCARIPFLOW, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCARIPFLOW_SFR_IPCARIPFLOW, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCEN);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCEN, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCEN_SFR_IPCEN);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCEN_SFR_IPCEN, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCEN_SFR_IPCEN, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCEN_SFR_IPCEN, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCEN_SFR_IPCEN, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCLPEN);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCLPEN, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCLPEN_SFR_IPCLPEN);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCLPEN_SFR_IPCLPEN, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCLPEN_SFR_IPCLPEN, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCLPEN_SFR_IPCLPEN, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCLPEN_SFR_IPCLPEN, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCOSC);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCOSC, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCOSC_SFR_IPCOSC);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCOSC_SFR_IPCOSC, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCOSC_SFR_IPCOSC, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCOSC_SFR_IPCOSC, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCOSC_SFR_IPCOSC, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCPLLMN);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCPLLMN, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCPLLMN_SFR_IPCPLLMN);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCPLLMN_SFR_IPCPLLMN, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCPLLMN_SFR_IPCPLLMN, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCPLLMN_SFR_IPCPLLMN, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCPLLMN_SFR_IPCPLLMN, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCPLLF);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCPLLF, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCPLLF_SFR_IPCPLLF);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCPLLF_SFR_IPCPLLF, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCPLLF_SFR_IPCPLLF, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCPLLF_SFR_IPCPLLF, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCPLLF_SFR_IPCPLLF, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCPLLQ);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCPLLQ, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCPLLQ_SFR_IPCPLLQ);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCPLLQ_SFR_IPCPLLQ, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCPLLQ_SFR_IPCPLLQ, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCPLLQ_SFR_IPCPLLQ, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCPLLQ_SFR_IPCPLLQ, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_IPCCR);
        sysctrl_csr.wo(utra::sysctrl::SFR_IPCCR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_IPCCR_SFR_IPCCR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_IPCCR_SFR_IPCCR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_IPCCR_SFR_IPCCR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_IPCCR_SFR_IPCCR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_IPCCR_SFR_IPCCR, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_apb_thru_csr() {
        use super::*;
        let mut apb_thru_csr = CSR::new(HW_APB_THRU_BASE as *mut u32);

        let foo = apb_thru_csr.r(utra::apb_thru::RESERVED0);
        apb_thru_csr.wo(utra::apb_thru::RESERVED0, foo);
        let bar = apb_thru_csr.rf(utra::apb_thru::RESERVED0_RESERVED0);
        apb_thru_csr.rmwf(utra::apb_thru::RESERVED0_RESERVED0, bar);
        let mut baz = apb_thru_csr.zf(utra::apb_thru::RESERVED0_RESERVED0, bar);
        baz |= apb_thru_csr.ms(utra::apb_thru::RESERVED0_RESERVED0, 1);
        apb_thru_csr.wfo(utra::apb_thru::RESERVED0_RESERVED0, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_iox_csr() {
        use super::*;
        let mut iox_csr = CSR::new(HW_IOX_BASE as *mut u32);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL0);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL0_CRAFSEL0);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL0_CRAFSEL0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL0_CRAFSEL0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL0_CRAFSEL0, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL0_CRAFSEL0, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL1);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL1_CRAFSEL1);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL1_CRAFSEL1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL1_CRAFSEL1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL1_CRAFSEL1, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL1_CRAFSEL1, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL2);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL2_CRAFSEL2);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL2_CRAFSEL2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL2_CRAFSEL2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL2_CRAFSEL2, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL2_CRAFSEL2, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL3);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL3_CRAFSEL3);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL3_CRAFSEL3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL3_CRAFSEL3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL3_CRAFSEL3, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL3_CRAFSEL3, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL4);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL4_CRAFSEL4);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL4_CRAFSEL4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL4_CRAFSEL4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL4_CRAFSEL4, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL4_CRAFSEL4, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL5);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL5_CRAFSEL5);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL5_CRAFSEL5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL5_CRAFSEL5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL5_CRAFSEL5, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL5_CRAFSEL5, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL6);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL6, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL6_CRAFSEL6);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL6_CRAFSEL6, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL6_CRAFSEL6, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL6_CRAFSEL6, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL6_CRAFSEL6, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL7);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL7, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL7_CRAFSEL7);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL7_CRAFSEL7, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL7_CRAFSEL7, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL7_CRAFSEL7, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL7_CRAFSEL7, baz);

        let foo = iox_csr.r(utra::iox::RESERVED8);
        iox_csr.wo(utra::iox::RESERVED8, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED8_RESERVED8);
        iox_csr.rmwf(utra::iox::RESERVED8_RESERVED8, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED8_RESERVED8, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED8_RESERVED8, 1);
        iox_csr.wfo(utra::iox::RESERVED8_RESERVED8, baz);

        let foo = iox_csr.r(utra::iox::RESERVED9);
        iox_csr.wo(utra::iox::RESERVED9, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED9_RESERVED9);
        iox_csr.rmwf(utra::iox::RESERVED9_RESERVED9, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED9_RESERVED9, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED9_RESERVED9, 1);
        iox_csr.wfo(utra::iox::RESERVED9_RESERVED9, baz);

        let foo = iox_csr.r(utra::iox::RESERVED10);
        iox_csr.wo(utra::iox::RESERVED10, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED10_RESERVED10);
        iox_csr.rmwf(utra::iox::RESERVED10_RESERVED10, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED10_RESERVED10, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED10_RESERVED10, 1);
        iox_csr.wfo(utra::iox::RESERVED10_RESERVED10, baz);

        let foo = iox_csr.r(utra::iox::RESERVED11);
        iox_csr.wo(utra::iox::RESERVED11, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED11_RESERVED11);
        iox_csr.rmwf(utra::iox::RESERVED11_RESERVED11, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED11_RESERVED11, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED11_RESERVED11, 1);
        iox_csr.wfo(utra::iox::RESERVED11_RESERVED11, baz);

        let foo = iox_csr.r(utra::iox::RESERVED12);
        iox_csr.wo(utra::iox::RESERVED12, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED12_RESERVED12);
        iox_csr.rmwf(utra::iox::RESERVED12_RESERVED12, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED12_RESERVED12, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED12_RESERVED12, 1);
        iox_csr.wfo(utra::iox::RESERVED12_RESERVED12, baz);

        let foo = iox_csr.r(utra::iox::RESERVED13);
        iox_csr.wo(utra::iox::RESERVED13, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED13_RESERVED13);
        iox_csr.rmwf(utra::iox::RESERVED13_RESERVED13, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED13_RESERVED13, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED13_RESERVED13, 1);
        iox_csr.wfo(utra::iox::RESERVED13_RESERVED13, baz);

        let foo = iox_csr.r(utra::iox::RESERVED14);
        iox_csr.wo(utra::iox::RESERVED14, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED14_RESERVED14);
        iox_csr.rmwf(utra::iox::RESERVED14_RESERVED14, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED14_RESERVED14, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED14_RESERVED14, 1);
        iox_csr.wfo(utra::iox::RESERVED14_RESERVED14, baz);

        let foo = iox_csr.r(utra::iox::RESERVED15);
        iox_csr.wo(utra::iox::RESERVED15, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED15_RESERVED15);
        iox_csr.rmwf(utra::iox::RESERVED15_RESERVED15, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED15_RESERVED15, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED15_RESERVED15, 1);
        iox_csr.wfo(utra::iox::RESERVED15_RESERVED15, baz);

        let foo = iox_csr.r(utra::iox::RESERVED16);
        iox_csr.wo(utra::iox::RESERVED16, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED16_RESERVED16);
        iox_csr.rmwf(utra::iox::RESERVED16_RESERVED16, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED16_RESERVED16, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED16_RESERVED16, 1);
        iox_csr.wfo(utra::iox::RESERVED16_RESERVED16, baz);

        let foo = iox_csr.r(utra::iox::RESERVED17);
        iox_csr.wo(utra::iox::RESERVED17, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED17_RESERVED17);
        iox_csr.rmwf(utra::iox::RESERVED17_RESERVED17, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED17_RESERVED17, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED17_RESERVED17, 1);
        iox_csr.wfo(utra::iox::RESERVED17_RESERVED17, baz);

        let foo = iox_csr.r(utra::iox::RESERVED18);
        iox_csr.wo(utra::iox::RESERVED18, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED18_RESERVED18);
        iox_csr.rmwf(utra::iox::RESERVED18_RESERVED18, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED18_RESERVED18, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED18_RESERVED18, 1);
        iox_csr.wfo(utra::iox::RESERVED18_RESERVED18, baz);

        let foo = iox_csr.r(utra::iox::RESERVED19);
        iox_csr.wo(utra::iox::RESERVED19, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED19_RESERVED19);
        iox_csr.rmwf(utra::iox::RESERVED19_RESERVED19, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED19_RESERVED19, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED19_RESERVED19, 1);
        iox_csr.wfo(utra::iox::RESERVED19_RESERVED19, baz);

        let foo = iox_csr.r(utra::iox::RESERVED20);
        iox_csr.wo(utra::iox::RESERVED20, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED20_RESERVED20);
        iox_csr.rmwf(utra::iox::RESERVED20_RESERVED20, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED20_RESERVED20, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED20_RESERVED20, 1);
        iox_csr.wfo(utra::iox::RESERVED20_RESERVED20, baz);

        let foo = iox_csr.r(utra::iox::RESERVED21);
        iox_csr.wo(utra::iox::RESERVED21, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED21_RESERVED21);
        iox_csr.rmwf(utra::iox::RESERVED21_RESERVED21, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED21_RESERVED21, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED21_RESERVED21, 1);
        iox_csr.wfo(utra::iox::RESERVED21_RESERVED21, baz);

        let foo = iox_csr.r(utra::iox::RESERVED22);
        iox_csr.wo(utra::iox::RESERVED22, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED22_RESERVED22);
        iox_csr.rmwf(utra::iox::RESERVED22_RESERVED22, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED22_RESERVED22, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED22_RESERVED22, 1);
        iox_csr.wfo(utra::iox::RESERVED22_RESERVED22, baz);

        let foo = iox_csr.r(utra::iox::RESERVED23);
        iox_csr.wo(utra::iox::RESERVED23, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED23_RESERVED23);
        iox_csr.rmwf(utra::iox::RESERVED23_RESERVED23, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED23_RESERVED23, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED23_RESERVED23, 1);
        iox_csr.wfo(utra::iox::RESERVED23_RESERVED23, baz);

        let foo = iox_csr.r(utra::iox::RESERVED24);
        iox_csr.wo(utra::iox::RESERVED24, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED24_RESERVED24);
        iox_csr.rmwf(utra::iox::RESERVED24_RESERVED24, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED24_RESERVED24, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED24_RESERVED24, 1);
        iox_csr.wfo(utra::iox::RESERVED24_RESERVED24, baz);

        let foo = iox_csr.r(utra::iox::RESERVED25);
        iox_csr.wo(utra::iox::RESERVED25, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED25_RESERVED25);
        iox_csr.rmwf(utra::iox::RESERVED25_RESERVED25, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED25_RESERVED25, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED25_RESERVED25, 1);
        iox_csr.wfo(utra::iox::RESERVED25_RESERVED25, baz);

        let foo = iox_csr.r(utra::iox::RESERVED26);
        iox_csr.wo(utra::iox::RESERVED26, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED26_RESERVED26);
        iox_csr.rmwf(utra::iox::RESERVED26_RESERVED26, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED26_RESERVED26, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED26_RESERVED26, 1);
        iox_csr.wfo(utra::iox::RESERVED26_RESERVED26, baz);

        let foo = iox_csr.r(utra::iox::RESERVED27);
        iox_csr.wo(utra::iox::RESERVED27, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED27_RESERVED27);
        iox_csr.rmwf(utra::iox::RESERVED27_RESERVED27, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED27_RESERVED27, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED27_RESERVED27, 1);
        iox_csr.wfo(utra::iox::RESERVED27_RESERVED27, baz);

        let foo = iox_csr.r(utra::iox::RESERVED28);
        iox_csr.wo(utra::iox::RESERVED28, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED28_RESERVED28);
        iox_csr.rmwf(utra::iox::RESERVED28_RESERVED28, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED28_RESERVED28, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED28_RESERVED28, 1);
        iox_csr.wfo(utra::iox::RESERVED28_RESERVED28, baz);

        let foo = iox_csr.r(utra::iox::RESERVED29);
        iox_csr.wo(utra::iox::RESERVED29, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED29_RESERVED29);
        iox_csr.rmwf(utra::iox::RESERVED29_RESERVED29, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED29_RESERVED29, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED29_RESERVED29, 1);
        iox_csr.wfo(utra::iox::RESERVED29_RESERVED29, baz);

        let foo = iox_csr.r(utra::iox::RESERVED30);
        iox_csr.wo(utra::iox::RESERVED30, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED30_RESERVED30);
        iox_csr.rmwf(utra::iox::RESERVED30_RESERVED30, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED30_RESERVED30, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED30_RESERVED30, 1);
        iox_csr.wfo(utra::iox::RESERVED30_RESERVED30, baz);

        let foo = iox_csr.r(utra::iox::RESERVED31);
        iox_csr.wo(utra::iox::RESERVED31, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED31_RESERVED31);
        iox_csr.rmwf(utra::iox::RESERVED31_RESERVED31, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED31_RESERVED31, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED31_RESERVED31, 1);
        iox_csr.wfo(utra::iox::RESERVED31_RESERVED31, baz);

        let foo = iox_csr.r(utra::iox::RESERVED32);
        iox_csr.wo(utra::iox::RESERVED32, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED32_RESERVED32);
        iox_csr.rmwf(utra::iox::RESERVED32_RESERVED32, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED32_RESERVED32, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED32_RESERVED32, 1);
        iox_csr.wfo(utra::iox::RESERVED32_RESERVED32, baz);

        let foo = iox_csr.r(utra::iox::RESERVED33);
        iox_csr.wo(utra::iox::RESERVED33, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED33_RESERVED33);
        iox_csr.rmwf(utra::iox::RESERVED33_RESERVED33, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED33_RESERVED33, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED33_RESERVED33, 1);
        iox_csr.wfo(utra::iox::RESERVED33_RESERVED33, baz);

        let foo = iox_csr.r(utra::iox::RESERVED34);
        iox_csr.wo(utra::iox::RESERVED34, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED34_RESERVED34);
        iox_csr.rmwf(utra::iox::RESERVED34_RESERVED34, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED34_RESERVED34, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED34_RESERVED34, 1);
        iox_csr.wfo(utra::iox::RESERVED34_RESERVED34, baz);

        let foo = iox_csr.r(utra::iox::RESERVED35);
        iox_csr.wo(utra::iox::RESERVED35, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED35_RESERVED35);
        iox_csr.rmwf(utra::iox::RESERVED35_RESERVED35, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED35_RESERVED35, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED35_RESERVED35, 1);
        iox_csr.wfo(utra::iox::RESERVED35_RESERVED35, baz);

        let foo = iox_csr.r(utra::iox::RESERVED36);
        iox_csr.wo(utra::iox::RESERVED36, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED36_RESERVED36);
        iox_csr.rmwf(utra::iox::RESERVED36_RESERVED36, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED36_RESERVED36, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED36_RESERVED36, 1);
        iox_csr.wfo(utra::iox::RESERVED36_RESERVED36, baz);

        let foo = iox_csr.r(utra::iox::RESERVED37);
        iox_csr.wo(utra::iox::RESERVED37, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED37_RESERVED37);
        iox_csr.rmwf(utra::iox::RESERVED37_RESERVED37, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED37_RESERVED37, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED37_RESERVED37, 1);
        iox_csr.wfo(utra::iox::RESERVED37_RESERVED37, baz);

        let foo = iox_csr.r(utra::iox::RESERVED38);
        iox_csr.wo(utra::iox::RESERVED38, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED38_RESERVED38);
        iox_csr.rmwf(utra::iox::RESERVED38_RESERVED38, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED38_RESERVED38, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED38_RESERVED38, 1);
        iox_csr.wfo(utra::iox::RESERVED38_RESERVED38, baz);

        let foo = iox_csr.r(utra::iox::RESERVED39);
        iox_csr.wo(utra::iox::RESERVED39, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED39_RESERVED39);
        iox_csr.rmwf(utra::iox::RESERVED39_RESERVED39, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED39_RESERVED39, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED39_RESERVED39, 1);
        iox_csr.wfo(utra::iox::RESERVED39_RESERVED39, baz);

        let foo = iox_csr.r(utra::iox::RESERVED40);
        iox_csr.wo(utra::iox::RESERVED40, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED40_RESERVED40);
        iox_csr.rmwf(utra::iox::RESERVED40_RESERVED40, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED40_RESERVED40, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED40_RESERVED40, 1);
        iox_csr.wfo(utra::iox::RESERVED40_RESERVED40, baz);

        let foo = iox_csr.r(utra::iox::RESERVED41);
        iox_csr.wo(utra::iox::RESERVED41, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED41_RESERVED41);
        iox_csr.rmwf(utra::iox::RESERVED41_RESERVED41, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED41_RESERVED41, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED41_RESERVED41, 1);
        iox_csr.wfo(utra::iox::RESERVED41_RESERVED41, baz);

        let foo = iox_csr.r(utra::iox::RESERVED42);
        iox_csr.wo(utra::iox::RESERVED42, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED42_RESERVED42);
        iox_csr.rmwf(utra::iox::RESERVED42_RESERVED42, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED42_RESERVED42, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED42_RESERVED42, 1);
        iox_csr.wfo(utra::iox::RESERVED42_RESERVED42, baz);

        let foo = iox_csr.r(utra::iox::RESERVED43);
        iox_csr.wo(utra::iox::RESERVED43, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED43_RESERVED43);
        iox_csr.rmwf(utra::iox::RESERVED43_RESERVED43, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED43_RESERVED43, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED43_RESERVED43, 1);
        iox_csr.wfo(utra::iox::RESERVED43_RESERVED43, baz);

        let foo = iox_csr.r(utra::iox::RESERVED44);
        iox_csr.wo(utra::iox::RESERVED44, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED44_RESERVED44);
        iox_csr.rmwf(utra::iox::RESERVED44_RESERVED44, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED44_RESERVED44, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED44_RESERVED44, 1);
        iox_csr.wfo(utra::iox::RESERVED44_RESERVED44, baz);

        let foo = iox_csr.r(utra::iox::RESERVED45);
        iox_csr.wo(utra::iox::RESERVED45, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED45_RESERVED45);
        iox_csr.rmwf(utra::iox::RESERVED45_RESERVED45, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED45_RESERVED45, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED45_RESERVED45, 1);
        iox_csr.wfo(utra::iox::RESERVED45_RESERVED45, baz);

        let foo = iox_csr.r(utra::iox::RESERVED46);
        iox_csr.wo(utra::iox::RESERVED46, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED46_RESERVED46);
        iox_csr.rmwf(utra::iox::RESERVED46_RESERVED46, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED46_RESERVED46, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED46_RESERVED46, 1);
        iox_csr.wfo(utra::iox::RESERVED46_RESERVED46, baz);

        let foo = iox_csr.r(utra::iox::RESERVED47);
        iox_csr.wo(utra::iox::RESERVED47, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED47_RESERVED47);
        iox_csr.rmwf(utra::iox::RESERVED47_RESERVED47, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED47_RESERVED47, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED47_RESERVED47, 1);
        iox_csr.wfo(utra::iox::RESERVED47_RESERVED47, baz);

        let foo = iox_csr.r(utra::iox::RESERVED48);
        iox_csr.wo(utra::iox::RESERVED48, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED48_RESERVED48);
        iox_csr.rmwf(utra::iox::RESERVED48_RESERVED48, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED48_RESERVED48, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED48_RESERVED48, 1);
        iox_csr.wfo(utra::iox::RESERVED48_RESERVED48, baz);

        let foo = iox_csr.r(utra::iox::RESERVED49);
        iox_csr.wo(utra::iox::RESERVED49, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED49_RESERVED49);
        iox_csr.rmwf(utra::iox::RESERVED49_RESERVED49, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED49_RESERVED49, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED49_RESERVED49, 1);
        iox_csr.wfo(utra::iox::RESERVED49_RESERVED49, baz);

        let foo = iox_csr.r(utra::iox::RESERVED50);
        iox_csr.wo(utra::iox::RESERVED50, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED50_RESERVED50);
        iox_csr.rmwf(utra::iox::RESERVED50_RESERVED50, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED50_RESERVED50, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED50_RESERVED50, 1);
        iox_csr.wfo(utra::iox::RESERVED50_RESERVED50, baz);

        let foo = iox_csr.r(utra::iox::RESERVED51);
        iox_csr.wo(utra::iox::RESERVED51, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED51_RESERVED51);
        iox_csr.rmwf(utra::iox::RESERVED51_RESERVED51, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED51_RESERVED51, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED51_RESERVED51, 1);
        iox_csr.wfo(utra::iox::RESERVED51_RESERVED51, baz);

        let foo = iox_csr.r(utra::iox::RESERVED52);
        iox_csr.wo(utra::iox::RESERVED52, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED52_RESERVED52);
        iox_csr.rmwf(utra::iox::RESERVED52_RESERVED52, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED52_RESERVED52, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED52_RESERVED52, 1);
        iox_csr.wfo(utra::iox::RESERVED52_RESERVED52, baz);

        let foo = iox_csr.r(utra::iox::RESERVED53);
        iox_csr.wo(utra::iox::RESERVED53, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED53_RESERVED53);
        iox_csr.rmwf(utra::iox::RESERVED53_RESERVED53, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED53_RESERVED53, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED53_RESERVED53, 1);
        iox_csr.wfo(utra::iox::RESERVED53_RESERVED53, baz);

        let foo = iox_csr.r(utra::iox::RESERVED54);
        iox_csr.wo(utra::iox::RESERVED54, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED54_RESERVED54);
        iox_csr.rmwf(utra::iox::RESERVED54_RESERVED54, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED54_RESERVED54, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED54_RESERVED54, 1);
        iox_csr.wfo(utra::iox::RESERVED54_RESERVED54, baz);

        let foo = iox_csr.r(utra::iox::RESERVED55);
        iox_csr.wo(utra::iox::RESERVED55, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED55_RESERVED55);
        iox_csr.rmwf(utra::iox::RESERVED55_RESERVED55, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED55_RESERVED55, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED55_RESERVED55, 1);
        iox_csr.wfo(utra::iox::RESERVED55_RESERVED55, baz);

        let foo = iox_csr.r(utra::iox::RESERVED56);
        iox_csr.wo(utra::iox::RESERVED56, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED56_RESERVED56);
        iox_csr.rmwf(utra::iox::RESERVED56_RESERVED56, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED56_RESERVED56, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED56_RESERVED56, 1);
        iox_csr.wfo(utra::iox::RESERVED56_RESERVED56, baz);

        let foo = iox_csr.r(utra::iox::RESERVED57);
        iox_csr.wo(utra::iox::RESERVED57, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED57_RESERVED57);
        iox_csr.rmwf(utra::iox::RESERVED57_RESERVED57, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED57_RESERVED57, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED57_RESERVED57, 1);
        iox_csr.wfo(utra::iox::RESERVED57_RESERVED57, baz);

        let foo = iox_csr.r(utra::iox::RESERVED58);
        iox_csr.wo(utra::iox::RESERVED58, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED58_RESERVED58);
        iox_csr.rmwf(utra::iox::RESERVED58_RESERVED58, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED58_RESERVED58, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED58_RESERVED58, 1);
        iox_csr.wfo(utra::iox::RESERVED58_RESERVED58, baz);

        let foo = iox_csr.r(utra::iox::RESERVED59);
        iox_csr.wo(utra::iox::RESERVED59, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED59_RESERVED59);
        iox_csr.rmwf(utra::iox::RESERVED59_RESERVED59, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED59_RESERVED59, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED59_RESERVED59, 1);
        iox_csr.wfo(utra::iox::RESERVED59_RESERVED59, baz);

        let foo = iox_csr.r(utra::iox::RESERVED60);
        iox_csr.wo(utra::iox::RESERVED60, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED60_RESERVED60);
        iox_csr.rmwf(utra::iox::RESERVED60_RESERVED60, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED60_RESERVED60, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED60_RESERVED60, 1);
        iox_csr.wfo(utra::iox::RESERVED60_RESERVED60, baz);

        let foo = iox_csr.r(utra::iox::RESERVED61);
        iox_csr.wo(utra::iox::RESERVED61, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED61_RESERVED61);
        iox_csr.rmwf(utra::iox::RESERVED61_RESERVED61, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED61_RESERVED61, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED61_RESERVED61, 1);
        iox_csr.wfo(utra::iox::RESERVED61_RESERVED61, baz);

        let foo = iox_csr.r(utra::iox::RESERVED62);
        iox_csr.wo(utra::iox::RESERVED62, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED62_RESERVED62);
        iox_csr.rmwf(utra::iox::RESERVED62_RESERVED62, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED62_RESERVED62, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED62_RESERVED62, 1);
        iox_csr.wfo(utra::iox::RESERVED62_RESERVED62, baz);

        let foo = iox_csr.r(utra::iox::RESERVED63);
        iox_csr.wo(utra::iox::RESERVED63, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED63_RESERVED63);
        iox_csr.rmwf(utra::iox::RESERVED63_RESERVED63, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED63_RESERVED63, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED63_RESERVED63, 1);
        iox_csr.wfo(utra::iox::RESERVED63_RESERVED63, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT0);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT0_CRINT0);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT0_CRINT0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT0_CRINT0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT0_CRINT0, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT0_CRINT0, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT1);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT1_CRINT1);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT1_CRINT1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT1_CRINT1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT1_CRINT1, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT1_CRINT1, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT2);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT2_CRINT2);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT2_CRINT2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT2_CRINT2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT2_CRINT2, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT2_CRINT2, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT3);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT3_CRINT3);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT3_CRINT3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT3_CRINT3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT3_CRINT3, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT3_CRINT3, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT4);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT4_CRINT4);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT4_CRINT4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT4_CRINT4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT4_CRINT4, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT4_CRINT4, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT5);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT5_CRINT5);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT5_CRINT5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT5_CRINT5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT5_CRINT5, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT5_CRINT5, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT6);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT6, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT6_CRINT6);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT6_CRINT6, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT6_CRINT6, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT6_CRINT6, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT6_CRINT6, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTCR_CRINT7);
        iox_csr.wo(utra::iox::SFR_INTCR_CRINT7, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTCR_CRINT7_CRINT7);
        iox_csr.rmwf(utra::iox::SFR_INTCR_CRINT7_CRINT7, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTCR_CRINT7_CRINT7, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTCR_CRINT7_CRINT7, 1);
        iox_csr.wfo(utra::iox::SFR_INTCR_CRINT7_CRINT7, baz);

        let foo = iox_csr.r(utra::iox::SFR_INTFR);
        iox_csr.wo(utra::iox::SFR_INTFR, foo);
        let bar = iox_csr.rf(utra::iox::SFR_INTFR_FRINT);
        iox_csr.rmwf(utra::iox::SFR_INTFR_FRINT, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_INTFR_FRINT, bar);
        baz |= iox_csr.ms(utra::iox::SFR_INTFR_FRINT, 1);
        iox_csr.wfo(utra::iox::SFR_INTFR_FRINT, baz);

        let foo = iox_csr.r(utra::iox::RESERVED73);
        iox_csr.wo(utra::iox::RESERVED73, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED73_RESERVED73);
        iox_csr.rmwf(utra::iox::RESERVED73_RESERVED73, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED73_RESERVED73, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED73_RESERVED73, 1);
        iox_csr.wfo(utra::iox::RESERVED73_RESERVED73, baz);

        let foo = iox_csr.r(utra::iox::RESERVED74);
        iox_csr.wo(utra::iox::RESERVED74, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED74_RESERVED74);
        iox_csr.rmwf(utra::iox::RESERVED74_RESERVED74, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED74_RESERVED74, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED74_RESERVED74, 1);
        iox_csr.wfo(utra::iox::RESERVED74_RESERVED74, baz);

        let foo = iox_csr.r(utra::iox::RESERVED75);
        iox_csr.wo(utra::iox::RESERVED75, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED75_RESERVED75);
        iox_csr.rmwf(utra::iox::RESERVED75_RESERVED75, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED75_RESERVED75, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED75_RESERVED75, 1);
        iox_csr.wfo(utra::iox::RESERVED75_RESERVED75, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOUT_CRGO0);
        iox_csr.wo(utra::iox::SFR_GPIOOUT_CRGO0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOUT_CRGO0_CRGO0);
        iox_csr.rmwf(utra::iox::SFR_GPIOOUT_CRGO0_CRGO0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOUT_CRGO0_CRGO0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOUT_CRGO0_CRGO0, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOUT_CRGO0_CRGO0, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOUT_CRGO1);
        iox_csr.wo(utra::iox::SFR_GPIOOUT_CRGO1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOUT_CRGO1_CRGO1);
        iox_csr.rmwf(utra::iox::SFR_GPIOOUT_CRGO1_CRGO1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOUT_CRGO1_CRGO1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOUT_CRGO1_CRGO1, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOUT_CRGO1_CRGO1, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOUT_CRGO2);
        iox_csr.wo(utra::iox::SFR_GPIOOUT_CRGO2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOUT_CRGO2_CRGO2);
        iox_csr.rmwf(utra::iox::SFR_GPIOOUT_CRGO2_CRGO2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOUT_CRGO2_CRGO2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOUT_CRGO2_CRGO2, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOUT_CRGO2_CRGO2, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOUT_CRGO3);
        iox_csr.wo(utra::iox::SFR_GPIOOUT_CRGO3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOUT_CRGO3_CRGO3);
        iox_csr.rmwf(utra::iox::SFR_GPIOOUT_CRGO3_CRGO3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOUT_CRGO3_CRGO3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOUT_CRGO3_CRGO3, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOUT_CRGO3_CRGO3, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOE_CRGOE0);
        iox_csr.wo(utra::iox::SFR_GPIOOE_CRGOE0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOE_CRGOE0_CRGOE0);
        iox_csr.rmwf(utra::iox::SFR_GPIOOE_CRGOE0_CRGOE0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOE_CRGOE0_CRGOE0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOE_CRGOE0_CRGOE0, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOE_CRGOE0_CRGOE0, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOE_CRGOE1);
        iox_csr.wo(utra::iox::SFR_GPIOOE_CRGOE1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOE_CRGOE1_CRGOE1);
        iox_csr.rmwf(utra::iox::SFR_GPIOOE_CRGOE1_CRGOE1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOE_CRGOE1_CRGOE1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOE_CRGOE1_CRGOE1, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOE_CRGOE1_CRGOE1, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOE_CRGOE2);
        iox_csr.wo(utra::iox::SFR_GPIOOE_CRGOE2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOE_CRGOE2_CRGOE2);
        iox_csr.rmwf(utra::iox::SFR_GPIOOE_CRGOE2_CRGOE2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOE_CRGOE2_CRGOE2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOE_CRGOE2_CRGOE2, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOE_CRGOE2_CRGOE2, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOE_CRGOE3);
        iox_csr.wo(utra::iox::SFR_GPIOOE_CRGOE3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOE_CRGOE3_CRGOE3);
        iox_csr.rmwf(utra::iox::SFR_GPIOOE_CRGOE3_CRGOE3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOE_CRGOE3_CRGOE3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOE_CRGOE3_CRGOE3, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOE_CRGOE3_CRGOE3, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOPU_CRGPU0);
        iox_csr.wo(utra::iox::SFR_GPIOPU_CRGPU0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOPU_CRGPU0_CRGPU0);
        iox_csr.rmwf(utra::iox::SFR_GPIOPU_CRGPU0_CRGPU0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOPU_CRGPU0_CRGPU0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOPU_CRGPU0_CRGPU0, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOPU_CRGPU0_CRGPU0, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOPU_CRGPU1);
        iox_csr.wo(utra::iox::SFR_GPIOPU_CRGPU1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOPU_CRGPU1_CRGPU1);
        iox_csr.rmwf(utra::iox::SFR_GPIOPU_CRGPU1_CRGPU1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOPU_CRGPU1_CRGPU1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOPU_CRGPU1_CRGPU1, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOPU_CRGPU1_CRGPU1, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOPU_CRGPU2);
        iox_csr.wo(utra::iox::SFR_GPIOPU_CRGPU2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOPU_CRGPU2_CRGPU2);
        iox_csr.rmwf(utra::iox::SFR_GPIOPU_CRGPU2_CRGPU2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOPU_CRGPU2_CRGPU2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOPU_CRGPU2_CRGPU2, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOPU_CRGPU2_CRGPU2, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOPU_CRGPU3);
        iox_csr.wo(utra::iox::SFR_GPIOPU_CRGPU3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOPU_CRGPU3_CRGPU3);
        iox_csr.rmwf(utra::iox::SFR_GPIOPU_CRGPU3_CRGPU3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOPU_CRGPU3_CRGPU3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOPU_CRGPU3_CRGPU3, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOPU_CRGPU3_CRGPU3, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOIN_SRGI0);
        iox_csr.wo(utra::iox::SFR_GPIOIN_SRGI0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOIN_SRGI0_SRGI0);
        iox_csr.rmwf(utra::iox::SFR_GPIOIN_SRGI0_SRGI0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOIN_SRGI0_SRGI0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOIN_SRGI0_SRGI0, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOIN_SRGI0_SRGI0, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOIN_SRGI1);
        iox_csr.wo(utra::iox::SFR_GPIOIN_SRGI1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOIN_SRGI1_SRGI1);
        iox_csr.rmwf(utra::iox::SFR_GPIOIN_SRGI1_SRGI1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOIN_SRGI1_SRGI1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOIN_SRGI1_SRGI1, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOIN_SRGI1_SRGI1, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOIN_SRGI2);
        iox_csr.wo(utra::iox::SFR_GPIOIN_SRGI2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOIN_SRGI2_SRGI2);
        iox_csr.rmwf(utra::iox::SFR_GPIOIN_SRGI2_SRGI2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOIN_SRGI2_SRGI2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOIN_SRGI2_SRGI2, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOIN_SRGI2_SRGI2, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOIN_SRGI3);
        iox_csr.wo(utra::iox::SFR_GPIOIN_SRGI3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOIN_SRGI3_SRGI3);
        iox_csr.rmwf(utra::iox::SFR_GPIOIN_SRGI3_SRGI3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOIN_SRGI3_SRGI3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOIN_SRGI3_SRGI3, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOIN_SRGI3_SRGI3, baz);

        let foo = iox_csr.r(utra::iox::RESERVED92);
        iox_csr.wo(utra::iox::RESERVED92, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED92_RESERVED92);
        iox_csr.rmwf(utra::iox::RESERVED92_RESERVED92, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED92_RESERVED92, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED92_RESERVED92, 1);
        iox_csr.wfo(utra::iox::RESERVED92_RESERVED92, baz);

        let foo = iox_csr.r(utra::iox::RESERVED93);
        iox_csr.wo(utra::iox::RESERVED93, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED93_RESERVED93);
        iox_csr.rmwf(utra::iox::RESERVED93_RESERVED93, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED93_RESERVED93, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED93_RESERVED93, 1);
        iox_csr.wfo(utra::iox::RESERVED93_RESERVED93, baz);

        let foo = iox_csr.r(utra::iox::RESERVED94);
        iox_csr.wo(utra::iox::RESERVED94, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED94_RESERVED94);
        iox_csr.rmwf(utra::iox::RESERVED94_RESERVED94, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED94_RESERVED94, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED94_RESERVED94, 1);
        iox_csr.wfo(utra::iox::RESERVED94_RESERVED94, baz);

        let foo = iox_csr.r(utra::iox::RESERVED95);
        iox_csr.wo(utra::iox::RESERVED95, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED95_RESERVED95);
        iox_csr.rmwf(utra::iox::RESERVED95_RESERVED95, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED95_RESERVED95, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED95_RESERVED95, 1);
        iox_csr.wfo(utra::iox::RESERVED95_RESERVED95, baz);

        let foo = iox_csr.r(utra::iox::RESERVED96);
        iox_csr.wo(utra::iox::RESERVED96, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED96_RESERVED96);
        iox_csr.rmwf(utra::iox::RESERVED96_RESERVED96, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED96_RESERVED96, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED96_RESERVED96, 1);
        iox_csr.wfo(utra::iox::RESERVED96_RESERVED96, baz);

        let foo = iox_csr.r(utra::iox::RESERVED97);
        iox_csr.wo(utra::iox::RESERVED97, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED97_RESERVED97);
        iox_csr.rmwf(utra::iox::RESERVED97_RESERVED97, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED97_RESERVED97, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED97_RESERVED97, 1);
        iox_csr.wfo(utra::iox::RESERVED97_RESERVED97, baz);

        let foo = iox_csr.r(utra::iox::RESERVED98);
        iox_csr.wo(utra::iox::RESERVED98, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED98_RESERVED98);
        iox_csr.rmwf(utra::iox::RESERVED98_RESERVED98, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED98_RESERVED98, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED98_RESERVED98, 1);
        iox_csr.wfo(utra::iox::RESERVED98_RESERVED98, baz);

        let foo = iox_csr.r(utra::iox::RESERVED99);
        iox_csr.wo(utra::iox::RESERVED99, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED99_RESERVED99);
        iox_csr.rmwf(utra::iox::RESERVED99_RESERVED99, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED99_RESERVED99, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED99_RESERVED99, 1);
        iox_csr.wfo(utra::iox::RESERVED99_RESERVED99, baz);

        let foo = iox_csr.r(utra::iox::RESERVED100);
        iox_csr.wo(utra::iox::RESERVED100, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED100_RESERVED100);
        iox_csr.rmwf(utra::iox::RESERVED100_RESERVED100, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED100_RESERVED100, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED100_RESERVED100, 1);
        iox_csr.wfo(utra::iox::RESERVED100_RESERVED100, baz);

        let foo = iox_csr.r(utra::iox::RESERVED101);
        iox_csr.wo(utra::iox::RESERVED101, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED101_RESERVED101);
        iox_csr.rmwf(utra::iox::RESERVED101_RESERVED101, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED101_RESERVED101, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED101_RESERVED101, 1);
        iox_csr.wfo(utra::iox::RESERVED101_RESERVED101, baz);

        let foo = iox_csr.r(utra::iox::RESERVED102);
        iox_csr.wo(utra::iox::RESERVED102, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED102_RESERVED102);
        iox_csr.rmwf(utra::iox::RESERVED102_RESERVED102, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED102_RESERVED102, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED102_RESERVED102, 1);
        iox_csr.wfo(utra::iox::RESERVED102_RESERVED102, baz);

        let foo = iox_csr.r(utra::iox::RESERVED103);
        iox_csr.wo(utra::iox::RESERVED103, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED103_RESERVED103);
        iox_csr.rmwf(utra::iox::RESERVED103_RESERVED103, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED103_RESERVED103, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED103_RESERVED103, 1);
        iox_csr.wfo(utra::iox::RESERVED103_RESERVED103, baz);

        let foo = iox_csr.r(utra::iox::RESERVED104);
        iox_csr.wo(utra::iox::RESERVED104, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED104_RESERVED104);
        iox_csr.rmwf(utra::iox::RESERVED104_RESERVED104, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED104_RESERVED104, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED104_RESERVED104, 1);
        iox_csr.wfo(utra::iox::RESERVED104_RESERVED104, baz);

        let foo = iox_csr.r(utra::iox::RESERVED105);
        iox_csr.wo(utra::iox::RESERVED105, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED105_RESERVED105);
        iox_csr.rmwf(utra::iox::RESERVED105_RESERVED105, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED105_RESERVED105, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED105_RESERVED105, 1);
        iox_csr.wfo(utra::iox::RESERVED105_RESERVED105, baz);

        let foo = iox_csr.r(utra::iox::RESERVED106);
        iox_csr.wo(utra::iox::RESERVED106, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED106_RESERVED106);
        iox_csr.rmwf(utra::iox::RESERVED106_RESERVED106, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED106_RESERVED106, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED106_RESERVED106, 1);
        iox_csr.wfo(utra::iox::RESERVED106_RESERVED106, baz);

        let foo = iox_csr.r(utra::iox::RESERVED107);
        iox_csr.wo(utra::iox::RESERVED107, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED107_RESERVED107);
        iox_csr.rmwf(utra::iox::RESERVED107_RESERVED107, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED107_RESERVED107, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED107_RESERVED107, 1);
        iox_csr.wfo(utra::iox::RESERVED107_RESERVED107, baz);

        let foo = iox_csr.r(utra::iox::RESERVED108);
        iox_csr.wo(utra::iox::RESERVED108, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED108_RESERVED108);
        iox_csr.rmwf(utra::iox::RESERVED108_RESERVED108, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED108_RESERVED108, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED108_RESERVED108, 1);
        iox_csr.wfo(utra::iox::RESERVED108_RESERVED108, baz);

        let foo = iox_csr.r(utra::iox::RESERVED109);
        iox_csr.wo(utra::iox::RESERVED109, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED109_RESERVED109);
        iox_csr.rmwf(utra::iox::RESERVED109_RESERVED109, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED109_RESERVED109, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED109_RESERVED109, 1);
        iox_csr.wfo(utra::iox::RESERVED109_RESERVED109, baz);

        let foo = iox_csr.r(utra::iox::RESERVED110);
        iox_csr.wo(utra::iox::RESERVED110, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED110_RESERVED110);
        iox_csr.rmwf(utra::iox::RESERVED110_RESERVED110, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED110_RESERVED110, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED110_RESERVED110, 1);
        iox_csr.wfo(utra::iox::RESERVED110_RESERVED110, baz);

        let foo = iox_csr.r(utra::iox::RESERVED111);
        iox_csr.wo(utra::iox::RESERVED111, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED111_RESERVED111);
        iox_csr.rmwf(utra::iox::RESERVED111_RESERVED111, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED111_RESERVED111, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED111_RESERVED111, 1);
        iox_csr.wfo(utra::iox::RESERVED111_RESERVED111, baz);

        let foo = iox_csr.r(utra::iox::RESERVED112);
        iox_csr.wo(utra::iox::RESERVED112, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED112_RESERVED112);
        iox_csr.rmwf(utra::iox::RESERVED112_RESERVED112, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED112_RESERVED112, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED112_RESERVED112, 1);
        iox_csr.wfo(utra::iox::RESERVED112_RESERVED112, baz);

        let foo = iox_csr.r(utra::iox::RESERVED113);
        iox_csr.wo(utra::iox::RESERVED113, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED113_RESERVED113);
        iox_csr.rmwf(utra::iox::RESERVED113_RESERVED113, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED113_RESERVED113, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED113_RESERVED113, 1);
        iox_csr.wfo(utra::iox::RESERVED113_RESERVED113, baz);

        let foo = iox_csr.r(utra::iox::RESERVED114);
        iox_csr.wo(utra::iox::RESERVED114, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED114_RESERVED114);
        iox_csr.rmwf(utra::iox::RESERVED114_RESERVED114, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED114_RESERVED114, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED114_RESERVED114, 1);
        iox_csr.wfo(utra::iox::RESERVED114_RESERVED114, baz);

        let foo = iox_csr.r(utra::iox::RESERVED115);
        iox_csr.wo(utra::iox::RESERVED115, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED115_RESERVED115);
        iox_csr.rmwf(utra::iox::RESERVED115_RESERVED115, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED115_RESERVED115, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED115_RESERVED115, 1);
        iox_csr.wfo(utra::iox::RESERVED115_RESERVED115, baz);

        let foo = iox_csr.r(utra::iox::RESERVED116);
        iox_csr.wo(utra::iox::RESERVED116, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED116_RESERVED116);
        iox_csr.rmwf(utra::iox::RESERVED116_RESERVED116, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED116_RESERVED116, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED116_RESERVED116, 1);
        iox_csr.wfo(utra::iox::RESERVED116_RESERVED116, baz);

        let foo = iox_csr.r(utra::iox::RESERVED117);
        iox_csr.wo(utra::iox::RESERVED117, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED117_RESERVED117);
        iox_csr.rmwf(utra::iox::RESERVED117_RESERVED117, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED117_RESERVED117, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED117_RESERVED117, 1);
        iox_csr.wfo(utra::iox::RESERVED117_RESERVED117, baz);

        let foo = iox_csr.r(utra::iox::RESERVED118);
        iox_csr.wo(utra::iox::RESERVED118, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED118_RESERVED118);
        iox_csr.rmwf(utra::iox::RESERVED118_RESERVED118, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED118_RESERVED118, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED118_RESERVED118, 1);
        iox_csr.wfo(utra::iox::RESERVED118_RESERVED118, baz);

        let foo = iox_csr.r(utra::iox::RESERVED119);
        iox_csr.wo(utra::iox::RESERVED119, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED119_RESERVED119);
        iox_csr.rmwf(utra::iox::RESERVED119_RESERVED119, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED119_RESERVED119, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED119_RESERVED119, 1);
        iox_csr.wfo(utra::iox::RESERVED119_RESERVED119, baz);

        let foo = iox_csr.r(utra::iox::RESERVED120);
        iox_csr.wo(utra::iox::RESERVED120, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED120_RESERVED120);
        iox_csr.rmwf(utra::iox::RESERVED120_RESERVED120, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED120_RESERVED120, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED120_RESERVED120, 1);
        iox_csr.wfo(utra::iox::RESERVED120_RESERVED120, baz);

        let foo = iox_csr.r(utra::iox::RESERVED121);
        iox_csr.wo(utra::iox::RESERVED121, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED121_RESERVED121);
        iox_csr.rmwf(utra::iox::RESERVED121_RESERVED121, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED121_RESERVED121, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED121_RESERVED121, 1);
        iox_csr.wfo(utra::iox::RESERVED121_RESERVED121, baz);

        let foo = iox_csr.r(utra::iox::RESERVED122);
        iox_csr.wo(utra::iox::RESERVED122, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED122_RESERVED122);
        iox_csr.rmwf(utra::iox::RESERVED122_RESERVED122, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED122_RESERVED122, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED122_RESERVED122, 1);
        iox_csr.wfo(utra::iox::RESERVED122_RESERVED122, baz);

        let foo = iox_csr.r(utra::iox::RESERVED123);
        iox_csr.wo(utra::iox::RESERVED123, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED123_RESERVED123);
        iox_csr.rmwf(utra::iox::RESERVED123_RESERVED123, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED123_RESERVED123, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED123_RESERVED123, 1);
        iox_csr.wfo(utra::iox::RESERVED123_RESERVED123, baz);

        let foo = iox_csr.r(utra::iox::RESERVED124);
        iox_csr.wo(utra::iox::RESERVED124, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED124_RESERVED124);
        iox_csr.rmwf(utra::iox::RESERVED124_RESERVED124, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED124_RESERVED124, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED124_RESERVED124, 1);
        iox_csr.wfo(utra::iox::RESERVED124_RESERVED124, baz);

        let foo = iox_csr.r(utra::iox::RESERVED125);
        iox_csr.wo(utra::iox::RESERVED125, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED125_RESERVED125);
        iox_csr.rmwf(utra::iox::RESERVED125_RESERVED125, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED125_RESERVED125, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED125_RESERVED125, 1);
        iox_csr.wfo(utra::iox::RESERVED125_RESERVED125, baz);

        let foo = iox_csr.r(utra::iox::RESERVED126);
        iox_csr.wo(utra::iox::RESERVED126, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED126_RESERVED126);
        iox_csr.rmwf(utra::iox::RESERVED126_RESERVED126, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED126_RESERVED126, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED126_RESERVED126, 1);
        iox_csr.wfo(utra::iox::RESERVED126_RESERVED126, baz);

        let foo = iox_csr.r(utra::iox::RESERVED127);
        iox_csr.wo(utra::iox::RESERVED127, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED127_RESERVED127);
        iox_csr.rmwf(utra::iox::RESERVED127_RESERVED127, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED127_RESERVED127, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED127_RESERVED127, 1);
        iox_csr.wfo(utra::iox::RESERVED127_RESERVED127, baz);

        let foo = iox_csr.r(utra::iox::SFR_PIOSEL);
        iox_csr.wo(utra::iox::SFR_PIOSEL, foo);
        let bar = iox_csr.rf(utra::iox::SFR_PIOSEL_PIOSEL);
        iox_csr.rmwf(utra::iox::SFR_PIOSEL_PIOSEL, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_PIOSEL_PIOSEL, bar);
        baz |= iox_csr.ms(utra::iox::SFR_PIOSEL_PIOSEL, 1);
        iox_csr.wfo(utra::iox::SFR_PIOSEL_PIOSEL, baz);

        let foo = iox_csr.r(utra::iox::RESERVED129);
        iox_csr.wo(utra::iox::RESERVED129, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED129_RESERVED129);
        iox_csr.rmwf(utra::iox::RESERVED129_RESERVED129, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED129_RESERVED129, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED129_RESERVED129, 1);
        iox_csr.wfo(utra::iox::RESERVED129_RESERVED129, baz);

        let foo = iox_csr.r(utra::iox::RESERVED130);
        iox_csr.wo(utra::iox::RESERVED130, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED130_RESERVED130);
        iox_csr.rmwf(utra::iox::RESERVED130_RESERVED130, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED130_RESERVED130, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED130_RESERVED130, 1);
        iox_csr.wfo(utra::iox::RESERVED130_RESERVED130, baz);

        let foo = iox_csr.r(utra::iox::RESERVED131);
        iox_csr.wo(utra::iox::RESERVED131, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED131_RESERVED131);
        iox_csr.rmwf(utra::iox::RESERVED131_RESERVED131, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED131_RESERVED131, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED131_RESERVED131, 1);
        iox_csr.wfo(utra::iox::RESERVED131_RESERVED131, baz);

        let foo = iox_csr.r(utra::iox::RESERVED132);
        iox_csr.wo(utra::iox::RESERVED132, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED132_RESERVED132);
        iox_csr.rmwf(utra::iox::RESERVED132_RESERVED132, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED132_RESERVED132, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED132_RESERVED132, 1);
        iox_csr.wfo(utra::iox::RESERVED132_RESERVED132, baz);

        let foo = iox_csr.r(utra::iox::RESERVED133);
        iox_csr.wo(utra::iox::RESERVED133, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED133_RESERVED133);
        iox_csr.rmwf(utra::iox::RESERVED133_RESERVED133, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED133_RESERVED133, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED133_RESERVED133, 1);
        iox_csr.wfo(utra::iox::RESERVED133_RESERVED133, baz);

        let foo = iox_csr.r(utra::iox::RESERVED134);
        iox_csr.wo(utra::iox::RESERVED134, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED134_RESERVED134);
        iox_csr.rmwf(utra::iox::RESERVED134_RESERVED134, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED134_RESERVED134, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED134_RESERVED134, 1);
        iox_csr.wfo(utra::iox::RESERVED134_RESERVED134, baz);

        let foo = iox_csr.r(utra::iox::RESERVED135);
        iox_csr.wo(utra::iox::RESERVED135, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED135_RESERVED135);
        iox_csr.rmwf(utra::iox::RESERVED135_RESERVED135, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED135_RESERVED135, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED135_RESERVED135, 1);
        iox_csr.wfo(utra::iox::RESERVED135_RESERVED135, baz);

        let foo = iox_csr.r(utra::iox::RESERVED136);
        iox_csr.wo(utra::iox::RESERVED136, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED136_RESERVED136);
        iox_csr.rmwf(utra::iox::RESERVED136_RESERVED136, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED136_RESERVED136, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED136_RESERVED136, 1);
        iox_csr.wfo(utra::iox::RESERVED136_RESERVED136, baz);

        let foo = iox_csr.r(utra::iox::RESERVED137);
        iox_csr.wo(utra::iox::RESERVED137, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED137_RESERVED137);
        iox_csr.rmwf(utra::iox::RESERVED137_RESERVED137, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED137_RESERVED137, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED137_RESERVED137, 1);
        iox_csr.wfo(utra::iox::RESERVED137_RESERVED137, baz);

        let foo = iox_csr.r(utra::iox::RESERVED138);
        iox_csr.wo(utra::iox::RESERVED138, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED138_RESERVED138);
        iox_csr.rmwf(utra::iox::RESERVED138_RESERVED138, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED138_RESERVED138, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED138_RESERVED138, 1);
        iox_csr.wfo(utra::iox::RESERVED138_RESERVED138, baz);

        let foo = iox_csr.r(utra::iox::RESERVED139);
        iox_csr.wo(utra::iox::RESERVED139, foo);
        let bar = iox_csr.rf(utra::iox::RESERVED139_RESERVED139);
        iox_csr.rmwf(utra::iox::RESERVED139_RESERVED139, bar);
        let mut baz = iox_csr.zf(utra::iox::RESERVED139_RESERVED139, bar);
        baz |= iox_csr.ms(utra::iox::RESERVED139_RESERVED139, 1);
        iox_csr.wfo(utra::iox::RESERVED139_RESERVED139, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0);
        iox_csr.wo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0);
        iox_csr.rmwf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1);
        iox_csr.wo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1);
        iox_csr.rmwf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2);
        iox_csr.wo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2);
        iox_csr.rmwf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3);
        iox_csr.wo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3);
        iox_csr.rmwf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0);
        iox_csr.wo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0);
        iox_csr.rmwf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1);
        iox_csr.wo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1);
        iox_csr.rmwf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2);
        iox_csr.wo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2);
        iox_csr.rmwf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3);
        iox_csr.wo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3);
        iox_csr.rmwf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0);
        iox_csr.wo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0);
        iox_csr.rmwf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1);
        iox_csr.wo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1);
        iox_csr.rmwf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2);
        iox_csr.wo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2);
        iox_csr.rmwf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3);
        iox_csr.wo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3);
        iox_csr.rmwf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_pwm_csr() {
        use super::*;
        let mut pwm_csr = CSR::new(HW_PWM_BASE as *mut u32);

        let foo = pwm_csr.r(utra::pwm::RESERVED0);
        pwm_csr.wo(utra::pwm::RESERVED0, foo);
        let bar = pwm_csr.rf(utra::pwm::RESERVED0_RESERVED0);
        pwm_csr.rmwf(utra::pwm::RESERVED0_RESERVED0, bar);
        let mut baz = pwm_csr.zf(utra::pwm::RESERVED0_RESERVED0, bar);
        baz |= pwm_csr.ms(utra::pwm::RESERVED0_RESERVED0, 1);
        pwm_csr.wfo(utra::pwm::RESERVED0_RESERVED0, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_sddc_csr() {
        use super::*;
        let mut sddc_csr = CSR::new(HW_SDDC_BASE as *mut u32);

        let foo = sddc_csr.r(utra::sddc::SFR_IO);
        sddc_csr.wo(utra::sddc::SFR_IO, foo);
        let bar = sddc_csr.rf(utra::sddc::SFR_IO_SFR_IO);
        sddc_csr.rmwf(utra::sddc::SFR_IO_SFR_IO, bar);
        let mut baz = sddc_csr.zf(utra::sddc::SFR_IO_SFR_IO, bar);
        baz |= sddc_csr.ms(utra::sddc::SFR_IO_SFR_IO, 1);
        sddc_csr.wfo(utra::sddc::SFR_IO_SFR_IO, baz);

        let foo = sddc_csr.r(utra::sddc::SFR_AR);
        sddc_csr.wo(utra::sddc::SFR_AR, foo);
        let bar = sddc_csr.rf(utra::sddc::SFR_AR_SFR_AR);
        sddc_csr.rmwf(utra::sddc::SFR_AR_SFR_AR, bar);
        let mut baz = sddc_csr.zf(utra::sddc::SFR_AR_SFR_AR, bar);
        baz |= sddc_csr.ms(utra::sddc::SFR_AR_SFR_AR, 1);
        sddc_csr.wfo(utra::sddc::SFR_AR_SFR_AR, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED2);
        sddc_csr.wo(utra::sddc::RESERVED2, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED2_RESERVED2);
        sddc_csr.rmwf(utra::sddc::RESERVED2_RESERVED2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED2_RESERVED2, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED2_RESERVED2, 1);
        sddc_csr.wfo(utra::sddc::RESERVED2_RESERVED2, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED3);
        sddc_csr.wo(utra::sddc::RESERVED3, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED3_RESERVED3);
        sddc_csr.rmwf(utra::sddc::RESERVED3_RESERVED3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED3_RESERVED3, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED3_RESERVED3, 1);
        sddc_csr.wfo(utra::sddc::RESERVED3_RESERVED3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_OCR);
        sddc_csr.wo(utra::sddc::CR_OCR, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_OCR_CR_OCR);
        sddc_csr.rmwf(utra::sddc::CR_OCR_CR_OCR, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_OCR_CR_OCR, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_OCR_CR_OCR, 1);
        sddc_csr.wfo(utra::sddc::CR_OCR_CR_OCR, baz);

        let foo = sddc_csr.r(utra::sddc::CR_RDFFTHRES);
        sddc_csr.wo(utra::sddc::CR_RDFFTHRES, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_RDFFTHRES_CR_RDFFTHRES);
        sddc_csr.rmwf(utra::sddc::CR_RDFFTHRES_CR_RDFFTHRES, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_RDFFTHRES_CR_RDFFTHRES, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_RDFFTHRES_CR_RDFFTHRES, 1);
        sddc_csr.wfo(utra::sddc::CR_RDFFTHRES_CR_RDFFTHRES, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REV);
        sddc_csr.wo(utra::sddc::CR_REV, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REV_CFG_REG_SD_SPEC_REVISION);
        sddc_csr.rmwf(utra::sddc::CR_REV_CFG_REG_SD_SPEC_REVISION, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REV_CFG_REG_SD_SPEC_REVISION, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REV_CFG_REG_SD_SPEC_REVISION, 1);
        sddc_csr.wfo(utra::sddc::CR_REV_CFG_REG_SD_SPEC_REVISION, baz);
        let bar = sddc_csr.rf(utra::sddc::CR_REV_CFG_REG_CCCR_SDIO_REVISION);
        sddc_csr.rmwf(utra::sddc::CR_REV_CFG_REG_CCCR_SDIO_REVISION, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REV_CFG_REG_CCCR_SDIO_REVISION, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REV_CFG_REG_CCCR_SDIO_REVISION, 1);
        sddc_csr.wfo(utra::sddc::CR_REV_CFG_REG_CCCR_SDIO_REVISION, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BACSA);
        sddc_csr.wo(utra::sddc::CR_BACSA, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BACSA_CFG_BASE_ADDR_CSA);
        sddc_csr.rmwf(utra::sddc::CR_BACSA_CFG_BASE_ADDR_CSA, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BACSA_CFG_BASE_ADDR_CSA, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BACSA_CFG_BASE_ADDR_CSA, 1);
        sddc_csr.wfo(utra::sddc::CR_BACSA_CFG_BASE_ADDR_CSA, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0_CFG_BASE_ADDR_IO_FUNC0);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0_CFG_BASE_ADDR_IO_FUNC0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0_CFG_BASE_ADDR_IO_FUNC0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0_CFG_BASE_ADDR_IO_FUNC0, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC0_CFG_BASE_ADDR_IO_FUNC0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1_CFG_BASE_ADDR_IO_FUNC1);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1_CFG_BASE_ADDR_IO_FUNC1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1_CFG_BASE_ADDR_IO_FUNC1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1_CFG_BASE_ADDR_IO_FUNC1, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC1_CFG_BASE_ADDR_IO_FUNC1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2_CFG_BASE_ADDR_IO_FUNC2);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2_CFG_BASE_ADDR_IO_FUNC2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2_CFG_BASE_ADDR_IO_FUNC2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2_CFG_BASE_ADDR_IO_FUNC2, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC2_CFG_BASE_ADDR_IO_FUNC2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3_CFG_BASE_ADDR_IO_FUNC3);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3_CFG_BASE_ADDR_IO_FUNC3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3_CFG_BASE_ADDR_IO_FUNC3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3_CFG_BASE_ADDR_IO_FUNC3, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC3_CFG_BASE_ADDR_IO_FUNC3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4_CFG_BASE_ADDR_IO_FUNC4);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4_CFG_BASE_ADDR_IO_FUNC4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4_CFG_BASE_ADDR_IO_FUNC4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4_CFG_BASE_ADDR_IO_FUNC4, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC4_CFG_BASE_ADDR_IO_FUNC4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5_CFG_BASE_ADDR_IO_FUNC5);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5_CFG_BASE_ADDR_IO_FUNC5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5_CFG_BASE_ADDR_IO_FUNC5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5_CFG_BASE_ADDR_IO_FUNC5, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC5_CFG_BASE_ADDR_IO_FUNC5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6_CFG_BASE_ADDR_IO_FUNC6);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6_CFG_BASE_ADDR_IO_FUNC6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6_CFG_BASE_ADDR_IO_FUNC6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6_CFG_BASE_ADDR_IO_FUNC6, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC6_CFG_BASE_ADDR_IO_FUNC6, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7);
        sddc_csr.wo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7_CFG_BASE_ADDR_IO_FUNC7);
        sddc_csr.rmwf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7_CFG_BASE_ADDR_IO_FUNC7, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7_CFG_BASE_ADDR_IO_FUNC7, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7_CFG_BASE_ADDR_IO_FUNC7, 1);
        sddc_csr.wfo(utra::sddc::CR_BAIOFN_CFG_BASE_ADDR_IO_FUNC7_CFG_BASE_ADDR_IO_FUNC7, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0_CFG_REG_FUNC_CIS_PTR0);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0_CFG_REG_FUNC_CIS_PTR0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0_CFG_REG_FUNC_CIS_PTR0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0_CFG_REG_FUNC_CIS_PTR0, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR0_CFG_REG_FUNC_CIS_PTR0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1_CFG_REG_FUNC_CIS_PTR1);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1_CFG_REG_FUNC_CIS_PTR1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1_CFG_REG_FUNC_CIS_PTR1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1_CFG_REG_FUNC_CIS_PTR1, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR1_CFG_REG_FUNC_CIS_PTR1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2_CFG_REG_FUNC_CIS_PTR2);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2_CFG_REG_FUNC_CIS_PTR2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2_CFG_REG_FUNC_CIS_PTR2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2_CFG_REG_FUNC_CIS_PTR2, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR2_CFG_REG_FUNC_CIS_PTR2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3_CFG_REG_FUNC_CIS_PTR3);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3_CFG_REG_FUNC_CIS_PTR3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3_CFG_REG_FUNC_CIS_PTR3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3_CFG_REG_FUNC_CIS_PTR3, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR3_CFG_REG_FUNC_CIS_PTR3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4_CFG_REG_FUNC_CIS_PTR4);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4_CFG_REG_FUNC_CIS_PTR4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4_CFG_REG_FUNC_CIS_PTR4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4_CFG_REG_FUNC_CIS_PTR4, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR4_CFG_REG_FUNC_CIS_PTR4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5_CFG_REG_FUNC_CIS_PTR5);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5_CFG_REG_FUNC_CIS_PTR5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5_CFG_REG_FUNC_CIS_PTR5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5_CFG_REG_FUNC_CIS_PTR5, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR5_CFG_REG_FUNC_CIS_PTR5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6_CFG_REG_FUNC_CIS_PTR6);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6_CFG_REG_FUNC_CIS_PTR6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6_CFG_REG_FUNC_CIS_PTR6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6_CFG_REG_FUNC_CIS_PTR6, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR6_CFG_REG_FUNC_CIS_PTR6, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7);
        sddc_csr.wo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7_CFG_REG_FUNC_CIS_PTR7);
        sddc_csr.rmwf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7_CFG_REG_FUNC_CIS_PTR7, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7_CFG_REG_FUNC_CIS_PTR7, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7_CFG_REG_FUNC_CIS_PTR7, 1);
        sddc_csr.wfo(utra::sddc::CR_FNCISPTR_CFG_REG_FUNC_CIS_PTR7_CFG_REG_FUNC_CIS_PTR7, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0_CFG_REG_FUNC_EXT_STD_CODE0);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0_CFG_REG_FUNC_EXT_STD_CODE0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0_CFG_REG_FUNC_EXT_STD_CODE0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0_CFG_REG_FUNC_EXT_STD_CODE0, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE0_CFG_REG_FUNC_EXT_STD_CODE0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1_CFG_REG_FUNC_EXT_STD_CODE1);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1_CFG_REG_FUNC_EXT_STD_CODE1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1_CFG_REG_FUNC_EXT_STD_CODE1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1_CFG_REG_FUNC_EXT_STD_CODE1, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE1_CFG_REG_FUNC_EXT_STD_CODE1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2_CFG_REG_FUNC_EXT_STD_CODE2);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2_CFG_REG_FUNC_EXT_STD_CODE2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2_CFG_REG_FUNC_EXT_STD_CODE2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2_CFG_REG_FUNC_EXT_STD_CODE2, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE2_CFG_REG_FUNC_EXT_STD_CODE2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3_CFG_REG_FUNC_EXT_STD_CODE3);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3_CFG_REG_FUNC_EXT_STD_CODE3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3_CFG_REG_FUNC_EXT_STD_CODE3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3_CFG_REG_FUNC_EXT_STD_CODE3, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE3_CFG_REG_FUNC_EXT_STD_CODE3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4_CFG_REG_FUNC_EXT_STD_CODE4);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4_CFG_REG_FUNC_EXT_STD_CODE4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4_CFG_REG_FUNC_EXT_STD_CODE4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4_CFG_REG_FUNC_EXT_STD_CODE4, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE4_CFG_REG_FUNC_EXT_STD_CODE4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5_CFG_REG_FUNC_EXT_STD_CODE5);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5_CFG_REG_FUNC_EXT_STD_CODE5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5_CFG_REG_FUNC_EXT_STD_CODE5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5_CFG_REG_FUNC_EXT_STD_CODE5, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE5_CFG_REG_FUNC_EXT_STD_CODE5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6_CFG_REG_FUNC_EXT_STD_CODE6);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6_CFG_REG_FUNC_EXT_STD_CODE6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6_CFG_REG_FUNC_EXT_STD_CODE6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6_CFG_REG_FUNC_EXT_STD_CODE6, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE6_CFG_REG_FUNC_EXT_STD_CODE6, baz);

        let foo = sddc_csr.r(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7);
        sddc_csr.wo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7_CFG_REG_FUNC_EXT_STD_CODE7);
        sddc_csr.rmwf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7_CFG_REG_FUNC_EXT_STD_CODE7, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7_CFG_REG_FUNC_EXT_STD_CODE7, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7_CFG_REG_FUNC_EXT_STD_CODE7, 1);
        sddc_csr.wfo(utra::sddc::CR_FNEXTSTDCODE_CFG_REG_FUNC_EXT_STD_CODE7_CFG_REG_FUNC_EXT_STD_CODE7, baz);

        let foo = sddc_csr.r(utra::sddc::CR_WRITE_PROTECT);
        sddc_csr.wo(utra::sddc::CR_WRITE_PROTECT, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_WRITE_PROTECT_CR_WRITE_PROTECT);
        sddc_csr.rmwf(utra::sddc::CR_WRITE_PROTECT_CR_WRITE_PROTECT, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_WRITE_PROTECT_CR_WRITE_PROTECT, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_WRITE_PROTECT_CR_WRITE_PROTECT, 1);
        sddc_csr.wfo(utra::sddc::CR_WRITE_PROTECT_CR_WRITE_PROTECT, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_DSR);
        sddc_csr.wo(utra::sddc::CR_REG_DSR, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_DSR_CR_REG_DSR);
        sddc_csr.rmwf(utra::sddc::CR_REG_DSR_CR_REG_DSR, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_DSR_CR_REG_DSR, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_DSR_CR_REG_DSR, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_DSR_CR_REG_DSR, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CID_CFG_REG_CID0);
        sddc_csr.wo(utra::sddc::CR_REG_CID_CFG_REG_CID0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CID_CFG_REG_CID0_CFG_REG_CID0);
        sddc_csr.rmwf(utra::sddc::CR_REG_CID_CFG_REG_CID0_CFG_REG_CID0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CID_CFG_REG_CID0_CFG_REG_CID0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CID_CFG_REG_CID0_CFG_REG_CID0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CID_CFG_REG_CID0_CFG_REG_CID0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CID_CFG_REG_CID1);
        sddc_csr.wo(utra::sddc::CR_REG_CID_CFG_REG_CID1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CID_CFG_REG_CID1_CFG_REG_CID1);
        sddc_csr.rmwf(utra::sddc::CR_REG_CID_CFG_REG_CID1_CFG_REG_CID1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CID_CFG_REG_CID1_CFG_REG_CID1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CID_CFG_REG_CID1_CFG_REG_CID1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CID_CFG_REG_CID1_CFG_REG_CID1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CID_CFG_REG_CID2);
        sddc_csr.wo(utra::sddc::CR_REG_CID_CFG_REG_CID2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CID_CFG_REG_CID2_CFG_REG_CID2);
        sddc_csr.rmwf(utra::sddc::CR_REG_CID_CFG_REG_CID2_CFG_REG_CID2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CID_CFG_REG_CID2_CFG_REG_CID2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CID_CFG_REG_CID2_CFG_REG_CID2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CID_CFG_REG_CID2_CFG_REG_CID2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CID_CFG_REG_CID3);
        sddc_csr.wo(utra::sddc::CR_REG_CID_CFG_REG_CID3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CID_CFG_REG_CID3_CFG_REG_CID3);
        sddc_csr.rmwf(utra::sddc::CR_REG_CID_CFG_REG_CID3_CFG_REG_CID3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CID_CFG_REG_CID3_CFG_REG_CID3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CID_CFG_REG_CID3_CFG_REG_CID3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CID_CFG_REG_CID3_CFG_REG_CID3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CSD_CFG_REG_CSD0);
        sddc_csr.wo(utra::sddc::CR_REG_CSD_CFG_REG_CSD0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CSD_CFG_REG_CSD0_CFG_REG_CSD0);
        sddc_csr.rmwf(utra::sddc::CR_REG_CSD_CFG_REG_CSD0_CFG_REG_CSD0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CSD_CFG_REG_CSD0_CFG_REG_CSD0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CSD_CFG_REG_CSD0_CFG_REG_CSD0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CSD_CFG_REG_CSD0_CFG_REG_CSD0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CSD_CFG_REG_CSD1);
        sddc_csr.wo(utra::sddc::CR_REG_CSD_CFG_REG_CSD1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CSD_CFG_REG_CSD1_CFG_REG_CSD1);
        sddc_csr.rmwf(utra::sddc::CR_REG_CSD_CFG_REG_CSD1_CFG_REG_CSD1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CSD_CFG_REG_CSD1_CFG_REG_CSD1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CSD_CFG_REG_CSD1_CFG_REG_CSD1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CSD_CFG_REG_CSD1_CFG_REG_CSD1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CSD_CFG_REG_CSD2);
        sddc_csr.wo(utra::sddc::CR_REG_CSD_CFG_REG_CSD2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CSD_CFG_REG_CSD2_CFG_REG_CSD2);
        sddc_csr.rmwf(utra::sddc::CR_REG_CSD_CFG_REG_CSD2_CFG_REG_CSD2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CSD_CFG_REG_CSD2_CFG_REG_CSD2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CSD_CFG_REG_CSD2_CFG_REG_CSD2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CSD_CFG_REG_CSD2_CFG_REG_CSD2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_CSD_CFG_REG_CSD3);
        sddc_csr.wo(utra::sddc::CR_REG_CSD_CFG_REG_CSD3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_CSD_CFG_REG_CSD3_CFG_REG_CSD3);
        sddc_csr.rmwf(utra::sddc::CR_REG_CSD_CFG_REG_CSD3_CFG_REG_CSD3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_CSD_CFG_REG_CSD3_CFG_REG_CSD3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_CSD_CFG_REG_CSD3_CFG_REG_CSD3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_CSD_CFG_REG_CSD3_CFG_REG_CSD3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SCR_CFG_REG_SCR0);
        sddc_csr.wo(utra::sddc::CR_REG_SCR_CFG_REG_SCR0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SCR_CFG_REG_SCR0_CFG_REG_SCR0);
        sddc_csr.rmwf(utra::sddc::CR_REG_SCR_CFG_REG_SCR0_CFG_REG_SCR0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SCR_CFG_REG_SCR0_CFG_REG_SCR0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SCR_CFG_REG_SCR0_CFG_REG_SCR0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SCR_CFG_REG_SCR0_CFG_REG_SCR0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SCR_CFG_REG_SCR1);
        sddc_csr.wo(utra::sddc::CR_REG_SCR_CFG_REG_SCR1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SCR_CFG_REG_SCR1_CFG_REG_SCR1);
        sddc_csr.rmwf(utra::sddc::CR_REG_SCR_CFG_REG_SCR1_CFG_REG_SCR1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SCR_CFG_REG_SCR1_CFG_REG_SCR1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SCR_CFG_REG_SCR1_CFG_REG_SCR1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SCR_CFG_REG_SCR1_CFG_REG_SCR1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0_CFG_REG_SD_STATUS0);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0_CFG_REG_SD_STATUS0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0_CFG_REG_SD_STATUS0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0_CFG_REG_SD_STATUS0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS0_CFG_REG_SD_STATUS0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1_CFG_REG_SD_STATUS1);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1_CFG_REG_SD_STATUS1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1_CFG_REG_SD_STATUS1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1_CFG_REG_SD_STATUS1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS1_CFG_REG_SD_STATUS1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2_CFG_REG_SD_STATUS2);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2_CFG_REG_SD_STATUS2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2_CFG_REG_SD_STATUS2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2_CFG_REG_SD_STATUS2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS2_CFG_REG_SD_STATUS2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3_CFG_REG_SD_STATUS3);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3_CFG_REG_SD_STATUS3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3_CFG_REG_SD_STATUS3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3_CFG_REG_SD_STATUS3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS3_CFG_REG_SD_STATUS3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4_CFG_REG_SD_STATUS4);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4_CFG_REG_SD_STATUS4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4_CFG_REG_SD_STATUS4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4_CFG_REG_SD_STATUS4, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS4_CFG_REG_SD_STATUS4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5_CFG_REG_SD_STATUS5);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5_CFG_REG_SD_STATUS5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5_CFG_REG_SD_STATUS5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5_CFG_REG_SD_STATUS5, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS5_CFG_REG_SD_STATUS5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6_CFG_REG_SD_STATUS6);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6_CFG_REG_SD_STATUS6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6_CFG_REG_SD_STATUS6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6_CFG_REG_SD_STATUS6, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS6_CFG_REG_SD_STATUS6, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7_CFG_REG_SD_STATUS7);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7_CFG_REG_SD_STATUS7, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7_CFG_REG_SD_STATUS7, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7_CFG_REG_SD_STATUS7, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS7_CFG_REG_SD_STATUS7, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8_CFG_REG_SD_STATUS8);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8_CFG_REG_SD_STATUS8, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8_CFG_REG_SD_STATUS8, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8_CFG_REG_SD_STATUS8, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS8_CFG_REG_SD_STATUS8, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9_CFG_REG_SD_STATUS9);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9_CFG_REG_SD_STATUS9, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9_CFG_REG_SD_STATUS9, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9_CFG_REG_SD_STATUS9, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS9_CFG_REG_SD_STATUS9, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10_CFG_REG_SD_STATUS10);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10_CFG_REG_SD_STATUS10, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10_CFG_REG_SD_STATUS10, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10_CFG_REG_SD_STATUS10, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS10_CFG_REG_SD_STATUS10, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11_CFG_REG_SD_STATUS11);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11_CFG_REG_SD_STATUS11, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11_CFG_REG_SD_STATUS11, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11_CFG_REG_SD_STATUS11, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS11_CFG_REG_SD_STATUS11, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12_CFG_REG_SD_STATUS12);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12_CFG_REG_SD_STATUS12, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12_CFG_REG_SD_STATUS12, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12_CFG_REG_SD_STATUS12, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS12_CFG_REG_SD_STATUS12, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13_CFG_REG_SD_STATUS13);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13_CFG_REG_SD_STATUS13, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13_CFG_REG_SD_STATUS13, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13_CFG_REG_SD_STATUS13, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS13_CFG_REG_SD_STATUS13, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14_CFG_REG_SD_STATUS14);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14_CFG_REG_SD_STATUS14, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14_CFG_REG_SD_STATUS14, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14_CFG_REG_SD_STATUS14, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS14_CFG_REG_SD_STATUS14, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15);
        sddc_csr.wo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15_CFG_REG_SD_STATUS15);
        sddc_csr.rmwf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15_CFG_REG_SD_STATUS15, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15_CFG_REG_SD_STATUS15, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15_CFG_REG_SD_STATUS15, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_SD_STATUS_CFG_REG_SD_STATUS15_CFG_REG_SD_STATUS15, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED60);
        sddc_csr.wo(utra::sddc::RESERVED60, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED60_RESERVED60);
        sddc_csr.rmwf(utra::sddc::RESERVED60_RESERVED60, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED60_RESERVED60, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED60_RESERVED60, 1);
        sddc_csr.wfo(utra::sddc::RESERVED60_RESERVED60, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED61);
        sddc_csr.wo(utra::sddc::RESERVED61, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED61_RESERVED61);
        sddc_csr.rmwf(utra::sddc::RESERVED61_RESERVED61, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED61_RESERVED61, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED61_RESERVED61, 1);
        sddc_csr.wfo(utra::sddc::RESERVED61_RESERVED61, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED62);
        sddc_csr.wo(utra::sddc::RESERVED62, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED62_RESERVED62);
        sddc_csr.rmwf(utra::sddc::RESERVED62_RESERVED62, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED62_RESERVED62, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED62_RESERVED62, 1);
        sddc_csr.wfo(utra::sddc::RESERVED62_RESERVED62, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED63);
        sddc_csr.wo(utra::sddc::RESERVED63, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED63_RESERVED63);
        sddc_csr.rmwf(utra::sddc::RESERVED63_RESERVED63, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED63_RESERVED63, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED63_RESERVED63, 1);
        sddc_csr.wfo(utra::sddc::RESERVED63_RESERVED63, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0_CFG_BASE_ADDR_MEM_FUNC0);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0_CFG_BASE_ADDR_MEM_FUNC0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0_CFG_BASE_ADDR_MEM_FUNC0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0_CFG_BASE_ADDR_MEM_FUNC0, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC0_CFG_BASE_ADDR_MEM_FUNC0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1_CFG_BASE_ADDR_MEM_FUNC1);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1_CFG_BASE_ADDR_MEM_FUNC1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1_CFG_BASE_ADDR_MEM_FUNC1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1_CFG_BASE_ADDR_MEM_FUNC1, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC1_CFG_BASE_ADDR_MEM_FUNC1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2_CFG_BASE_ADDR_MEM_FUNC2);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2_CFG_BASE_ADDR_MEM_FUNC2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2_CFG_BASE_ADDR_MEM_FUNC2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2_CFG_BASE_ADDR_MEM_FUNC2, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC2_CFG_BASE_ADDR_MEM_FUNC2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3_CFG_BASE_ADDR_MEM_FUNC3);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3_CFG_BASE_ADDR_MEM_FUNC3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3_CFG_BASE_ADDR_MEM_FUNC3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3_CFG_BASE_ADDR_MEM_FUNC3, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC3_CFG_BASE_ADDR_MEM_FUNC3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4_CFG_BASE_ADDR_MEM_FUNC4);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4_CFG_BASE_ADDR_MEM_FUNC4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4_CFG_BASE_ADDR_MEM_FUNC4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4_CFG_BASE_ADDR_MEM_FUNC4, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC4_CFG_BASE_ADDR_MEM_FUNC4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5_CFG_BASE_ADDR_MEM_FUNC5);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5_CFG_BASE_ADDR_MEM_FUNC5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5_CFG_BASE_ADDR_MEM_FUNC5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5_CFG_BASE_ADDR_MEM_FUNC5, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC5_CFG_BASE_ADDR_MEM_FUNC5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6_CFG_BASE_ADDR_MEM_FUNC6);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6_CFG_BASE_ADDR_MEM_FUNC6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6_CFG_BASE_ADDR_MEM_FUNC6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6_CFG_BASE_ADDR_MEM_FUNC6, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC6_CFG_BASE_ADDR_MEM_FUNC6, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7_CFG_BASE_ADDR_MEM_FUNC7);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7_CFG_BASE_ADDR_MEM_FUNC7, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7_CFG_BASE_ADDR_MEM_FUNC7, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7_CFG_BASE_ADDR_MEM_FUNC7, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC7_CFG_BASE_ADDR_MEM_FUNC7, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8_CFG_BASE_ADDR_MEM_FUNC8);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8_CFG_BASE_ADDR_MEM_FUNC8, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8_CFG_BASE_ADDR_MEM_FUNC8, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8_CFG_BASE_ADDR_MEM_FUNC8, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC8_CFG_BASE_ADDR_MEM_FUNC8, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9_CFG_BASE_ADDR_MEM_FUNC9);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9_CFG_BASE_ADDR_MEM_FUNC9, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9_CFG_BASE_ADDR_MEM_FUNC9, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9_CFG_BASE_ADDR_MEM_FUNC9, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC9_CFG_BASE_ADDR_MEM_FUNC9, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10_CFG_BASE_ADDR_MEM_FUNC10);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10_CFG_BASE_ADDR_MEM_FUNC10, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10_CFG_BASE_ADDR_MEM_FUNC10, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10_CFG_BASE_ADDR_MEM_FUNC10, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC10_CFG_BASE_ADDR_MEM_FUNC10, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11_CFG_BASE_ADDR_MEM_FUNC11);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11_CFG_BASE_ADDR_MEM_FUNC11, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11_CFG_BASE_ADDR_MEM_FUNC11, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11_CFG_BASE_ADDR_MEM_FUNC11, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC11_CFG_BASE_ADDR_MEM_FUNC11, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12_CFG_BASE_ADDR_MEM_FUNC12);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12_CFG_BASE_ADDR_MEM_FUNC12, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12_CFG_BASE_ADDR_MEM_FUNC12, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12_CFG_BASE_ADDR_MEM_FUNC12, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC12_CFG_BASE_ADDR_MEM_FUNC12, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13_CFG_BASE_ADDR_MEM_FUNC13);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13_CFG_BASE_ADDR_MEM_FUNC13, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13_CFG_BASE_ADDR_MEM_FUNC13, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13_CFG_BASE_ADDR_MEM_FUNC13, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC13_CFG_BASE_ADDR_MEM_FUNC13, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14_CFG_BASE_ADDR_MEM_FUNC14);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14_CFG_BASE_ADDR_MEM_FUNC14, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14_CFG_BASE_ADDR_MEM_FUNC14, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14_CFG_BASE_ADDR_MEM_FUNC14, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC14_CFG_BASE_ADDR_MEM_FUNC14, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15_CFG_BASE_ADDR_MEM_FUNC15);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15_CFG_BASE_ADDR_MEM_FUNC15, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15_CFG_BASE_ADDR_MEM_FUNC15, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15_CFG_BASE_ADDR_MEM_FUNC15, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC15_CFG_BASE_ADDR_MEM_FUNC15, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16_CFG_BASE_ADDR_MEM_FUNC16);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16_CFG_BASE_ADDR_MEM_FUNC16, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16_CFG_BASE_ADDR_MEM_FUNC16, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16_CFG_BASE_ADDR_MEM_FUNC16, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC16_CFG_BASE_ADDR_MEM_FUNC16, baz);

        let foo = sddc_csr.r(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17);
        sddc_csr.wo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17_CFG_BASE_ADDR_MEM_FUNC17);
        sddc_csr.rmwf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17_CFG_BASE_ADDR_MEM_FUNC17, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17_CFG_BASE_ADDR_MEM_FUNC17, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17_CFG_BASE_ADDR_MEM_FUNC17, 1);
        sddc_csr.wfo(utra::sddc::CR_BASE_ADDR_MEM_FUNC_CFG_BASE_ADDR_MEM_FUNC17_CFG_BASE_ADDR_MEM_FUNC17, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0_CFG_REG_FUNC_ISDIO_INTERFACE_CODE0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1_CFG_REG_FUNC_ISDIO_INTERFACE_CODE1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2_CFG_REG_FUNC_ISDIO_INTERFACE_CODE2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3_CFG_REG_FUNC_ISDIO_INTERFACE_CODE3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4_CFG_REG_FUNC_ISDIO_INTERFACE_CODE4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5_CFG_REG_FUNC_ISDIO_INTERFACE_CODE5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_INTERFACE_CODE_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6_CFG_REG_FUNC_ISDIO_INTERFACE_CODE6, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED89);
        sddc_csr.wo(utra::sddc::RESERVED89, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED89_RESERVED89);
        sddc_csr.rmwf(utra::sddc::RESERVED89_RESERVED89, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED89_RESERVED89, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED89_RESERVED89, 1);
        sddc_csr.wfo(utra::sddc::RESERVED89_RESERVED89, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0_CFG_REG_FUNC_MANUFACT_CODE0);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0_CFG_REG_FUNC_MANUFACT_CODE0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0_CFG_REG_FUNC_MANUFACT_CODE0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0_CFG_REG_FUNC_MANUFACT_CODE0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE0_CFG_REG_FUNC_MANUFACT_CODE0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1_CFG_REG_FUNC_MANUFACT_CODE1);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1_CFG_REG_FUNC_MANUFACT_CODE1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1_CFG_REG_FUNC_MANUFACT_CODE1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1_CFG_REG_FUNC_MANUFACT_CODE1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE1_CFG_REG_FUNC_MANUFACT_CODE1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2_CFG_REG_FUNC_MANUFACT_CODE2);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2_CFG_REG_FUNC_MANUFACT_CODE2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2_CFG_REG_FUNC_MANUFACT_CODE2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2_CFG_REG_FUNC_MANUFACT_CODE2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE2_CFG_REG_FUNC_MANUFACT_CODE2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3_CFG_REG_FUNC_MANUFACT_CODE3);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3_CFG_REG_FUNC_MANUFACT_CODE3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3_CFG_REG_FUNC_MANUFACT_CODE3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3_CFG_REG_FUNC_MANUFACT_CODE3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE3_CFG_REG_FUNC_MANUFACT_CODE3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4_CFG_REG_FUNC_MANUFACT_CODE4);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4_CFG_REG_FUNC_MANUFACT_CODE4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4_CFG_REG_FUNC_MANUFACT_CODE4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4_CFG_REG_FUNC_MANUFACT_CODE4, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE4_CFG_REG_FUNC_MANUFACT_CODE4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5_CFG_REG_FUNC_MANUFACT_CODE5);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5_CFG_REG_FUNC_MANUFACT_CODE5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5_CFG_REG_FUNC_MANUFACT_CODE5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5_CFG_REG_FUNC_MANUFACT_CODE5, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE5_CFG_REG_FUNC_MANUFACT_CODE5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6_CFG_REG_FUNC_MANUFACT_CODE6);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6_CFG_REG_FUNC_MANUFACT_CODE6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6_CFG_REG_FUNC_MANUFACT_CODE6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6_CFG_REG_FUNC_MANUFACT_CODE6, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_CODE_CFG_REG_FUNC_MANUFACT_CODE6_CFG_REG_FUNC_MANUFACT_CODE6, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED97);
        sddc_csr.wo(utra::sddc::RESERVED97, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED97_RESERVED97);
        sddc_csr.rmwf(utra::sddc::RESERVED97_RESERVED97, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED97_RESERVED97, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED97_RESERVED97, 1);
        sddc_csr.wfo(utra::sddc::RESERVED97_RESERVED97, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0_CFG_REG_FUNC_MANUFACT_INFO0);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0_CFG_REG_FUNC_MANUFACT_INFO0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0_CFG_REG_FUNC_MANUFACT_INFO0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0_CFG_REG_FUNC_MANUFACT_INFO0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO0_CFG_REG_FUNC_MANUFACT_INFO0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1_CFG_REG_FUNC_MANUFACT_INFO1);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1_CFG_REG_FUNC_MANUFACT_INFO1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1_CFG_REG_FUNC_MANUFACT_INFO1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1_CFG_REG_FUNC_MANUFACT_INFO1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO1_CFG_REG_FUNC_MANUFACT_INFO1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2_CFG_REG_FUNC_MANUFACT_INFO2);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2_CFG_REG_FUNC_MANUFACT_INFO2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2_CFG_REG_FUNC_MANUFACT_INFO2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2_CFG_REG_FUNC_MANUFACT_INFO2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO2_CFG_REG_FUNC_MANUFACT_INFO2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3_CFG_REG_FUNC_MANUFACT_INFO3);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3_CFG_REG_FUNC_MANUFACT_INFO3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3_CFG_REG_FUNC_MANUFACT_INFO3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3_CFG_REG_FUNC_MANUFACT_INFO3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO3_CFG_REG_FUNC_MANUFACT_INFO3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4_CFG_REG_FUNC_MANUFACT_INFO4);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4_CFG_REG_FUNC_MANUFACT_INFO4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4_CFG_REG_FUNC_MANUFACT_INFO4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4_CFG_REG_FUNC_MANUFACT_INFO4, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO4_CFG_REG_FUNC_MANUFACT_INFO4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5_CFG_REG_FUNC_MANUFACT_INFO5);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5_CFG_REG_FUNC_MANUFACT_INFO5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5_CFG_REG_FUNC_MANUFACT_INFO5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5_CFG_REG_FUNC_MANUFACT_INFO5, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO5_CFG_REG_FUNC_MANUFACT_INFO5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6_CFG_REG_FUNC_MANUFACT_INFO6);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6_CFG_REG_FUNC_MANUFACT_INFO6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6_CFG_REG_FUNC_MANUFACT_INFO6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6_CFG_REG_FUNC_MANUFACT_INFO6, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_MANUFACT_INFO_CFG_REG_FUNC_MANUFACT_INFO6_CFG_REG_FUNC_MANUFACT_INFO6, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED105);
        sddc_csr.wo(utra::sddc::RESERVED105, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED105_RESERVED105);
        sddc_csr.rmwf(utra::sddc::RESERVED105_RESERVED105, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED105_RESERVED105, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED105_RESERVED105, 1);
        sddc_csr.wfo(utra::sddc::RESERVED105_RESERVED105, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_ISDIO_TYPE_SUP_CODE_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6_CFG_REG_FUNC_ISDIO_TYPE_SUP_CODE6, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED113);
        sddc_csr.wo(utra::sddc::RESERVED113, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED113_RESERVED113);
        sddc_csr.rmwf(utra::sddc::RESERVED113_RESERVED113, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED113_RESERVED113, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED113_RESERVED113, 1);
        sddc_csr.wfo(utra::sddc::RESERVED113_RESERVED113, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0_CFG_REG_FUNC_INFO0);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0_CFG_REG_FUNC_INFO0, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0_CFG_REG_FUNC_INFO0, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0_CFG_REG_FUNC_INFO0, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO0_CFG_REG_FUNC_INFO0, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1_CFG_REG_FUNC_INFO1);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1_CFG_REG_FUNC_INFO1, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1_CFG_REG_FUNC_INFO1, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1_CFG_REG_FUNC_INFO1, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO1_CFG_REG_FUNC_INFO1, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2_CFG_REG_FUNC_INFO2);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2_CFG_REG_FUNC_INFO2, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2_CFG_REG_FUNC_INFO2, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2_CFG_REG_FUNC_INFO2, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO2_CFG_REG_FUNC_INFO2, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3_CFG_REG_FUNC_INFO3);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3_CFG_REG_FUNC_INFO3, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3_CFG_REG_FUNC_INFO3, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3_CFG_REG_FUNC_INFO3, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO3_CFG_REG_FUNC_INFO3, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4_CFG_REG_FUNC_INFO4);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4_CFG_REG_FUNC_INFO4, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4_CFG_REG_FUNC_INFO4, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4_CFG_REG_FUNC_INFO4, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO4_CFG_REG_FUNC_INFO4, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5_CFG_REG_FUNC_INFO5);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5_CFG_REG_FUNC_INFO5, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5_CFG_REG_FUNC_INFO5, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5_CFG_REG_FUNC_INFO5, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO5_CFG_REG_FUNC_INFO5, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6);
        sddc_csr.wo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6_CFG_REG_FUNC_INFO6);
        sddc_csr.rmwf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6_CFG_REG_FUNC_INFO6, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6_CFG_REG_FUNC_INFO6, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6_CFG_REG_FUNC_INFO6, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_FUNC_INFO_CFG_REG_FUNC_INFO6_CFG_REG_FUNC_INFO6, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED121);
        sddc_csr.wo(utra::sddc::RESERVED121, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED121_RESERVED121);
        sddc_csr.rmwf(utra::sddc::RESERVED121_RESERVED121, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED121_RESERVED121, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED121_RESERVED121, 1);
        sddc_csr.wfo(utra::sddc::RESERVED121_RESERVED121, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED122);
        sddc_csr.wo(utra::sddc::RESERVED122, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED122_RESERVED122);
        sddc_csr.rmwf(utra::sddc::RESERVED122_RESERVED122, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED122_RESERVED122, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED122_RESERVED122, 1);
        sddc_csr.wfo(utra::sddc::RESERVED122_RESERVED122, baz);

        let foo = sddc_csr.r(utra::sddc::RESERVED123);
        sddc_csr.wo(utra::sddc::RESERVED123, foo);
        let bar = sddc_csr.rf(utra::sddc::RESERVED123_RESERVED123);
        sddc_csr.rmwf(utra::sddc::RESERVED123_RESERVED123, bar);
        let mut baz = sddc_csr.zf(utra::sddc::RESERVED123_RESERVED123, bar);
        baz |= sddc_csr.ms(utra::sddc::RESERVED123_RESERVED123, 1);
        sddc_csr.wfo(utra::sddc::RESERVED123_RESERVED123, baz);

        let foo = sddc_csr.r(utra::sddc::CR_REG_UHS_1_SUPPORT);
        sddc_csr.wo(utra::sddc::CR_REG_UHS_1_SUPPORT, foo);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT);
        sddc_csr.rmwf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT, baz);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION);
        sddc_csr.rmwf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION, baz);
        let bar = sddc_csr.rf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT);
        sddc_csr.rmwf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT, bar);
        let mut baz = sddc_csr.zf(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT, bar);
        baz |= sddc_csr.ms(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT, 1);
        sddc_csr.wfo(utra::sddc::CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_mdma_csr() {
        use super::*;
        let mut mdma_csr = CSR::new(HW_MDMA_BASE as *mut u32);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL0);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL0, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL0_CR_EVSEL0);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL0_CR_EVSEL0, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL0_CR_EVSEL0, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL0_CR_EVSEL0, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL0_CR_EVSEL0, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL1);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL1, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL1_CR_EVSEL1);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL1_CR_EVSEL1, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL1_CR_EVSEL1, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL1_CR_EVSEL1, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL1_CR_EVSEL1, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL2);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL2, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL2_CR_EVSEL2);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL2_CR_EVSEL2, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL2_CR_EVSEL2, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL2_CR_EVSEL2, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL2_CR_EVSEL2, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL3);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL3, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL3_CR_EVSEL3);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL3_CR_EVSEL3, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL3_CR_EVSEL3, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL3_CR_EVSEL3, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL3_CR_EVSEL3, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL4);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL4, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL4_CR_EVSEL4);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL4_CR_EVSEL4, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL4_CR_EVSEL4, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL4_CR_EVSEL4, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL4_CR_EVSEL4, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL5);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL5, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL5_CR_EVSEL5);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL5_CR_EVSEL5, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL5_CR_EVSEL5, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL5_CR_EVSEL5, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL5_CR_EVSEL5, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL6);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL6, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL6_CR_EVSEL6);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL6_CR_EVSEL6, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL6_CR_EVSEL6, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL6_CR_EVSEL6, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL6_CR_EVSEL6, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_EVSEL_CR_EVSEL7);
        mdma_csr.wo(utra::mdma::SFR_EVSEL_CR_EVSEL7, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_EVSEL_CR_EVSEL7_CR_EVSEL7);
        mdma_csr.rmwf(utra::mdma::SFR_EVSEL_CR_EVSEL7_CR_EVSEL7, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_EVSEL_CR_EVSEL7_CR_EVSEL7, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_EVSEL_CR_EVSEL7_CR_EVSEL7, 1);
        mdma_csr.wfo(utra::mdma::SFR_EVSEL_CR_EVSEL7_CR_EVSEL7, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ0);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ0, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ0_CR_MDMAREQ0);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ0_CR_MDMAREQ0, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ0_CR_MDMAREQ0, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ0_CR_MDMAREQ0, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ0_CR_MDMAREQ0, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ1);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ1, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ1_CR_MDMAREQ1);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ1_CR_MDMAREQ1, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ1_CR_MDMAREQ1, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ1_CR_MDMAREQ1, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ1_CR_MDMAREQ1, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ2);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ2, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ2_CR_MDMAREQ2);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ2_CR_MDMAREQ2, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ2_CR_MDMAREQ2, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ2_CR_MDMAREQ2, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ2_CR_MDMAREQ2, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ3);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ3, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ3_CR_MDMAREQ3);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ3_CR_MDMAREQ3, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ3_CR_MDMAREQ3, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ3_CR_MDMAREQ3, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ3_CR_MDMAREQ3, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ4);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ4, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ4_CR_MDMAREQ4);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ4_CR_MDMAREQ4, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ4_CR_MDMAREQ4, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ4_CR_MDMAREQ4, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ4_CR_MDMAREQ4, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ5);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ5, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ5_CR_MDMAREQ5);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ5_CR_MDMAREQ5, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ5_CR_MDMAREQ5, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ5_CR_MDMAREQ5, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ5_CR_MDMAREQ5, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ6);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ6, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ6_CR_MDMAREQ6);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ6_CR_MDMAREQ6, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ6_CR_MDMAREQ6, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ6_CR_MDMAREQ6, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ6_CR_MDMAREQ6, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_CR_CR_MDMAREQ7);
        mdma_csr.wo(utra::mdma::SFR_CR_CR_MDMAREQ7, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_CR_CR_MDMAREQ7_CR_MDMAREQ7);
        mdma_csr.rmwf(utra::mdma::SFR_CR_CR_MDMAREQ7_CR_MDMAREQ7, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_CR_CR_MDMAREQ7_CR_MDMAREQ7, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_CR_CR_MDMAREQ7_CR_MDMAREQ7, 1);
        mdma_csr.wfo(utra::mdma::SFR_CR_CR_MDMAREQ7_CR_MDMAREQ7, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ0);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ0, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ0_SR_MDMAREQ0);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ0_SR_MDMAREQ0, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ0_SR_MDMAREQ0, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ0_SR_MDMAREQ0, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ0_SR_MDMAREQ0, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ1);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ1, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ1_SR_MDMAREQ1);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ1_SR_MDMAREQ1, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ1_SR_MDMAREQ1, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ1_SR_MDMAREQ1, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ1_SR_MDMAREQ1, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ2);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ2, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ2_SR_MDMAREQ2);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ2_SR_MDMAREQ2, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ2_SR_MDMAREQ2, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ2_SR_MDMAREQ2, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ2_SR_MDMAREQ2, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ3);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ3, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ3_SR_MDMAREQ3);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ3_SR_MDMAREQ3, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ3_SR_MDMAREQ3, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ3_SR_MDMAREQ3, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ3_SR_MDMAREQ3, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ4);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ4, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ4_SR_MDMAREQ4);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ4_SR_MDMAREQ4, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ4_SR_MDMAREQ4, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ4_SR_MDMAREQ4, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ4_SR_MDMAREQ4, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ5);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ5, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ5_SR_MDMAREQ5);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ5_SR_MDMAREQ5, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ5_SR_MDMAREQ5, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ5_SR_MDMAREQ5, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ5_SR_MDMAREQ5, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ6);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ6, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ6_SR_MDMAREQ6);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ6_SR_MDMAREQ6, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ6_SR_MDMAREQ6, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ6_SR_MDMAREQ6, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ6_SR_MDMAREQ6, baz);

        let foo = mdma_csr.r(utra::mdma::SFR_SR_SR_MDMAREQ7);
        mdma_csr.wo(utra::mdma::SFR_SR_SR_MDMAREQ7, foo);
        let bar = mdma_csr.rf(utra::mdma::SFR_SR_SR_MDMAREQ7_SR_MDMAREQ7);
        mdma_csr.rmwf(utra::mdma::SFR_SR_SR_MDMAREQ7_SR_MDMAREQ7, bar);
        let mut baz = mdma_csr.zf(utra::mdma::SFR_SR_SR_MDMAREQ7_SR_MDMAREQ7, bar);
        baz |= mdma_csr.ms(utra::mdma::SFR_SR_SR_MDMAREQ7_SR_MDMAREQ7, 1);
        mdma_csr.wfo(utra::mdma::SFR_SR_SR_MDMAREQ7_SR_MDMAREQ7, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_qfc_csr() {
        use super::*;
        let mut qfc_csr = CSR::new(HW_QFC_BASE as *mut u32);

        let foo = qfc_csr.r(utra::qfc::SFR_IO);
        qfc_csr.wo(utra::qfc::SFR_IO, foo);
        let bar = qfc_csr.rf(utra::qfc::SFR_IO_SFR_IO);
        qfc_csr.rmwf(utra::qfc::SFR_IO_SFR_IO, bar);
        let mut baz = qfc_csr.zf(utra::qfc::SFR_IO_SFR_IO, bar);
        baz |= qfc_csr.ms(utra::qfc::SFR_IO_SFR_IO, 1);
        qfc_csr.wfo(utra::qfc::SFR_IO_SFR_IO, baz);

        let foo = qfc_csr.r(utra::qfc::SFR_AR);
        qfc_csr.wo(utra::qfc::SFR_AR, foo);
        let bar = qfc_csr.rf(utra::qfc::SFR_AR_SFR_AR);
        qfc_csr.rmwf(utra::qfc::SFR_AR_SFR_AR, bar);
        let mut baz = qfc_csr.zf(utra::qfc::SFR_AR_SFR_AR, bar);
        baz |= qfc_csr.ms(utra::qfc::SFR_AR_SFR_AR, 1);
        qfc_csr.wfo(utra::qfc::SFR_AR_SFR_AR, baz);

        let foo = qfc_csr.r(utra::qfc::SFR_IODRV);
        qfc_csr.wo(utra::qfc::SFR_IODRV, foo);
        let bar = qfc_csr.rf(utra::qfc::SFR_IODRV_PADDRVSEL);
        qfc_csr.rmwf(utra::qfc::SFR_IODRV_PADDRVSEL, bar);
        let mut baz = qfc_csr.zf(utra::qfc::SFR_IODRV_PADDRVSEL, bar);
        baz |= qfc_csr.ms(utra::qfc::SFR_IODRV_PADDRVSEL, 1);
        qfc_csr.wfo(utra::qfc::SFR_IODRV_PADDRVSEL, baz);

        let foo = qfc_csr.r(utra::qfc::RESERVED3);
        qfc_csr.wo(utra::qfc::RESERVED3, foo);
        let bar = qfc_csr.rf(utra::qfc::RESERVED3_RESERVED3);
        qfc_csr.rmwf(utra::qfc::RESERVED3_RESERVED3, bar);
        let mut baz = qfc_csr.zf(utra::qfc::RESERVED3_RESERVED3, bar);
        baz |= qfc_csr.ms(utra::qfc::RESERVED3_RESERVED3, 1);
        qfc_csr.wfo(utra::qfc::RESERVED3_RESERVED3, baz);

        let foo = qfc_csr.r(utra::qfc::CR_XIP_ADDRMODE);
        qfc_csr.wo(utra::qfc::CR_XIP_ADDRMODE, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_XIP_ADDRMODE_CR_XIP_ADDRMODE);
        qfc_csr.rmwf(utra::qfc::CR_XIP_ADDRMODE_CR_XIP_ADDRMODE, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_XIP_ADDRMODE_CR_XIP_ADDRMODE, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_XIP_ADDRMODE_CR_XIP_ADDRMODE, 1);
        qfc_csr.wfo(utra::qfc::CR_XIP_ADDRMODE_CR_XIP_ADDRMODE, baz);

        let foo = qfc_csr.r(utra::qfc::CR_XIP_OPCODE);
        qfc_csr.wo(utra::qfc::CR_XIP_OPCODE, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_XIP_OPCODE_CR_XIP_OPCODE);
        qfc_csr.rmwf(utra::qfc::CR_XIP_OPCODE_CR_XIP_OPCODE, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_XIP_OPCODE_CR_XIP_OPCODE, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_XIP_OPCODE_CR_XIP_OPCODE, 1);
        qfc_csr.wfo(utra::qfc::CR_XIP_OPCODE_CR_XIP_OPCODE, baz);

        let foo = qfc_csr.r(utra::qfc::CR_XIP_WIDTH);
        qfc_csr.wo(utra::qfc::CR_XIP_WIDTH, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_XIP_WIDTH_CR_XIP_WIDTH);
        qfc_csr.rmwf(utra::qfc::CR_XIP_WIDTH_CR_XIP_WIDTH, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_XIP_WIDTH_CR_XIP_WIDTH, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_XIP_WIDTH_CR_XIP_WIDTH, 1);
        qfc_csr.wfo(utra::qfc::CR_XIP_WIDTH_CR_XIP_WIDTH, baz);

        let foo = qfc_csr.r(utra::qfc::CR_XIP_SSEL);
        qfc_csr.wo(utra::qfc::CR_XIP_SSEL, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_XIP_SSEL_CR_XIP_SSEL);
        qfc_csr.rmwf(utra::qfc::CR_XIP_SSEL_CR_XIP_SSEL, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_XIP_SSEL_CR_XIP_SSEL, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_XIP_SSEL_CR_XIP_SSEL, 1);
        qfc_csr.wfo(utra::qfc::CR_XIP_SSEL_CR_XIP_SSEL, baz);

        let foo = qfc_csr.r(utra::qfc::CR_XIP_DUMCYC);
        qfc_csr.wo(utra::qfc::CR_XIP_DUMCYC, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_XIP_DUMCYC_CR_XIP_DUMCYC);
        qfc_csr.rmwf(utra::qfc::CR_XIP_DUMCYC_CR_XIP_DUMCYC, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_XIP_DUMCYC_CR_XIP_DUMCYC, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_XIP_DUMCYC_CR_XIP_DUMCYC, 1);
        qfc_csr.wfo(utra::qfc::CR_XIP_DUMCYC_CR_XIP_DUMCYC, baz);

        let foo = qfc_csr.r(utra::qfc::CR_XIP_CFG);
        qfc_csr.wo(utra::qfc::CR_XIP_CFG, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_XIP_CFG_CR_XIP_CFG);
        qfc_csr.rmwf(utra::qfc::CR_XIP_CFG_CR_XIP_CFG, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_XIP_CFG_CR_XIP_CFG, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_XIP_CFG_CR_XIP_CFG, 1);
        qfc_csr.wfo(utra::qfc::CR_XIP_CFG_CR_XIP_CFG, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_pl230_csr() {
        use super::*;
        let mut pl230_csr = CSR::new(HW_PL230_BASE as *mut u32);

        let foo = pl230_csr.r(utra::pl230::PL230);
        pl230_csr.wo(utra::pl230::PL230, foo);
        let bar = pl230_csr.rf(utra::pl230::PL230_PLACEHOLDER);
        pl230_csr.rmwf(utra::pl230::PL230_PLACEHOLDER, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PL230_PLACEHOLDER, bar);
        baz |= pl230_csr.ms(utra::pl230::PL230_PLACEHOLDER, 1);
        pl230_csr.wfo(utra::pl230::PL230_PLACEHOLDER, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_gluechain_csr() {
        use super::*;
        let mut gluechain_csr = CSR::new(HW_GLUECHAIN_BASE as *mut u32);

        let foo = gluechain_csr.r(utra::gluechain::SFR_GCMASK);
        gluechain_csr.wo(utra::gluechain::SFR_GCMASK, foo);
        let bar = gluechain_csr.rf(utra::gluechain::SFR_GCMASK_CR_GCMASK);
        gluechain_csr.rmwf(utra::gluechain::SFR_GCMASK_CR_GCMASK, bar);
        let mut baz = gluechain_csr.zf(utra::gluechain::SFR_GCMASK_CR_GCMASK, bar);
        baz |= gluechain_csr.ms(utra::gluechain::SFR_GCMASK_CR_GCMASK, 1);
        gluechain_csr.wfo(utra::gluechain::SFR_GCMASK_CR_GCMASK, baz);

        let foo = gluechain_csr.r(utra::gluechain::SFR_GCSR);
        gluechain_csr.wo(utra::gluechain::SFR_GCSR, foo);
        let bar = gluechain_csr.rf(utra::gluechain::SFR_GCSR_GLUEREG);
        gluechain_csr.rmwf(utra::gluechain::SFR_GCSR_GLUEREG, bar);
        let mut baz = gluechain_csr.zf(utra::gluechain::SFR_GCSR_GLUEREG, bar);
        baz |= gluechain_csr.ms(utra::gluechain::SFR_GCSR_GLUEREG, 1);
        gluechain_csr.wfo(utra::gluechain::SFR_GCSR_GLUEREG, baz);

        let foo = gluechain_csr.r(utra::gluechain::SFR_GCRST);
        gluechain_csr.wo(utra::gluechain::SFR_GCRST, foo);
        let bar = gluechain_csr.rf(utra::gluechain::SFR_GCRST_GLUERST);
        gluechain_csr.rmwf(utra::gluechain::SFR_GCRST_GLUERST, bar);
        let mut baz = gluechain_csr.zf(utra::gluechain::SFR_GCRST_GLUERST, bar);
        baz |= gluechain_csr.ms(utra::gluechain::SFR_GCRST_GLUERST, 1);
        gluechain_csr.wfo(utra::gluechain::SFR_GCRST_GLUERST, baz);

        let foo = gluechain_csr.r(utra::gluechain::SFR_GCTEST);
        gluechain_csr.wo(utra::gluechain::SFR_GCTEST, foo);
        let bar = gluechain_csr.rf(utra::gluechain::SFR_GCTEST_GLUETEST);
        gluechain_csr.rmwf(utra::gluechain::SFR_GCTEST_GLUETEST, bar);
        let mut baz = gluechain_csr.zf(utra::gluechain::SFR_GCTEST_GLUETEST, bar);
        baz |= gluechain_csr.ms(utra::gluechain::SFR_GCTEST_GLUETEST, 1);
        gluechain_csr.wfo(utra::gluechain::SFR_GCTEST_GLUETEST, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_mesh_csr() {
        use super::*;
        let mut mesh_csr = CSR::new(HW_MESH_BASE as *mut u32);

        let foo = mesh_csr.r(utra::mesh::SFR_MLDRV_CR_MLDRV0);
        mesh_csr.wo(utra::mesh::SFR_MLDRV_CR_MLDRV0, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLDRV_CR_MLDRV0_CR_MLDRV0);
        mesh_csr.rmwf(utra::mesh::SFR_MLDRV_CR_MLDRV0_CR_MLDRV0, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLDRV_CR_MLDRV0_CR_MLDRV0, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLDRV_CR_MLDRV0_CR_MLDRV0, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLDRV_CR_MLDRV0_CR_MLDRV0, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLIE_CR_MLIE0);
        mesh_csr.wo(utra::mesh::SFR_MLIE_CR_MLIE0, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLIE_CR_MLIE0_CR_MLIE0);
        mesh_csr.rmwf(utra::mesh::SFR_MLIE_CR_MLIE0_CR_MLIE0, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLIE_CR_MLIE0_CR_MLIE0, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLIE_CR_MLIE0_CR_MLIE0, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLIE_CR_MLIE0_CR_MLIE0, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR0);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR0, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR0_SR_MLSR0);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR0_SR_MLSR0, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR0_SR_MLSR0, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR0_SR_MLSR0, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR0_SR_MLSR0, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR1);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR1, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR1_SR_MLSR1);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR1_SR_MLSR1, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR1_SR_MLSR1, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR1_SR_MLSR1, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR1_SR_MLSR1, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR2);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR2, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR2_SR_MLSR2);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR2_SR_MLSR2, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR2_SR_MLSR2, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR2_SR_MLSR2, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR2_SR_MLSR2, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR3);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR3, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR3_SR_MLSR3);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR3_SR_MLSR3, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR3_SR_MLSR3, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR3_SR_MLSR3, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR3_SR_MLSR3, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR4);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR4, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR4_SR_MLSR4);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR4_SR_MLSR4, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR4_SR_MLSR4, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR4_SR_MLSR4, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR4_SR_MLSR4, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR5);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR5, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR5_SR_MLSR5);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR5_SR_MLSR5, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR5_SR_MLSR5, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR5_SR_MLSR5, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR5_SR_MLSR5, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR6);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR6, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR6_SR_MLSR6);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR6_SR_MLSR6, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR6_SR_MLSR6, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR6_SR_MLSR6, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR6_SR_MLSR6, baz);

        let foo = mesh_csr.r(utra::mesh::SFR_MLSR_SR_MLSR7);
        mesh_csr.wo(utra::mesh::SFR_MLSR_SR_MLSR7, foo);
        let bar = mesh_csr.rf(utra::mesh::SFR_MLSR_SR_MLSR7_SR_MLSR7);
        mesh_csr.rmwf(utra::mesh::SFR_MLSR_SR_MLSR7_SR_MLSR7, bar);
        let mut baz = mesh_csr.zf(utra::mesh::SFR_MLSR_SR_MLSR7_SR_MLSR7, bar);
        baz |= mesh_csr.ms(utra::mesh::SFR_MLSR_SR_MLSR7_SR_MLSR7, 1);
        mesh_csr.wfo(utra::mesh::SFR_MLSR_SR_MLSR7_SR_MLSR7, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_sensorc_csr() {
        use super::*;
        let mut sensorc_csr = CSR::new(HW_SENSORC_BASE as *mut u32);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDMASK0);
        sensorc_csr.wo(utra::sensorc::SFR_VDMASK0, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDMASK0_CR_VDMASK0);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDMASK0_CR_VDMASK0, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDMASK0_CR_VDMASK0, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDMASK0_CR_VDMASK0, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDMASK0_CR_VDMASK0, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDMASK1);
        sensorc_csr.wo(utra::sensorc::SFR_VDMASK1, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDMASK1_CR_VDMASK1);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDMASK1_CR_VDMASK1, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDMASK1_CR_VDMASK1, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDMASK1_CR_VDMASK1, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDMASK1_CR_VDMASK1, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDSR);
        sensorc_csr.wo(utra::sensorc::SFR_VDSR, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDSR_SR_VDSR);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDSR_SR_VDSR, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDSR_SR_VDSR, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDSR_SR_VDSR, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDSR_SR_VDSR, baz);

        let foo = sensorc_csr.r(utra::sensorc::RESERVED3);
        sensorc_csr.wo(utra::sensorc::RESERVED3, foo);
        let bar = sensorc_csr.rf(utra::sensorc::RESERVED3_RESERVED3);
        sensorc_csr.rmwf(utra::sensorc::RESERVED3_RESERVED3, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::RESERVED3_RESERVED3, bar);
        baz |= sensorc_csr.ms(utra::sensorc::RESERVED3_RESERVED3, 1);
        sensorc_csr.wfo(utra::sensorc::RESERVED3_RESERVED3, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_LDMASK);
        sensorc_csr.wo(utra::sensorc::SFR_LDMASK, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_LDMASK_CR_LDMASK);
        sensorc_csr.rmwf(utra::sensorc::SFR_LDMASK_CR_LDMASK, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_LDMASK_CR_LDMASK, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_LDMASK_CR_LDMASK, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_LDMASK_CR_LDMASK, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_LDSR);
        sensorc_csr.wo(utra::sensorc::SFR_LDSR, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_LDSR_SR_LDSR);
        sensorc_csr.rmwf(utra::sensorc::SFR_LDSR_SR_LDSR, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_LDSR_SR_LDSR, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_LDSR_SR_LDSR, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_LDSR_SR_LDSR, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_LDCFG);
        sensorc_csr.wo(utra::sensorc::SFR_LDCFG, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_LDCFG_SFR_LDCFG);
        sensorc_csr.rmwf(utra::sensorc::SFR_LDCFG_SFR_LDCFG, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_LDCFG_SFR_LDCFG, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_LDCFG_SFR_LDCFG, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_LDCFG_SFR_LDCFG, baz);

        let foo = sensorc_csr.r(utra::sensorc::RESERVED7);
        sensorc_csr.wo(utra::sensorc::RESERVED7, foo);
        let bar = sensorc_csr.rf(utra::sensorc::RESERVED7_RESERVED7);
        sensorc_csr.rmwf(utra::sensorc::RESERVED7_RESERVED7, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::RESERVED7_RESERVED7, bar);
        baz |= sensorc_csr.ms(utra::sensorc::RESERVED7_RESERVED7, 1);
        sensorc_csr.wfo(utra::sensorc::RESERVED7_RESERVED7, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG0);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG0, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG0_CR_VDCFG0);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG0_CR_VDCFG0, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG0_CR_VDCFG0, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG0_CR_VDCFG0, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG0_CR_VDCFG0, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG1);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG1, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG1_CR_VDCFG1);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG1_CR_VDCFG1, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG1_CR_VDCFG1, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG1_CR_VDCFG1, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG1_CR_VDCFG1, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG2);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG2, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG2_CR_VDCFG2);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG2_CR_VDCFG2, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG2_CR_VDCFG2, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG2_CR_VDCFG2, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG2_CR_VDCFG2, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG3);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG3, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG3_CR_VDCFG3);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG3_CR_VDCFG3, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG3_CR_VDCFG3, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG3_CR_VDCFG3, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG3_CR_VDCFG3, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG4);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG4, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG4_CR_VDCFG4);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG4_CR_VDCFG4, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG4_CR_VDCFG4, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG4_CR_VDCFG4, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG4_CR_VDCFG4, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG5);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG5, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG5_CR_VDCFG5);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG5_CR_VDCFG5, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG5_CR_VDCFG5, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG5_CR_VDCFG5, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG5_CR_VDCFG5, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG6);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG6, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG6_CR_VDCFG6);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG6_CR_VDCFG6, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG6_CR_VDCFG6, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG6_CR_VDCFG6, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG6_CR_VDCFG6, baz);

        let foo = sensorc_csr.r(utra::sensorc::SFR_VDCFG_CR_VDCFG7);
        sensorc_csr.wo(utra::sensorc::SFR_VDCFG_CR_VDCFG7, foo);
        let bar = sensorc_csr.rf(utra::sensorc::SFR_VDCFG_CR_VDCFG7_CR_VDCFG7);
        sensorc_csr.rmwf(utra::sensorc::SFR_VDCFG_CR_VDCFG7_CR_VDCFG7, bar);
        let mut baz = sensorc_csr.zf(utra::sensorc::SFR_VDCFG_CR_VDCFG7_CR_VDCFG7, bar);
        baz |= sensorc_csr.ms(utra::sensorc::SFR_VDCFG_CR_VDCFG7_CR_VDCFG7, 1);
        sensorc_csr.wfo(utra::sensorc::SFR_VDCFG_CR_VDCFG7_CR_VDCFG7, baz);
  }
}
