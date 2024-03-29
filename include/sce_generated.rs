
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

// Physical base addresses of registers
pub const HW_AES_BASE :   usize = 0x4002d000;
pub const HW_COMBOHASH_BASE :   usize = 0x4002b000;
pub const HW_PKE_BASE :   usize = 0x4002c000;
pub const HW_SCEDMA_BASE :   usize = 0x40029000;
pub const HW_SCE_GLBSFR_BASE :   usize = 0x40028000;
pub const HW_TRNG_BASE :   usize = 0x4002e000;
pub const HW_ALU_BASE :   usize = 0x4002f000;


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
}
