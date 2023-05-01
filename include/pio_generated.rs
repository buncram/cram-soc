
#![allow(dead_code)]
use core::convert::TryInto;
use core::sync::atomic::AtomicPtr;

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
        // Asserts don't work in const fn yet.
        // assert!(width != 0, "field width cannot be 0");
        // assert!((width + offset) < 32, "field with and offset must fit within a 32-bit value");
        // It would be lovely if we could call `usize::pow()` in a const fn.
        let mask = match width {
            0 => 0,
            1 => 1,
            2 => 3,
            3 => 7,
            4 => 15,
            5 => 31,
            6 => 63,
            7 => 127,
            8 => 255,
            9 => 511,
            10 => 1023,
            11 => 2047,
            12 => 4095,
            13 => 8191,
            14 => 16383,
            15 => 32767,
            16 => 65535,
            17 => 131071,
            18 => 262143,
            19 => 524287,
            20 => 1048575,
            21 => 2097151,
            22 => 4194303,
            23 => 8388607,
            24 => 16777215,
            25 => 33554431,
            26 => 67108863,
            27 => 134217727,
            28 => 268435455,
            29 => 536870911,
            30 => 1073741823,
            31 => 2147483647,
            32 => 4294967295,
            _ => 0,
        };
        Field {
            mask,
            offset,
            register,
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct CSR<T> {
    pub base: *mut T,
}
impl<T> CSR<T>
where
    T: core::convert::TryFrom<usize> + core::convert::TryInto<usize> + core::default::Default,
{
    pub fn new(base: *mut T) -> Self {
        CSR { base }
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
pub struct AtomicCsr<T> {
    pub base: AtomicPtr<T>,
}
impl<T> AtomicCsr<T>
where
    T: core::convert::TryFrom<usize> + core::convert::TryInto<usize> + core::default::Default,
{
    pub fn new(base: *mut T) -> Self {
        AtomicCsr {
            base: AtomicPtr::new(base)
        }
    }
    /// In reality, we should wrap this in an `Arc` so we can be truly safe across a multi-core
    /// implementation, but for our single-core system this is fine. The reason we don't do it
    /// immediately is that UTRA also needs to work in a `no_std` environment, where `Arc`
    /// does not exist, and so additional config flags would need to be introduced to not break
    /// that compability issue. If migrating to multicore, this technical debt would have to be
    /// addressed.
    pub fn clone(&self) -> Self {
        AtomicCsr {
            base: AtomicPtr::new(self.base.load(core::sync::atomic::Ordering::SeqCst))
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
pub const HW_PIO_MEM:     usize = 0x40202000;
pub const HW_PIO_MEM_LEN: usize = 32768;

// Physical base addresses of registers
pub const HW_PIO_BASE :   usize = 0x40202000;


pub mod utra {

    pub mod pio {
        pub const PIO_NUMREGS: usize = 81;

        pub const SFR_CTRL: crate::Register = crate::Register::new(0, 0xfff);
        pub const SFR_CTRL_EN: crate::Field = crate::Field::new(4, 0, SFR_CTRL);
        pub const SFR_CTRL_RESTART: crate::Field = crate::Field::new(4, 4, SFR_CTRL);
        pub const SFR_CTRL_CLKDIV_RESTART: crate::Field = crate::Field::new(4, 8, SFR_CTRL);

        pub const SFR_FSTAT: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_FSTAT_RX_FULL: crate::Field = crate::Field::new(4, 0, SFR_FSTAT);
        pub const SFR_FSTAT_CONSTANT0: crate::Field = crate::Field::new(4, 4, SFR_FSTAT);
        pub const SFR_FSTAT_RX_EMPTY: crate::Field = crate::Field::new(4, 8, SFR_FSTAT);
        pub const SFR_FSTAT_CONSTANT1: crate::Field = crate::Field::new(4, 12, SFR_FSTAT);
        pub const SFR_FSTAT_TX_FULL: crate::Field = crate::Field::new(4, 16, SFR_FSTAT);
        pub const SFR_FSTAT_CONSTANT2: crate::Field = crate::Field::new(4, 20, SFR_FSTAT);
        pub const SFR_FSTAT_TX_EMPTY: crate::Field = crate::Field::new(4, 24, SFR_FSTAT);
        pub const SFR_FSTAT_CONSTANT3: crate::Field = crate::Field::new(4, 28, SFR_FSTAT);

        pub const SFR_FDEBUG: crate::Register = crate::Register::new(2, 0xffffffff);
        pub const SFR_FDEBUG_RXSTALL: crate::Field = crate::Field::new(4, 0, SFR_FDEBUG);
        pub const SFR_FDEBUG_CONSTANT0: crate::Field = crate::Field::new(4, 4, SFR_FDEBUG);
        pub const SFR_FDEBUG_RXUNDER: crate::Field = crate::Field::new(4, 8, SFR_FDEBUG);
        pub const SFR_FDEBUG_CONSTANT1: crate::Field = crate::Field::new(4, 12, SFR_FDEBUG);
        pub const SFR_FDEBUG_TXOVER: crate::Field = crate::Field::new(4, 16, SFR_FDEBUG);
        pub const SFR_FDEBUG_CONSTANT2: crate::Field = crate::Field::new(4, 20, SFR_FDEBUG);
        pub const SFR_FDEBUG_TXSTALL: crate::Field = crate::Field::new(4, 24, SFR_FDEBUG);
        pub const SFR_FDEBUG_CONSTANT3: crate::Field = crate::Field::new(4, 28, SFR_FDEBUG);

        pub const SFR_FLEVEL: crate::Register = crate::Register::new(3, 0xffffffff);
        pub const SFR_FLEVEL_TX_LEVEL0: crate::Field = crate::Field::new(3, 0, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT0: crate::Field = crate::Field::new(1, 3, SFR_FLEVEL);
        pub const SFR_FLEVEL_RX_LEVEL0: crate::Field = crate::Field::new(3, 4, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT1: crate::Field = crate::Field::new(1, 7, SFR_FLEVEL);
        pub const SFR_FLEVEL_TX_LEVEL1: crate::Field = crate::Field::new(3, 8, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT2: crate::Field = crate::Field::new(1, 11, SFR_FLEVEL);
        pub const SFR_FLEVEL_RX_LEVEL1: crate::Field = crate::Field::new(3, 12, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT3: crate::Field = crate::Field::new(1, 15, SFR_FLEVEL);
        pub const SFR_FLEVEL_TX_LEVEL2: crate::Field = crate::Field::new(3, 16, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT4: crate::Field = crate::Field::new(1, 19, SFR_FLEVEL);
        pub const SFR_FLEVEL_RX_LEVEL2: crate::Field = crate::Field::new(3, 20, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT5: crate::Field = crate::Field::new(1, 23, SFR_FLEVEL);
        pub const SFR_FLEVEL_TX_LEVEL3: crate::Field = crate::Field::new(3, 24, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT6: crate::Field = crate::Field::new(1, 27, SFR_FLEVEL);
        pub const SFR_FLEVEL_RX_LEVEL3: crate::Field = crate::Field::new(3, 28, SFR_FLEVEL);
        pub const SFR_FLEVEL_CONSTANT7: crate::Field = crate::Field::new(1, 31, SFR_FLEVEL);

        pub const SFR_TXF0: crate::Register = crate::Register::new(4, 0xffffffff);
        pub const SFR_TXF0_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF0);

        pub const SFR_TXF1: crate::Register = crate::Register::new(5, 0xffffffff);
        pub const SFR_TXF1_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF1);

        pub const SFR_TXF2: crate::Register = crate::Register::new(6, 0xffffffff);
        pub const SFR_TXF2_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF2);

        pub const RESERVED7: crate::Register = crate::Register::new(7, 0x1);
        pub const RESERVED7_RESERVED7: crate::Field = crate::Field::new(1, 0, RESERVED7);

        pub const SFR_RXF0: crate::Register = crate::Register::new(8, 0xffffffff);
        pub const SFR_RXF0_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF0);

        pub const SFR_RXF1: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_RXF1_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF1);

        pub const SFR_RXF2: crate::Register = crate::Register::new(10, 0xffffffff);
        pub const SFR_RXF2_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF2);

        pub const SFR_TXF3: crate::Register = crate::Register::new(11, 0xffffffff);
        pub const SFR_TXF3_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF3);

        pub const SFR_IRQ: crate::Register = crate::Register::new(12, 0xff);
        pub const SFR_IRQ_SFR_IRQ: crate::Field = crate::Field::new(8, 0, SFR_IRQ);

        pub const SFR_IRQ_FORCE: crate::Register = crate::Register::new(13, 0xff);
        pub const SFR_IRQ_FORCE_SFR_IRQ_FORCE: crate::Field = crate::Field::new(8, 0, SFR_IRQ_FORCE);

        pub const SFR_SYNC_BYPASS: crate::Register = crate::Register::new(14, 0xffffffff);
        pub const SFR_SYNC_BYPASS_SFR_SYNC_BYPASS: crate::Field = crate::Field::new(32, 0, SFR_SYNC_BYPASS);

        pub const SFR_DBG_PADOUT: crate::Register = crate::Register::new(15, 0xffffffff);
        pub const SFR_DBG_PADOUT_GPIO_IN: crate::Field = crate::Field::new(32, 0, SFR_DBG_PADOUT);

        pub const SFR_DBG_PADOE: crate::Register = crate::Register::new(16, 0xffffffff);
        pub const SFR_DBG_PADOE_GPIO_DIR: crate::Field = crate::Field::new(32, 0, SFR_DBG_PADOE);

        pub const SFR_DBG_CFGINFO: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const SFR_DBG_CFGINFO_CONSTANT0: crate::Field = crate::Field::new(8, 0, SFR_DBG_CFGINFO);
        pub const SFR_DBG_CFGINFO_CONSTANT1: crate::Field = crate::Field::new(8, 8, SFR_DBG_CFGINFO);
        pub const SFR_DBG_CFGINFO_CONSTANT2: crate::Field = crate::Field::new(16, 16, SFR_DBG_CFGINFO);

        pub const SFR_INSTR_MEM0: crate::Register = crate::Register::new(18, 0xffffffff);
        pub const SFR_INSTR_MEM0_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM0);

        pub const SFR_INSTR_MEM1: crate::Register = crate::Register::new(19, 0xffffffff);
        pub const SFR_INSTR_MEM1_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM1);

        pub const SFR_INSTR_MEM2: crate::Register = crate::Register::new(20, 0xffffffff);
        pub const SFR_INSTR_MEM2_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM2);

        pub const SFR_INSTR_MEM3: crate::Register = crate::Register::new(21, 0xffffffff);
        pub const SFR_INSTR_MEM3_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM3);

        pub const SFR_INSTR_MEM4: crate::Register = crate::Register::new(22, 0xffffffff);
        pub const SFR_INSTR_MEM4_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM4);

        pub const SFR_INSTR_MEM5: crate::Register = crate::Register::new(23, 0xffffffff);
        pub const SFR_INSTR_MEM5_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM5);

        pub const SFR_INSTR_MEM6: crate::Register = crate::Register::new(24, 0xffffffff);
        pub const SFR_INSTR_MEM6_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM6);

        pub const SFR_INSTR_MEM7: crate::Register = crate::Register::new(25, 0xffffffff);
        pub const SFR_INSTR_MEM7_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM7);

        pub const SFR_INSTR_MEM8: crate::Register = crate::Register::new(26, 0xffffffff);
        pub const SFR_INSTR_MEM8_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM8);

        pub const SFR_INSTR_MEM9: crate::Register = crate::Register::new(27, 0xffffffff);
        pub const SFR_INSTR_MEM9_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM9);

        pub const SFR_INSTR_MEM10: crate::Register = crate::Register::new(28, 0xffffffff);
        pub const SFR_INSTR_MEM10_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM10);

        pub const SFR_INSTR_MEM11: crate::Register = crate::Register::new(29, 0xffffffff);
        pub const SFR_INSTR_MEM11_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM11);

        pub const SFR_INSTR_MEM12: crate::Register = crate::Register::new(30, 0xffffffff);
        pub const SFR_INSTR_MEM12_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM12);

        pub const SFR_INSTR_MEM13: crate::Register = crate::Register::new(31, 0xffffffff);
        pub const SFR_INSTR_MEM13_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM13);

        pub const SFR_INSTR_MEM14: crate::Register = crate::Register::new(32, 0xffffffff);
        pub const SFR_INSTR_MEM14_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM14);

        pub const SFR_INSTR_MEM15: crate::Register = crate::Register::new(33, 0xffffffff);
        pub const SFR_INSTR_MEM15_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM15);

        pub const SFR_INSTR_MEM16: crate::Register = crate::Register::new(34, 0xffffffff);
        pub const SFR_INSTR_MEM16_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM16);

        pub const SFR_INSTR_MEM17: crate::Register = crate::Register::new(35, 0xffffffff);
        pub const SFR_INSTR_MEM17_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM17);

        pub const SFR_INSTR_MEM18: crate::Register = crate::Register::new(36, 0xffffffff);
        pub const SFR_INSTR_MEM18_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM18);

        pub const SFR_INSTR_MEM19: crate::Register = crate::Register::new(37, 0xffffffff);
        pub const SFR_INSTR_MEM19_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM19);

        pub const SFR_INSTR_MEM20: crate::Register = crate::Register::new(38, 0xffffffff);
        pub const SFR_INSTR_MEM20_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM20);

        pub const SFR_INSTR_MEM21: crate::Register = crate::Register::new(39, 0xffffffff);
        pub const SFR_INSTR_MEM21_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM21);

        pub const SFR_INSTR_MEM22: crate::Register = crate::Register::new(40, 0xffffffff);
        pub const SFR_INSTR_MEM22_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM22);

        pub const SFR_INSTR_MEM23: crate::Register = crate::Register::new(41, 0xffffffff);
        pub const SFR_INSTR_MEM23_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM23);

        pub const SFR_INSTR_MEM24: crate::Register = crate::Register::new(42, 0xffffffff);
        pub const SFR_INSTR_MEM24_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM24);

        pub const SFR_INSTR_MEM25: crate::Register = crate::Register::new(43, 0xffffffff);
        pub const SFR_INSTR_MEM25_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM25);

        pub const SFR_INSTR_MEM26: crate::Register = crate::Register::new(44, 0xffffffff);
        pub const SFR_INSTR_MEM26_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM26);

        pub const SFR_INSTR_MEM27: crate::Register = crate::Register::new(45, 0xffffffff);
        pub const SFR_INSTR_MEM27_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM27);

        pub const SFR_INSTR_MEM28: crate::Register = crate::Register::new(46, 0xffffffff);
        pub const SFR_INSTR_MEM28_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM28);

        pub const SFR_INSTR_MEM29: crate::Register = crate::Register::new(47, 0xffffffff);
        pub const SFR_INSTR_MEM29_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM29);

        pub const SFR_INSTR_MEM30: crate::Register = crate::Register::new(48, 0xffffffff);
        pub const SFR_INSTR_MEM30_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM30);

        pub const SFR_INSTR_MEM31: crate::Register = crate::Register::new(49, 0xffffffff);
        pub const SFR_INSTR_MEM31_INSTR: crate::Field = crate::Field::new(32, 0, SFR_INSTR_MEM31);

        pub const SFR_SM0_CLKDIV: crate::Register = crate::Register::new(50, 0xffffffff);
        pub const SFR_SM0_CLKDIV_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_SM0_CLKDIV);
        pub const SFR_SM0_CLKDIV_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_SM0_CLKDIV);
        pub const SFR_SM0_CLKDIV_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_SM0_CLKDIV);

        pub const SFR_SM0_EXECCTRL: crate::Register = crate::Register::new(51, 0xffffffff);
        pub const SFR_SM0_EXECCTRL_STATUS_N: crate::Field = crate::Field::new(4, 0, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_STATUS_SEL: crate::Field = crate::Field::new(1, 4, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_RESVD_EXEC: crate::Field = crate::Field::new(2, 5, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_WRAP_TARGET: crate::Field = crate::Field::new(5, 7, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_PEND: crate::Field = crate::Field::new(5, 12, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_OUT_STICKY: crate::Field = crate::Field::new(1, 17, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_INLINE_OUT_EN: crate::Field = crate::Field::new(1, 18, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_OUT_EN_SEL: crate::Field = crate::Field::new(5, 19, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_JMP_PIN: crate::Field = crate::Field::new(5, 24, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_SIDE_PINDIR: crate::Field = crate::Field::new(1, 29, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT: crate::Field = crate::Field::new(1, 30, SFR_SM0_EXECCTRL);
        pub const SFR_SM0_EXECCTRL_EXEC_STALLED: crate::Field = crate::Field::new(1, 31, SFR_SM0_EXECCTRL);

        pub const SFR_SM0_SHIFTCTRL: crate::Register = crate::Register::new(52, 0xffffffff);
        pub const SFR_SM0_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_RESVD_JOIN: crate::Field = crate::Field::new(2, 30, SFR_SM0_SHIFTCTRL);

        pub const SFR_SM0_ADDR: crate::Register = crate::Register::new(53, 0x1f);
        pub const SFR_SM0_ADDR_PC: crate::Field = crate::Field::new(5, 0, SFR_SM0_ADDR);

        pub const SFR_SM0_INSTR: crate::Register = crate::Register::new(54, 0xffff);
        pub const SFR_SM0_INSTR_IMM_INSTR: crate::Field = crate::Field::new(16, 0, SFR_SM0_INSTR);

        pub const SFR_SM0_PINCTRL: crate::Register = crate::Register::new(55, 0xffffffff);
        pub const SFR_SM0_PINCTRL_PINS_OUT_BASE: crate::Field = crate::Field::new(5, 0, SFR_SM0_PINCTRL);
        pub const SFR_SM0_PINCTRL_PINS_SET_BASE: crate::Field = crate::Field::new(5, 5, SFR_SM0_PINCTRL);
        pub const SFR_SM0_PINCTRL_PINS_SIDE_BASE: crate::Field = crate::Field::new(5, 10, SFR_SM0_PINCTRL);
        pub const SFR_SM0_PINCTRL_PINS_IN_BASE: crate::Field = crate::Field::new(5, 15, SFR_SM0_PINCTRL);
        pub const SFR_SM0_PINCTRL_PINS_OUT_COUNT: crate::Field = crate::Field::new(6, 20, SFR_SM0_PINCTRL);
        pub const SFR_SM0_PINCTRL_PINS_SET_COUNT: crate::Field = crate::Field::new(3, 26, SFR_SM0_PINCTRL);
        pub const SFR_SM0_PINCTRL_PINS_SIDE_COUNT: crate::Field = crate::Field::new(3, 29, SFR_SM0_PINCTRL);

        pub const SFR_SM1_CLKDIV: crate::Register = crate::Register::new(56, 0xffffffff);
        pub const SFR_SM1_CLKDIV_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_SM1_CLKDIV);
        pub const SFR_SM1_CLKDIV_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_SM1_CLKDIV);
        pub const SFR_SM1_CLKDIV_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_SM1_CLKDIV);

        pub const SFR_SM1_EXECCTRL: crate::Register = crate::Register::new(57, 0xffffffff);
        pub const SFR_SM1_EXECCTRL_STATUS_N: crate::Field = crate::Field::new(4, 0, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_STATUS_SEL: crate::Field = crate::Field::new(1, 4, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_RESVD_EXEC: crate::Field = crate::Field::new(2, 5, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_WRAP_TARGET: crate::Field = crate::Field::new(5, 7, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_PEND: crate::Field = crate::Field::new(5, 12, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_OUT_STICKY: crate::Field = crate::Field::new(1, 17, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_INLINE_OUT_EN: crate::Field = crate::Field::new(1, 18, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_OUT_EN_SEL: crate::Field = crate::Field::new(5, 19, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_JMP_PIN: crate::Field = crate::Field::new(5, 24, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_SIDE_PINDIR: crate::Field = crate::Field::new(1, 29, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT: crate::Field = crate::Field::new(1, 30, SFR_SM1_EXECCTRL);
        pub const SFR_SM1_EXECCTRL_EXEC_STALLED: crate::Field = crate::Field::new(1, 31, SFR_SM1_EXECCTRL);

        pub const SFR_SM1_SHIFTCTRL: crate::Register = crate::Register::new(58, 0xffffffff);
        pub const SFR_SM1_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_RESVD_JOIN: crate::Field = crate::Field::new(2, 30, SFR_SM1_SHIFTCTRL);

        pub const SFR_SM1_ADDR: crate::Register = crate::Register::new(59, 0x1f);
        pub const SFR_SM1_ADDR_PC: crate::Field = crate::Field::new(5, 0, SFR_SM1_ADDR);

        pub const SFR_SM1_INSTR: crate::Register = crate::Register::new(60, 0xffff);
        pub const SFR_SM1_INSTR_IMM_INSTR: crate::Field = crate::Field::new(16, 0, SFR_SM1_INSTR);

        pub const SFR_SM1_PINCTRL: crate::Register = crate::Register::new(61, 0xffffffff);
        pub const SFR_SM1_PINCTRL_PINS_OUT_BASE: crate::Field = crate::Field::new(5, 0, SFR_SM1_PINCTRL);
        pub const SFR_SM1_PINCTRL_PINS_SET_BASE: crate::Field = crate::Field::new(5, 5, SFR_SM1_PINCTRL);
        pub const SFR_SM1_PINCTRL_PINS_SIDE_BASE: crate::Field = crate::Field::new(5, 10, SFR_SM1_PINCTRL);
        pub const SFR_SM1_PINCTRL_PINS_IN_BASE: crate::Field = crate::Field::new(5, 15, SFR_SM1_PINCTRL);
        pub const SFR_SM1_PINCTRL_PINS_OUT_COUNT: crate::Field = crate::Field::new(6, 20, SFR_SM1_PINCTRL);
        pub const SFR_SM1_PINCTRL_PINS_SET_COUNT: crate::Field = crate::Field::new(3, 26, SFR_SM1_PINCTRL);
        pub const SFR_SM1_PINCTRL_PINS_SIDE_COUNT: crate::Field = crate::Field::new(3, 29, SFR_SM1_PINCTRL);

        pub const SFR_SM2_CLKDIV: crate::Register = crate::Register::new(62, 0xffffffff);
        pub const SFR_SM2_CLKDIV_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_SM2_CLKDIV);
        pub const SFR_SM2_CLKDIV_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_SM2_CLKDIV);
        pub const SFR_SM2_CLKDIV_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_SM2_CLKDIV);

        pub const SFR_SM2_EXECCTRL: crate::Register = crate::Register::new(63, 0xffffffff);
        pub const SFR_SM2_EXECCTRL_STATUS_N: crate::Field = crate::Field::new(4, 0, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_STATUS_SEL: crate::Field = crate::Field::new(1, 4, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_RESVD_EXEC: crate::Field = crate::Field::new(2, 5, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_WRAP_TARGET: crate::Field = crate::Field::new(5, 7, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_PEND: crate::Field = crate::Field::new(5, 12, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_OUT_STICKY: crate::Field = crate::Field::new(1, 17, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_INLINE_OUT_EN: crate::Field = crate::Field::new(1, 18, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_OUT_EN_SEL: crate::Field = crate::Field::new(5, 19, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_JMP_PIN: crate::Field = crate::Field::new(5, 24, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_SIDE_PINDIR: crate::Field = crate::Field::new(1, 29, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT: crate::Field = crate::Field::new(1, 30, SFR_SM2_EXECCTRL);
        pub const SFR_SM2_EXECCTRL_EXEC_STALLED: crate::Field = crate::Field::new(1, 31, SFR_SM2_EXECCTRL);

        pub const SFR_SM2_SHIFTCTRL: crate::Register = crate::Register::new(64, 0xffffffff);
        pub const SFR_SM2_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_RESVD_JOIN: crate::Field = crate::Field::new(2, 30, SFR_SM2_SHIFTCTRL);

        pub const SFR_SM2_ADDR: crate::Register = crate::Register::new(65, 0x1f);
        pub const SFR_SM2_ADDR_PC: crate::Field = crate::Field::new(5, 0, SFR_SM2_ADDR);

        pub const SFR_SM2_INSTR: crate::Register = crate::Register::new(66, 0xffff);
        pub const SFR_SM2_INSTR_IMM_INSTR: crate::Field = crate::Field::new(16, 0, SFR_SM2_INSTR);

        pub const SFR_SM2_PINCTRL: crate::Register = crate::Register::new(67, 0xffffffff);
        pub const SFR_SM2_PINCTRL_PINS_OUT_BASE: crate::Field = crate::Field::new(5, 0, SFR_SM2_PINCTRL);
        pub const SFR_SM2_PINCTRL_PINS_SET_BASE: crate::Field = crate::Field::new(5, 5, SFR_SM2_PINCTRL);
        pub const SFR_SM2_PINCTRL_PINS_SIDE_BASE: crate::Field = crate::Field::new(5, 10, SFR_SM2_PINCTRL);
        pub const SFR_SM2_PINCTRL_PINS_IN_BASE: crate::Field = crate::Field::new(5, 15, SFR_SM2_PINCTRL);
        pub const SFR_SM2_PINCTRL_PINS_OUT_COUNT: crate::Field = crate::Field::new(6, 20, SFR_SM2_PINCTRL);
        pub const SFR_SM2_PINCTRL_PINS_SET_COUNT: crate::Field = crate::Field::new(3, 26, SFR_SM2_PINCTRL);
        pub const SFR_SM2_PINCTRL_PINS_SIDE_COUNT: crate::Field = crate::Field::new(3, 29, SFR_SM2_PINCTRL);

        pub const SFR_SM3_CLKDIV: crate::Register = crate::Register::new(68, 0xffffffff);
        pub const SFR_SM3_CLKDIV_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_SM3_CLKDIV);
        pub const SFR_SM3_CLKDIV_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_SM3_CLKDIV);
        pub const SFR_SM3_CLKDIV_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_SM3_CLKDIV);

        pub const SFR_SM3_EXECCTRL: crate::Register = crate::Register::new(69, 0xffffffff);
        pub const SFR_SM3_EXECCTRL_STATUS_N: crate::Field = crate::Field::new(4, 0, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_STATUS_SEL: crate::Field = crate::Field::new(1, 4, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_RESVD_EXEC: crate::Field = crate::Field::new(2, 5, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_WRAP_TARGET: crate::Field = crate::Field::new(5, 7, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_PEND: crate::Field = crate::Field::new(5, 12, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_OUT_STICKY: crate::Field = crate::Field::new(1, 17, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_INLINE_OUT_EN: crate::Field = crate::Field::new(1, 18, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_OUT_EN_SEL: crate::Field = crate::Field::new(5, 19, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_JMP_PIN: crate::Field = crate::Field::new(5, 24, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_SIDE_PINDIR: crate::Field = crate::Field::new(1, 29, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT: crate::Field = crate::Field::new(1, 30, SFR_SM3_EXECCTRL);
        pub const SFR_SM3_EXECCTRL_EXEC_STALLED: crate::Field = crate::Field::new(1, 31, SFR_SM3_EXECCTRL);

        pub const SFR_SM3_SHIFTCTRL: crate::Register = crate::Register::new(70, 0xffffffff);
        pub const SFR_SM3_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_RESVD_JOIN: crate::Field = crate::Field::new(2, 30, SFR_SM3_SHIFTCTRL);

        pub const SFR_SM3_ADDR: crate::Register = crate::Register::new(71, 0x1f);
        pub const SFR_SM3_ADDR_PC: crate::Field = crate::Field::new(5, 0, SFR_SM3_ADDR);

        pub const SFR_SM3_INSTR: crate::Register = crate::Register::new(72, 0xffff);
        pub const SFR_SM3_INSTR_IMM_INSTR: crate::Field = crate::Field::new(16, 0, SFR_SM3_INSTR);

        pub const SFR_SM3_PINCTRL: crate::Register = crate::Register::new(73, 0xffffffff);
        pub const SFR_SM3_PINCTRL_PINS_OUT_BASE: crate::Field = crate::Field::new(5, 0, SFR_SM3_PINCTRL);
        pub const SFR_SM3_PINCTRL_PINS_SET_BASE: crate::Field = crate::Field::new(5, 5, SFR_SM3_PINCTRL);
        pub const SFR_SM3_PINCTRL_PINS_SIDE_BASE: crate::Field = crate::Field::new(5, 10, SFR_SM3_PINCTRL);
        pub const SFR_SM3_PINCTRL_PINS_IN_BASE: crate::Field = crate::Field::new(5, 15, SFR_SM3_PINCTRL);
        pub const SFR_SM3_PINCTRL_PINS_OUT_COUNT: crate::Field = crate::Field::new(6, 20, SFR_SM3_PINCTRL);
        pub const SFR_SM3_PINCTRL_PINS_SET_COUNT: crate::Field = crate::Field::new(3, 26, SFR_SM3_PINCTRL);
        pub const SFR_SM3_PINCTRL_PINS_SIDE_COUNT: crate::Field = crate::Field::new(3, 29, SFR_SM3_PINCTRL);

        pub const SFR_INTR: crate::Register = crate::Register::new(74, 0xfff);
        pub const SFR_INTR_INTR_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_INTR);
        pub const SFR_INTR_INTR_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_INTR);
        pub const SFR_INTR_INTR_SM: crate::Field = crate::Field::new(4, 8, SFR_INTR);

        pub const SFR_IRQ0_INTE: crate::Register = crate::Register::new(75, 0xfff);
        pub const SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_IRQ0_INTE);
        pub const SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_IRQ0_INTE);
        pub const SFR_IRQ0_INTE_IRQ0_INTE_SM: crate::Field = crate::Field::new(4, 8, SFR_IRQ0_INTE);

        pub const SFR_IRQ0_INTF: crate::Register = crate::Register::new(76, 0xfff);
        pub const SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_IRQ0_INTF);
        pub const SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_IRQ0_INTF);
        pub const SFR_IRQ0_INTF_IRQ0_INTF_SM: crate::Field = crate::Field::new(4, 8, SFR_IRQ0_INTF);

        pub const SFR_IRQ0_INTS: crate::Register = crate::Register::new(77, 0xfff);
        pub const SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_IRQ0_INTS);
        pub const SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_IRQ0_INTS);
        pub const SFR_IRQ0_INTS_IRQ0_INTS_SM: crate::Field = crate::Field::new(4, 8, SFR_IRQ0_INTS);

        pub const SFR_IRQ1_INTE: crate::Register = crate::Register::new(78, 0xfff);
        pub const SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_IRQ1_INTE);
        pub const SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_IRQ1_INTE);
        pub const SFR_IRQ1_INTE_IRQ1_INTE_SM: crate::Field = crate::Field::new(4, 8, SFR_IRQ1_INTE);

        pub const SFR_IRQ1_INTF: crate::Register = crate::Register::new(79, 0xfff);
        pub const SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_IRQ1_INTF);
        pub const SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_IRQ1_INTF);
        pub const SFR_IRQ1_INTF_IRQ1_INTF_SM: crate::Field = crate::Field::new(4, 8, SFR_IRQ1_INTF);

        pub const SFR_IRQ1_INTS: crate::Register = crate::Register::new(80, 0xfff);
        pub const SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY: crate::Field = crate::Field::new(4, 0, SFR_IRQ1_INTS);
        pub const SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL: crate::Field = crate::Field::new(4, 4, SFR_IRQ1_INTS);
        pub const SFR_IRQ1_INTS_IRQ1_INTS_SM: crate::Field = crate::Field::new(4, 8, SFR_IRQ1_INTS);

        pub const HW_PIO_BASE: usize = 0x40202000;
    }
}

// Litex auto-generated constants


#[cfg(test)]
mod tests {

    #[test]
    #[ignore]
    fn compile_check_pio_csr() {
        use super::*;
        let mut pio_csr = CSR::new(HW_PIO_BASE as *mut u32);

        let foo = pio_csr.r(utra::pio::SFR_CTRL);
        pio_csr.wo(utra::pio::SFR_CTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_CTRL_EN);
        pio_csr.rmwf(utra::pio::SFR_CTRL_EN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_CTRL_EN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_CTRL_EN, 1);
        pio_csr.wfo(utra::pio::SFR_CTRL_EN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_CTRL_RESTART);
        pio_csr.rmwf(utra::pio::SFR_CTRL_RESTART, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_CTRL_RESTART, bar);
        baz |= pio_csr.ms(utra::pio::SFR_CTRL_RESTART, 1);
        pio_csr.wfo(utra::pio::SFR_CTRL_RESTART, baz);
        let bar = pio_csr.rf(utra::pio::SFR_CTRL_CLKDIV_RESTART);
        pio_csr.rmwf(utra::pio::SFR_CTRL_CLKDIV_RESTART, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_CTRL_CLKDIV_RESTART, bar);
        baz |= pio_csr.ms(utra::pio::SFR_CTRL_CLKDIV_RESTART, 1);
        pio_csr.wfo(utra::pio::SFR_CTRL_CLKDIV_RESTART, baz);

        let foo = pio_csr.r(utra::pio::SFR_FSTAT);
        pio_csr.wo(utra::pio::SFR_FSTAT, foo);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_RX_FULL);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_RX_FULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_RX_FULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_RX_FULL, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_RX_FULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_CONSTANT0);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_CONSTANT0, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_CONSTANT0, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_CONSTANT0, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_CONSTANT0, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_RX_EMPTY);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_RX_EMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_RX_EMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_RX_EMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_RX_EMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_CONSTANT1);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_CONSTANT1, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_CONSTANT1, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_CONSTANT1, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_CONSTANT1, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_TX_FULL);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_TX_FULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_TX_FULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_TX_FULL, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_TX_FULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_CONSTANT2);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_CONSTANT2, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_CONSTANT2, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_CONSTANT2, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_CONSTANT2, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_TX_EMPTY);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_TX_EMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_TX_EMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_TX_EMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_TX_EMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FSTAT_CONSTANT3);
        pio_csr.rmwf(utra::pio::SFR_FSTAT_CONSTANT3, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FSTAT_CONSTANT3, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FSTAT_CONSTANT3, 1);
        pio_csr.wfo(utra::pio::SFR_FSTAT_CONSTANT3, baz);

        let foo = pio_csr.r(utra::pio::SFR_FDEBUG);
        pio_csr.wo(utra::pio::SFR_FDEBUG, foo);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_RXSTALL);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_RXSTALL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_RXSTALL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_RXSTALL, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_RXSTALL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_CONSTANT0);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_CONSTANT0, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_CONSTANT0, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_CONSTANT0, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_CONSTANT0, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_RXUNDER);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_RXUNDER, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_RXUNDER, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_RXUNDER, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_RXUNDER, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_CONSTANT1);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_CONSTANT1, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_CONSTANT1, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_CONSTANT1, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_CONSTANT1, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_TXOVER);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_TXOVER, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_TXOVER, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_TXOVER, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_TXOVER, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_CONSTANT2);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_CONSTANT2, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_CONSTANT2, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_CONSTANT2, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_CONSTANT2, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_TXSTALL);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_TXSTALL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_TXSTALL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_TXSTALL, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_TXSTALL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FDEBUG_CONSTANT3);
        pio_csr.rmwf(utra::pio::SFR_FDEBUG_CONSTANT3, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FDEBUG_CONSTANT3, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FDEBUG_CONSTANT3, 1);
        pio_csr.wfo(utra::pio::SFR_FDEBUG_CONSTANT3, baz);

        let foo = pio_csr.r(utra::pio::SFR_FLEVEL);
        pio_csr.wo(utra::pio::SFR_FLEVEL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_TX_LEVEL0);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_TX_LEVEL0, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_TX_LEVEL0, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_TX_LEVEL0, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_TX_LEVEL0, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT0);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT0, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT0, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT0, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT0, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_RX_LEVEL0);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_RX_LEVEL0, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_RX_LEVEL0, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_RX_LEVEL0, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_RX_LEVEL0, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT1);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT1, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT1, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT1, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT1, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_TX_LEVEL1);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_TX_LEVEL1, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_TX_LEVEL1, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_TX_LEVEL1, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_TX_LEVEL1, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT2);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT2, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT2, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT2, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT2, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_RX_LEVEL1);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_RX_LEVEL1, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_RX_LEVEL1, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_RX_LEVEL1, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_RX_LEVEL1, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT3);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT3, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT3, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT3, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT3, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_TX_LEVEL2);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_TX_LEVEL2, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_TX_LEVEL2, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_TX_LEVEL2, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_TX_LEVEL2, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT4);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT4, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT4, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT4, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT4, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_RX_LEVEL2);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_RX_LEVEL2, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_RX_LEVEL2, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_RX_LEVEL2, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_RX_LEVEL2, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT5);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT5, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT5, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT5, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT5, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_TX_LEVEL3);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_TX_LEVEL3, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_TX_LEVEL3, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_TX_LEVEL3, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_TX_LEVEL3, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT6);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT6, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT6, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT6, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT6, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_RX_LEVEL3);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_RX_LEVEL3, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_RX_LEVEL3, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_RX_LEVEL3, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_RX_LEVEL3, baz);
        let bar = pio_csr.rf(utra::pio::SFR_FLEVEL_CONSTANT7);
        pio_csr.rmwf(utra::pio::SFR_FLEVEL_CONSTANT7, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_FLEVEL_CONSTANT7, bar);
        baz |= pio_csr.ms(utra::pio::SFR_FLEVEL_CONSTANT7, 1);
        pio_csr.wfo(utra::pio::SFR_FLEVEL_CONSTANT7, baz);

        let foo = pio_csr.r(utra::pio::SFR_TXF0);
        pio_csr.wo(utra::pio::SFR_TXF0, foo);
        let bar = pio_csr.rf(utra::pio::SFR_TXF0_FDIN);
        pio_csr.rmwf(utra::pio::SFR_TXF0_FDIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_TXF0_FDIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_TXF0_FDIN, 1);
        pio_csr.wfo(utra::pio::SFR_TXF0_FDIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_TXF1);
        pio_csr.wo(utra::pio::SFR_TXF1, foo);
        let bar = pio_csr.rf(utra::pio::SFR_TXF1_FDIN);
        pio_csr.rmwf(utra::pio::SFR_TXF1_FDIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_TXF1_FDIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_TXF1_FDIN, 1);
        pio_csr.wfo(utra::pio::SFR_TXF1_FDIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_TXF2);
        pio_csr.wo(utra::pio::SFR_TXF2, foo);
        let bar = pio_csr.rf(utra::pio::SFR_TXF2_FDIN);
        pio_csr.rmwf(utra::pio::SFR_TXF2_FDIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_TXF2_FDIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_TXF2_FDIN, 1);
        pio_csr.wfo(utra::pio::SFR_TXF2_FDIN, baz);

        let foo = pio_csr.r(utra::pio::RESERVED7);
        pio_csr.wo(utra::pio::RESERVED7, foo);
        let bar = pio_csr.rf(utra::pio::RESERVED7_RESERVED7);
        pio_csr.rmwf(utra::pio::RESERVED7_RESERVED7, bar);
        let mut baz = pio_csr.zf(utra::pio::RESERVED7_RESERVED7, bar);
        baz |= pio_csr.ms(utra::pio::RESERVED7_RESERVED7, 1);
        pio_csr.wfo(utra::pio::RESERVED7_RESERVED7, baz);

        let foo = pio_csr.r(utra::pio::SFR_RXF0);
        pio_csr.wo(utra::pio::SFR_RXF0, foo);
        let bar = pio_csr.rf(utra::pio::SFR_RXF0_PDOUT);
        pio_csr.rmwf(utra::pio::SFR_RXF0_PDOUT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_RXF0_PDOUT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_RXF0_PDOUT, 1);
        pio_csr.wfo(utra::pio::SFR_RXF0_PDOUT, baz);

        let foo = pio_csr.r(utra::pio::SFR_RXF1);
        pio_csr.wo(utra::pio::SFR_RXF1, foo);
        let bar = pio_csr.rf(utra::pio::SFR_RXF1_PDOUT);
        pio_csr.rmwf(utra::pio::SFR_RXF1_PDOUT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_RXF1_PDOUT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_RXF1_PDOUT, 1);
        pio_csr.wfo(utra::pio::SFR_RXF1_PDOUT, baz);

        let foo = pio_csr.r(utra::pio::SFR_RXF2);
        pio_csr.wo(utra::pio::SFR_RXF2, foo);
        let bar = pio_csr.rf(utra::pio::SFR_RXF2_PDOUT);
        pio_csr.rmwf(utra::pio::SFR_RXF2_PDOUT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_RXF2_PDOUT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_RXF2_PDOUT, 1);
        pio_csr.wfo(utra::pio::SFR_RXF2_PDOUT, baz);

        let foo = pio_csr.r(utra::pio::SFR_TXF3);
        pio_csr.wo(utra::pio::SFR_TXF3, foo);
        let bar = pio_csr.rf(utra::pio::SFR_TXF3_FDIN);
        pio_csr.rmwf(utra::pio::SFR_TXF3_FDIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_TXF3_FDIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_TXF3_FDIN, 1);
        pio_csr.wfo(utra::pio::SFR_TXF3_FDIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ);
        pio_csr.wo(utra::pio::SFR_IRQ, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ_SFR_IRQ);
        pio_csr.rmwf(utra::pio::SFR_IRQ_SFR_IRQ, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ_SFR_IRQ, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ_SFR_IRQ, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ_SFR_IRQ, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ_FORCE);
        pio_csr.wo(utra::pio::SFR_IRQ_FORCE, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE);
        pio_csr.rmwf(utra::pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, baz);

        let foo = pio_csr.r(utra::pio::SFR_SYNC_BYPASS);
        pio_csr.wo(utra::pio::SFR_SYNC_BYPASS, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS);
        pio_csr.rmwf(utra::pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, 1);
        pio_csr.wfo(utra::pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, baz);

        let foo = pio_csr.r(utra::pio::SFR_DBG_PADOUT);
        pio_csr.wo(utra::pio::SFR_DBG_PADOUT, foo);
        let bar = pio_csr.rf(utra::pio::SFR_DBG_PADOUT_GPIO_IN);
        pio_csr.rmwf(utra::pio::SFR_DBG_PADOUT_GPIO_IN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_DBG_PADOUT_GPIO_IN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_DBG_PADOUT_GPIO_IN, 1);
        pio_csr.wfo(utra::pio::SFR_DBG_PADOUT_GPIO_IN, baz);

        let foo = pio_csr.r(utra::pio::SFR_DBG_PADOE);
        pio_csr.wo(utra::pio::SFR_DBG_PADOE, foo);
        let bar = pio_csr.rf(utra::pio::SFR_DBG_PADOE_GPIO_DIR);
        pio_csr.rmwf(utra::pio::SFR_DBG_PADOE_GPIO_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_DBG_PADOE_GPIO_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_DBG_PADOE_GPIO_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_DBG_PADOE_GPIO_DIR, baz);

        let foo = pio_csr.r(utra::pio::SFR_DBG_CFGINFO);
        pio_csr.wo(utra::pio::SFR_DBG_CFGINFO, foo);
        let bar = pio_csr.rf(utra::pio::SFR_DBG_CFGINFO_CONSTANT0);
        pio_csr.rmwf(utra::pio::SFR_DBG_CFGINFO_CONSTANT0, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_DBG_CFGINFO_CONSTANT0, bar);
        baz |= pio_csr.ms(utra::pio::SFR_DBG_CFGINFO_CONSTANT0, 1);
        pio_csr.wfo(utra::pio::SFR_DBG_CFGINFO_CONSTANT0, baz);
        let bar = pio_csr.rf(utra::pio::SFR_DBG_CFGINFO_CONSTANT1);
        pio_csr.rmwf(utra::pio::SFR_DBG_CFGINFO_CONSTANT1, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_DBG_CFGINFO_CONSTANT1, bar);
        baz |= pio_csr.ms(utra::pio::SFR_DBG_CFGINFO_CONSTANT1, 1);
        pio_csr.wfo(utra::pio::SFR_DBG_CFGINFO_CONSTANT1, baz);
        let bar = pio_csr.rf(utra::pio::SFR_DBG_CFGINFO_CONSTANT2);
        pio_csr.rmwf(utra::pio::SFR_DBG_CFGINFO_CONSTANT2, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_DBG_CFGINFO_CONSTANT2, bar);
        baz |= pio_csr.ms(utra::pio::SFR_DBG_CFGINFO_CONSTANT2, 1);
        pio_csr.wfo(utra::pio::SFR_DBG_CFGINFO_CONSTANT2, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM0);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM0, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM0_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM0_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM0_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM0_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM0_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM1);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM1, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM1_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM1_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM1_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM1_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM1_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM2);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM2, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM2_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM2_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM2_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM2_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM2_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM3);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM3, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM3_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM3_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM3_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM3_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM3_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM4);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM4, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM4_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM4_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM4_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM4_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM4_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM5);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM5, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM5_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM5_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM5_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM5_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM5_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM6);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM6, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM6_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM6_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM6_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM6_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM6_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM7);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM7, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM7_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM7_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM7_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM7_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM7_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM8);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM8, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM8_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM8_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM8_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM8_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM8_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM9);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM9, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM9_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM9_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM9_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM9_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM9_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM10);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM10, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM10_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM10_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM10_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM10_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM10_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM11);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM11, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM11_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM11_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM11_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM11_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM11_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM12);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM12, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM12_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM12_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM12_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM12_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM12_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM13);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM13, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM13_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM13_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM13_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM13_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM13_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM14);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM14, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM14_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM14_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM14_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM14_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM14_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM15);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM15, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM15_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM15_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM15_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM15_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM15_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM16);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM16, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM16_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM16_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM16_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM16_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM16_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM17);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM17, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM17_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM17_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM17_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM17_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM17_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM18);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM18, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM18_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM18_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM18_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM18_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM18_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM19);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM19, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM19_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM19_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM19_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM19_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM19_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM20);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM20, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM20_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM20_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM20_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM20_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM20_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM21);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM21, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM21_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM21_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM21_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM21_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM21_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM22);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM22, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM22_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM22_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM22_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM22_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM22_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM23);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM23, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM23_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM23_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM23_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM23_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM23_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM24);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM24, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM24_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM24_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM24_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM24_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM24_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM25);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM25, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM25_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM25_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM25_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM25_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM25_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM26);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM26, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM26_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM26_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM26_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM26_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM26_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM27);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM27, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM27_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM27_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM27_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM27_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM27_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM28);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM28, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM28_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM28_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM28_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM28_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM28_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM29);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM29, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM29_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM29_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM29_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM29_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM29_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM30);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM30, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM30_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM30_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM30_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM30_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM30_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_INSTR_MEM31);
        pio_csr.wo(utra::pio::SFR_INSTR_MEM31, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INSTR_MEM31_INSTR);
        pio_csr.rmwf(utra::pio::SFR_INSTR_MEM31_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INSTR_MEM31_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INSTR_MEM31_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_INSTR_MEM31_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM0_CLKDIV);
        pio_csr.wo(utra::pio::SFR_SM0_CLKDIV, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_CLKDIV_UNUSED_DIV);
        pio_csr.rmwf(utra::pio::SFR_SM0_CLKDIV_UNUSED_DIV, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_CLKDIV_UNUSED_DIV, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_CLKDIV_UNUSED_DIV, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_CLKDIV_UNUSED_DIV, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_CLKDIV_DIV_FRAC);
        pio_csr.rmwf(utra::pio::SFR_SM0_CLKDIV_DIV_FRAC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_CLKDIV_DIV_FRAC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_CLKDIV_DIV_FRAC, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_CLKDIV_DIV_FRAC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_CLKDIV_DIV_INT);
        pio_csr.rmwf(utra::pio::SFR_SM0_CLKDIV_DIV_INT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_CLKDIV_DIV_INT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_CLKDIV_DIV_INT, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_CLKDIV_DIV_INT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM0_EXECCTRL);
        pio_csr.wo(utra::pio::SFR_SM0_EXECCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_STATUS_N);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_STATUS_N, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_STATUS_N, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_STATUS_N, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_STATUS_N, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_STATUS_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_STATUS_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_STATUS_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_STATUS_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_STATUS_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_RESVD_EXEC);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_RESVD_EXEC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_RESVD_EXEC, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_RESVD_EXEC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_WRAP_TARGET);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_WRAP_TARGET, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_WRAP_TARGET, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_WRAP_TARGET, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_PEND);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_PEND, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_PEND, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_PEND, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_PEND, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_OUT_STICKY);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_OUT_STICKY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_OUT_STICKY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_OUT_STICKY, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_OUT_STICKY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_OUT_EN_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_JMP_PIN);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_JMP_PIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_JMP_PIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_JMP_PIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_JMP_PIN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_SIDE_PINDIR);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_EXECCTRL_EXEC_STALLED);
        pio_csr.rmwf(utra::pio::SFR_SM0_EXECCTRL_EXEC_STALLED, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_EXECCTRL_EXEC_STALLED, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_EXECCTRL_EXEC_STALLED, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_EXECCTRL_EXEC_STALLED, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM0_SHIFTCTRL);
        pio_csr.wo(utra::pio::SFR_SM0_SHIFTCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PULL);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_JOIN);
        pio_csr.rmwf(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_JOIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_JOIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_JOIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_SHIFTCTRL_RESVD_JOIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM0_ADDR);
        pio_csr.wo(utra::pio::SFR_SM0_ADDR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_ADDR_PC);
        pio_csr.rmwf(utra::pio::SFR_SM0_ADDR_PC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_ADDR_PC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_ADDR_PC, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_ADDR_PC, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM0_INSTR);
        pio_csr.wo(utra::pio::SFR_SM0_INSTR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_INSTR_IMM_INSTR);
        pio_csr.rmwf(utra::pio::SFR_SM0_INSTR_IMM_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_INSTR_IMM_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_INSTR_IMM_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_INSTR_IMM_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM0_PINCTRL);
        pio_csr.wo(utra::pio::SFR_SM0_PINCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_SET_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_SET_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_SET_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_SET_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_IN_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_IN_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_IN_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_IN_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_SET_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM1_CLKDIV);
        pio_csr.wo(utra::pio::SFR_SM1_CLKDIV, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_CLKDIV_UNUSED_DIV);
        pio_csr.rmwf(utra::pio::SFR_SM1_CLKDIV_UNUSED_DIV, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_CLKDIV_UNUSED_DIV, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_CLKDIV_UNUSED_DIV, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_CLKDIV_UNUSED_DIV, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_CLKDIV_DIV_FRAC);
        pio_csr.rmwf(utra::pio::SFR_SM1_CLKDIV_DIV_FRAC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_CLKDIV_DIV_FRAC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_CLKDIV_DIV_FRAC, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_CLKDIV_DIV_FRAC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_CLKDIV_DIV_INT);
        pio_csr.rmwf(utra::pio::SFR_SM1_CLKDIV_DIV_INT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_CLKDIV_DIV_INT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_CLKDIV_DIV_INT, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_CLKDIV_DIV_INT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM1_EXECCTRL);
        pio_csr.wo(utra::pio::SFR_SM1_EXECCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_STATUS_N);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_STATUS_N, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_STATUS_N, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_STATUS_N, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_STATUS_N, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_STATUS_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_STATUS_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_STATUS_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_STATUS_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_STATUS_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_RESVD_EXEC);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_RESVD_EXEC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_RESVD_EXEC, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_RESVD_EXEC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_WRAP_TARGET);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_WRAP_TARGET, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_WRAP_TARGET, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_WRAP_TARGET, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_PEND);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_PEND, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_PEND, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_PEND, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_PEND, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_OUT_STICKY);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_OUT_STICKY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_OUT_STICKY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_OUT_STICKY, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_OUT_STICKY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_OUT_EN_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_JMP_PIN);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_JMP_PIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_JMP_PIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_JMP_PIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_JMP_PIN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_SIDE_PINDIR);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_EXECCTRL_EXEC_STALLED);
        pio_csr.rmwf(utra::pio::SFR_SM1_EXECCTRL_EXEC_STALLED, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_EXECCTRL_EXEC_STALLED, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_EXECCTRL_EXEC_STALLED, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_EXECCTRL_EXEC_STALLED, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM1_SHIFTCTRL);
        pio_csr.wo(utra::pio::SFR_SM1_SHIFTCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PULL);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_JOIN);
        pio_csr.rmwf(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_JOIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_JOIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_JOIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_SHIFTCTRL_RESVD_JOIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM1_ADDR);
        pio_csr.wo(utra::pio::SFR_SM1_ADDR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_ADDR_PC);
        pio_csr.rmwf(utra::pio::SFR_SM1_ADDR_PC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_ADDR_PC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_ADDR_PC, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_ADDR_PC, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM1_INSTR);
        pio_csr.wo(utra::pio::SFR_SM1_INSTR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_INSTR_IMM_INSTR);
        pio_csr.rmwf(utra::pio::SFR_SM1_INSTR_IMM_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_INSTR_IMM_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_INSTR_IMM_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_INSTR_IMM_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM1_PINCTRL);
        pio_csr.wo(utra::pio::SFR_SM1_PINCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_SET_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_SET_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_SET_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_SET_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_IN_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_IN_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_IN_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_IN_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_SET_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM2_CLKDIV);
        pio_csr.wo(utra::pio::SFR_SM2_CLKDIV, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_CLKDIV_UNUSED_DIV);
        pio_csr.rmwf(utra::pio::SFR_SM2_CLKDIV_UNUSED_DIV, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_CLKDIV_UNUSED_DIV, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_CLKDIV_UNUSED_DIV, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_CLKDIV_UNUSED_DIV, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_CLKDIV_DIV_FRAC);
        pio_csr.rmwf(utra::pio::SFR_SM2_CLKDIV_DIV_FRAC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_CLKDIV_DIV_FRAC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_CLKDIV_DIV_FRAC, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_CLKDIV_DIV_FRAC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_CLKDIV_DIV_INT);
        pio_csr.rmwf(utra::pio::SFR_SM2_CLKDIV_DIV_INT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_CLKDIV_DIV_INT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_CLKDIV_DIV_INT, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_CLKDIV_DIV_INT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM2_EXECCTRL);
        pio_csr.wo(utra::pio::SFR_SM2_EXECCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_STATUS_N);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_STATUS_N, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_STATUS_N, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_STATUS_N, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_STATUS_N, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_STATUS_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_STATUS_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_STATUS_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_STATUS_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_STATUS_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_RESVD_EXEC);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_RESVD_EXEC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_RESVD_EXEC, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_RESVD_EXEC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_WRAP_TARGET);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_WRAP_TARGET, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_WRAP_TARGET, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_WRAP_TARGET, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_PEND);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_PEND, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_PEND, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_PEND, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_PEND, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_OUT_STICKY);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_OUT_STICKY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_OUT_STICKY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_OUT_STICKY, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_OUT_STICKY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_OUT_EN_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_JMP_PIN);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_JMP_PIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_JMP_PIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_JMP_PIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_JMP_PIN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_SIDE_PINDIR);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_EXECCTRL_EXEC_STALLED);
        pio_csr.rmwf(utra::pio::SFR_SM2_EXECCTRL_EXEC_STALLED, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_EXECCTRL_EXEC_STALLED, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_EXECCTRL_EXEC_STALLED, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_EXECCTRL_EXEC_STALLED, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM2_SHIFTCTRL);
        pio_csr.wo(utra::pio::SFR_SM2_SHIFTCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PULL);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_JOIN);
        pio_csr.rmwf(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_JOIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_JOIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_JOIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_SHIFTCTRL_RESVD_JOIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM2_ADDR);
        pio_csr.wo(utra::pio::SFR_SM2_ADDR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_ADDR_PC);
        pio_csr.rmwf(utra::pio::SFR_SM2_ADDR_PC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_ADDR_PC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_ADDR_PC, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_ADDR_PC, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM2_INSTR);
        pio_csr.wo(utra::pio::SFR_SM2_INSTR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_INSTR_IMM_INSTR);
        pio_csr.rmwf(utra::pio::SFR_SM2_INSTR_IMM_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_INSTR_IMM_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_INSTR_IMM_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_INSTR_IMM_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM2_PINCTRL);
        pio_csr.wo(utra::pio::SFR_SM2_PINCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_SET_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_SET_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_SET_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_SET_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_IN_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_IN_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_IN_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_IN_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_SET_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM3_CLKDIV);
        pio_csr.wo(utra::pio::SFR_SM3_CLKDIV, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_CLKDIV_UNUSED_DIV);
        pio_csr.rmwf(utra::pio::SFR_SM3_CLKDIV_UNUSED_DIV, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_CLKDIV_UNUSED_DIV, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_CLKDIV_UNUSED_DIV, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_CLKDIV_UNUSED_DIV, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_CLKDIV_DIV_FRAC);
        pio_csr.rmwf(utra::pio::SFR_SM3_CLKDIV_DIV_FRAC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_CLKDIV_DIV_FRAC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_CLKDIV_DIV_FRAC, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_CLKDIV_DIV_FRAC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_CLKDIV_DIV_INT);
        pio_csr.rmwf(utra::pio::SFR_SM3_CLKDIV_DIV_INT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_CLKDIV_DIV_INT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_CLKDIV_DIV_INT, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_CLKDIV_DIV_INT, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM3_EXECCTRL);
        pio_csr.wo(utra::pio::SFR_SM3_EXECCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_STATUS_N);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_STATUS_N, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_STATUS_N, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_STATUS_N, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_STATUS_N, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_STATUS_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_STATUS_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_STATUS_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_STATUS_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_STATUS_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_RESVD_EXEC);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_RESVD_EXEC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_RESVD_EXEC, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_RESVD_EXEC, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_WRAP_TARGET);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_WRAP_TARGET, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_WRAP_TARGET, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_WRAP_TARGET, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_PEND);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_PEND, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_PEND, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_PEND, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_PEND, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_OUT_STICKY);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_OUT_STICKY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_OUT_STICKY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_OUT_STICKY, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_OUT_STICKY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_OUT_EN_SEL);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_JMP_PIN);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_JMP_PIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_JMP_PIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_JMP_PIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_JMP_PIN, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_SIDE_PINDIR);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_EXECCTRL_EXEC_STALLED);
        pio_csr.rmwf(utra::pio::SFR_SM3_EXECCTRL_EXEC_STALLED, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_EXECCTRL_EXEC_STALLED, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_EXECCTRL_EXEC_STALLED, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_EXECCTRL_EXEC_STALLED, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM3_SHIFTCTRL);
        pio_csr.wo(utra::pio::SFR_SM3_SHIFTCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PULL);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_JOIN);
        pio_csr.rmwf(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_JOIN, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_JOIN, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_JOIN, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_SHIFTCTRL_RESVD_JOIN, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM3_ADDR);
        pio_csr.wo(utra::pio::SFR_SM3_ADDR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_ADDR_PC);
        pio_csr.rmwf(utra::pio::SFR_SM3_ADDR_PC, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_ADDR_PC, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_ADDR_PC, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_ADDR_PC, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM3_INSTR);
        pio_csr.wo(utra::pio::SFR_SM3_INSTR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_INSTR_IMM_INSTR);
        pio_csr.rmwf(utra::pio::SFR_SM3_INSTR_IMM_INSTR, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_INSTR_IMM_INSTR, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_INSTR_IMM_INSTR, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_INSTR_IMM_INSTR, baz);

        let foo = pio_csr.r(utra::pio::SFR_SM3_PINCTRL);
        pio_csr.wo(utra::pio::SFR_SM3_PINCTRL, foo);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_SET_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_SET_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_SET_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_SET_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_IN_BASE);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_IN_BASE, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_IN_BASE, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_IN_BASE, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_SET_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, baz);
        let bar = pio_csr.rf(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT);
        pio_csr.rmwf(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= pio_csr.ms(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, 1);
        pio_csr.wfo(utra::pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = pio_csr.r(utra::pio::SFR_INTR);
        pio_csr.wo(utra::pio::SFR_INTR, foo);
        let bar = pio_csr.rf(utra::pio::SFR_INTR_INTR_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_INTR_INTR_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INTR_INTR_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INTR_INTR_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_INTR_INTR_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_INTR_INTR_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_INTR_INTR_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INTR_INTR_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INTR_INTR_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_INTR_INTR_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_INTR_INTR_SM);
        pio_csr.rmwf(utra::pio::SFR_INTR_INTR_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_INTR_INTR_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_INTR_INTR_SM, 1);
        pio_csr.wfo(utra::pio::SFR_INTR_INTR_SM, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ0_INTE);
        pio_csr.wo(utra::pio::SFR_IRQ0_INTE, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_SM);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ0_INTF);
        pio_csr.wo(utra::pio::SFR_IRQ0_INTF, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_SM);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ0_INTS);
        pio_csr.wo(utra::pio::SFR_IRQ0_INTS, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_SM);
        pio_csr.rmwf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ1_INTE);
        pio_csr.wo(utra::pio::SFR_IRQ1_INTE, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_SM);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ1_INTF);
        pio_csr.wo(utra::pio::SFR_IRQ1_INTF, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_SM);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, baz);

        let foo = pio_csr.r(utra::pio::SFR_IRQ1_INTS);
        pio_csr.wo(utra::pio::SFR_IRQ1_INTS, foo);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, baz);
        let bar = pio_csr.rf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_SM);
        pio_csr.rmwf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, bar);
        let mut baz = pio_csr.zf(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, bar);
        baz |= pio_csr.ms(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, 1);
        pio_csr.wfo(utra::pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, baz);
  }
}
