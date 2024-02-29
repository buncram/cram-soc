
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
pub const HW_IFSUB_MEM_LEN: usize = 12288;
pub const HW_CORESUB_MEM:     usize = 0x40010000;
pub const HW_CORESUB_MEM_LEN: usize = 65536;
pub const HW_SECSUB_MEM:     usize = 0x40050000;
pub const HW_SECSUB_MEM_LEN: usize = 65536;
pub const HW_PIO_MEM:     usize = 0x50123000;
pub const HW_PIO_MEM_LEN: usize = 4096;
pub const HW_BIO_MEM:     usize = 0x50124000;
pub const HW_BIO_MEM_LEN: usize = 4096;
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
pub const HW_SEG_PCON_MEM_LEN: usize = 0;
pub const HW_SEG_PKB_MEM:     usize = 0x40020800;
pub const HW_SEG_PKB_MEM_LEN: usize = 512;
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
pub const HW_IFRAM0_MEM:     usize = 0x50000000;
pub const HW_IFRAM0_MEM_LEN: usize = 131072;
pub const HW_IFRAM1_MEM:     usize = 0x50020000;
pub const HW_IFRAM1_MEM_LEN: usize = 131072;
pub const HW_NULL_MEM:     usize = 0x50040000;
pub const HW_NULL_MEM_LEN: usize = 65536;
pub const HW_UDMA_MEM:     usize = 0x50100000;
pub const HW_UDMA_MEM_LEN: usize = 131072;
pub const HW_UDP_MEM:     usize = 0x50122000;
pub const HW_UDP_MEM_LEN: usize = 4096;
pub const HW_SDDC_DAT_MEM:     usize = 0x50140000;
pub const HW_SDDC_DAT_MEM_LEN: usize = 65536;
pub const HW_UDC_MEM:     usize = 0x50200000;
pub const HW_UDC_MEM_LEN: usize = 65536;
pub const HW_SRAM_MEM:     usize = 0x61000000;
pub const HW_SRAM_MEM_LEN: usize = 2097152;
pub const HW_RERAM_MEM:     usize = 0x60000000;
pub const HW_RERAM_MEM_LEN: usize = 4194304;
pub const HW_XIP_MEM:     usize = 0x70000000;
pub const HW_XIP_MEM_LEN: usize = 134217728;
pub const HW_PL230_MEM:     usize = 0x40011000;
pub const HW_PL230_MEM_LEN: usize = 4096;
pub const HW_MDMA_MEM:     usize = 0x40012000;
pub const HW_MDMA_MEM_LEN: usize = 4096;
pub const HW_MBOX_APB_MEM:     usize = 0x40013000;
pub const HW_MBOX_APB_MEM_LEN: usize = 4096;
pub const HW_IOX_MEM:     usize = 0x5012f000;
pub const HW_IOX_MEM_LEN: usize = 4096;
pub const HW_AOC_MEM:     usize = 0x40060000;
pub const HW_AOC_MEM_LEN: usize = 4096;
pub const HW_BIO_RAM_MEM:     usize = 0x50125000;
pub const HW_BIO_RAM_MEM_LEN: usize = 4096;

// Physical base addresses of registers
pub const HW_PL230_BASE :   usize = 0x40011000;
pub const HW_UDMA_CTRL_BASE :   usize = 0x50100000;
pub const HW_UDMA_UART_0_BASE :   usize = 0x50101000;
pub const HW_UDMA_UART_1_BASE :   usize = 0x50102000;
pub const HW_UDMA_UART_2_BASE :   usize = 0x50103000;
pub const HW_UDMA_UART_3_BASE :   usize = 0x50104000;
pub const HW_UDMA_SPIM_0_BASE :   usize = 0x50105000;
pub const HW_UDMA_SPIM_1_BASE :   usize = 0x50106000;
pub const HW_UDMA_SPIM_2_BASE :   usize = 0x50107000;
pub const HW_UDMA_SPIM_3_BASE :   usize = 0x50108000;
pub const HW_UDMA_I2C_0_BASE :   usize = 0x50109000;
pub const HW_UDMA_I2C_1_BASE :   usize = 0x5010a000;
pub const HW_UDMA_I2C_2_BASE :   usize = 0x5010b000;
pub const HW_UDMA_I2C_3_BASE :   usize = 0x5010c000;
pub const HW_UDMA_SDIO_BASE :   usize = 0x5010d000;
pub const HW_UDMA_I2S_BASE :   usize = 0x5010e000;
pub const HW_UDMA_CAMERA_BASE :   usize = 0x5010f000;
pub const HW_UDMA_FILTER_BASE :   usize = 0x50110000;
pub const HW_UDMA_SCIF_BASE :   usize = 0x50111000;
pub const HW_UDMA_SPIS_0_BASE :   usize = 0x50112000;
pub const HW_UDMA_SPIS_1_BASE :   usize = 0x50113000;
pub const HW_UDMA_ADC_BASE :   usize = 0x50114000;
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
pub const HW_RP_PIO_BASE :   usize = 0x50123000;
pub const HW_BIO_BASE :   usize = 0x50124000;
pub const HW_CORESUB_SRAMTRM_BASE :   usize = 0x40014000;
pub const HW_MDMA_BASE :   usize = 0x40012000;
pub const HW_QFC_BASE :   usize = 0x40010000;
pub const HW_MBOX_APB_BASE :   usize = 0x40013000;
pub const HW_GLUECHAIN_BASE :   usize = 0x40054000;


pub mod utra {

    pub mod pl230 {
        pub const PL230_NUMREGS: usize = 20;

        pub const STATUS: crate::Register = crate::Register::new(0, 0xf01f00f1);
        pub const STATUS_TEST_STATUS: crate::Field = crate::Field::new(4, 28, STATUS);
        pub const STATUS_CHNLS_MINUS1: crate::Field = crate::Field::new(5, 16, STATUS);
        pub const STATUS_STATE: crate::Field = crate::Field::new(4, 4, STATUS);
        pub const STATUS_MASTER_ENABLE: crate::Field = crate::Field::new(1, 0, STATUS);

        pub const CFG: crate::Register = crate::Register::new(1, 0xe1);
        pub const CFG_CHNL_PROT_CTRL: crate::Field = crate::Field::new(3, 5, CFG);
        pub const CFG_MASTER_ENABLE: crate::Field = crate::Field::new(1, 0, CFG);

        pub const CTRLBASEPTR: crate::Register = crate::Register::new(2, 0xffffff00);
        pub const CTRLBASEPTR_CTRL_BASE_PTR: crate::Field = crate::Field::new(24, 8, CTRLBASEPTR);

        pub const ALTCTRLBASEPTR: crate::Register = crate::Register::new(3, 0xffffffff);
        pub const ALTCTRLBASEPTR_ALT_CTRL_BASE_PTR: crate::Field = crate::Field::new(32, 0, ALTCTRLBASEPTR);

        pub const DMA_WAITONREQ_STATUS: crate::Register = crate::Register::new(4, 0xff);
        pub const DMA_WAITONREQ_STATUS_DMA_WAITONREQ_STATUS: crate::Field = crate::Field::new(8, 0, DMA_WAITONREQ_STATUS);

        pub const CHNLSWREQUEST: crate::Register = crate::Register::new(5, 0xff);
        pub const CHNLSWREQUEST_CHNL_SW_REQUEST: crate::Field = crate::Field::new(8, 0, CHNLSWREQUEST);

        pub const CHNLUSEBURSTSET: crate::Register = crate::Register::new(6, 0xff);
        pub const CHNLUSEBURSTSET_CHNL_USEBURST_SET: crate::Field = crate::Field::new(8, 0, CHNLUSEBURSTSET);

        pub const CHNLUSEBURSTCLR: crate::Register = crate::Register::new(7, 0xff);
        pub const CHNLUSEBURSTCLR_CHNL_USEBURST_CLR: crate::Field = crate::Field::new(8, 0, CHNLUSEBURSTCLR);

        pub const CHNLREQMASKSET: crate::Register = crate::Register::new(8, 0xff);
        pub const CHNLREQMASKSET_CHNL_REQ_MASK_SET: crate::Field = crate::Field::new(8, 0, CHNLREQMASKSET);

        pub const CHNLREQMASKCLR: crate::Register = crate::Register::new(9, 0xff);
        pub const CHNLREQMASKCLR_CHNL_REQ_MASK_CLR: crate::Field = crate::Field::new(8, 0, CHNLREQMASKCLR);

        pub const CHNLENABLESET: crate::Register = crate::Register::new(10, 0xff);
        pub const CHNLENABLESET_CHNL_ENABLE_SET: crate::Field = crate::Field::new(8, 0, CHNLENABLESET);

        pub const CHNLENABLECLR: crate::Register = crate::Register::new(11, 0xff);
        pub const CHNLENABLECLR_CHNL_ENABLE_CLR: crate::Field = crate::Field::new(8, 0, CHNLENABLECLR);

        pub const CHNLPRIALTSET: crate::Register = crate::Register::new(12, 0xff);
        pub const CHNLPRIALTSET_CHNL_PRI_ALT_SET: crate::Field = crate::Field::new(8, 0, CHNLPRIALTSET);

        pub const CHNLPRIALTCLR: crate::Register = crate::Register::new(13, 0xff);
        pub const CHNLPRIALTCLR_CHNL_PRI_ALT_CLR: crate::Field = crate::Field::new(8, 0, CHNLPRIALTCLR);

        pub const CHNLPRIORITYSET: crate::Register = crate::Register::new(14, 0xff);
        pub const CHNLPRIORITYSET_CHNL_PRIORITY_SET: crate::Field = crate::Field::new(8, 0, CHNLPRIORITYSET);

        pub const CHNLPRIORITYCLR: crate::Register = crate::Register::new(15, 0xff);
        pub const CHNLPRIORITYCLR_CHNL_PRIORITY_CLR: crate::Field = crate::Field::new(8, 0, CHNLPRIORITYCLR);

        pub const ERRCLR: crate::Register = crate::Register::new(19, 0x1);
        pub const ERRCLR_ERR_CLR: crate::Field = crate::Field::new(1, 0, ERRCLR);

        pub const PERIPH_ID_0: crate::Register = crate::Register::new(1016, 0xff);
        pub const PERIPH_ID_0_PART_NUMBER_LSB: crate::Field = crate::Field::new(8, 0, PERIPH_ID_0);

        pub const PERIPH_ID_1: crate::Register = crate::Register::new(1017, 0x7f);
        pub const PERIPH_ID_1_PART_NUMBER_MSB: crate::Field = crate::Field::new(4, 0, PERIPH_ID_1);
        pub const PERIPH_ID_1_JEP106_LSB: crate::Field = crate::Field::new(3, 4, PERIPH_ID_1);

        pub const PERIPH_ID_2: crate::Register = crate::Register::new(1018, 0xff);
        pub const PERIPH_ID_2_JEP106_MSB: crate::Field = crate::Field::new(3, 0, PERIPH_ID_2);
        pub const PERIPH_ID_2_JEDEC_USED: crate::Field = crate::Field::new(1, 3, PERIPH_ID_2);
        pub const PERIPH_ID_2_REVISION: crate::Field = crate::Field::new(4, 4, PERIPH_ID_2);

        pub const HW_PL230_BASE: usize = 0x40011000;
    }

    pub mod udma_ctrl {
        pub const UDMA_CTRL_NUMREGS: usize = 3;

        pub const REG_CG: crate::Register = crate::Register::new(0, 0x3f);
        pub const REG_CG_R_CG: crate::Field = crate::Field::new(6, 0, REG_CG);

        pub const REG_CFG_EVT: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const REG_CFG_EVT_R_CMP_EVT_0: crate::Field = crate::Field::new(8, 0, REG_CFG_EVT);
        pub const REG_CFG_EVT_R_CMP_EVT_1: crate::Field = crate::Field::new(8, 8, REG_CFG_EVT);
        pub const REG_CFG_EVT_R_CMP_EVT_2: crate::Field = crate::Field::new(8, 16, REG_CFG_EVT);
        pub const REG_CFG_EVT_R_CMP_EVT_3: crate::Field = crate::Field::new(8, 24, REG_CFG_EVT);

        pub const REG_RST: crate::Register = crate::Register::new(2, 0x3f);
        pub const REG_RST_R_RST: crate::Field = crate::Field::new(6, 0, REG_RST);

        pub const HW_UDMA_CTRL_BASE: usize = 0x50100000;
    }

    pub mod udma_uart_0 {
        pub const UDMA_UART_0_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(8, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const REG_UART_SETUP: crate::Register = crate::Register::new(9, 0xffff033f);
        pub const REG_UART_SETUP_R_UART_PARITY_EN: crate::Field = crate::Field::new(1, 0, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_BITS: crate::Field = crate::Field::new(2, 1, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_STOP_BITS: crate::Field = crate::Field::new(1, 3, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_POLLING_EN: crate::Field = crate::Field::new(1, 4, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_CLEAN_FIFO: crate::Field = crate::Field::new(1, 5, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_TX: crate::Field = crate::Field::new(1, 8, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_RX: crate::Field = crate::Field::new(1, 9, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_DIV: crate::Field = crate::Field::new(16, 16, REG_UART_SETUP);

        pub const REG_ERROR: crate::Register = crate::Register::new(10, 0x3);
        pub const REG_ERROR_R_ERR_OVERFLOW: crate::Field = crate::Field::new(1, 0, REG_ERROR);
        pub const REG_ERROR_R_ERR_PARITY: crate::Field = crate::Field::new(1, 1, REG_ERROR);

        pub const REG_IRQ_EN: crate::Register = crate::Register::new(11, 0x3);
        pub const REG_IRQ_EN_R_UART_RX_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_IRQ_EN);
        pub const REG_IRQ_EN_R_UART_ERR_IRQ_EN: crate::Field = crate::Field::new(1, 1, REG_IRQ_EN);

        pub const REG_VALID: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_VALID_R_UART_RX_DATA_VALID: crate::Field = crate::Field::new(1, 0, REG_VALID);

        pub const REG_DATA: crate::Register = crate::Register::new(13, 0xff);
        pub const REG_DATA_R_UART_RX_DATA: crate::Field = crate::Field::new(8, 0, REG_DATA);

        pub const HW_UDMA_UART_0_BASE: usize = 0x50101000;
    }

    pub mod udma_uart_1 {
        pub const UDMA_UART_1_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(8, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const REG_UART_SETUP: crate::Register = crate::Register::new(9, 0xffff033f);
        pub const REG_UART_SETUP_R_UART_PARITY_EN: crate::Field = crate::Field::new(1, 0, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_BITS: crate::Field = crate::Field::new(2, 1, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_STOP_BITS: crate::Field = crate::Field::new(1, 3, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_POLLING_EN: crate::Field = crate::Field::new(1, 4, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_CLEAN_FIFO: crate::Field = crate::Field::new(1, 5, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_TX: crate::Field = crate::Field::new(1, 8, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_RX: crate::Field = crate::Field::new(1, 9, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_DIV: crate::Field = crate::Field::new(16, 16, REG_UART_SETUP);

        pub const REG_ERROR: crate::Register = crate::Register::new(10, 0x3);
        pub const REG_ERROR_R_ERR_OVERFLOW: crate::Field = crate::Field::new(1, 0, REG_ERROR);
        pub const REG_ERROR_R_ERR_PARITY: crate::Field = crate::Field::new(1, 1, REG_ERROR);

        pub const REG_IRQ_EN: crate::Register = crate::Register::new(11, 0x3);
        pub const REG_IRQ_EN_R_UART_RX_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_IRQ_EN);
        pub const REG_IRQ_EN_R_UART_ERR_IRQ_EN: crate::Field = crate::Field::new(1, 1, REG_IRQ_EN);

        pub const REG_VALID: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_VALID_R_UART_RX_DATA_VALID: crate::Field = crate::Field::new(1, 0, REG_VALID);

        pub const REG_DATA: crate::Register = crate::Register::new(13, 0xff);
        pub const REG_DATA_R_UART_RX_DATA: crate::Field = crate::Field::new(8, 0, REG_DATA);

        pub const HW_UDMA_UART_1_BASE: usize = 0x50102000;
    }

    pub mod udma_uart_2 {
        pub const UDMA_UART_2_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(8, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const REG_UART_SETUP: crate::Register = crate::Register::new(9, 0xffff033f);
        pub const REG_UART_SETUP_R_UART_PARITY_EN: crate::Field = crate::Field::new(1, 0, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_BITS: crate::Field = crate::Field::new(2, 1, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_STOP_BITS: crate::Field = crate::Field::new(1, 3, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_POLLING_EN: crate::Field = crate::Field::new(1, 4, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_CLEAN_FIFO: crate::Field = crate::Field::new(1, 5, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_TX: crate::Field = crate::Field::new(1, 8, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_RX: crate::Field = crate::Field::new(1, 9, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_DIV: crate::Field = crate::Field::new(16, 16, REG_UART_SETUP);

        pub const REG_ERROR: crate::Register = crate::Register::new(10, 0x3);
        pub const REG_ERROR_R_ERR_OVERFLOW: crate::Field = crate::Field::new(1, 0, REG_ERROR);
        pub const REG_ERROR_R_ERR_PARITY: crate::Field = crate::Field::new(1, 1, REG_ERROR);

        pub const REG_IRQ_EN: crate::Register = crate::Register::new(11, 0x3);
        pub const REG_IRQ_EN_R_UART_RX_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_IRQ_EN);
        pub const REG_IRQ_EN_R_UART_ERR_IRQ_EN: crate::Field = crate::Field::new(1, 1, REG_IRQ_EN);

        pub const REG_VALID: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_VALID_R_UART_RX_DATA_VALID: crate::Field = crate::Field::new(1, 0, REG_VALID);

        pub const REG_DATA: crate::Register = crate::Register::new(13, 0xff);
        pub const REG_DATA_R_UART_RX_DATA: crate::Field = crate::Field::new(8, 0, REG_DATA);

        pub const HW_UDMA_UART_2_BASE: usize = 0x50103000;
    }

    pub mod udma_uart_3 {
        pub const UDMA_UART_3_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(8, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const REG_UART_SETUP: crate::Register = crate::Register::new(9, 0xffff033f);
        pub const REG_UART_SETUP_R_UART_PARITY_EN: crate::Field = crate::Field::new(1, 0, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_BITS: crate::Field = crate::Field::new(2, 1, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_STOP_BITS: crate::Field = crate::Field::new(1, 3, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_POLLING_EN: crate::Field = crate::Field::new(1, 4, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_RX_CLEAN_FIFO: crate::Field = crate::Field::new(1, 5, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_TX: crate::Field = crate::Field::new(1, 8, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_EN_RX: crate::Field = crate::Field::new(1, 9, REG_UART_SETUP);
        pub const REG_UART_SETUP_R_UART_DIV: crate::Field = crate::Field::new(16, 16, REG_UART_SETUP);

        pub const REG_ERROR: crate::Register = crate::Register::new(10, 0x3);
        pub const REG_ERROR_R_ERR_OVERFLOW: crate::Field = crate::Field::new(1, 0, REG_ERROR);
        pub const REG_ERROR_R_ERR_PARITY: crate::Field = crate::Field::new(1, 1, REG_ERROR);

        pub const REG_IRQ_EN: crate::Register = crate::Register::new(11, 0x3);
        pub const REG_IRQ_EN_R_UART_RX_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_IRQ_EN);
        pub const REG_IRQ_EN_R_UART_ERR_IRQ_EN: crate::Field = crate::Field::new(1, 1, REG_IRQ_EN);

        pub const REG_VALID: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_VALID_R_UART_RX_DATA_VALID: crate::Field = crate::Field::new(1, 0, REG_VALID);

        pub const REG_DATA: crate::Register = crate::Register::new(13, 0xff);
        pub const REG_DATA_R_UART_RX_DATA: crate::Field = crate::Field::new(8, 0, REG_DATA);

        pub const HW_UDMA_UART_3_BASE: usize = 0x50104000;
    }

    pub mod udma_spim_0 {
        pub const UDMA_SPIM_0_NUMREGS: usize = 10;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x57);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x57);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const HW_UDMA_SPIM_0_BASE: usize = 0x50105000;
    }

    pub mod udma_spim_1 {
        pub const UDMA_SPIM_1_NUMREGS: usize = 10;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x57);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x57);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const HW_UDMA_SPIM_1_BASE: usize = 0x50106000;
    }

    pub mod udma_spim_2 {
        pub const UDMA_SPIM_2_NUMREGS: usize = 10;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x57);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x57);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const HW_UDMA_SPIM_2_BASE: usize = 0x50107000;
    }

    pub mod udma_spim_3 {
        pub const UDMA_SPIM_3_NUMREGS: usize = 10;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x57);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x57);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const HW_UDMA_SPIM_3_BASE: usize = 0x50108000;
    }

    pub mod udma_i2c_0 {
        pub const UDMA_I2C_0_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x3);
        pub const REG_STATUS_R_BUSY: crate::Field = crate::Field::new(1, 0, REG_STATUS);
        pub const REG_STATUS_R_AL: crate::Field = crate::Field::new(1, 1, REG_STATUS);

        pub const REG_SETUP: crate::Register = crate::Register::new(13, 0x1);
        pub const REG_SETUP_R_DO_RST: crate::Field = crate::Field::new(1, 0, REG_SETUP);

        pub const REG_ACK: crate::Register = crate::Register::new(14, 0x1);
        pub const REG_ACK_R_NACK: crate::Field = crate::Field::new(1, 0, REG_ACK);

        pub const HW_UDMA_I2C_0_BASE: usize = 0x50109000;
    }

    pub mod udma_i2c_1 {
        pub const UDMA_I2C_1_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x3);
        pub const REG_STATUS_R_BUSY: crate::Field = crate::Field::new(1, 0, REG_STATUS);
        pub const REG_STATUS_R_AL: crate::Field = crate::Field::new(1, 1, REG_STATUS);

        pub const REG_SETUP: crate::Register = crate::Register::new(13, 0x1);
        pub const REG_SETUP_R_DO_RST: crate::Field = crate::Field::new(1, 0, REG_SETUP);

        pub const REG_ACK: crate::Register = crate::Register::new(14, 0x1);
        pub const REG_ACK_R_NACK: crate::Field = crate::Field::new(1, 0, REG_ACK);

        pub const HW_UDMA_I2C_1_BASE: usize = 0x5010a000;
    }

    pub mod udma_i2c_2 {
        pub const UDMA_I2C_2_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x3);
        pub const REG_STATUS_R_BUSY: crate::Field = crate::Field::new(1, 0, REG_STATUS);
        pub const REG_STATUS_R_AL: crate::Field = crate::Field::new(1, 1, REG_STATUS);

        pub const REG_SETUP: crate::Register = crate::Register::new(13, 0x1);
        pub const REG_SETUP_R_DO_RST: crate::Field = crate::Field::new(1, 0, REG_SETUP);

        pub const REG_ACK: crate::Register = crate::Register::new(14, 0x1);
        pub const REG_ACK_R_NACK: crate::Field = crate::Field::new(1, 0, REG_ACK);

        pub const HW_UDMA_I2C_2_BASE: usize = 0x5010b000;
    }

    pub mod udma_i2c_3 {
        pub const UDMA_I2C_3_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_CMD_SADDR: crate::Register = crate::Register::new(8, 0xfff);
        pub const REG_CMD_SADDR_R_CMD_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_CMD_SADDR);

        pub const REG_CMD_SIZE: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_CMD_SIZE_R_CMD_SIZE: crate::Field = crate::Field::new(16, 0, REG_CMD_SIZE);

        pub const REG_CMD_CFG: crate::Register = crate::Register::new(10, 0x51);
        pub const REG_CMD_CFG_R_CMD_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_EN: crate::Field = crate::Field::new(1, 4, REG_CMD_CFG);
        pub const REG_CMD_CFG_R_CMD_CLR: crate::Field = crate::Field::new(1, 6, REG_CMD_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(12, 0x3);
        pub const REG_STATUS_R_BUSY: crate::Field = crate::Field::new(1, 0, REG_STATUS);
        pub const REG_STATUS_R_AL: crate::Field = crate::Field::new(1, 1, REG_STATUS);

        pub const REG_SETUP: crate::Register = crate::Register::new(13, 0x1);
        pub const REG_SETUP_R_DO_RST: crate::Field = crate::Field::new(1, 0, REG_SETUP);

        pub const REG_ACK: crate::Register = crate::Register::new(14, 0x1);
        pub const REG_ACK_R_NACK: crate::Field = crate::Field::new(1, 0, REG_ACK);

        pub const HW_UDMA_I2C_3_BASE: usize = 0x5010c000;
    }

    pub mod udma_sdio {
        pub const UDMA_SDIO_NUMREGS: usize = 15;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x31);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 5, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x31);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 5, REG_TX_CFG);

        pub const REG_CMD_OP: crate::Register = crate::Register::new(8, 0x33f07);
        pub const REG_CMD_OP_R_CMD_RSP_TYPE: crate::Field = crate::Field::new(3, 0, REG_CMD_OP);
        pub const REG_CMD_OP_R_CMD_OP: crate::Field = crate::Field::new(6, 8, REG_CMD_OP);
        pub const REG_CMD_OP_R_CMD_STOPOPT: crate::Field = crate::Field::new(2, 16, REG_CMD_OP);

        pub const REG_DATA_SETUP: crate::Register = crate::Register::new(10, 0x3ffff07);
        pub const REG_DATA_SETUP_R_DATA_EN: crate::Field = crate::Field::new(1, 0, REG_DATA_SETUP);
        pub const REG_DATA_SETUP_R_DATA_RWN: crate::Field = crate::Field::new(1, 1, REG_DATA_SETUP);
        pub const REG_DATA_SETUP_R_DATA_QUAD: crate::Field = crate::Field::new(1, 2, REG_DATA_SETUP);
        pub const REG_DATA_SETUP_R_DATA_BLOCK_NUM: crate::Field = crate::Field::new(8, 8, REG_DATA_SETUP);
        pub const REG_DATA_SETUP_R_DATA_BLOCK_SIZE: crate::Field = crate::Field::new(10, 16, REG_DATA_SETUP);

        pub const REG_START: crate::Register = crate::Register::new(11, 0x1);
        pub const REG_START_R_SDIO_START: crate::Field = crate::Field::new(1, 0, REG_START);

        pub const REG_RSP0: crate::Register = crate::Register::new(12, 0xffffffff);
        pub const REG_RSP0_CFG_RSP_DATA_I_31_0: crate::Field = crate::Field::new(32, 0, REG_RSP0);

        pub const REG_RSP1: crate::Register = crate::Register::new(13, 0xffffffff);
        pub const REG_RSP1_CFG_RSP_DATA_I_63_32: crate::Field = crate::Field::new(32, 0, REG_RSP1);

        pub const REG_RSP2: crate::Register = crate::Register::new(14, 0xffffffff);
        pub const REG_RSP2_CFG_RSP_DATA_I_95_64: crate::Field = crate::Field::new(32, 0, REG_RSP2);

        pub const REG_RSP3: crate::Register = crate::Register::new(15, 0xffffffff);
        pub const REG_RSP3_CFG_RSP_DATA_I_127_96: crate::Field = crate::Field::new(32, 0, REG_RSP3);

        pub const REG_CLK_DIV: crate::Register = crate::Register::new(16, 0x1ff);
        pub const REG_CLK_DIV_R_CLK_DIV_DATA: crate::Field = crate::Field::new(8, 0, REG_CLK_DIV);
        pub const REG_CLK_DIV_R_CLK_DIV_VALID: crate::Field = crate::Field::new(1, 8, REG_CLK_DIV);

        pub const REG_STATUS: crate::Register = crate::Register::new(17, 0x3);
        pub const REG_STATUS_R_EOT: crate::Field = crate::Field::new(1, 0, REG_STATUS);
        pub const REG_STATUS_R_ERR: crate::Field = crate::Field::new(1, 1, REG_STATUS);

        pub const HW_UDMA_SDIO_BASE: usize = 0x5010d000;
    }

    pub mod udma_i2s {
        pub const UDMA_I2S_NUMREGS: usize = 10;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x37);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 5, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x37);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 5, REG_TX_CFG);

        pub const REG_I2S_CLKCFG_SETUP: crate::Register = crate::Register::new(8, 0xf7ffffff);
        pub const REG_I2S_CLKCFG_SETUP_R_MASTER_GEN_CLK_DIV: crate::Field = crate::Field::new(8, 0, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_SLAVE_GEN_CLK_DIV: crate::Field = crate::Field::new(8, 8, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_COMMON_GEN_CLK_DIV: crate::Field = crate::Field::new(8, 16, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_SLAVE_CLK_EN: crate::Field = crate::Field::new(1, 24, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_MASTER_CLK_EN: crate::Field = crate::Field::new(1, 25, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_PDM_CLK_EN: crate::Field = crate::Field::new(1, 26, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_EXT: crate::Field = crate::Field::new(1, 28, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_NUM: crate::Field = crate::Field::new(1, 29, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_EXT: crate::Field = crate::Field::new(1, 30, REG_I2S_CLKCFG_SETUP);
        pub const REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_NUM: crate::Field = crate::Field::new(1, 31, REG_I2S_CLKCFG_SETUP);

        pub const REG_I2S_SLV_SETUP: crate::Register = crate::Register::new(9, 0x80031f07);
        pub const REG_I2S_SLV_SETUP_R_SLAVE_I2S_WORDS: crate::Field = crate::Field::new(3, 0, REG_I2S_SLV_SETUP);
        pub const REG_I2S_SLV_SETUP_R_SLAVE_I2S_BITS_WORD: crate::Field = crate::Field::new(5, 8, REG_I2S_SLV_SETUP);
        pub const REG_I2S_SLV_SETUP_R_SLAVE_I2S_LSB_FIRST: crate::Field = crate::Field::new(1, 16, REG_I2S_SLV_SETUP);
        pub const REG_I2S_SLV_SETUP_R_SLAVE_I2S_2CH: crate::Field = crate::Field::new(1, 17, REG_I2S_SLV_SETUP);
        pub const REG_I2S_SLV_SETUP_R_SLAVE_I2S_EN: crate::Field = crate::Field::new(1, 31, REG_I2S_SLV_SETUP);

        pub const REG_I2S_MST_SETUP: crate::Register = crate::Register::new(10, 0x80031f07);
        pub const REG_I2S_MST_SETUP_R_MASTER_I2S_WORDS: crate::Field = crate::Field::new(3, 0, REG_I2S_MST_SETUP);
        pub const REG_I2S_MST_SETUP_R_MASTER_I2S_BITS_WORD: crate::Field = crate::Field::new(5, 8, REG_I2S_MST_SETUP);
        pub const REG_I2S_MST_SETUP_R_MASTER_I2S_LSB_FIRST: crate::Field = crate::Field::new(1, 16, REG_I2S_MST_SETUP);
        pub const REG_I2S_MST_SETUP_R_MASTER_I2S_2CH: crate::Field = crate::Field::new(1, 17, REG_I2S_MST_SETUP);
        pub const REG_I2S_MST_SETUP_R_MASTER_I2S_EN: crate::Field = crate::Field::new(1, 31, REG_I2S_MST_SETUP);

        pub const REG_I2S_PDM_SETUP: crate::Register = crate::Register::new(11, 0x80007fff);
        pub const REG_I2S_PDM_SETUP_R_SLAVE_PDM_SHIFT: crate::Field = crate::Field::new(3, 0, REG_I2S_PDM_SETUP);
        pub const REG_I2S_PDM_SETUP_R_SLAVE_PDM_DECIMATION: crate::Field = crate::Field::new(10, 3, REG_I2S_PDM_SETUP);
        pub const REG_I2S_PDM_SETUP_R_SLAVE_PDM_MODE: crate::Field = crate::Field::new(2, 13, REG_I2S_PDM_SETUP);
        pub const REG_I2S_PDM_SETUP_R_SLAVE_PDM_EN: crate::Field = crate::Field::new(1, 31, REG_I2S_PDM_SETUP);

        pub const HW_UDMA_I2S_BASE: usize = 0x5010e000;
    }

    pub mod udma_camera {
        pub const UDMA_CAMERA_NUMREGS: usize = 9;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x57);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_DATASIZE: crate::Field = crate::Field::new(2, 1, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_CAM_CFG_GLOB: crate::Register = crate::Register::new(8, 0x7fffffff);
        pub const REG_CAM_CFG_GLOB_R_CAM_CFG: crate::Field = crate::Field::new(30, 0, REG_CAM_CFG_GLOB);
        pub const REG_CAM_CFG_GLOB_CFG_CAM_IP_EN_I: crate::Field = crate::Field::new(1, 30, REG_CAM_CFG_GLOB);

        pub const REG_CAM_CFG_LL: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const REG_CAM_CFG_LL_R_CAM_CFG_LL: crate::Field = crate::Field::new(32, 0, REG_CAM_CFG_LL);

        pub const REG_CAM_CFG_UR: crate::Register = crate::Register::new(10, 0xffffffff);
        pub const REG_CAM_CFG_UR_R_CAM_CFG_UR: crate::Field = crate::Field::new(32, 0, REG_CAM_CFG_UR);

        pub const REG_CAM_CFG_SIZE: crate::Register = crate::Register::new(11, 0xffffffff);
        pub const REG_CAM_CFG_SIZE_R_CAM_CFG_SIZE: crate::Field = crate::Field::new(32, 0, REG_CAM_CFG_SIZE);

        pub const REG_CAM_CFG_FILTER: crate::Register = crate::Register::new(12, 0xffffffff);
        pub const REG_CAM_CFG_FILTER_R_CAM_CFG_FILTER: crate::Field = crate::Field::new(32, 0, REG_CAM_CFG_FILTER);

        pub const REG_CAM_VSYNC_POLARITY: crate::Register = crate::Register::new(13, 0x3);
        pub const REG_CAM_VSYNC_POLARITY_R_CAM_VSYNC_POLARITY: crate::Field = crate::Field::new(1, 0, REG_CAM_VSYNC_POLARITY);
        pub const REG_CAM_VSYNC_POLARITY_R_CAM_HSYNC_POLARITY: crate::Field = crate::Field::new(1, 1, REG_CAM_VSYNC_POLARITY);

        pub const HW_UDMA_CAMERA_BASE: usize = 0x5010f000;
    }

    pub mod udma_filter {
        pub const UDMA_FILTER_NUMREGS: usize = 24;

        pub const REG_TX_CH0_ADD: crate::Register = crate::Register::new(0, 0x7fff);
        pub const REG_TX_CH0_ADD_R_FILTER_TX_START_ADDR_0: crate::Field = crate::Field::new(15, 0, REG_TX_CH0_ADD);

        pub const REG_TX_CH0_CFG: crate::Register = crate::Register::new(1, 0x303);
        pub const REG_TX_CH0_CFG_R_FILTER_TX_DATASIZE_0: crate::Field = crate::Field::new(2, 0, REG_TX_CH0_CFG);
        pub const REG_TX_CH0_CFG_R_FILTER_TX_MODE_0: crate::Field = crate::Field::new(2, 8, REG_TX_CH0_CFG);

        pub const REG_TX_CH0_LEN0: crate::Register = crate::Register::new(2, 0x7fff);
        pub const REG_TX_CH0_LEN0_R_FILTER_TX_LEN0_0: crate::Field = crate::Field::new(15, 0, REG_TX_CH0_LEN0);

        pub const REG_TX_CH0_LEN1: crate::Register = crate::Register::new(3, 0x7fff);
        pub const REG_TX_CH0_LEN1_R_FILTER_TX_LEN1_0: crate::Field = crate::Field::new(15, 0, REG_TX_CH0_LEN1);

        pub const REG_TX_CH0_LEN2: crate::Register = crate::Register::new(4, 0x7fff);
        pub const REG_TX_CH0_LEN2_R_FILTER_TX_LEN2_0: crate::Field = crate::Field::new(15, 0, REG_TX_CH0_LEN2);

        pub const REG_TX_CH1_ADD: crate::Register = crate::Register::new(5, 0x7fff);
        pub const REG_TX_CH1_ADD_R_FILTER_TX_START_ADDR_1: crate::Field = crate::Field::new(15, 0, REG_TX_CH1_ADD);

        pub const REG_TX_CH1_CFG: crate::Register = crate::Register::new(6, 0x303);
        pub const REG_TX_CH1_CFG_R_FILTER_TX_DATASIZE_1: crate::Field = crate::Field::new(2, 0, REG_TX_CH1_CFG);
        pub const REG_TX_CH1_CFG_R_FILTER_TX_MODE_1: crate::Field = crate::Field::new(2, 8, REG_TX_CH1_CFG);

        pub const REG_TX_CH1_LEN0: crate::Register = crate::Register::new(7, 0x7fff);
        pub const REG_TX_CH1_LEN0_R_FILTER_TX_LEN0_1: crate::Field = crate::Field::new(15, 0, REG_TX_CH1_LEN0);

        pub const REG_TX_CH1_LEN1: crate::Register = crate::Register::new(8, 0x7fff);
        pub const REG_TX_CH1_LEN1_R_FILTER_TX_LEN1_1: crate::Field = crate::Field::new(15, 0, REG_TX_CH1_LEN1);

        pub const REG_TX_CH1_LEN2: crate::Register = crate::Register::new(9, 0x7fff);
        pub const REG_TX_CH1_LEN2_R_FILTER_TX_LEN2_1: crate::Field = crate::Field::new(15, 0, REG_TX_CH1_LEN2);

        pub const REG_RX_CH_ADD: crate::Register = crate::Register::new(10, 0x7fff);
        pub const REG_RX_CH_ADD_R_FILTER_RX_START_ADDR: crate::Field = crate::Field::new(15, 0, REG_RX_CH_ADD);

        pub const REG_RX_CH_CFG: crate::Register = crate::Register::new(11, 0x303);
        pub const REG_RX_CH_CFG_R_FILTER_RX_DATASIZE: crate::Field = crate::Field::new(2, 0, REG_RX_CH_CFG);
        pub const REG_RX_CH_CFG_R_FILTER_RX_MODE: crate::Field = crate::Field::new(2, 8, REG_RX_CH_CFG);

        pub const REG_RX_CH_LEN0: crate::Register = crate::Register::new(12, 0xffff);
        pub const REG_RX_CH_LEN0_R_FILTER_RX_LEN0: crate::Field = crate::Field::new(16, 0, REG_RX_CH_LEN0);

        pub const REG_RX_CH_LEN1: crate::Register = crate::Register::new(13, 0xffff);
        pub const REG_RX_CH_LEN1_R_FILTER_RX_LEN1: crate::Field = crate::Field::new(16, 0, REG_RX_CH_LEN1);

        pub const REG_RX_CH_LEN2: crate::Register = crate::Register::new(14, 0xffff);
        pub const REG_RX_CH_LEN2_R_FILTER_RX_LEN2: crate::Field = crate::Field::new(16, 0, REG_RX_CH_LEN2);

        pub const REG_AU_CFG: crate::Register = crate::Register::new(15, 0x1f0f03);
        pub const REG_AU_CFG_R_AU_USE_SIGNED: crate::Field = crate::Field::new(1, 0, REG_AU_CFG);
        pub const REG_AU_CFG_R_AU_BYPASS: crate::Field = crate::Field::new(1, 1, REG_AU_CFG);
        pub const REG_AU_CFG_R_AU_MODE: crate::Field = crate::Field::new(4, 8, REG_AU_CFG);
        pub const REG_AU_CFG_R_AU_SHIFT: crate::Field = crate::Field::new(5, 16, REG_AU_CFG);

        pub const REG_AU_REG0: crate::Register = crate::Register::new(16, 0xffffffff);
        pub const REG_AU_REG0_R_COMMIT_AU_REG0: crate::Field = crate::Field::new(32, 0, REG_AU_REG0);

        pub const REG_AU_REG1: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const REG_AU_REG1_R_COMMIT_AU_REG1: crate::Field = crate::Field::new(32, 0, REG_AU_REG1);

        pub const REG_BINCU_TH: crate::Register = crate::Register::new(18, 0xffffffff);
        pub const REG_BINCU_TH_R_COMMIT_BINCU_THRESHOLD: crate::Field = crate::Field::new(32, 0, REG_BINCU_TH);

        pub const REG_BINCU_CNT: crate::Register = crate::Register::new(19, 0x80007fff);
        pub const REG_BINCU_CNT_R_BINCU_COUNTER: crate::Field = crate::Field::new(15, 0, REG_BINCU_CNT);
        pub const REG_BINCU_CNT_R_BINCU_EN_COUNTER: crate::Field = crate::Field::new(1, 31, REG_BINCU_CNT);

        pub const REG_BINCU_SETUP: crate::Register = crate::Register::new(20, 0x3);
        pub const REG_BINCU_SETUP_R_BINCU_DATASIZE: crate::Field = crate::Field::new(2, 0, REG_BINCU_SETUP);

        pub const REG_BINCU_VAL: crate::Register = crate::Register::new(21, 0x7fff);
        pub const REG_BINCU_VAL_BINCU_COUNTER_I: crate::Field = crate::Field::new(15, 0, REG_BINCU_VAL);

        pub const REG_FILT: crate::Register = crate::Register::new(22, 0xf);
        pub const REG_FILT_R_FILTER_MODE: crate::Field = crate::Field::new(4, 0, REG_FILT);

        pub const REG_STATUS: crate::Register = crate::Register::new(24, 0x1);
        pub const REG_STATUS_R_FILTER_DONE: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const HW_UDMA_FILTER_BASE: usize = 0x50110000;
    }

    pub mod udma_scif {
        pub const UDMA_SCIF_NUMREGS: usize = 13;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x1);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x1);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);

        pub const REG_STATUS: crate::Register = crate::Register::new(8, 0x1);
        pub const REG_STATUS_STATUS_I: crate::Field = crate::Field::new(1, 0, REG_STATUS);

        pub const REG_SCIF_SETUP: crate::Register = crate::Register::new(9, 0xffffc33f);
        pub const REG_SCIF_SETUP_R_SCIF_PARITY_EN: crate::Field = crate::Field::new(1, 0, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_BITS: crate::Field = crate::Field::new(2, 1, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_STOP_BITS: crate::Field = crate::Field::new(1, 3, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_RX_POLLING_EN: crate::Field = crate::Field::new(1, 4, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_RX_CLEAN_FIFO: crate::Field = crate::Field::new(1, 5, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_EN_TX: crate::Field = crate::Field::new(1, 8, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_EN_RX: crate::Field = crate::Field::new(1, 9, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_CLKSEL: crate::Field = crate::Field::new(2, 14, REG_SCIF_SETUP);
        pub const REG_SCIF_SETUP_R_SCIF_DIV: crate::Field = crate::Field::new(16, 16, REG_SCIF_SETUP);

        pub const REG_ERROR: crate::Register = crate::Register::new(10, 0x3);
        pub const REG_ERROR_R_ERR_OVERFLOW: crate::Field = crate::Field::new(1, 0, REG_ERROR);
        pub const REG_ERROR_R_ERR_PARITY: crate::Field = crate::Field::new(1, 1, REG_ERROR);

        pub const REG_IRQ_EN: crate::Register = crate::Register::new(11, 0x3);
        pub const REG_IRQ_EN_R_SCIF_RX_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_IRQ_EN);
        pub const REG_IRQ_EN_R_SCIF_ERR_IRQ_EN: crate::Field = crate::Field::new(1, 1, REG_IRQ_EN);

        pub const REG_VALID: crate::Register = crate::Register::new(12, 0x1);
        pub const REG_VALID_R_SCIF_RX_DATA_VALID: crate::Field = crate::Field::new(1, 0, REG_VALID);

        pub const REG_DATA: crate::Register = crate::Register::new(13, 0xff);
        pub const REG_DATA_R_SCIF_RX_DATA: crate::Field = crate::Field::new(8, 0, REG_DATA);

        pub const REG_SCIF_ETU: crate::Register = crate::Register::new(14, 0xffff);
        pub const REG_SCIF_ETU_R_SCIF_ETU: crate::Field = crate::Field::new(16, 0, REG_SCIF_ETU);

        pub const HW_UDMA_SCIF_BASE: usize = 0x50111000;
    }

    pub mod udma_spis_0 {
        pub const UDMA_SPIS_0_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_SPIS_SETUP: crate::Register = crate::Register::new(8, 0x3);
        pub const REG_SPIS_SETUP_CFGCPOL: crate::Field = crate::Field::new(1, 0, REG_SPIS_SETUP);
        pub const REG_SPIS_SETUP_CFGCPHA: crate::Field = crate::Field::new(1, 1, REG_SPIS_SETUP);

        pub const REG_SEOT_CNT: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_SEOT_CNT_SR_SEOT_CNT: crate::Field = crate::Field::new(16, 0, REG_SEOT_CNT);

        pub const REG_SPIS_IRQ_EN: crate::Register = crate::Register::new(10, 0x1);
        pub const REG_SPIS_IRQ_EN_SEOT_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_SPIS_IRQ_EN);

        pub const REG_SPIS_RXCNT: crate::Register = crate::Register::new(11, 0xffff);
        pub const REG_SPIS_RXCNT_CFGRXCNT: crate::Field = crate::Field::new(16, 0, REG_SPIS_RXCNT);

        pub const REG_SPIS_TXCNT: crate::Register = crate::Register::new(12, 0xffff);
        pub const REG_SPIS_TXCNT_CFGTXCNT: crate::Field = crate::Field::new(16, 0, REG_SPIS_TXCNT);

        pub const REG_SPIS_DMCNT: crate::Register = crate::Register::new(13, 0xffff);
        pub const REG_SPIS_DMCNT_CFGDMCNT: crate::Field = crate::Field::new(16, 0, REG_SPIS_DMCNT);

        pub const HW_UDMA_SPIS_0_BASE: usize = 0x50112000;
    }

    pub mod udma_spis_1 {
        pub const UDMA_SPIS_1_NUMREGS: usize = 12;

        pub const REG_RX_SADDR: crate::Register = crate::Register::new(0, 0xfff);
        pub const REG_RX_SADDR_R_RX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_RX_SADDR);

        pub const REG_RX_SIZE: crate::Register = crate::Register::new(1, 0xffff);
        pub const REG_RX_SIZE_R_RX_SIZE: crate::Field = crate::Field::new(16, 0, REG_RX_SIZE);

        pub const REG_RX_CFG: crate::Register = crate::Register::new(2, 0x51);
        pub const REG_RX_CFG_R_RX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_EN: crate::Field = crate::Field::new(1, 4, REG_RX_CFG);
        pub const REG_RX_CFG_R_RX_CLR: crate::Field = crate::Field::new(1, 6, REG_RX_CFG);

        pub const REG_TX_SADDR: crate::Register = crate::Register::new(4, 0xfff);
        pub const REG_TX_SADDR_R_TX_STARTADDR: crate::Field = crate::Field::new(12, 0, REG_TX_SADDR);

        pub const REG_TX_SIZE: crate::Register = crate::Register::new(5, 0xffff);
        pub const REG_TX_SIZE_R_TX_SIZE: crate::Field = crate::Field::new(16, 0, REG_TX_SIZE);

        pub const REG_TX_CFG: crate::Register = crate::Register::new(6, 0x51);
        pub const REG_TX_CFG_R_TX_CONTINUOUS: crate::Field = crate::Field::new(1, 0, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_EN: crate::Field = crate::Field::new(1, 4, REG_TX_CFG);
        pub const REG_TX_CFG_R_TX_CLR: crate::Field = crate::Field::new(1, 6, REG_TX_CFG);

        pub const REG_SPIS_SETUP: crate::Register = crate::Register::new(8, 0x3);
        pub const REG_SPIS_SETUP_CFGCPOL: crate::Field = crate::Field::new(1, 0, REG_SPIS_SETUP);
        pub const REG_SPIS_SETUP_CFGCPHA: crate::Field = crate::Field::new(1, 1, REG_SPIS_SETUP);

        pub const REG_SEOT_CNT: crate::Register = crate::Register::new(9, 0xffff);
        pub const REG_SEOT_CNT_SR_SEOT_CNT: crate::Field = crate::Field::new(16, 0, REG_SEOT_CNT);

        pub const REG_SPIS_IRQ_EN: crate::Register = crate::Register::new(10, 0x1);
        pub const REG_SPIS_IRQ_EN_SEOT_IRQ_EN: crate::Field = crate::Field::new(1, 0, REG_SPIS_IRQ_EN);

        pub const REG_SPIS_RXCNT: crate::Register = crate::Register::new(11, 0xffff);
        pub const REG_SPIS_RXCNT_CFGRXCNT: crate::Field = crate::Field::new(16, 0, REG_SPIS_RXCNT);

        pub const REG_SPIS_TXCNT: crate::Register = crate::Register::new(12, 0xffff);
        pub const REG_SPIS_TXCNT_CFGTXCNT: crate::Field = crate::Field::new(16, 0, REG_SPIS_TXCNT);

        pub const REG_SPIS_DMCNT: crate::Register = crate::Register::new(13, 0xffff);
        pub const REG_SPIS_DMCNT_CFGDMCNT: crate::Field = crate::Field::new(16, 0, REG_SPIS_DMCNT);

        pub const HW_UDMA_SPIS_1_BASE: usize = 0x50113000;
    }

    pub mod udma_adc {
        pub const UDMA_ADC_NUMREGS: usize = 0;

        pub const HW_UDMA_ADC_BASE: usize = 0x50114000;
    }

    pub mod aes {
        pub const AES_NUMREGS: usize = 13;

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

        pub const SFR_OPTLTX: crate::Register = crate::Register::new(6, 0x3f);
        pub const SFR_OPTLTX_SFR_OPTLTX: crate::Field = crate::Field::new(6, 0, SFR_OPTLTX);

        pub const SFR_MASKSEED: crate::Register = crate::Register::new(8, 0xffffffff);
        pub const SFR_MASKSEED_SFR_MASKSEED: crate::Field = crate::Field::new(32, 0, SFR_MASKSEED);

        pub const SFR_MASKSEEDAR: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_MASKSEEDAR_SFR_MASKSEEDAR: crate::Field = crate::Field::new(32, 0, SFR_MASKSEEDAR);

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
        pub const COMBOHASH_NUMREGS: usize = 14;

        pub const SFR_CRFUNC: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_CRFUNC_CR_FUNC: crate::Field = crate::Field::new(8, 0, SFR_CRFUNC);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const SFR_SRMFSM: crate::Register = crate::Register::new(2, 0xff);
        pub const SFR_SRMFSM_MFSM: crate::Field = crate::Field::new(8, 0, SFR_SRMFSM);

        pub const SFR_FR: crate::Register = crate::Register::new(3, 0x3f);
        pub const SFR_FR_MFSM_DONE: crate::Field = crate::Field::new(1, 0, SFR_FR);
        pub const SFR_FR_HASH_DONE: crate::Field = crate::Field::new(1, 1, SFR_FR);
        pub const SFR_FR_CHNLO_DONE: crate::Field = crate::Field::new(1, 2, SFR_FR);
        pub const SFR_FR_CHNLI_DONE: crate::Field = crate::Field::new(1, 3, SFR_FR);
        pub const SFR_FR_CHKDONE: crate::Field = crate::Field::new(1, 4, SFR_FR);
        pub const SFR_FR_CHKPASS: crate::Field = crate::Field::new(1, 5, SFR_FR);

        pub const SFR_OPT1: crate::Register = crate::Register::new(4, 0xffff);
        pub const SFR_OPT1_CR_OPT_HASHCNT: crate::Field = crate::Field::new(16, 0, SFR_OPT1);

        pub const SFR_OPT2: crate::Register = crate::Register::new(5, 0x7);
        pub const SFR_OPT2_CR_OPT_SCRTCHK: crate::Field = crate::Field::new(1, 0, SFR_OPT2);
        pub const SFR_OPT2_CR_OPT_IFSOB: crate::Field = crate::Field::new(1, 1, SFR_OPT2);
        pub const SFR_OPT2_CR_OPT_IFSTART: crate::Field = crate::Field::new(1, 2, SFR_OPT2);

        pub const SFR_OPT3: crate::Register = crate::Register::new(6, 0xff);
        pub const SFR_OPT3_SFR_OPT3: crate::Field = crate::Field::new(8, 0, SFR_OPT3);

        pub const SFR_BLKT0: crate::Register = crate::Register::new(7, 0xff);
        pub const SFR_BLKT0_SFR_BLKT0: crate::Field = crate::Field::new(8, 0, SFR_BLKT0);

        pub const SFR_SEGPTR_SEGID_LKEY: crate::Register = crate::Register::new(8, 0xfff);
        pub const SFR_SEGPTR_SEGID_LKEY_SEGID_LKEY: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_LKEY);

        pub const SFR_SEGPTR_SEGID_KEY: crate::Register = crate::Register::new(9, 0xfff);
        pub const SFR_SEGPTR_SEGID_KEY_SEGID_KEY: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_KEY);

        pub const SFR_SEGPTR_SEGID_SCRT: crate::Register = crate::Register::new(11, 0xfff);
        pub const SFR_SEGPTR_SEGID_SCRT_SEGID_SCRT: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_SCRT);

        pub const SFR_SEGPTR_SEGID_MSG: crate::Register = crate::Register::new(12, 0xfff);
        pub const SFR_SEGPTR_SEGID_MSG_SEGID_MSG: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_MSG);

        pub const SFR_SEGPTR_SEGID_HOUT: crate::Register = crate::Register::new(13, 0xfff);
        pub const SFR_SEGPTR_SEGID_HOUT_SEGID_HOUT: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_HOUT);

        pub const SFR_SEGPTR_SEGID_HOUT2: crate::Register = crate::Register::new(15, 0xfff);
        pub const SFR_SEGPTR_SEGID_HOUT2_SEGID_HOUT2: crate::Field = crate::Field::new(12, 0, SFR_SEGPTR_SEGID_HOUT2);

        pub const HW_COMBOHASH_BASE: usize = 0x4002b000;
    }

    pub mod pke {
        pub const PKE_NUMREGS: usize = 14;

        pub const SFR_CRFUNC: crate::Register = crate::Register::new(0, 0xffff);
        pub const SFR_CRFUNC_CR_FUNC: crate::Field = crate::Field::new(8, 0, SFR_CRFUNC);
        pub const SFR_CRFUNC_CR_PCOREIR: crate::Field = crate::Field::new(8, 8, SFR_CRFUNC);

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

        pub const SFR_OPTRW: crate::Register = crate::Register::new(6, 0x3ff);
        pub const SFR_OPTRW_SFR_OPTRW: crate::Field = crate::Field::new(10, 0, SFR_OPTRW);

        pub const SFR_OPTLTX: crate::Register = crate::Register::new(7, 0x1f);
        pub const SFR_OPTLTX_SFR_OPTLTX: crate::Field = crate::Field::new(5, 0, SFR_OPTLTX);

        pub const SFR_OPTMASK: crate::Register = crate::Register::new(8, 0xffff);
        pub const SFR_OPTMASK_SFR_OPTMASK: crate::Field = crate::Field::new(16, 0, SFR_OPTMASK);

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
        pub const SCEDMA_NUMREGS: usize = 18;

        pub const SFR_SCHSTART_AR: crate::Register = crate::Register::new(0, 0xffffffff);
        pub const SFR_SCHSTART_AR_SFR_SCHSTART_AR: crate::Field = crate::Field::new(32, 0, SFR_SCHSTART_AR);

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
        pub const SCE_GLBSFR_NUMREGS: usize = 18;

        pub const SFR_SCEMODE: crate::Register = crate::Register::new(0, 0x3);
        pub const SFR_SCEMODE_CR_SCEMODE: crate::Field = crate::Field::new(2, 0, SFR_SCEMODE);

        pub const SFR_SUBEN: crate::Register = crate::Register::new(1, 0xffff);
        pub const SFR_SUBEN_CR_SUBEN: crate::Field = crate::Field::new(16, 0, SFR_SUBEN);

        pub const SFR_AHBS: crate::Register = crate::Register::new(2, 0x1f);
        pub const SFR_AHBS_CR_AHBSOPT: crate::Field = crate::Field::new(5, 0, SFR_AHBS);

        pub const SFR_SRBUSY: crate::Register = crate::Register::new(4, 0xffff);
        pub const SFR_SRBUSY_SR_BUSY: crate::Field = crate::Field::new(16, 0, SFR_SRBUSY);

        pub const SFR_FRDONE: crate::Register = crate::Register::new(5, 0xffff);
        pub const SFR_FRDONE_FR_DONE: crate::Field = crate::Field::new(16, 0, SFR_FRDONE);

        pub const SFR_FRERR: crate::Register = crate::Register::new(6, 0xffff);
        pub const SFR_FRERR_FR_ERR: crate::Field = crate::Field::new(16, 0, SFR_FRERR);

        pub const SFR_ARCLR: crate::Register = crate::Register::new(7, 0xffffffff);
        pub const SFR_ARCLR_AR_CLRRAM: crate::Field = crate::Field::new(32, 0, SFR_ARCLR);

        pub const SFR_FRACERR: crate::Register = crate::Register::new(8, 0xff);
        pub const SFR_FRACERR_FR_ACERR: crate::Field = crate::Field::new(8, 0, SFR_FRACERR);

        pub const SFR_TICKCNT: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_TICKCNT_SFR_TICKCNT: crate::Field = crate::Field::new(32, 0, SFR_TICKCNT);

        pub const SFR_FFEN: crate::Register = crate::Register::new(12, 0x3f);
        pub const SFR_FFEN_CR_FFEN: crate::Field = crate::Field::new(6, 0, SFR_FFEN);

        pub const SFR_FFCLR: crate::Register = crate::Register::new(13, 0xffffffff);
        pub const SFR_FFCLR_AR_FFCLR: crate::Field = crate::Field::new(32, 0, SFR_FFCLR);

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

        pub const SFR_TS: crate::Register = crate::Register::new(63, 0xffff);
        pub const SFR_TS_CR_TS: crate::Field = crate::Field::new(16, 0, SFR_TS);

        pub const HW_SCE_GLBSFR_BASE: usize = 0x40028000;
    }

    pub mod trng {
        pub const TRNG_NUMREGS: usize = 13;

        pub const SFR_CRSRC: crate::Register = crate::Register::new(0, 0xfff);
        pub const SFR_CRSRC_SFR_CRSRC: crate::Field = crate::Field::new(12, 0, SFR_CRSRC);

        pub const SFR_CRANA: crate::Register = crate::Register::new(1, 0xffff);
        pub const SFR_CRANA_SFR_CRANA: crate::Field = crate::Field::new(16, 0, SFR_CRANA);

        pub const SFR_PP: crate::Register = crate::Register::new(2, 0x1ffff);
        pub const SFR_PP_SFR_PP: crate::Field = crate::Field::new(17, 0, SFR_PP);

        pub const SFR_OPT: crate::Register = crate::Register::new(3, 0x1ffff);
        pub const SFR_OPT_SFR_OPT: crate::Field = crate::Field::new(17, 0, SFR_OPT);

        pub const SFR_SR: crate::Register = crate::Register::new(4, 0xffffffff);
        pub const SFR_SR_SR_RNG: crate::Field = crate::Field::new(32, 0, SFR_SR);

        pub const SFR_AR_GEN: crate::Register = crate::Register::new(5, 0xffffffff);
        pub const SFR_AR_GEN_SFR_AR_GEN: crate::Field = crate::Field::new(32, 0, SFR_AR_GEN);

        pub const SFR_FR: crate::Register = crate::Register::new(6, 0x3);
        pub const SFR_FR_SFR_FR: crate::Field = crate::Field::new(2, 0, SFR_FR);

        pub const SFR_DRPSZ: crate::Register = crate::Register::new(8, 0xffffffff);
        pub const SFR_DRPSZ_SFR_DRPSZ: crate::Field = crate::Field::new(32, 0, SFR_DRPSZ);

        pub const SFR_DRGEN: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_DRGEN_SFR_DRGEN: crate::Field = crate::Field::new(32, 0, SFR_DRGEN);

        pub const SFR_DRRESEED: crate::Register = crate::Register::new(10, 0xffffffff);
        pub const SFR_DRRESEED_SFR_DRRESEED: crate::Field = crate::Field::new(32, 0, SFR_DRRESEED);

        pub const SFR_BUF: crate::Register = crate::Register::new(12, 0xffffffff);
        pub const SFR_BUF_SFR_BUF: crate::Field = crate::Field::new(32, 0, SFR_BUF);

        pub const SFR_CHAIN_RNGCHAINEN0: crate::Register = crate::Register::new(16, 0xffffffff);
        pub const SFR_CHAIN_RNGCHAINEN0_RNGCHAINEN0: crate::Field = crate::Field::new(32, 0, SFR_CHAIN_RNGCHAINEN0);

        pub const SFR_CHAIN_RNGCHAINEN1: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const SFR_CHAIN_RNGCHAINEN1_RNGCHAINEN1: crate::Field = crate::Field::new(32, 0, SFR_CHAIN_RNGCHAINEN1);

        pub const HW_TRNG_BASE: usize = 0x4002e000;
    }

    pub mod alu {
        pub const ALU_NUMREGS: usize = 0;

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
        pub const WDG_INTF_NUMREGS: usize = 0;

        pub const HW_WDG_INTF_BASE: usize = 0x40041000;
    }

    pub mod timer_intf {
        pub const TIMER_INTF_NUMREGS: usize = 0;

        pub const HW_TIMER_INTF_BASE: usize = 0x40043000;
    }

    pub mod evc {
        pub const EVC_NUMREGS: usize = 22;

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

        pub const SFR_TMREVSEL: crate::Register = crate::Register::new(12, 0xffff);
        pub const SFR_TMREVSEL_TMR_EVSEL: crate::Field = crate::Field::new(16, 0, SFR_TMREVSEL);

        pub const SFR_PWMEVSEL: crate::Register = crate::Register::new(13, 0xffffffff);
        pub const SFR_PWMEVSEL_PWM_EVSEL: crate::Field = crate::Field::new(32, 0, SFR_PWMEVSEL);

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

        pub const SFR_CM7ERRFR: crate::Register = crate::Register::new(32, 0xffffffff);
        pub const SFR_CM7ERRFR_ERRIN: crate::Field = crate::Field::new(32, 0, SFR_CM7ERRFR);

        pub const HW_EVC_BASE: usize = 0x40044000;
    }

    pub mod sysctrl {
        pub const SYSCTRL_NUMREGS: usize = 35;

        pub const SFR_CGUSEC: crate::Register = crate::Register::new(0, 0xffff);
        pub const SFR_CGUSEC_SFR_CGUSEC: crate::Field = crate::Field::new(16, 0, SFR_CGUSEC);

        pub const SFR_CGULP: crate::Register = crate::Register::new(1, 0xffff);
        pub const SFR_CGULP_SFR_CGULP: crate::Field = crate::Field::new(16, 0, SFR_CGULP);

        pub const SFR_SEED: crate::Register = crate::Register::new(2, 0xffffffff);
        pub const SFR_SEED_SFR_SEED: crate::Field = crate::Field::new(32, 0, SFR_SEED);

        pub const SFR_SEEDAR: crate::Register = crate::Register::new(3, 0xffffffff);
        pub const SFR_SEEDAR_SFR_SEEDAR: crate::Field = crate::Field::new(32, 0, SFR_SEEDAR);

        pub const SFR_CGUSEL0: crate::Register = crate::Register::new(4, 0x3);
        pub const SFR_CGUSEL0_SFR_CGUSEL0: crate::Field = crate::Field::new(2, 0, SFR_CGUSEL0);

        pub const SFR_CGUFD_CFGFDCR_0_4_0: crate::Register = crate::Register::new(5, 0xffff);
        pub const SFR_CGUFD_CFGFDCR_0_4_0_CFGFDCR_0_4_0: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR_0_4_0);

        pub const SFR_CGUFD_CFGFDCR_0_4_1: crate::Register = crate::Register::new(6, 0xffff);
        pub const SFR_CGUFD_CFGFDCR_0_4_1_CFGFDCR_0_4_1: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR_0_4_1);

        pub const SFR_CGUFD_CFGFDCR_0_4_2: crate::Register = crate::Register::new(7, 0xffff);
        pub const SFR_CGUFD_CFGFDCR_0_4_2_CFGFDCR_0_4_2: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR_0_4_2);

        pub const SFR_CGUFD_CFGFDCR_0_4_3: crate::Register = crate::Register::new(8, 0xffff);
        pub const SFR_CGUFD_CFGFDCR_0_4_3_CFGFDCR_0_4_3: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR_0_4_3);

        pub const SFR_CGUFD_CFGFDCR_0_4_4: crate::Register = crate::Register::new(9, 0xffff);
        pub const SFR_CGUFD_CFGFDCR_0_4_4_CFGFDCR_0_4_4: crate::Field = crate::Field::new(16, 0, SFR_CGUFD_CFGFDCR_0_4_4);

        pub const SFR_CGUFDAO: crate::Register = crate::Register::new(10, 0xffff);
        pub const SFR_CGUFDAO_CFGFDCR: crate::Field = crate::Field::new(16, 0, SFR_CGUFDAO);

        pub const SFR_CGUSET: crate::Register = crate::Register::new(11, 0xffffffff);
        pub const SFR_CGUSET_SFR_CGUSET: crate::Field = crate::Field::new(32, 0, SFR_CGUSET);

        pub const SFR_CGUSEL1: crate::Register = crate::Register::new(12, 0x1);
        pub const SFR_CGUSEL1_SFR_CGUSEL1: crate::Field = crate::Field::new(1, 0, SFR_CGUSEL1);

        pub const SFR_CGUFDPKE: crate::Register = crate::Register::new(13, 0x1ff);
        pub const SFR_CGUFDPKE_SFR_CGUFDPKE: crate::Field = crate::Field::new(9, 0, SFR_CGUFDPKE);

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

        pub const SFR_ACLKGR: crate::Register = crate::Register::new(24, 0xff);
        pub const SFR_ACLKGR_SFR_ACLKGR: crate::Field = crate::Field::new(8, 0, SFR_ACLKGR);

        pub const SFR_HCLKGR: crate::Register = crate::Register::new(25, 0xff);
        pub const SFR_HCLKGR_SFR_HCLKGR: crate::Field = crate::Field::new(8, 0, SFR_HCLKGR);

        pub const SFR_ICLKGR: crate::Register = crate::Register::new(26, 0xff);
        pub const SFR_ICLKGR_SFR_ICLKGR: crate::Field = crate::Field::new(8, 0, SFR_ICLKGR);

        pub const SFR_PCLKGR: crate::Register = crate::Register::new(27, 0xff);
        pub const SFR_PCLKGR_SFR_PCLKGR: crate::Field = crate::Field::new(8, 0, SFR_PCLKGR);

        pub const SFR_RCURST0: crate::Register = crate::Register::new(32, 0xffffffff);
        pub const SFR_RCURST0_SFR_RCURST0: crate::Field = crate::Field::new(32, 0, SFR_RCURST0);

        pub const SFR_RCURST1: crate::Register = crate::Register::new(33, 0xffffffff);
        pub const SFR_RCURST1_SFR_RCURST1: crate::Field = crate::Field::new(32, 0, SFR_RCURST1);

        pub const SFR_RCUSRCFR: crate::Register = crate::Register::new(34, 0xffff);
        pub const SFR_RCUSRCFR_SFR_RCUSRCFR: crate::Field = crate::Field::new(16, 0, SFR_RCUSRCFR);

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
        pub const APB_THRU_NUMREGS: usize = 0;

        pub const HW_APB_THRU_BASE: usize = 0x50122000;
    }

    pub mod iox {
        pub const IOX_NUMREGS: usize = 64;

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

        pub const SFR_AFSEL_CRAFSEL8: crate::Register = crate::Register::new(8, 0xffff);
        pub const SFR_AFSEL_CRAFSEL8_CRAFSEL8: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL8);

        pub const SFR_AFSEL_CRAFSEL9: crate::Register = crate::Register::new(9, 0xffff);
        pub const SFR_AFSEL_CRAFSEL9_CRAFSEL9: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL9);

        pub const SFR_AFSEL_CRAFSEL10: crate::Register = crate::Register::new(10, 0xffff);
        pub const SFR_AFSEL_CRAFSEL10_CRAFSEL10: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL10);

        pub const SFR_AFSEL_CRAFSEL11: crate::Register = crate::Register::new(11, 0xffff);
        pub const SFR_AFSEL_CRAFSEL11_CRAFSEL11: crate::Field = crate::Field::new(16, 0, SFR_AFSEL_CRAFSEL11);

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

        pub const SFR_GPIOOUT_CRGO0: crate::Register = crate::Register::new(76, 0xffff);
        pub const SFR_GPIOOUT_CRGO0_CRGO0: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO0);

        pub const SFR_GPIOOUT_CRGO1: crate::Register = crate::Register::new(77, 0xffff);
        pub const SFR_GPIOOUT_CRGO1_CRGO1: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO1);

        pub const SFR_GPIOOUT_CRGO2: crate::Register = crate::Register::new(78, 0xffff);
        pub const SFR_GPIOOUT_CRGO2_CRGO2: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO2);

        pub const SFR_GPIOOUT_CRGO3: crate::Register = crate::Register::new(79, 0xffff);
        pub const SFR_GPIOOUT_CRGO3_CRGO3: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO3);

        pub const SFR_GPIOOUT_CRGO4: crate::Register = crate::Register::new(80, 0xffff);
        pub const SFR_GPIOOUT_CRGO4_CRGO4: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO4);

        pub const SFR_GPIOOUT_CRGO5: crate::Register = crate::Register::new(81, 0xffff);
        pub const SFR_GPIOOUT_CRGO5_CRGO5: crate::Field = crate::Field::new(16, 0, SFR_GPIOOUT_CRGO5);

        pub const SFR_GPIOOE_CRGOE0: crate::Register = crate::Register::new(82, 0xffff);
        pub const SFR_GPIOOE_CRGOE0_CRGOE0: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE0);

        pub const SFR_GPIOOE_CRGOE1: crate::Register = crate::Register::new(83, 0xffff);
        pub const SFR_GPIOOE_CRGOE1_CRGOE1: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE1);

        pub const SFR_GPIOOE_CRGOE2: crate::Register = crate::Register::new(84, 0xffff);
        pub const SFR_GPIOOE_CRGOE2_CRGOE2: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE2);

        pub const SFR_GPIOOE_CRGOE3: crate::Register = crate::Register::new(85, 0xffff);
        pub const SFR_GPIOOE_CRGOE3_CRGOE3: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE3);

        pub const SFR_GPIOOE_CRGOE4: crate::Register = crate::Register::new(86, 0xffff);
        pub const SFR_GPIOOE_CRGOE4_CRGOE4: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE4);

        pub const SFR_GPIOOE_CRGOE5: crate::Register = crate::Register::new(87, 0xffff);
        pub const SFR_GPIOOE_CRGOE5_CRGOE5: crate::Field = crate::Field::new(16, 0, SFR_GPIOOE_CRGOE5);

        pub const SFR_GPIOPU_CRGPU0: crate::Register = crate::Register::new(88, 0xffff);
        pub const SFR_GPIOPU_CRGPU0_CRGPU0: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU0);

        pub const SFR_GPIOPU_CRGPU1: crate::Register = crate::Register::new(89, 0xffff);
        pub const SFR_GPIOPU_CRGPU1_CRGPU1: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU1);

        pub const SFR_GPIOPU_CRGPU2: crate::Register = crate::Register::new(90, 0xffff);
        pub const SFR_GPIOPU_CRGPU2_CRGPU2: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU2);

        pub const SFR_GPIOPU_CRGPU3: crate::Register = crate::Register::new(91, 0xffff);
        pub const SFR_GPIOPU_CRGPU3_CRGPU3: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU3);

        pub const SFR_GPIOPU_CRGPU4: crate::Register = crate::Register::new(92, 0xffff);
        pub const SFR_GPIOPU_CRGPU4_CRGPU4: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU4);

        pub const SFR_GPIOPU_CRGPU5: crate::Register = crate::Register::new(93, 0xffff);
        pub const SFR_GPIOPU_CRGPU5_CRGPU5: crate::Field = crate::Field::new(16, 0, SFR_GPIOPU_CRGPU5);

        pub const SFR_GPIOIN_SRGI0: crate::Register = crate::Register::new(94, 0xffff);
        pub const SFR_GPIOIN_SRGI0_SRGI0: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI0);

        pub const SFR_GPIOIN_SRGI1: crate::Register = crate::Register::new(95, 0xffff);
        pub const SFR_GPIOIN_SRGI1_SRGI1: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI1);

        pub const SFR_GPIOIN_SRGI2: crate::Register = crate::Register::new(96, 0xffff);
        pub const SFR_GPIOIN_SRGI2_SRGI2: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI2);

        pub const SFR_GPIOIN_SRGI3: crate::Register = crate::Register::new(97, 0xffff);
        pub const SFR_GPIOIN_SRGI3_SRGI3: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI3);

        pub const SFR_GPIOIN_SRGI4: crate::Register = crate::Register::new(98, 0xffff);
        pub const SFR_GPIOIN_SRGI4_SRGI4: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI4);

        pub const SFR_GPIOIN_SRGI5: crate::Register = crate::Register::new(99, 0xffff);
        pub const SFR_GPIOIN_SRGI5_SRGI5: crate::Field = crate::Field::new(16, 0, SFR_GPIOIN_SRGI5);

        pub const SFR_PIOSEL: crate::Register = crate::Register::new(128, 0xffffffff);
        pub const SFR_PIOSEL_PIOSEL: crate::Field = crate::Field::new(32, 0, SFR_PIOSEL);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL0: crate::Register = crate::Register::new(140, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL0_CR_CFG_SCHMSEL0: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL0);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL1: crate::Register = crate::Register::new(141, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL1_CR_CFG_SCHMSEL1: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL1);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL2: crate::Register = crate::Register::new(142, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL2_CR_CFG_SCHMSEL2: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL2);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL3: crate::Register = crate::Register::new(143, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL3_CR_CFG_SCHMSEL3: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL3);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL4: crate::Register = crate::Register::new(144, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL4_CR_CFG_SCHMSEL4: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL4);

        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL5: crate::Register = crate::Register::new(145, 0xffff);
        pub const SFR_CFG_SCHM_CR_CFG_SCHMSEL5_CR_CFG_SCHMSEL5: crate::Field = crate::Field::new(16, 0, SFR_CFG_SCHM_CR_CFG_SCHMSEL5);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW0: crate::Register = crate::Register::new(146, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW0_CR_CFG_SLEWSLOW0: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW0);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW1: crate::Register = crate::Register::new(147, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW1_CR_CFG_SLEWSLOW1: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW1);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW2: crate::Register = crate::Register::new(148, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW2_CR_CFG_SLEWSLOW2: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW2);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW3: crate::Register = crate::Register::new(149, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW3_CR_CFG_SLEWSLOW3: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW3);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW4: crate::Register = crate::Register::new(150, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW4_CR_CFG_SLEWSLOW4: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW4);

        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW5: crate::Register = crate::Register::new(151, 0xffff);
        pub const SFR_CFG_SLEW_CR_CFG_SLEWSLOW5_CR_CFG_SLEWSLOW5: crate::Field = crate::Field::new(16, 0, SFR_CFG_SLEW_CR_CFG_SLEWSLOW5);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL0: crate::Register = crate::Register::new(152, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL0_CR_CFG_DRVSEL0: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL0);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL1: crate::Register = crate::Register::new(153, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL1_CR_CFG_DRVSEL1: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL1);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL2: crate::Register = crate::Register::new(154, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL2_CR_CFG_DRVSEL2: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL2);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL3: crate::Register = crate::Register::new(155, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL3_CR_CFG_DRVSEL3: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL3);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL4: crate::Register = crate::Register::new(156, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL4_CR_CFG_DRVSEL4: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL4);

        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL5: crate::Register = crate::Register::new(157, 0xffffffff);
        pub const SFR_CFG_DRVSEL_CR_CFG_DRVSEL5_CR_CFG_DRVSEL5: crate::Field = crate::Field::new(32, 0, SFR_CFG_DRVSEL_CR_CFG_DRVSEL5);

        pub const HW_IOX_BASE: usize = 0x5012f000;
    }

    pub mod pwm {
        pub const PWM_NUMREGS: usize = 0;

        pub const HW_PWM_BASE: usize = 0x50120000;
    }

    pub mod sddc {
        pub const SDDC_NUMREGS: usize = 112;

        pub const SFR_IO: crate::Register = crate::Register::new(0, 0x3);
        pub const SFR_IO_SFR_IO: crate::Field = crate::Field::new(2, 0, SFR_IO);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

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

        pub const CR_REG_UHS_1_SUPPORT: crate::Register = crate::Register::new(124, 0xffffffff);
        pub const CR_REG_UHS_1_SUPPORT_CFG_REG_MAX_CURRENT: crate::Field = crate::Field::new(16, 0, CR_REG_UHS_1_SUPPORT);
        pub const CR_REG_UHS_1_SUPPORT_CFG_REG_DATA_STRC_VERSION: crate::Field = crate::Field::new(8, 16, CR_REG_UHS_1_SUPPORT);
        pub const CR_REG_UHS_1_SUPPORT_CFG_REG_UHS_1_SUPPORT: crate::Field = crate::Field::new(8, 24, CR_REG_UHS_1_SUPPORT);

        pub const HW_SDDC_BASE: usize = 0x50121000;
    }

    pub mod rp_pio {
        pub const RP_PIO_NUMREGS: usize = 89;

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
        pub const SFR_FDEBUG_NC_DBG3: crate::Field = crate::Field::new(4, 4, SFR_FDEBUG);
        pub const SFR_FDEBUG_RXUNDER: crate::Field = crate::Field::new(4, 8, SFR_FDEBUG);
        pub const SFR_FDEBUG_NC_DBG2: crate::Field = crate::Field::new(4, 12, SFR_FDEBUG);
        pub const SFR_FDEBUG_TXOVER: crate::Field = crate::Field::new(4, 16, SFR_FDEBUG);
        pub const SFR_FDEBUG_NC_DBG1: crate::Field = crate::Field::new(4, 20, SFR_FDEBUG);
        pub const SFR_FDEBUG_TXSTALL: crate::Field = crate::Field::new(4, 24, SFR_FDEBUG);
        pub const SFR_FDEBUG_NC_DBG0: crate::Field = crate::Field::new(4, 28, SFR_FDEBUG);

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

        pub const SFR_TXF3: crate::Register = crate::Register::new(7, 0xffffffff);
        pub const SFR_TXF3_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF3);

        pub const SFR_RXF0: crate::Register = crate::Register::new(8, 0xffffffff);
        pub const SFR_RXF0_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF0);

        pub const SFR_RXF1: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_RXF1_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF1);

        pub const SFR_RXF2: crate::Register = crate::Register::new(10, 0xffffffff);
        pub const SFR_RXF2_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF2);

        pub const SFR_RXF3: crate::Register = crate::Register::new(11, 0xffffffff);
        pub const SFR_RXF3_PDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF3);

        pub const SFR_IRQ: crate::Register = crate::Register::new(12, 0xff);
        pub const SFR_IRQ_SFR_IRQ: crate::Field = crate::Field::new(8, 0, SFR_IRQ);

        pub const SFR_IRQ_FORCE: crate::Register = crate::Register::new(13, 0xff);
        pub const SFR_IRQ_FORCE_SFR_IRQ_FORCE: crate::Field = crate::Field::new(8, 0, SFR_IRQ_FORCE);

        pub const SFR_SYNC_BYPASS: crate::Register = crate::Register::new(14, 0xffffffff);
        pub const SFR_SYNC_BYPASS_SFR_SYNC_BYPASS: crate::Field = crate::Field::new(32, 0, SFR_SYNC_BYPASS);

        pub const SFR_DBG_PADOUT: crate::Register = crate::Register::new(15, 0xffffffff);
        pub const SFR_DBG_PADOUT_SFR_DBG_PADOUT: crate::Field = crate::Field::new(32, 0, SFR_DBG_PADOUT);

        pub const SFR_DBG_PADOE: crate::Register = crate::Register::new(16, 0xffffffff);
        pub const SFR_DBG_PADOE_SFR_DBG_PADOE: crate::Field = crate::Field::new(32, 0, SFR_DBG_PADOE);

        pub const SFR_DBG_CFGINFO: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const SFR_DBG_CFGINFO_CONSTANT0: crate::Field = crate::Field::new(8, 0, SFR_DBG_CFGINFO);
        pub const SFR_DBG_CFGINFO_CONSTANT1: crate::Field = crate::Field::new(8, 8, SFR_DBG_CFGINFO);
        pub const SFR_DBG_CFGINFO_CONSTANT2: crate::Field = crate::Field::new(16, 16, SFR_DBG_CFGINFO);

        pub const SFR_INSTR_MEM0: crate::Register = crate::Register::new(18, 0xffff);
        pub const SFR_INSTR_MEM0_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM0);

        pub const SFR_INSTR_MEM1: crate::Register = crate::Register::new(19, 0xffff);
        pub const SFR_INSTR_MEM1_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM1);

        pub const SFR_INSTR_MEM2: crate::Register = crate::Register::new(20, 0xffff);
        pub const SFR_INSTR_MEM2_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM2);

        pub const SFR_INSTR_MEM3: crate::Register = crate::Register::new(21, 0xffff);
        pub const SFR_INSTR_MEM3_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM3);

        pub const SFR_INSTR_MEM4: crate::Register = crate::Register::new(22, 0xffff);
        pub const SFR_INSTR_MEM4_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM4);

        pub const SFR_INSTR_MEM5: crate::Register = crate::Register::new(23, 0xffff);
        pub const SFR_INSTR_MEM5_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM5);

        pub const SFR_INSTR_MEM6: crate::Register = crate::Register::new(24, 0xffff);
        pub const SFR_INSTR_MEM6_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM6);

        pub const SFR_INSTR_MEM7: crate::Register = crate::Register::new(25, 0xffff);
        pub const SFR_INSTR_MEM7_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM7);

        pub const SFR_INSTR_MEM8: crate::Register = crate::Register::new(26, 0xffff);
        pub const SFR_INSTR_MEM8_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM8);

        pub const SFR_INSTR_MEM9: crate::Register = crate::Register::new(27, 0xffff);
        pub const SFR_INSTR_MEM9_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM9);

        pub const SFR_INSTR_MEM10: crate::Register = crate::Register::new(28, 0xffff);
        pub const SFR_INSTR_MEM10_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM10);

        pub const SFR_INSTR_MEM11: crate::Register = crate::Register::new(29, 0xffff);
        pub const SFR_INSTR_MEM11_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM11);

        pub const SFR_INSTR_MEM12: crate::Register = crate::Register::new(30, 0xffff);
        pub const SFR_INSTR_MEM12_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM12);

        pub const SFR_INSTR_MEM13: crate::Register = crate::Register::new(31, 0xffff);
        pub const SFR_INSTR_MEM13_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM13);

        pub const SFR_INSTR_MEM14: crate::Register = crate::Register::new(32, 0xffff);
        pub const SFR_INSTR_MEM14_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM14);

        pub const SFR_INSTR_MEM15: crate::Register = crate::Register::new(33, 0xffff);
        pub const SFR_INSTR_MEM15_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM15);

        pub const SFR_INSTR_MEM16: crate::Register = crate::Register::new(34, 0xffff);
        pub const SFR_INSTR_MEM16_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM16);

        pub const SFR_INSTR_MEM17: crate::Register = crate::Register::new(35, 0xffff);
        pub const SFR_INSTR_MEM17_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM17);

        pub const SFR_INSTR_MEM18: crate::Register = crate::Register::new(36, 0xffff);
        pub const SFR_INSTR_MEM18_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM18);

        pub const SFR_INSTR_MEM19: crate::Register = crate::Register::new(37, 0xffff);
        pub const SFR_INSTR_MEM19_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM19);

        pub const SFR_INSTR_MEM20: crate::Register = crate::Register::new(38, 0xffff);
        pub const SFR_INSTR_MEM20_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM20);

        pub const SFR_INSTR_MEM21: crate::Register = crate::Register::new(39, 0xffff);
        pub const SFR_INSTR_MEM21_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM21);

        pub const SFR_INSTR_MEM22: crate::Register = crate::Register::new(40, 0xffff);
        pub const SFR_INSTR_MEM22_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM22);

        pub const SFR_INSTR_MEM23: crate::Register = crate::Register::new(41, 0xffff);
        pub const SFR_INSTR_MEM23_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM23);

        pub const SFR_INSTR_MEM24: crate::Register = crate::Register::new(42, 0xffff);
        pub const SFR_INSTR_MEM24_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM24);

        pub const SFR_INSTR_MEM25: crate::Register = crate::Register::new(43, 0xffff);
        pub const SFR_INSTR_MEM25_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM25);

        pub const SFR_INSTR_MEM26: crate::Register = crate::Register::new(44, 0xffff);
        pub const SFR_INSTR_MEM26_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM26);

        pub const SFR_INSTR_MEM27: crate::Register = crate::Register::new(45, 0xffff);
        pub const SFR_INSTR_MEM27_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM27);

        pub const SFR_INSTR_MEM28: crate::Register = crate::Register::new(46, 0xffff);
        pub const SFR_INSTR_MEM28_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM28);

        pub const SFR_INSTR_MEM29: crate::Register = crate::Register::new(47, 0xffff);
        pub const SFR_INSTR_MEM29_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM29);

        pub const SFR_INSTR_MEM30: crate::Register = crate::Register::new(48, 0xffff);
        pub const SFR_INSTR_MEM30_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM30);

        pub const SFR_INSTR_MEM31: crate::Register = crate::Register::new(49, 0xffff);
        pub const SFR_INSTR_MEM31_INSTR: crate::Field = crate::Field::new(16, 0, SFR_INSTR_MEM31);

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
        pub const SFR_SM0_EXECCTRL_EXEC_STALLED_RO0: crate::Field = crate::Field::new(1, 31, SFR_SM0_EXECCTRL);

        pub const SFR_SM0_SHIFTCTRL: crate::Register = crate::Register::new(52, 0xffffffff);
        pub const SFR_SM0_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_JOIN_TX: crate::Field = crate::Field::new(1, 30, SFR_SM0_SHIFTCTRL);
        pub const SFR_SM0_SHIFTCTRL_JOIN_RX: crate::Field = crate::Field::new(1, 31, SFR_SM0_SHIFTCTRL);

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
        pub const SFR_SM1_EXECCTRL_EXEC_STALLED_RO1: crate::Field = crate::Field::new(1, 31, SFR_SM1_EXECCTRL);

        pub const SFR_SM1_SHIFTCTRL: crate::Register = crate::Register::new(58, 0xffffffff);
        pub const SFR_SM1_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_JOIN_TX: crate::Field = crate::Field::new(1, 30, SFR_SM1_SHIFTCTRL);
        pub const SFR_SM1_SHIFTCTRL_JOIN_RX: crate::Field = crate::Field::new(1, 31, SFR_SM1_SHIFTCTRL);

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
        pub const SFR_SM2_EXECCTRL_EXEC_STALLED_RO2: crate::Field = crate::Field::new(1, 31, SFR_SM2_EXECCTRL);

        pub const SFR_SM2_SHIFTCTRL: crate::Register = crate::Register::new(64, 0xffffffff);
        pub const SFR_SM2_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_JOIN_TX: crate::Field = crate::Field::new(1, 30, SFR_SM2_SHIFTCTRL);
        pub const SFR_SM2_SHIFTCTRL_JOIN_RX: crate::Field = crate::Field::new(1, 31, SFR_SM2_SHIFTCTRL);

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
        pub const SFR_SM3_EXECCTRL_EXEC_STALLED_RO3: crate::Field = crate::Field::new(1, 31, SFR_SM3_EXECCTRL);

        pub const SFR_SM3_SHIFTCTRL: crate::Register = crate::Register::new(70, 0xffffffff);
        pub const SFR_SM3_SHIFTCTRL_RESVD_SHIFT: crate::Field = crate::Field::new(16, 0, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_AUTO_PUSH: crate::Field = crate::Field::new(1, 16, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_AUTO_PULL: crate::Field = crate::Field::new(1, 17, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR: crate::Field = crate::Field::new(1, 18, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR: crate::Field = crate::Field::new(1, 19, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_ISR_THRESHOLD: crate::Field = crate::Field::new(5, 20, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_OSR_THRESHOLD: crate::Field = crate::Field::new(5, 25, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_JOIN_TX: crate::Field = crate::Field::new(1, 30, SFR_SM3_SHIFTCTRL);
        pub const SFR_SM3_SHIFTCTRL_JOIN_RX: crate::Field = crate::Field::new(1, 31, SFR_SM3_SHIFTCTRL);

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

        pub const SFR_IO_OE_INV: crate::Register = crate::Register::new(96, 0xffffffff);
        pub const SFR_IO_OE_INV_SFR_IO_OE_INV: crate::Field = crate::Field::new(32, 0, SFR_IO_OE_INV);

        pub const SFR_IO_O_INV: crate::Register = crate::Register::new(97, 0xffffffff);
        pub const SFR_IO_O_INV_SFR_IO_O_INV: crate::Field = crate::Field::new(32, 0, SFR_IO_O_INV);

        pub const SFR_IO_I_INV: crate::Register = crate::Register::new(98, 0xffffffff);
        pub const SFR_IO_I_INV_SFR_IO_I_INV: crate::Field = crate::Field::new(32, 0, SFR_IO_I_INV);

        pub const SFR_FIFO_MARGIN: crate::Register = crate::Register::new(99, 0xffff);
        pub const SFR_FIFO_MARGIN_FIFO_TX_MARGIN0: crate::Field = crate::Field::new(2, 0, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_RX_MARGIN0: crate::Field = crate::Field::new(2, 2, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_TX_MARGIN1: crate::Field = crate::Field::new(2, 4, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_RX_MARGIN1: crate::Field = crate::Field::new(2, 6, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_TX_MARGIN2: crate::Field = crate::Field::new(2, 8, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_RX_MARGIN2: crate::Field = crate::Field::new(2, 10, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_TX_MARGIN3: crate::Field = crate::Field::new(2, 12, SFR_FIFO_MARGIN);
        pub const SFR_FIFO_MARGIN_FIFO_RX_MARGIN3: crate::Field = crate::Field::new(2, 14, SFR_FIFO_MARGIN);

        pub const SFR_ZERO0: crate::Register = crate::Register::new(100, 0xffffffff);
        pub const SFR_ZERO0_SFR_ZERO0: crate::Field = crate::Field::new(32, 0, SFR_ZERO0);

        pub const SFR_ZERO1: crate::Register = crate::Register::new(101, 0xffffffff);
        pub const SFR_ZERO1_SFR_ZERO1: crate::Field = crate::Field::new(32, 0, SFR_ZERO1);

        pub const SFR_ZERO2: crate::Register = crate::Register::new(102, 0xffffffff);
        pub const SFR_ZERO2_SFR_ZERO2: crate::Field = crate::Field::new(32, 0, SFR_ZERO2);

        pub const SFR_ZERO3: crate::Register = crate::Register::new(103, 0xffffffff);
        pub const SFR_ZERO3_SFR_ZERO3: crate::Field = crate::Field::new(32, 0, SFR_ZERO3);

        pub const HW_RP_PIO_BASE: usize = 0x50123000;
    }

    pub mod bio {
        pub const BIO_NUMREGS: usize = 32;

        pub const SFR_CTRL: crate::Register = crate::Register::new(0, 0xfff);
        pub const SFR_CTRL_EN: crate::Field = crate::Field::new(4, 0, SFR_CTRL);
        pub const SFR_CTRL_RESTART: crate::Field = crate::Field::new(4, 4, SFR_CTRL);
        pub const SFR_CTRL_CLKDIV_RESTART: crate::Field = crate::Field::new(4, 8, SFR_CTRL);

        pub const SFR_CFGINFO: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_CFGINFO_CONSTANT0: crate::Field = crate::Field::new(8, 0, SFR_CFGINFO);
        pub const SFR_CFGINFO_CONSTANT1: crate::Field = crate::Field::new(8, 8, SFR_CFGINFO);
        pub const SFR_CFGINFO_CONSTANT2: crate::Field = crate::Field::new(16, 16, SFR_CFGINFO);

        pub const SFR_FLEVEL: crate::Register = crate::Register::new(3, 0xffff);
        pub const SFR_FLEVEL_PCLK_REGFIFO_LEVEL0: crate::Field = crate::Field::new(4, 0, SFR_FLEVEL);
        pub const SFR_FLEVEL_PCLK_REGFIFO_LEVEL1: crate::Field = crate::Field::new(4, 4, SFR_FLEVEL);
        pub const SFR_FLEVEL_PCLK_REGFIFO_LEVEL2: crate::Field = crate::Field::new(4, 8, SFR_FLEVEL);
        pub const SFR_FLEVEL_PCLK_REGFIFO_LEVEL3: crate::Field = crate::Field::new(4, 12, SFR_FLEVEL);

        pub const SFR_TXF0: crate::Register = crate::Register::new(4, 0xffffffff);
        pub const SFR_TXF0_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF0);

        pub const SFR_TXF1: crate::Register = crate::Register::new(5, 0xffffffff);
        pub const SFR_TXF1_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF1);

        pub const SFR_TXF2: crate::Register = crate::Register::new(6, 0xffffffff);
        pub const SFR_TXF2_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF2);

        pub const SFR_TXF3: crate::Register = crate::Register::new(7, 0xffffffff);
        pub const SFR_TXF3_FDIN: crate::Field = crate::Field::new(32, 0, SFR_TXF3);

        pub const SFR_RXF0: crate::Register = crate::Register::new(8, 0xffffffff);
        pub const SFR_RXF0_FDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF0);

        pub const SFR_RXF1: crate::Register = crate::Register::new(9, 0xffffffff);
        pub const SFR_RXF1_FDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF1);

        pub const SFR_RXF2: crate::Register = crate::Register::new(10, 0xffffffff);
        pub const SFR_RXF2_FDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF2);

        pub const SFR_RXF3: crate::Register = crate::Register::new(11, 0xffffffff);
        pub const SFR_RXF3_FDOUT: crate::Field = crate::Field::new(32, 0, SFR_RXF3);

        pub const SFR_ELEVEL0: crate::Register = crate::Register::new(12, 0xffffffff);
        pub const SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL0: crate::Field = crate::Field::new(8, 0, SFR_ELEVEL0);
        pub const SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL1: crate::Field = crate::Field::new(8, 8, SFR_ELEVEL0);
        pub const SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL2: crate::Field = crate::Field::new(8, 16, SFR_ELEVEL0);
        pub const SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL3: crate::Field = crate::Field::new(8, 24, SFR_ELEVEL0);

        pub const SFR_ELEVEL1: crate::Register = crate::Register::new(13, 0xffffffff);
        pub const SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL4: crate::Field = crate::Field::new(8, 0, SFR_ELEVEL1);
        pub const SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL5: crate::Field = crate::Field::new(8, 8, SFR_ELEVEL1);
        pub const SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL6: crate::Field = crate::Field::new(8, 16, SFR_ELEVEL1);
        pub const SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL7: crate::Field = crate::Field::new(8, 24, SFR_ELEVEL1);

        pub const SFR_ETYPE: crate::Register = crate::Register::new(14, 0xffffff);
        pub const SFR_ETYPE_PCLK_FIFO_EVENT_LT_MASK: crate::Field = crate::Field::new(8, 0, SFR_ETYPE);
        pub const SFR_ETYPE_PCLK_FIFO_EVENT_EQ_MASK: crate::Field = crate::Field::new(8, 8, SFR_ETYPE);
        pub const SFR_ETYPE_PCLK_FIFO_EVENT_GT_MASK: crate::Field = crate::Field::new(8, 16, SFR_ETYPE);

        pub const SFR_EVENT_SET: crate::Register = crate::Register::new(15, 0xffffff);
        pub const SFR_EVENT_SET_SFR_EVENT_SET: crate::Field = crate::Field::new(24, 0, SFR_EVENT_SET);

        pub const SFR_EVENT_CLR: crate::Register = crate::Register::new(16, 0xffffff);
        pub const SFR_EVENT_CLR_SFR_EVENT_CLR: crate::Field = crate::Field::new(24, 0, SFR_EVENT_CLR);

        pub const SFR_EVENT_STATUS: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const SFR_EVENT_STATUS_SFR_EVENT_STATUS: crate::Field = crate::Field::new(32, 0, SFR_EVENT_STATUS);

        pub const SFR_QDIV0: crate::Register = crate::Register::new(20, 0xffffffff);
        pub const SFR_QDIV0_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_QDIV0);
        pub const SFR_QDIV0_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_QDIV0);
        pub const SFR_QDIV0_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_QDIV0);

        pub const SFR_QDIV1: crate::Register = crate::Register::new(21, 0xffffffff);
        pub const SFR_QDIV1_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_QDIV1);
        pub const SFR_QDIV1_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_QDIV1);
        pub const SFR_QDIV1_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_QDIV1);

        pub const SFR_QDIV2: crate::Register = crate::Register::new(22, 0xffffffff);
        pub const SFR_QDIV2_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_QDIV2);
        pub const SFR_QDIV2_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_QDIV2);
        pub const SFR_QDIV2_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_QDIV2);

        pub const SFR_QDIV3: crate::Register = crate::Register::new(23, 0xffffffff);
        pub const SFR_QDIV3_UNUSED_DIV: crate::Field = crate::Field::new(8, 0, SFR_QDIV3);
        pub const SFR_QDIV3_DIV_FRAC: crate::Field = crate::Field::new(8, 8, SFR_QDIV3);
        pub const SFR_QDIV3_DIV_INT: crate::Field = crate::Field::new(16, 16, SFR_QDIV3);

        pub const SFR_SYNC_BYPASS: crate::Register = crate::Register::new(24, 0xffffffff);
        pub const SFR_SYNC_BYPASS_SFR_SYNC_BYPASS: crate::Field = crate::Field::new(32, 0, SFR_SYNC_BYPASS);

        pub const SFR_IO_OE_INV: crate::Register = crate::Register::new(25, 0xffffffff);
        pub const SFR_IO_OE_INV_SFR_IO_OE_INV: crate::Field = crate::Field::new(32, 0, SFR_IO_OE_INV);

        pub const SFR_IO_O_INV: crate::Register = crate::Register::new(26, 0xffffffff);
        pub const SFR_IO_O_INV_SFR_IO_O_INV: crate::Field = crate::Field::new(32, 0, SFR_IO_O_INV);

        pub const SFR_IO_I_INV: crate::Register = crate::Register::new(27, 0xffffffff);
        pub const SFR_IO_I_INV_SFR_IO_I_INV: crate::Field = crate::Field::new(32, 0, SFR_IO_I_INV);

        pub const SFR_IRQMASK_0: crate::Register = crate::Register::new(28, 0xffffffff);
        pub const SFR_IRQMASK_0_SFR_IRQMASK_0: crate::Field = crate::Field::new(32, 0, SFR_IRQMASK_0);

        pub const SFR_IRQMASK_1: crate::Register = crate::Register::new(29, 0xffffffff);
        pub const SFR_IRQMASK_1_SFR_IRQMASK_1: crate::Field = crate::Field::new(32, 0, SFR_IRQMASK_1);

        pub const SFR_IRQMASK_2: crate::Register = crate::Register::new(30, 0xffffffff);
        pub const SFR_IRQMASK_2_SFR_IRQMASK_2: crate::Field = crate::Field::new(32, 0, SFR_IRQMASK_2);

        pub const SFR_IRQMASK_3: crate::Register = crate::Register::new(31, 0xffffffff);
        pub const SFR_IRQMASK_3_SFR_IRQMASK_3: crate::Field = crate::Field::new(32, 0, SFR_IRQMASK_3);

        pub const SFR_IRQ_EDGE: crate::Register = crate::Register::new(32, 0xf);
        pub const SFR_IRQ_EDGE_SFR_IRQ_EDGE: crate::Field = crate::Field::new(4, 0, SFR_IRQ_EDGE);

        pub const SFR_DBG_PADOUT: crate::Register = crate::Register::new(33, 0xffffffff);
        pub const SFR_DBG_PADOUT_SFR_DBG_PADOUT: crate::Field = crate::Field::new(32, 0, SFR_DBG_PADOUT);

        pub const SFR_DBG_PADOE: crate::Register = crate::Register::new(34, 0xffffffff);
        pub const SFR_DBG_PADOE_SFR_DBG_PADOE: crate::Field = crate::Field::new(32, 0, SFR_DBG_PADOE);

        pub const HW_BIO_BASE: usize = 0x50124000;
    }

    pub mod coresub_sramtrm {
        pub const CORESUB_SRAMTRM_NUMREGS: usize = 7;

        pub const SFR_CACHE: crate::Register = crate::Register::new(0, 0x7);
        pub const SFR_CACHE_SFR_CACHE: crate::Field = crate::Field::new(3, 0, SFR_CACHE);

        pub const SFR_ITCM: crate::Register = crate::Register::new(1, 0x1f);
        pub const SFR_ITCM_SFR_ITCM: crate::Field = crate::Field::new(5, 0, SFR_ITCM);

        pub const SFR_DTCM: crate::Register = crate::Register::new(2, 0x1f);
        pub const SFR_DTCM_SFR_DTCM: crate::Field = crate::Field::new(5, 0, SFR_DTCM);

        pub const SFR_SRAM0: crate::Register = crate::Register::new(3, 0x1f);
        pub const SFR_SRAM0_SFR_SRAM0: crate::Field = crate::Field::new(5, 0, SFR_SRAM0);

        pub const SFR_SRAM1: crate::Register = crate::Register::new(4, 0x1f);
        pub const SFR_SRAM1_SFR_SRAM1: crate::Field = crate::Field::new(5, 0, SFR_SRAM1);

        pub const SFR_VEXRAM: crate::Register = crate::Register::new(5, 0x7);
        pub const SFR_VEXRAM_SFR_VEXRAM: crate::Field = crate::Field::new(3, 0, SFR_VEXRAM);

        pub const SFR_SRAMERR: crate::Register = crate::Register::new(8, 0xf);
        pub const SFR_SRAMERR_SRAMBANKERR: crate::Field = crate::Field::new(4, 0, SFR_SRAMERR);

        pub const HW_CORESUB_SRAMTRM_BASE: usize = 0x40014000;
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

        pub const HW_MDMA_BASE: usize = 0x40012000;
    }

    pub mod qfc {
        pub const QFC_NUMREGS: usize = 14;

        pub const SFR_IO: crate::Register = crate::Register::new(0, 0xff);
        pub const SFR_IO_SFR_IO: crate::Field = crate::Field::new(8, 0, SFR_IO);

        pub const SFR_AR: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_AR_SFR_AR: crate::Field = crate::Field::new(32, 0, SFR_AR);

        pub const SFR_IODRV: crate::Register = crate::Register::new(2, 0xfff);
        pub const SFR_IODRV_PADDRVSEL: crate::Field = crate::Field::new(12, 0, SFR_IODRV);

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

        pub const CR_AESKEY_AESKEYIN0: crate::Register = crate::Register::new(16, 0xffffffff);
        pub const CR_AESKEY_AESKEYIN0_AESKEYIN0: crate::Field = crate::Field::new(32, 0, CR_AESKEY_AESKEYIN0);

        pub const CR_AESKEY_AESKEYIN1: crate::Register = crate::Register::new(17, 0xffffffff);
        pub const CR_AESKEY_AESKEYIN1_AESKEYIN1: crate::Field = crate::Field::new(32, 0, CR_AESKEY_AESKEYIN1);

        pub const CR_AESKEY_AESKEYIN2: crate::Register = crate::Register::new(18, 0xffffffff);
        pub const CR_AESKEY_AESKEYIN2_AESKEYIN2: crate::Field = crate::Field::new(32, 0, CR_AESKEY_AESKEYIN2);

        pub const CR_AESKEY_AESKEYIN3: crate::Register = crate::Register::new(19, 0xffffffff);
        pub const CR_AESKEY_AESKEYIN3_AESKEYIN3: crate::Field = crate::Field::new(32, 0, CR_AESKEY_AESKEYIN3);

        pub const CR_AESENA: crate::Register = crate::Register::new(20, 0x1);
        pub const CR_AESENA_CR_AESENA: crate::Field = crate::Field::new(1, 0, CR_AESENA);

        pub const HW_QFC_BASE: usize = 0x40010000;
    }

    pub mod mbox_apb {
        pub const MBOX_APB_NUMREGS: usize = 5;

        pub const SFR_WDATA: crate::Register = crate::Register::new(0, 0xffffffff);
        pub const SFR_WDATA_SFR_WDATA: crate::Field = crate::Field::new(32, 0, SFR_WDATA);

        pub const SFR_RDATA: crate::Register = crate::Register::new(1, 0xffffffff);
        pub const SFR_RDATA_SFR_RDATA: crate::Field = crate::Field::new(32, 0, SFR_RDATA);

        pub const SFR_STATUS: crate::Register = crate::Register::new(2, 0x3f);
        pub const SFR_STATUS_RX_AVAIL: crate::Field = crate::Field::new(1, 0, SFR_STATUS);
        pub const SFR_STATUS_TX_FREE: crate::Field = crate::Field::new(1, 1, SFR_STATUS);
        pub const SFR_STATUS_ABORT_IN_PROGRESS: crate::Field = crate::Field::new(1, 2, SFR_STATUS);
        pub const SFR_STATUS_ABORT_ACK: crate::Field = crate::Field::new(1, 3, SFR_STATUS);
        pub const SFR_STATUS_TX_ERR: crate::Field = crate::Field::new(1, 4, SFR_STATUS);
        pub const SFR_STATUS_RX_ERR: crate::Field = crate::Field::new(1, 5, SFR_STATUS);

        pub const SFR_ABORT: crate::Register = crate::Register::new(6, 0xffffffff);
        pub const SFR_ABORT_SFR_ABORT: crate::Field = crate::Field::new(32, 0, SFR_ABORT);

        pub const SFR_DONE: crate::Register = crate::Register::new(7, 0xffffffff);
        pub const SFR_DONE_SFR_DONE: crate::Field = crate::Field::new(32, 0, SFR_DONE);

        pub const HW_MBOX_APB_BASE: usize = 0x40013000;
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
}

// Litex auto-generated constants


#[cfg(test)]
mod tests {

    #[test]
    #[ignore]
    fn compile_check_pl230_csr() {
        use super::*;
        let mut pl230_csr = CSR::new(HW_PL230_BASE as *mut u32);

        let foo = pl230_csr.r(utra::pl230::STATUS);
        pl230_csr.wo(utra::pl230::STATUS, foo);
        let bar = pl230_csr.rf(utra::pl230::STATUS_TEST_STATUS);
        pl230_csr.rmwf(utra::pl230::STATUS_TEST_STATUS, bar);
        let mut baz = pl230_csr.zf(utra::pl230::STATUS_TEST_STATUS, bar);
        baz |= pl230_csr.ms(utra::pl230::STATUS_TEST_STATUS, 1);
        pl230_csr.wfo(utra::pl230::STATUS_TEST_STATUS, baz);
        let bar = pl230_csr.rf(utra::pl230::STATUS_CHNLS_MINUS1);
        pl230_csr.rmwf(utra::pl230::STATUS_CHNLS_MINUS1, bar);
        let mut baz = pl230_csr.zf(utra::pl230::STATUS_CHNLS_MINUS1, bar);
        baz |= pl230_csr.ms(utra::pl230::STATUS_CHNLS_MINUS1, 1);
        pl230_csr.wfo(utra::pl230::STATUS_CHNLS_MINUS1, baz);
        let bar = pl230_csr.rf(utra::pl230::STATUS_STATE);
        pl230_csr.rmwf(utra::pl230::STATUS_STATE, bar);
        let mut baz = pl230_csr.zf(utra::pl230::STATUS_STATE, bar);
        baz |= pl230_csr.ms(utra::pl230::STATUS_STATE, 1);
        pl230_csr.wfo(utra::pl230::STATUS_STATE, baz);
        let bar = pl230_csr.rf(utra::pl230::STATUS_MASTER_ENABLE);
        pl230_csr.rmwf(utra::pl230::STATUS_MASTER_ENABLE, bar);
        let mut baz = pl230_csr.zf(utra::pl230::STATUS_MASTER_ENABLE, bar);
        baz |= pl230_csr.ms(utra::pl230::STATUS_MASTER_ENABLE, 1);
        pl230_csr.wfo(utra::pl230::STATUS_MASTER_ENABLE, baz);

        let foo = pl230_csr.r(utra::pl230::CFG);
        pl230_csr.wo(utra::pl230::CFG, foo);
        let bar = pl230_csr.rf(utra::pl230::CFG_CHNL_PROT_CTRL);
        pl230_csr.rmwf(utra::pl230::CFG_CHNL_PROT_CTRL, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CFG_CHNL_PROT_CTRL, bar);
        baz |= pl230_csr.ms(utra::pl230::CFG_CHNL_PROT_CTRL, 1);
        pl230_csr.wfo(utra::pl230::CFG_CHNL_PROT_CTRL, baz);
        let bar = pl230_csr.rf(utra::pl230::CFG_MASTER_ENABLE);
        pl230_csr.rmwf(utra::pl230::CFG_MASTER_ENABLE, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CFG_MASTER_ENABLE, bar);
        baz |= pl230_csr.ms(utra::pl230::CFG_MASTER_ENABLE, 1);
        pl230_csr.wfo(utra::pl230::CFG_MASTER_ENABLE, baz);

        let foo = pl230_csr.r(utra::pl230::CTRLBASEPTR);
        pl230_csr.wo(utra::pl230::CTRLBASEPTR, foo);
        let bar = pl230_csr.rf(utra::pl230::CTRLBASEPTR_CTRL_BASE_PTR);
        pl230_csr.rmwf(utra::pl230::CTRLBASEPTR_CTRL_BASE_PTR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CTRLBASEPTR_CTRL_BASE_PTR, bar);
        baz |= pl230_csr.ms(utra::pl230::CTRLBASEPTR_CTRL_BASE_PTR, 1);
        pl230_csr.wfo(utra::pl230::CTRLBASEPTR_CTRL_BASE_PTR, baz);

        let foo = pl230_csr.r(utra::pl230::ALTCTRLBASEPTR);
        pl230_csr.wo(utra::pl230::ALTCTRLBASEPTR, foo);
        let bar = pl230_csr.rf(utra::pl230::ALTCTRLBASEPTR_ALT_CTRL_BASE_PTR);
        pl230_csr.rmwf(utra::pl230::ALTCTRLBASEPTR_ALT_CTRL_BASE_PTR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::ALTCTRLBASEPTR_ALT_CTRL_BASE_PTR, bar);
        baz |= pl230_csr.ms(utra::pl230::ALTCTRLBASEPTR_ALT_CTRL_BASE_PTR, 1);
        pl230_csr.wfo(utra::pl230::ALTCTRLBASEPTR_ALT_CTRL_BASE_PTR, baz);

        let foo = pl230_csr.r(utra::pl230::DMA_WAITONREQ_STATUS);
        pl230_csr.wo(utra::pl230::DMA_WAITONREQ_STATUS, foo);
        let bar = pl230_csr.rf(utra::pl230::DMA_WAITONREQ_STATUS_DMA_WAITONREQ_STATUS);
        pl230_csr.rmwf(utra::pl230::DMA_WAITONREQ_STATUS_DMA_WAITONREQ_STATUS, bar);
        let mut baz = pl230_csr.zf(utra::pl230::DMA_WAITONREQ_STATUS_DMA_WAITONREQ_STATUS, bar);
        baz |= pl230_csr.ms(utra::pl230::DMA_WAITONREQ_STATUS_DMA_WAITONREQ_STATUS, 1);
        pl230_csr.wfo(utra::pl230::DMA_WAITONREQ_STATUS_DMA_WAITONREQ_STATUS, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLSWREQUEST);
        pl230_csr.wo(utra::pl230::CHNLSWREQUEST, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLSWREQUEST_CHNL_SW_REQUEST);
        pl230_csr.rmwf(utra::pl230::CHNLSWREQUEST_CHNL_SW_REQUEST, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLSWREQUEST_CHNL_SW_REQUEST, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLSWREQUEST_CHNL_SW_REQUEST, 1);
        pl230_csr.wfo(utra::pl230::CHNLSWREQUEST_CHNL_SW_REQUEST, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLUSEBURSTSET);
        pl230_csr.wo(utra::pl230::CHNLUSEBURSTSET, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLUSEBURSTSET_CHNL_USEBURST_SET);
        pl230_csr.rmwf(utra::pl230::CHNLUSEBURSTSET_CHNL_USEBURST_SET, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLUSEBURSTSET_CHNL_USEBURST_SET, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLUSEBURSTSET_CHNL_USEBURST_SET, 1);
        pl230_csr.wfo(utra::pl230::CHNLUSEBURSTSET_CHNL_USEBURST_SET, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLUSEBURSTCLR);
        pl230_csr.wo(utra::pl230::CHNLUSEBURSTCLR, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLUSEBURSTCLR_CHNL_USEBURST_CLR);
        pl230_csr.rmwf(utra::pl230::CHNLUSEBURSTCLR_CHNL_USEBURST_CLR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLUSEBURSTCLR_CHNL_USEBURST_CLR, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLUSEBURSTCLR_CHNL_USEBURST_CLR, 1);
        pl230_csr.wfo(utra::pl230::CHNLUSEBURSTCLR_CHNL_USEBURST_CLR, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLREQMASKSET);
        pl230_csr.wo(utra::pl230::CHNLREQMASKSET, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLREQMASKSET_CHNL_REQ_MASK_SET);
        pl230_csr.rmwf(utra::pl230::CHNLREQMASKSET_CHNL_REQ_MASK_SET, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLREQMASKSET_CHNL_REQ_MASK_SET, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLREQMASKSET_CHNL_REQ_MASK_SET, 1);
        pl230_csr.wfo(utra::pl230::CHNLREQMASKSET_CHNL_REQ_MASK_SET, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLREQMASKCLR);
        pl230_csr.wo(utra::pl230::CHNLREQMASKCLR, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLREQMASKCLR_CHNL_REQ_MASK_CLR);
        pl230_csr.rmwf(utra::pl230::CHNLREQMASKCLR_CHNL_REQ_MASK_CLR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLREQMASKCLR_CHNL_REQ_MASK_CLR, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLREQMASKCLR_CHNL_REQ_MASK_CLR, 1);
        pl230_csr.wfo(utra::pl230::CHNLREQMASKCLR_CHNL_REQ_MASK_CLR, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLENABLESET);
        pl230_csr.wo(utra::pl230::CHNLENABLESET, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLENABLESET_CHNL_ENABLE_SET);
        pl230_csr.rmwf(utra::pl230::CHNLENABLESET_CHNL_ENABLE_SET, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLENABLESET_CHNL_ENABLE_SET, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLENABLESET_CHNL_ENABLE_SET, 1);
        pl230_csr.wfo(utra::pl230::CHNLENABLESET_CHNL_ENABLE_SET, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLENABLECLR);
        pl230_csr.wo(utra::pl230::CHNLENABLECLR, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLENABLECLR_CHNL_ENABLE_CLR);
        pl230_csr.rmwf(utra::pl230::CHNLENABLECLR_CHNL_ENABLE_CLR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLENABLECLR_CHNL_ENABLE_CLR, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLENABLECLR_CHNL_ENABLE_CLR, 1);
        pl230_csr.wfo(utra::pl230::CHNLENABLECLR_CHNL_ENABLE_CLR, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLPRIALTSET);
        pl230_csr.wo(utra::pl230::CHNLPRIALTSET, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLPRIALTSET_CHNL_PRI_ALT_SET);
        pl230_csr.rmwf(utra::pl230::CHNLPRIALTSET_CHNL_PRI_ALT_SET, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLPRIALTSET_CHNL_PRI_ALT_SET, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLPRIALTSET_CHNL_PRI_ALT_SET, 1);
        pl230_csr.wfo(utra::pl230::CHNLPRIALTSET_CHNL_PRI_ALT_SET, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLPRIALTCLR);
        pl230_csr.wo(utra::pl230::CHNLPRIALTCLR, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLPRIALTCLR_CHNL_PRI_ALT_CLR);
        pl230_csr.rmwf(utra::pl230::CHNLPRIALTCLR_CHNL_PRI_ALT_CLR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLPRIALTCLR_CHNL_PRI_ALT_CLR, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLPRIALTCLR_CHNL_PRI_ALT_CLR, 1);
        pl230_csr.wfo(utra::pl230::CHNLPRIALTCLR_CHNL_PRI_ALT_CLR, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLPRIORITYSET);
        pl230_csr.wo(utra::pl230::CHNLPRIORITYSET, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLPRIORITYSET_CHNL_PRIORITY_SET);
        pl230_csr.rmwf(utra::pl230::CHNLPRIORITYSET_CHNL_PRIORITY_SET, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLPRIORITYSET_CHNL_PRIORITY_SET, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLPRIORITYSET_CHNL_PRIORITY_SET, 1);
        pl230_csr.wfo(utra::pl230::CHNLPRIORITYSET_CHNL_PRIORITY_SET, baz);

        let foo = pl230_csr.r(utra::pl230::CHNLPRIORITYCLR);
        pl230_csr.wo(utra::pl230::CHNLPRIORITYCLR, foo);
        let bar = pl230_csr.rf(utra::pl230::CHNLPRIORITYCLR_CHNL_PRIORITY_CLR);
        pl230_csr.rmwf(utra::pl230::CHNLPRIORITYCLR_CHNL_PRIORITY_CLR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::CHNLPRIORITYCLR_CHNL_PRIORITY_CLR, bar);
        baz |= pl230_csr.ms(utra::pl230::CHNLPRIORITYCLR_CHNL_PRIORITY_CLR, 1);
        pl230_csr.wfo(utra::pl230::CHNLPRIORITYCLR_CHNL_PRIORITY_CLR, baz);

        let foo = pl230_csr.r(utra::pl230::ERRCLR);
        pl230_csr.wo(utra::pl230::ERRCLR, foo);
        let bar = pl230_csr.rf(utra::pl230::ERRCLR_ERR_CLR);
        pl230_csr.rmwf(utra::pl230::ERRCLR_ERR_CLR, bar);
        let mut baz = pl230_csr.zf(utra::pl230::ERRCLR_ERR_CLR, bar);
        baz |= pl230_csr.ms(utra::pl230::ERRCLR_ERR_CLR, 1);
        pl230_csr.wfo(utra::pl230::ERRCLR_ERR_CLR, baz);

        let foo = pl230_csr.r(utra::pl230::PERIPH_ID_0);
        pl230_csr.wo(utra::pl230::PERIPH_ID_0, foo);
        let bar = pl230_csr.rf(utra::pl230::PERIPH_ID_0_PART_NUMBER_LSB);
        pl230_csr.rmwf(utra::pl230::PERIPH_ID_0_PART_NUMBER_LSB, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PERIPH_ID_0_PART_NUMBER_LSB, bar);
        baz |= pl230_csr.ms(utra::pl230::PERIPH_ID_0_PART_NUMBER_LSB, 1);
        pl230_csr.wfo(utra::pl230::PERIPH_ID_0_PART_NUMBER_LSB, baz);

        let foo = pl230_csr.r(utra::pl230::PERIPH_ID_1);
        pl230_csr.wo(utra::pl230::PERIPH_ID_1, foo);
        let bar = pl230_csr.rf(utra::pl230::PERIPH_ID_1_PART_NUMBER_MSB);
        pl230_csr.rmwf(utra::pl230::PERIPH_ID_1_PART_NUMBER_MSB, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PERIPH_ID_1_PART_NUMBER_MSB, bar);
        baz |= pl230_csr.ms(utra::pl230::PERIPH_ID_1_PART_NUMBER_MSB, 1);
        pl230_csr.wfo(utra::pl230::PERIPH_ID_1_PART_NUMBER_MSB, baz);
        let bar = pl230_csr.rf(utra::pl230::PERIPH_ID_1_JEP106_LSB);
        pl230_csr.rmwf(utra::pl230::PERIPH_ID_1_JEP106_LSB, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PERIPH_ID_1_JEP106_LSB, bar);
        baz |= pl230_csr.ms(utra::pl230::PERIPH_ID_1_JEP106_LSB, 1);
        pl230_csr.wfo(utra::pl230::PERIPH_ID_1_JEP106_LSB, baz);

        let foo = pl230_csr.r(utra::pl230::PERIPH_ID_2);
        pl230_csr.wo(utra::pl230::PERIPH_ID_2, foo);
        let bar = pl230_csr.rf(utra::pl230::PERIPH_ID_2_JEP106_MSB);
        pl230_csr.rmwf(utra::pl230::PERIPH_ID_2_JEP106_MSB, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PERIPH_ID_2_JEP106_MSB, bar);
        baz |= pl230_csr.ms(utra::pl230::PERIPH_ID_2_JEP106_MSB, 1);
        pl230_csr.wfo(utra::pl230::PERIPH_ID_2_JEP106_MSB, baz);
        let bar = pl230_csr.rf(utra::pl230::PERIPH_ID_2_JEDEC_USED);
        pl230_csr.rmwf(utra::pl230::PERIPH_ID_2_JEDEC_USED, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PERIPH_ID_2_JEDEC_USED, bar);
        baz |= pl230_csr.ms(utra::pl230::PERIPH_ID_2_JEDEC_USED, 1);
        pl230_csr.wfo(utra::pl230::PERIPH_ID_2_JEDEC_USED, baz);
        let bar = pl230_csr.rf(utra::pl230::PERIPH_ID_2_REVISION);
        pl230_csr.rmwf(utra::pl230::PERIPH_ID_2_REVISION, bar);
        let mut baz = pl230_csr.zf(utra::pl230::PERIPH_ID_2_REVISION, bar);
        baz |= pl230_csr.ms(utra::pl230::PERIPH_ID_2_REVISION, 1);
        pl230_csr.wfo(utra::pl230::PERIPH_ID_2_REVISION, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_ctrl_csr() {
        use super::*;
        let mut udma_ctrl_csr = CSR::new(HW_UDMA_CTRL_BASE as *mut u32);

        let foo = udma_ctrl_csr.r(utra::udma_ctrl::REG_CG);
        udma_ctrl_csr.wo(utra::udma_ctrl::REG_CG, foo);
        let bar = udma_ctrl_csr.rf(utra::udma_ctrl::REG_CG_R_CG);
        udma_ctrl_csr.rmwf(utra::udma_ctrl::REG_CG_R_CG, bar);
        let mut baz = udma_ctrl_csr.zf(utra::udma_ctrl::REG_CG_R_CG, bar);
        baz |= udma_ctrl_csr.ms(utra::udma_ctrl::REG_CG_R_CG, 1);
        udma_ctrl_csr.wfo(utra::udma_ctrl::REG_CG_R_CG, baz);

        let foo = udma_ctrl_csr.r(utra::udma_ctrl::REG_CFG_EVT);
        udma_ctrl_csr.wo(utra::udma_ctrl::REG_CFG_EVT, foo);
        let bar = udma_ctrl_csr.rf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_0);
        udma_ctrl_csr.rmwf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_0, bar);
        let mut baz = udma_ctrl_csr.zf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_0, bar);
        baz |= udma_ctrl_csr.ms(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_0, 1);
        udma_ctrl_csr.wfo(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_0, baz);
        let bar = udma_ctrl_csr.rf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_1);
        udma_ctrl_csr.rmwf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_1, bar);
        let mut baz = udma_ctrl_csr.zf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_1, bar);
        baz |= udma_ctrl_csr.ms(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_1, 1);
        udma_ctrl_csr.wfo(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_1, baz);
        let bar = udma_ctrl_csr.rf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_2);
        udma_ctrl_csr.rmwf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_2, bar);
        let mut baz = udma_ctrl_csr.zf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_2, bar);
        baz |= udma_ctrl_csr.ms(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_2, 1);
        udma_ctrl_csr.wfo(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_2, baz);
        let bar = udma_ctrl_csr.rf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_3);
        udma_ctrl_csr.rmwf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_3, bar);
        let mut baz = udma_ctrl_csr.zf(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_3, bar);
        baz |= udma_ctrl_csr.ms(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_3, 1);
        udma_ctrl_csr.wfo(utra::udma_ctrl::REG_CFG_EVT_R_CMP_EVT_3, baz);

        let foo = udma_ctrl_csr.r(utra::udma_ctrl::REG_RST);
        udma_ctrl_csr.wo(utra::udma_ctrl::REG_RST, foo);
        let bar = udma_ctrl_csr.rf(utra::udma_ctrl::REG_RST_R_RST);
        udma_ctrl_csr.rmwf(utra::udma_ctrl::REG_RST_R_RST, bar);
        let mut baz = udma_ctrl_csr.zf(utra::udma_ctrl::REG_RST_R_RST, bar);
        baz |= udma_ctrl_csr.ms(utra::udma_ctrl::REG_RST_R_RST, 1);
        udma_ctrl_csr.wfo(utra::udma_ctrl::REG_RST_R_RST, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_uart_0_csr() {
        use super::*;
        let mut udma_uart_0_csr = CSR::new(HW_UDMA_UART_0_BASE as *mut u32);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_RX_SADDR);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_RX_SADDR, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_RX_SADDR_R_RX_STARTADDR);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_RX_SIZE);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_RX_SIZE, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_RX_SIZE_R_RX_SIZE);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_RX_CFG);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_RX_CFG, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_RX_CFG_R_RX_EN);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_RX_CFG_R_RX_EN, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_RX_CFG_R_RX_CLR);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_RX_CFG_R_RX_CLR, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_TX_SADDR);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_TX_SADDR, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_TX_SADDR_R_TX_STARTADDR);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_TX_SIZE);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_TX_SIZE, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_TX_SIZE_R_TX_SIZE);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_TX_CFG);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_TX_CFG, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_TX_CFG_R_TX_EN);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_TX_CFG_R_TX_EN, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_TX_CFG_R_TX_CLR);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_TX_CFG_R_TX_CLR, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_STATUS);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_STATUS, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_STATUS_STATUS_I);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_STATUS_STATUS_I, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_STATUS_STATUS_I, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_STATUS_STATUS_I, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_UART_SETUP);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_UART_SETUP, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_PARITY_EN);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_PARITY_EN, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_PARITY_EN, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_BITS);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_BITS, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_BITS, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_BITS, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_BITS, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_STOP_BITS);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_STOP_BITS, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_STOP_BITS, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_POLLING_EN);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_POLLING_EN, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_POLLING_EN, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_TX);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_TX, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_TX, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_TX, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_TX, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_RX);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_RX, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_RX, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_RX, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_EN_RX, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_UART_SETUP_R_UART_DIV);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_UART_SETUP_R_UART_DIV, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_UART_SETUP_R_UART_DIV, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_UART_SETUP_R_UART_DIV, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_UART_SETUP_R_UART_DIV, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_ERROR);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_ERROR, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_ERROR_R_ERR_OVERFLOW);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_ERROR_R_ERR_OVERFLOW, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_ERROR_R_ERR_OVERFLOW, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_ERROR_R_ERR_OVERFLOW, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_ERROR_R_ERR_OVERFLOW, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_ERROR_R_ERR_PARITY);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_ERROR_R_ERR_PARITY, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_ERROR_R_ERR_PARITY, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_ERROR_R_ERR_PARITY, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_ERROR_R_ERR_PARITY, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_IRQ_EN);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_IRQ_EN, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_IRQ_EN_R_UART_RX_IRQ_EN);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_IRQ_EN_R_UART_RX_IRQ_EN, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_IRQ_EN_R_UART_RX_IRQ_EN, baz);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_IRQ_EN_R_UART_ERR_IRQ_EN);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_IRQ_EN_R_UART_ERR_IRQ_EN, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_IRQ_EN_R_UART_ERR_IRQ_EN, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_VALID);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_VALID, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_VALID_R_UART_RX_DATA_VALID);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_VALID_R_UART_RX_DATA_VALID, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_VALID_R_UART_RX_DATA_VALID, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_VALID_R_UART_RX_DATA_VALID, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_VALID_R_UART_RX_DATA_VALID, baz);

        let foo = udma_uart_0_csr.r(utra::udma_uart_0::REG_DATA);
        udma_uart_0_csr.wo(utra::udma_uart_0::REG_DATA, foo);
        let bar = udma_uart_0_csr.rf(utra::udma_uart_0::REG_DATA_R_UART_RX_DATA);
        udma_uart_0_csr.rmwf(utra::udma_uart_0::REG_DATA_R_UART_RX_DATA, bar);
        let mut baz = udma_uart_0_csr.zf(utra::udma_uart_0::REG_DATA_R_UART_RX_DATA, bar);
        baz |= udma_uart_0_csr.ms(utra::udma_uart_0::REG_DATA_R_UART_RX_DATA, 1);
        udma_uart_0_csr.wfo(utra::udma_uart_0::REG_DATA_R_UART_RX_DATA, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_uart_1_csr() {
        use super::*;
        let mut udma_uart_1_csr = CSR::new(HW_UDMA_UART_1_BASE as *mut u32);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_RX_SADDR);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_RX_SADDR, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_RX_SADDR_R_RX_STARTADDR);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_RX_SIZE);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_RX_SIZE, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_RX_SIZE_R_RX_SIZE);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_RX_CFG);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_RX_CFG, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_RX_CFG_R_RX_EN);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_RX_CFG_R_RX_EN, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_RX_CFG_R_RX_CLR);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_RX_CFG_R_RX_CLR, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_TX_SADDR);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_TX_SADDR, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_TX_SADDR_R_TX_STARTADDR);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_TX_SIZE);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_TX_SIZE, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_TX_SIZE_R_TX_SIZE);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_TX_CFG);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_TX_CFG, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_TX_CFG_R_TX_EN);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_TX_CFG_R_TX_EN, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_TX_CFG_R_TX_CLR);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_TX_CFG_R_TX_CLR, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_STATUS);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_STATUS, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_STATUS_STATUS_I);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_STATUS_STATUS_I, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_STATUS_STATUS_I, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_STATUS_STATUS_I, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_UART_SETUP);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_UART_SETUP, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_PARITY_EN);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_PARITY_EN, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_PARITY_EN, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_BITS);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_BITS, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_BITS, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_BITS, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_BITS, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_STOP_BITS);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_STOP_BITS, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_STOP_BITS, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_POLLING_EN);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_POLLING_EN, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_POLLING_EN, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_TX);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_TX, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_TX, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_TX, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_TX, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_RX);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_RX, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_RX, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_RX, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_EN_RX, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_UART_SETUP_R_UART_DIV);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_UART_SETUP_R_UART_DIV, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_UART_SETUP_R_UART_DIV, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_UART_SETUP_R_UART_DIV, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_UART_SETUP_R_UART_DIV, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_ERROR);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_ERROR, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_ERROR_R_ERR_OVERFLOW);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_ERROR_R_ERR_OVERFLOW, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_ERROR_R_ERR_OVERFLOW, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_ERROR_R_ERR_OVERFLOW, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_ERROR_R_ERR_OVERFLOW, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_ERROR_R_ERR_PARITY);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_ERROR_R_ERR_PARITY, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_ERROR_R_ERR_PARITY, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_ERROR_R_ERR_PARITY, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_ERROR_R_ERR_PARITY, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_IRQ_EN);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_IRQ_EN, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_IRQ_EN_R_UART_RX_IRQ_EN);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_IRQ_EN_R_UART_RX_IRQ_EN, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_IRQ_EN_R_UART_RX_IRQ_EN, baz);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_IRQ_EN_R_UART_ERR_IRQ_EN);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_IRQ_EN_R_UART_ERR_IRQ_EN, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_IRQ_EN_R_UART_ERR_IRQ_EN, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_VALID);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_VALID, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_VALID_R_UART_RX_DATA_VALID);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_VALID_R_UART_RX_DATA_VALID, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_VALID_R_UART_RX_DATA_VALID, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_VALID_R_UART_RX_DATA_VALID, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_VALID_R_UART_RX_DATA_VALID, baz);

        let foo = udma_uart_1_csr.r(utra::udma_uart_1::REG_DATA);
        udma_uart_1_csr.wo(utra::udma_uart_1::REG_DATA, foo);
        let bar = udma_uart_1_csr.rf(utra::udma_uart_1::REG_DATA_R_UART_RX_DATA);
        udma_uart_1_csr.rmwf(utra::udma_uart_1::REG_DATA_R_UART_RX_DATA, bar);
        let mut baz = udma_uart_1_csr.zf(utra::udma_uart_1::REG_DATA_R_UART_RX_DATA, bar);
        baz |= udma_uart_1_csr.ms(utra::udma_uart_1::REG_DATA_R_UART_RX_DATA, 1);
        udma_uart_1_csr.wfo(utra::udma_uart_1::REG_DATA_R_UART_RX_DATA, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_uart_2_csr() {
        use super::*;
        let mut udma_uart_2_csr = CSR::new(HW_UDMA_UART_2_BASE as *mut u32);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_RX_SADDR);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_RX_SADDR, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_RX_SADDR_R_RX_STARTADDR);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_RX_SIZE);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_RX_SIZE, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_RX_SIZE_R_RX_SIZE);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_RX_CFG);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_RX_CFG, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_RX_CFG_R_RX_EN);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_RX_CFG_R_RX_EN, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_RX_CFG_R_RX_CLR);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_RX_CFG_R_RX_CLR, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_TX_SADDR);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_TX_SADDR, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_TX_SADDR_R_TX_STARTADDR);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_TX_SIZE);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_TX_SIZE, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_TX_SIZE_R_TX_SIZE);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_TX_CFG);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_TX_CFG, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_TX_CFG_R_TX_EN);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_TX_CFG_R_TX_EN, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_TX_CFG_R_TX_CLR);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_TX_CFG_R_TX_CLR, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_STATUS);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_STATUS, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_STATUS_STATUS_I);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_STATUS_STATUS_I, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_STATUS_STATUS_I, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_STATUS_STATUS_I, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_UART_SETUP);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_UART_SETUP, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_PARITY_EN);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_PARITY_EN, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_PARITY_EN, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_BITS);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_BITS, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_BITS, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_BITS, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_BITS, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_STOP_BITS);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_STOP_BITS, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_STOP_BITS, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_POLLING_EN);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_POLLING_EN, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_POLLING_EN, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_TX);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_TX, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_TX, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_TX, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_TX, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_RX);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_RX, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_RX, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_RX, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_EN_RX, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_UART_SETUP_R_UART_DIV);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_UART_SETUP_R_UART_DIV, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_UART_SETUP_R_UART_DIV, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_UART_SETUP_R_UART_DIV, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_UART_SETUP_R_UART_DIV, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_ERROR);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_ERROR, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_ERROR_R_ERR_OVERFLOW);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_ERROR_R_ERR_OVERFLOW, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_ERROR_R_ERR_OVERFLOW, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_ERROR_R_ERR_OVERFLOW, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_ERROR_R_ERR_OVERFLOW, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_ERROR_R_ERR_PARITY);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_ERROR_R_ERR_PARITY, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_ERROR_R_ERR_PARITY, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_ERROR_R_ERR_PARITY, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_ERROR_R_ERR_PARITY, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_IRQ_EN);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_IRQ_EN, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_IRQ_EN_R_UART_RX_IRQ_EN);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_IRQ_EN_R_UART_RX_IRQ_EN, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_IRQ_EN_R_UART_RX_IRQ_EN, baz);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_IRQ_EN_R_UART_ERR_IRQ_EN);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_IRQ_EN_R_UART_ERR_IRQ_EN, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_IRQ_EN_R_UART_ERR_IRQ_EN, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_VALID);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_VALID, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_VALID_R_UART_RX_DATA_VALID);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_VALID_R_UART_RX_DATA_VALID, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_VALID_R_UART_RX_DATA_VALID, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_VALID_R_UART_RX_DATA_VALID, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_VALID_R_UART_RX_DATA_VALID, baz);

        let foo = udma_uart_2_csr.r(utra::udma_uart_2::REG_DATA);
        udma_uart_2_csr.wo(utra::udma_uart_2::REG_DATA, foo);
        let bar = udma_uart_2_csr.rf(utra::udma_uart_2::REG_DATA_R_UART_RX_DATA);
        udma_uart_2_csr.rmwf(utra::udma_uart_2::REG_DATA_R_UART_RX_DATA, bar);
        let mut baz = udma_uart_2_csr.zf(utra::udma_uart_2::REG_DATA_R_UART_RX_DATA, bar);
        baz |= udma_uart_2_csr.ms(utra::udma_uart_2::REG_DATA_R_UART_RX_DATA, 1);
        udma_uart_2_csr.wfo(utra::udma_uart_2::REG_DATA_R_UART_RX_DATA, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_uart_3_csr() {
        use super::*;
        let mut udma_uart_3_csr = CSR::new(HW_UDMA_UART_3_BASE as *mut u32);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_RX_SADDR);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_RX_SADDR, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_RX_SADDR_R_RX_STARTADDR);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_RX_SIZE);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_RX_SIZE, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_RX_SIZE_R_RX_SIZE);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_RX_CFG);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_RX_CFG, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_RX_CFG_R_RX_EN);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_RX_CFG_R_RX_EN, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_RX_CFG_R_RX_CLR);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_RX_CFG_R_RX_CLR, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_TX_SADDR);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_TX_SADDR, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_TX_SADDR_R_TX_STARTADDR);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_TX_SIZE);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_TX_SIZE, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_TX_SIZE_R_TX_SIZE);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_TX_CFG);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_TX_CFG, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_TX_CFG_R_TX_EN);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_TX_CFG_R_TX_EN, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_TX_CFG_R_TX_CLR);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_TX_CFG_R_TX_CLR, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_STATUS);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_STATUS, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_STATUS_STATUS_I);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_STATUS_STATUS_I, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_STATUS_STATUS_I, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_STATUS_STATUS_I, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_UART_SETUP);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_UART_SETUP, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_PARITY_EN);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_PARITY_EN, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_PARITY_EN, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_PARITY_EN, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_BITS);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_BITS, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_BITS, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_BITS, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_BITS, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_STOP_BITS);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_STOP_BITS, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_STOP_BITS, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_STOP_BITS, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_POLLING_EN);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_POLLING_EN, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_POLLING_EN, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_POLLING_EN, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_RX_CLEAN_FIFO, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_TX);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_TX, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_TX, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_TX, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_TX, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_RX);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_RX, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_RX, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_RX, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_EN_RX, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_UART_SETUP_R_UART_DIV);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_UART_SETUP_R_UART_DIV, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_UART_SETUP_R_UART_DIV, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_UART_SETUP_R_UART_DIV, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_UART_SETUP_R_UART_DIV, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_ERROR);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_ERROR, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_ERROR_R_ERR_OVERFLOW);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_ERROR_R_ERR_OVERFLOW, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_ERROR_R_ERR_OVERFLOW, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_ERROR_R_ERR_OVERFLOW, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_ERROR_R_ERR_OVERFLOW, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_ERROR_R_ERR_PARITY);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_ERROR_R_ERR_PARITY, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_ERROR_R_ERR_PARITY, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_ERROR_R_ERR_PARITY, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_ERROR_R_ERR_PARITY, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_IRQ_EN);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_IRQ_EN, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_IRQ_EN_R_UART_RX_IRQ_EN);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_IRQ_EN_R_UART_RX_IRQ_EN, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_IRQ_EN_R_UART_RX_IRQ_EN, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_IRQ_EN_R_UART_RX_IRQ_EN, baz);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_IRQ_EN_R_UART_ERR_IRQ_EN);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_IRQ_EN_R_UART_ERR_IRQ_EN, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_IRQ_EN_R_UART_ERR_IRQ_EN, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_IRQ_EN_R_UART_ERR_IRQ_EN, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_VALID);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_VALID, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_VALID_R_UART_RX_DATA_VALID);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_VALID_R_UART_RX_DATA_VALID, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_VALID_R_UART_RX_DATA_VALID, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_VALID_R_UART_RX_DATA_VALID, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_VALID_R_UART_RX_DATA_VALID, baz);

        let foo = udma_uart_3_csr.r(utra::udma_uart_3::REG_DATA);
        udma_uart_3_csr.wo(utra::udma_uart_3::REG_DATA, foo);
        let bar = udma_uart_3_csr.rf(utra::udma_uart_3::REG_DATA_R_UART_RX_DATA);
        udma_uart_3_csr.rmwf(utra::udma_uart_3::REG_DATA_R_UART_RX_DATA, bar);
        let mut baz = udma_uart_3_csr.zf(utra::udma_uart_3::REG_DATA_R_UART_RX_DATA, bar);
        baz |= udma_uart_3_csr.ms(utra::udma_uart_3::REG_DATA_R_UART_RX_DATA, 1);
        udma_uart_3_csr.wfo(utra::udma_uart_3::REG_DATA_R_UART_RX_DATA, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_spim_0_csr() {
        use super::*;
        let mut udma_spim_0_csr = CSR::new(HW_UDMA_SPIM_0_BASE as *mut u32);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_RX_SADDR);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_RX_SADDR, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_RX_SADDR_R_RX_STARTADDR);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_RX_SIZE);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_RX_SIZE, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_RX_SIZE_R_RX_SIZE);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_RX_CFG);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_RX_CFG, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_RX_CFG_R_RX_DATASIZE);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_RX_CFG_R_RX_DATASIZE, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_RX_CFG_R_RX_DATASIZE, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_RX_CFG_R_RX_DATASIZE, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_RX_CFG_R_RX_DATASIZE, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_RX_CFG_R_RX_EN);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_RX_CFG_R_RX_EN, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_RX_CFG_R_RX_CLR);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_RX_CFG_R_RX_CLR, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_TX_SADDR);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_TX_SADDR, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_TX_SADDR_R_TX_STARTADDR);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_TX_SIZE);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_TX_SIZE, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_TX_SIZE_R_TX_SIZE);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_TX_CFG);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_TX_CFG, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_TX_CFG_R_TX_DATASIZE);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_TX_CFG_R_TX_DATASIZE, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_TX_CFG_R_TX_DATASIZE, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_TX_CFG_R_TX_DATASIZE, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_TX_CFG_R_TX_DATASIZE, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_TX_CFG_R_TX_EN);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_TX_CFG_R_TX_EN, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_TX_CFG_R_TX_CLR);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_TX_CFG_R_TX_CLR, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_CMD_SADDR);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_CMD_SADDR, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_CMD_SIZE);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_CMD_SIZE, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_CMD_SIZE_R_CMD_SIZE);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_CMD_CFG);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_CMD_CFG, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_EN);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_CMD_CFG_R_CMD_EN, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CLR);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_spim_0_csr.r(utra::udma_spim_0::REG_STATUS);
        udma_spim_0_csr.wo(utra::udma_spim_0::REG_STATUS, foo);
        let bar = udma_spim_0_csr.rf(utra::udma_spim_0::REG_STATUS_STATUS_I);
        udma_spim_0_csr.rmwf(utra::udma_spim_0::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_spim_0_csr.zf(utra::udma_spim_0::REG_STATUS_STATUS_I, bar);
        baz |= udma_spim_0_csr.ms(utra::udma_spim_0::REG_STATUS_STATUS_I, 1);
        udma_spim_0_csr.wfo(utra::udma_spim_0::REG_STATUS_STATUS_I, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_spim_1_csr() {
        use super::*;
        let mut udma_spim_1_csr = CSR::new(HW_UDMA_SPIM_1_BASE as *mut u32);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_RX_SADDR);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_RX_SADDR, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_RX_SADDR_R_RX_STARTADDR);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_RX_SIZE);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_RX_SIZE, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_RX_SIZE_R_RX_SIZE);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_RX_CFG);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_RX_CFG, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_RX_CFG_R_RX_DATASIZE);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_RX_CFG_R_RX_DATASIZE, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_RX_CFG_R_RX_DATASIZE, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_RX_CFG_R_RX_DATASIZE, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_RX_CFG_R_RX_DATASIZE, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_RX_CFG_R_RX_EN);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_RX_CFG_R_RX_EN, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_RX_CFG_R_RX_CLR);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_RX_CFG_R_RX_CLR, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_TX_SADDR);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_TX_SADDR, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_TX_SADDR_R_TX_STARTADDR);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_TX_SIZE);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_TX_SIZE, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_TX_SIZE_R_TX_SIZE);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_TX_CFG);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_TX_CFG, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_TX_CFG_R_TX_DATASIZE);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_TX_CFG_R_TX_DATASIZE, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_TX_CFG_R_TX_DATASIZE, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_TX_CFG_R_TX_DATASIZE, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_TX_CFG_R_TX_DATASIZE, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_TX_CFG_R_TX_EN);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_TX_CFG_R_TX_EN, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_TX_CFG_R_TX_CLR);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_TX_CFG_R_TX_CLR, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_CMD_SADDR);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_CMD_SADDR, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_CMD_SIZE);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_CMD_SIZE, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_CMD_SIZE_R_CMD_SIZE);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_CMD_CFG);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_CMD_CFG, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_EN);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_CMD_CFG_R_CMD_EN, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CLR);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_spim_1_csr.r(utra::udma_spim_1::REG_STATUS);
        udma_spim_1_csr.wo(utra::udma_spim_1::REG_STATUS, foo);
        let bar = udma_spim_1_csr.rf(utra::udma_spim_1::REG_STATUS_STATUS_I);
        udma_spim_1_csr.rmwf(utra::udma_spim_1::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_spim_1_csr.zf(utra::udma_spim_1::REG_STATUS_STATUS_I, bar);
        baz |= udma_spim_1_csr.ms(utra::udma_spim_1::REG_STATUS_STATUS_I, 1);
        udma_spim_1_csr.wfo(utra::udma_spim_1::REG_STATUS_STATUS_I, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_spim_2_csr() {
        use super::*;
        let mut udma_spim_2_csr = CSR::new(HW_UDMA_SPIM_2_BASE as *mut u32);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_RX_SADDR);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_RX_SADDR, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_RX_SADDR_R_RX_STARTADDR);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_RX_SIZE);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_RX_SIZE, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_RX_SIZE_R_RX_SIZE);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_RX_CFG);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_RX_CFG, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_RX_CFG_R_RX_DATASIZE);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_RX_CFG_R_RX_DATASIZE, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_RX_CFG_R_RX_DATASIZE, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_RX_CFG_R_RX_DATASIZE, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_RX_CFG_R_RX_DATASIZE, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_RX_CFG_R_RX_EN);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_RX_CFG_R_RX_EN, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_RX_CFG_R_RX_CLR);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_RX_CFG_R_RX_CLR, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_TX_SADDR);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_TX_SADDR, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_TX_SADDR_R_TX_STARTADDR);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_TX_SIZE);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_TX_SIZE, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_TX_SIZE_R_TX_SIZE);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_TX_CFG);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_TX_CFG, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_TX_CFG_R_TX_DATASIZE);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_TX_CFG_R_TX_DATASIZE, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_TX_CFG_R_TX_DATASIZE, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_TX_CFG_R_TX_DATASIZE, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_TX_CFG_R_TX_DATASIZE, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_TX_CFG_R_TX_EN);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_TX_CFG_R_TX_EN, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_TX_CFG_R_TX_CLR);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_TX_CFG_R_TX_CLR, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_CMD_SADDR);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_CMD_SADDR, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_CMD_SIZE);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_CMD_SIZE, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_CMD_SIZE_R_CMD_SIZE);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_CMD_CFG);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_CMD_CFG, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_EN);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_CMD_CFG_R_CMD_EN, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CLR);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_spim_2_csr.r(utra::udma_spim_2::REG_STATUS);
        udma_spim_2_csr.wo(utra::udma_spim_2::REG_STATUS, foo);
        let bar = udma_spim_2_csr.rf(utra::udma_spim_2::REG_STATUS_STATUS_I);
        udma_spim_2_csr.rmwf(utra::udma_spim_2::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_spim_2_csr.zf(utra::udma_spim_2::REG_STATUS_STATUS_I, bar);
        baz |= udma_spim_2_csr.ms(utra::udma_spim_2::REG_STATUS_STATUS_I, 1);
        udma_spim_2_csr.wfo(utra::udma_spim_2::REG_STATUS_STATUS_I, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_spim_3_csr() {
        use super::*;
        let mut udma_spim_3_csr = CSR::new(HW_UDMA_SPIM_3_BASE as *mut u32);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_RX_SADDR);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_RX_SADDR, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_RX_SADDR_R_RX_STARTADDR);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_RX_SIZE);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_RX_SIZE, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_RX_SIZE_R_RX_SIZE);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_RX_CFG);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_RX_CFG, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_RX_CFG_R_RX_DATASIZE);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_RX_CFG_R_RX_DATASIZE, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_RX_CFG_R_RX_DATASIZE, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_RX_CFG_R_RX_DATASIZE, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_RX_CFG_R_RX_DATASIZE, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_RX_CFG_R_RX_EN);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_RX_CFG_R_RX_EN, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_RX_CFG_R_RX_CLR);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_RX_CFG_R_RX_CLR, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_TX_SADDR);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_TX_SADDR, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_TX_SADDR_R_TX_STARTADDR);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_TX_SIZE);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_TX_SIZE, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_TX_SIZE_R_TX_SIZE);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_TX_CFG);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_TX_CFG, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_TX_CFG_R_TX_DATASIZE);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_TX_CFG_R_TX_DATASIZE, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_TX_CFG_R_TX_DATASIZE, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_TX_CFG_R_TX_DATASIZE, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_TX_CFG_R_TX_DATASIZE, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_TX_CFG_R_TX_EN);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_TX_CFG_R_TX_EN, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_TX_CFG_R_TX_CLR);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_TX_CFG_R_TX_CLR, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_CMD_SADDR);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_CMD_SADDR, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_CMD_SIZE);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_CMD_SIZE, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_CMD_SIZE_R_CMD_SIZE);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_CMD_CFG);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_CMD_CFG, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_EN);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_CMD_CFG_R_CMD_EN, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CLR);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_spim_3_csr.r(utra::udma_spim_3::REG_STATUS);
        udma_spim_3_csr.wo(utra::udma_spim_3::REG_STATUS, foo);
        let bar = udma_spim_3_csr.rf(utra::udma_spim_3::REG_STATUS_STATUS_I);
        udma_spim_3_csr.rmwf(utra::udma_spim_3::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_spim_3_csr.zf(utra::udma_spim_3::REG_STATUS_STATUS_I, bar);
        baz |= udma_spim_3_csr.ms(utra::udma_spim_3::REG_STATUS_STATUS_I, 1);
        udma_spim_3_csr.wfo(utra::udma_spim_3::REG_STATUS_STATUS_I, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_i2c_0_csr() {
        use super::*;
        let mut udma_i2c_0_csr = CSR::new(HW_UDMA_I2C_0_BASE as *mut u32);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_RX_SADDR);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_RX_SADDR, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_RX_SADDR_R_RX_STARTADDR);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_RX_SIZE);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_RX_SIZE, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_RX_SIZE_R_RX_SIZE);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_RX_CFG);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_RX_CFG, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_RX_CFG_R_RX_EN);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_RX_CFG_R_RX_EN, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_RX_CFG_R_RX_CLR);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_RX_CFG_R_RX_CLR, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_TX_SADDR);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_TX_SADDR, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_TX_SADDR_R_TX_STARTADDR);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_TX_SIZE);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_TX_SIZE, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_TX_SIZE_R_TX_SIZE);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_TX_CFG);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_TX_CFG, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_TX_CFG_R_TX_EN);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_TX_CFG_R_TX_EN, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_TX_CFG_R_TX_CLR);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_TX_CFG_R_TX_CLR, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_CMD_SADDR);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_CMD_SADDR, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_CMD_SIZE);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_CMD_SIZE, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_CMD_SIZE_R_CMD_SIZE);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_CMD_CFG);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_CMD_CFG, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_EN);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_EN, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CLR);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_STATUS);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_STATUS, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_STATUS_R_BUSY);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_STATUS_R_BUSY, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_STATUS_R_BUSY, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_STATUS_R_BUSY, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_STATUS_R_BUSY, baz);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_STATUS_R_AL);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_STATUS_R_AL, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_STATUS_R_AL, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_STATUS_R_AL, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_STATUS_R_AL, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_SETUP);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_SETUP, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_SETUP_R_DO_RST);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_SETUP_R_DO_RST, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_SETUP_R_DO_RST, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_SETUP_R_DO_RST, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_SETUP_R_DO_RST, baz);

        let foo = udma_i2c_0_csr.r(utra::udma_i2c_0::REG_ACK);
        udma_i2c_0_csr.wo(utra::udma_i2c_0::REG_ACK, foo);
        let bar = udma_i2c_0_csr.rf(utra::udma_i2c_0::REG_ACK_R_NACK);
        udma_i2c_0_csr.rmwf(utra::udma_i2c_0::REG_ACK_R_NACK, bar);
        let mut baz = udma_i2c_0_csr.zf(utra::udma_i2c_0::REG_ACK_R_NACK, bar);
        baz |= udma_i2c_0_csr.ms(utra::udma_i2c_0::REG_ACK_R_NACK, 1);
        udma_i2c_0_csr.wfo(utra::udma_i2c_0::REG_ACK_R_NACK, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_i2c_1_csr() {
        use super::*;
        let mut udma_i2c_1_csr = CSR::new(HW_UDMA_I2C_1_BASE as *mut u32);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_RX_SADDR);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_RX_SADDR, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_RX_SADDR_R_RX_STARTADDR);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_RX_SIZE);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_RX_SIZE, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_RX_SIZE_R_RX_SIZE);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_RX_CFG);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_RX_CFG, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_RX_CFG_R_RX_EN);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_RX_CFG_R_RX_EN, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_RX_CFG_R_RX_CLR);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_RX_CFG_R_RX_CLR, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_TX_SADDR);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_TX_SADDR, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_TX_SADDR_R_TX_STARTADDR);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_TX_SIZE);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_TX_SIZE, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_TX_SIZE_R_TX_SIZE);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_TX_CFG);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_TX_CFG, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_TX_CFG_R_TX_EN);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_TX_CFG_R_TX_EN, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_TX_CFG_R_TX_CLR);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_TX_CFG_R_TX_CLR, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_CMD_SADDR);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_CMD_SADDR, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_CMD_SIZE);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_CMD_SIZE, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_CMD_SIZE_R_CMD_SIZE);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_CMD_CFG);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_CMD_CFG, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_EN);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_EN, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CLR);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_STATUS);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_STATUS, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_STATUS_R_BUSY);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_STATUS_R_BUSY, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_STATUS_R_BUSY, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_STATUS_R_BUSY, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_STATUS_R_BUSY, baz);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_STATUS_R_AL);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_STATUS_R_AL, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_STATUS_R_AL, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_STATUS_R_AL, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_STATUS_R_AL, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_SETUP);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_SETUP, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_SETUP_R_DO_RST);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_SETUP_R_DO_RST, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_SETUP_R_DO_RST, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_SETUP_R_DO_RST, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_SETUP_R_DO_RST, baz);

        let foo = udma_i2c_1_csr.r(utra::udma_i2c_1::REG_ACK);
        udma_i2c_1_csr.wo(utra::udma_i2c_1::REG_ACK, foo);
        let bar = udma_i2c_1_csr.rf(utra::udma_i2c_1::REG_ACK_R_NACK);
        udma_i2c_1_csr.rmwf(utra::udma_i2c_1::REG_ACK_R_NACK, bar);
        let mut baz = udma_i2c_1_csr.zf(utra::udma_i2c_1::REG_ACK_R_NACK, bar);
        baz |= udma_i2c_1_csr.ms(utra::udma_i2c_1::REG_ACK_R_NACK, 1);
        udma_i2c_1_csr.wfo(utra::udma_i2c_1::REG_ACK_R_NACK, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_i2c_2_csr() {
        use super::*;
        let mut udma_i2c_2_csr = CSR::new(HW_UDMA_I2C_2_BASE as *mut u32);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_RX_SADDR);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_RX_SADDR, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_RX_SADDR_R_RX_STARTADDR);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_RX_SIZE);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_RX_SIZE, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_RX_SIZE_R_RX_SIZE);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_RX_CFG);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_RX_CFG, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_RX_CFG_R_RX_EN);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_RX_CFG_R_RX_EN, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_RX_CFG_R_RX_CLR);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_RX_CFG_R_RX_CLR, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_TX_SADDR);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_TX_SADDR, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_TX_SADDR_R_TX_STARTADDR);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_TX_SIZE);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_TX_SIZE, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_TX_SIZE_R_TX_SIZE);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_TX_CFG);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_TX_CFG, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_TX_CFG_R_TX_EN);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_TX_CFG_R_TX_EN, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_TX_CFG_R_TX_CLR);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_TX_CFG_R_TX_CLR, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_CMD_SADDR);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_CMD_SADDR, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_CMD_SIZE);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_CMD_SIZE, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_CMD_SIZE_R_CMD_SIZE);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_CMD_CFG);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_CMD_CFG, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_EN);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_EN, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CLR);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_STATUS);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_STATUS, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_STATUS_R_BUSY);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_STATUS_R_BUSY, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_STATUS_R_BUSY, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_STATUS_R_BUSY, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_STATUS_R_BUSY, baz);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_STATUS_R_AL);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_STATUS_R_AL, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_STATUS_R_AL, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_STATUS_R_AL, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_STATUS_R_AL, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_SETUP);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_SETUP, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_SETUP_R_DO_RST);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_SETUP_R_DO_RST, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_SETUP_R_DO_RST, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_SETUP_R_DO_RST, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_SETUP_R_DO_RST, baz);

        let foo = udma_i2c_2_csr.r(utra::udma_i2c_2::REG_ACK);
        udma_i2c_2_csr.wo(utra::udma_i2c_2::REG_ACK, foo);
        let bar = udma_i2c_2_csr.rf(utra::udma_i2c_2::REG_ACK_R_NACK);
        udma_i2c_2_csr.rmwf(utra::udma_i2c_2::REG_ACK_R_NACK, bar);
        let mut baz = udma_i2c_2_csr.zf(utra::udma_i2c_2::REG_ACK_R_NACK, bar);
        baz |= udma_i2c_2_csr.ms(utra::udma_i2c_2::REG_ACK_R_NACK, 1);
        udma_i2c_2_csr.wfo(utra::udma_i2c_2::REG_ACK_R_NACK, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_i2c_3_csr() {
        use super::*;
        let mut udma_i2c_3_csr = CSR::new(HW_UDMA_I2C_3_BASE as *mut u32);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_RX_SADDR);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_RX_SADDR, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_RX_SADDR_R_RX_STARTADDR);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_RX_SIZE);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_RX_SIZE, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_RX_SIZE_R_RX_SIZE);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_RX_CFG);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_RX_CFG, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_RX_CFG_R_RX_EN);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_RX_CFG_R_RX_EN, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_RX_CFG_R_RX_CLR);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_RX_CFG_R_RX_CLR, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_TX_SADDR);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_TX_SADDR, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_TX_SADDR_R_TX_STARTADDR);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_TX_SIZE);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_TX_SIZE, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_TX_SIZE_R_TX_SIZE);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_TX_CFG);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_TX_CFG, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_TX_CFG_R_TX_EN);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_TX_CFG_R_TX_EN, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_TX_CFG_R_TX_CLR);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_TX_CFG_R_TX_CLR, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_CMD_SADDR);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_CMD_SADDR, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_CMD_SADDR_R_CMD_STARTADDR);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_CMD_SADDR_R_CMD_STARTADDR, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_CMD_SADDR_R_CMD_STARTADDR, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_CMD_SADDR_R_CMD_STARTADDR, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_CMD_SIZE);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_CMD_SIZE, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_CMD_SIZE_R_CMD_SIZE);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_CMD_SIZE_R_CMD_SIZE, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_CMD_SIZE_R_CMD_SIZE, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_CMD_SIZE_R_CMD_SIZE, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_CMD_SIZE_R_CMD_SIZE, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_CMD_CFG);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_CMD_CFG, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CONTINUOUS);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CONTINUOUS, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CONTINUOUS, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CONTINUOUS, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_EN);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_EN, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_EN, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_EN, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_EN, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CLR);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CLR, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CLR, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CLR, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_CMD_CFG_R_CMD_CLR, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_STATUS);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_STATUS, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_STATUS_R_BUSY);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_STATUS_R_BUSY, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_STATUS_R_BUSY, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_STATUS_R_BUSY, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_STATUS_R_BUSY, baz);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_STATUS_R_AL);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_STATUS_R_AL, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_STATUS_R_AL, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_STATUS_R_AL, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_STATUS_R_AL, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_SETUP);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_SETUP, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_SETUP_R_DO_RST);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_SETUP_R_DO_RST, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_SETUP_R_DO_RST, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_SETUP_R_DO_RST, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_SETUP_R_DO_RST, baz);

        let foo = udma_i2c_3_csr.r(utra::udma_i2c_3::REG_ACK);
        udma_i2c_3_csr.wo(utra::udma_i2c_3::REG_ACK, foo);
        let bar = udma_i2c_3_csr.rf(utra::udma_i2c_3::REG_ACK_R_NACK);
        udma_i2c_3_csr.rmwf(utra::udma_i2c_3::REG_ACK_R_NACK, bar);
        let mut baz = udma_i2c_3_csr.zf(utra::udma_i2c_3::REG_ACK_R_NACK, bar);
        baz |= udma_i2c_3_csr.ms(utra::udma_i2c_3::REG_ACK_R_NACK, 1);
        udma_i2c_3_csr.wfo(utra::udma_i2c_3::REG_ACK_R_NACK, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_sdio_csr() {
        use super::*;
        let mut udma_sdio_csr = CSR::new(HW_UDMA_SDIO_BASE as *mut u32);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RX_SADDR);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RX_SADDR, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RX_SADDR_R_RX_STARTADDR);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RX_SIZE);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RX_SIZE, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RX_SIZE_R_RX_SIZE);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RX_CFG);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RX_CFG, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RX_CFG_R_RX_EN);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RX_CFG_R_RX_EN, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RX_CFG_R_RX_CLR);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RX_CFG_R_RX_CLR, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_TX_SADDR);
        udma_sdio_csr.wo(utra::udma_sdio::REG_TX_SADDR, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_TX_SADDR_R_TX_STARTADDR);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_TX_SIZE);
        udma_sdio_csr.wo(utra::udma_sdio::REG_TX_SIZE, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_TX_SIZE_R_TX_SIZE);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_TX_CFG);
        udma_sdio_csr.wo(utra::udma_sdio::REG_TX_CFG, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_TX_CFG_R_TX_EN);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_TX_CFG_R_TX_EN, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_TX_CFG_R_TX_CLR);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_TX_CFG_R_TX_CLR, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_CMD_OP);
        udma_sdio_csr.wo(utra::udma_sdio::REG_CMD_OP, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_CMD_OP_R_CMD_RSP_TYPE);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_CMD_OP_R_CMD_RSP_TYPE, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_CMD_OP_R_CMD_RSP_TYPE, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_CMD_OP_R_CMD_RSP_TYPE, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_CMD_OP_R_CMD_RSP_TYPE, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_CMD_OP_R_CMD_OP);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_CMD_OP_R_CMD_OP, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_CMD_OP_R_CMD_OP, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_CMD_OP_R_CMD_OP, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_CMD_OP_R_CMD_OP, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_CMD_OP_R_CMD_STOPOPT);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_CMD_OP_R_CMD_STOPOPT, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_CMD_OP_R_CMD_STOPOPT, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_CMD_OP_R_CMD_STOPOPT, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_CMD_OP_R_CMD_STOPOPT, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_DATA_SETUP);
        udma_sdio_csr.wo(utra::udma_sdio::REG_DATA_SETUP, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_EN);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_EN, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_EN, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_DATA_SETUP_R_DATA_EN, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_DATA_SETUP_R_DATA_EN, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_RWN);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_RWN, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_RWN, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_DATA_SETUP_R_DATA_RWN, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_DATA_SETUP_R_DATA_RWN, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_QUAD);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_QUAD, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_QUAD, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_DATA_SETUP_R_DATA_QUAD, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_DATA_SETUP_R_DATA_QUAD, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_NUM);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_NUM, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_NUM, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_NUM, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_NUM, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_SIZE);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_SIZE, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_SIZE, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_SIZE, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_DATA_SETUP_R_DATA_BLOCK_SIZE, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_START);
        udma_sdio_csr.wo(utra::udma_sdio::REG_START, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_START_R_SDIO_START);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_START_R_SDIO_START, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_START_R_SDIO_START, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_START_R_SDIO_START, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_START_R_SDIO_START, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RSP0);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RSP0, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RSP0_CFG_RSP_DATA_I_31_0);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RSP0_CFG_RSP_DATA_I_31_0, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RSP0_CFG_RSP_DATA_I_31_0, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RSP0_CFG_RSP_DATA_I_31_0, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RSP0_CFG_RSP_DATA_I_31_0, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RSP1);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RSP1, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RSP1_CFG_RSP_DATA_I_63_32);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RSP1_CFG_RSP_DATA_I_63_32, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RSP1_CFG_RSP_DATA_I_63_32, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RSP1_CFG_RSP_DATA_I_63_32, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RSP1_CFG_RSP_DATA_I_63_32, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RSP2);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RSP2, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RSP2_CFG_RSP_DATA_I_95_64);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RSP2_CFG_RSP_DATA_I_95_64, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RSP2_CFG_RSP_DATA_I_95_64, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RSP2_CFG_RSP_DATA_I_95_64, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RSP2_CFG_RSP_DATA_I_95_64, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_RSP3);
        udma_sdio_csr.wo(utra::udma_sdio::REG_RSP3, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_RSP3_CFG_RSP_DATA_I_127_96);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_RSP3_CFG_RSP_DATA_I_127_96, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_RSP3_CFG_RSP_DATA_I_127_96, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_RSP3_CFG_RSP_DATA_I_127_96, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_RSP3_CFG_RSP_DATA_I_127_96, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_CLK_DIV);
        udma_sdio_csr.wo(utra::udma_sdio::REG_CLK_DIV, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_DATA);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_DATA, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_DATA, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_DATA, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_DATA, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_VALID);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_VALID, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_VALID, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_VALID, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_CLK_DIV_R_CLK_DIV_VALID, baz);

        let foo = udma_sdio_csr.r(utra::udma_sdio::REG_STATUS);
        udma_sdio_csr.wo(utra::udma_sdio::REG_STATUS, foo);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_STATUS_R_EOT);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_STATUS_R_EOT, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_STATUS_R_EOT, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_STATUS_R_EOT, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_STATUS_R_EOT, baz);
        let bar = udma_sdio_csr.rf(utra::udma_sdio::REG_STATUS_R_ERR);
        udma_sdio_csr.rmwf(utra::udma_sdio::REG_STATUS_R_ERR, bar);
        let mut baz = udma_sdio_csr.zf(utra::udma_sdio::REG_STATUS_R_ERR, bar);
        baz |= udma_sdio_csr.ms(utra::udma_sdio::REG_STATUS_R_ERR, 1);
        udma_sdio_csr.wfo(utra::udma_sdio::REG_STATUS_R_ERR, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_i2s_csr() {
        use super::*;
        let mut udma_i2s_csr = CSR::new(HW_UDMA_I2S_BASE as *mut u32);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_RX_SADDR);
        udma_i2s_csr.wo(utra::udma_i2s::REG_RX_SADDR, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_RX_SADDR_R_RX_STARTADDR);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_RX_SIZE);
        udma_i2s_csr.wo(utra::udma_i2s::REG_RX_SIZE, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_RX_SIZE_R_RX_SIZE);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_RX_CFG);
        udma_i2s_csr.wo(utra::udma_i2s::REG_RX_CFG, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_RX_CFG_R_RX_DATASIZE);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_RX_CFG_R_RX_DATASIZE, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_RX_CFG_R_RX_DATASIZE, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_RX_CFG_R_RX_DATASIZE, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_RX_CFG_R_RX_DATASIZE, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_RX_CFG_R_RX_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_RX_CFG_R_RX_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_RX_CFG_R_RX_CLR);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_RX_CFG_R_RX_CLR, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_TX_SADDR);
        udma_i2s_csr.wo(utra::udma_i2s::REG_TX_SADDR, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_TX_SADDR_R_TX_STARTADDR);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_TX_SIZE);
        udma_i2s_csr.wo(utra::udma_i2s::REG_TX_SIZE, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_TX_SIZE_R_TX_SIZE);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_TX_CFG);
        udma_i2s_csr.wo(utra::udma_i2s::REG_TX_CFG, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_TX_CFG_R_TX_DATASIZE);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_TX_CFG_R_TX_DATASIZE, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_TX_CFG_R_TX_DATASIZE, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_TX_CFG_R_TX_DATASIZE, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_TX_CFG_R_TX_DATASIZE, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_TX_CFG_R_TX_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_TX_CFG_R_TX_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_TX_CFG_R_TX_CLR);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_TX_CFG_R_TX_CLR, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_I2S_CLKCFG_SETUP);
        udma_i2s_csr.wo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_GEN_CLK_DIV);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_GEN_CLK_DIV, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_GEN_CLK_DIV, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_GEN_CLK_DIV, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_GEN_CLK_DIV, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_GEN_CLK_DIV);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_GEN_CLK_DIV, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_GEN_CLK_DIV, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_GEN_CLK_DIV, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_GEN_CLK_DIV, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_COMMON_GEN_CLK_DIV);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_COMMON_GEN_CLK_DIV, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_COMMON_GEN_CLK_DIV, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_COMMON_GEN_CLK_DIV, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_COMMON_GEN_CLK_DIV, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_CLK_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_CLK_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_CLK_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_CLK_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_CLK_EN, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_CLK_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_CLK_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_CLK_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_CLK_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_CLK_EN, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_PDM_CLK_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_PDM_CLK_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_PDM_CLK_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_PDM_CLK_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_PDM_CLK_EN, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_EXT);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_EXT, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_EXT, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_EXT, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_EXT, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_NUM);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_NUM, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_NUM, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_NUM, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_SLAVE_SEL_NUM, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_EXT);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_EXT, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_EXT, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_EXT, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_EXT, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_NUM);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_NUM, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_NUM, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_NUM, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_CLKCFG_SETUP_R_MASTER_SEL_NUM, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_I2S_SLV_SETUP);
        udma_i2s_csr.wo(utra::udma_i2s::REG_I2S_SLV_SETUP, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_WORDS);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_WORDS, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_WORDS, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_WORDS, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_WORDS, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_BITS_WORD);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_BITS_WORD, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_BITS_WORD, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_BITS_WORD, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_BITS_WORD, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_LSB_FIRST);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_LSB_FIRST, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_LSB_FIRST, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_LSB_FIRST, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_LSB_FIRST, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_2CH);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_2CH, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_2CH, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_2CH, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_2CH, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_SLV_SETUP_R_SLAVE_I2S_EN, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_I2S_MST_SETUP);
        udma_i2s_csr.wo(utra::udma_i2s::REG_I2S_MST_SETUP, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_WORDS);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_WORDS, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_WORDS, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_WORDS, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_WORDS, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_BITS_WORD);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_BITS_WORD, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_BITS_WORD, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_BITS_WORD, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_BITS_WORD, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_LSB_FIRST);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_LSB_FIRST, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_LSB_FIRST, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_LSB_FIRST, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_LSB_FIRST, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_2CH);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_2CH, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_2CH, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_2CH, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_2CH, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_MST_SETUP_R_MASTER_I2S_EN, baz);

        let foo = udma_i2s_csr.r(utra::udma_i2s::REG_I2S_PDM_SETUP);
        udma_i2s_csr.wo(utra::udma_i2s::REG_I2S_PDM_SETUP, foo);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_SHIFT);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_SHIFT, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_SHIFT, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_SHIFT, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_SHIFT, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_DECIMATION);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_DECIMATION, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_DECIMATION, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_DECIMATION, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_DECIMATION, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_MODE);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_MODE, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_MODE, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_MODE, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_MODE, baz);
        let bar = udma_i2s_csr.rf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_EN);
        udma_i2s_csr.rmwf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_EN, bar);
        let mut baz = udma_i2s_csr.zf(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_EN, bar);
        baz |= udma_i2s_csr.ms(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_EN, 1);
        udma_i2s_csr.wfo(utra::udma_i2s::REG_I2S_PDM_SETUP_R_SLAVE_PDM_EN, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_camera_csr() {
        use super::*;
        let mut udma_camera_csr = CSR::new(HW_UDMA_CAMERA_BASE as *mut u32);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_RX_SADDR);
        udma_camera_csr.wo(utra::udma_camera::REG_RX_SADDR, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_RX_SADDR_R_RX_STARTADDR);
        udma_camera_csr.rmwf(utra::udma_camera::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_RX_SIZE);
        udma_camera_csr.wo(utra::udma_camera::REG_RX_SIZE, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_RX_SIZE_R_RX_SIZE);
        udma_camera_csr.rmwf(utra::udma_camera::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_RX_CFG);
        udma_camera_csr.wo(utra::udma_camera::REG_RX_CFG, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_camera_csr.rmwf(utra::udma_camera::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_RX_CFG_R_RX_DATASIZE);
        udma_camera_csr.rmwf(utra::udma_camera::REG_RX_CFG_R_RX_DATASIZE, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_RX_CFG_R_RX_DATASIZE, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_RX_CFG_R_RX_DATASIZE, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_RX_CFG_R_RX_DATASIZE, baz);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_RX_CFG_R_RX_EN);
        udma_camera_csr.rmwf(utra::udma_camera::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_RX_CFG_R_RX_EN, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_RX_CFG_R_RX_CLR);
        udma_camera_csr.rmwf(utra::udma_camera::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_RX_CFG_R_RX_CLR, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_CAM_CFG_GLOB);
        udma_camera_csr.wo(utra::udma_camera::REG_CAM_CFG_GLOB, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_CFG_GLOB_R_CAM_CFG);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_CFG_GLOB_R_CAM_CFG, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_CFG_GLOB_R_CAM_CFG, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_CFG_GLOB_R_CAM_CFG, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_CFG_GLOB_R_CAM_CFG, baz);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_CFG_GLOB_CFG_CAM_IP_EN_I);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_CFG_GLOB_CFG_CAM_IP_EN_I, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_CFG_GLOB_CFG_CAM_IP_EN_I, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_CFG_GLOB_CFG_CAM_IP_EN_I, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_CFG_GLOB_CFG_CAM_IP_EN_I, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_CAM_CFG_LL);
        udma_camera_csr.wo(utra::udma_camera::REG_CAM_CFG_LL, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_CFG_LL_R_CAM_CFG_LL);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_CFG_LL_R_CAM_CFG_LL, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_CFG_LL_R_CAM_CFG_LL, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_CFG_LL_R_CAM_CFG_LL, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_CFG_LL_R_CAM_CFG_LL, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_CAM_CFG_UR);
        udma_camera_csr.wo(utra::udma_camera::REG_CAM_CFG_UR, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_CFG_UR_R_CAM_CFG_UR);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_CFG_UR_R_CAM_CFG_UR, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_CFG_UR_R_CAM_CFG_UR, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_CFG_UR_R_CAM_CFG_UR, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_CFG_UR_R_CAM_CFG_UR, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_CAM_CFG_SIZE);
        udma_camera_csr.wo(utra::udma_camera::REG_CAM_CFG_SIZE, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_CFG_SIZE_R_CAM_CFG_SIZE);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_CFG_SIZE_R_CAM_CFG_SIZE, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_CFG_SIZE_R_CAM_CFG_SIZE, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_CFG_SIZE_R_CAM_CFG_SIZE, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_CFG_SIZE_R_CAM_CFG_SIZE, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_CAM_CFG_FILTER);
        udma_camera_csr.wo(utra::udma_camera::REG_CAM_CFG_FILTER, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_CFG_FILTER_R_CAM_CFG_FILTER);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_CFG_FILTER_R_CAM_CFG_FILTER, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_CFG_FILTER_R_CAM_CFG_FILTER, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_CFG_FILTER_R_CAM_CFG_FILTER, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_CFG_FILTER_R_CAM_CFG_FILTER, baz);

        let foo = udma_camera_csr.r(utra::udma_camera::REG_CAM_VSYNC_POLARITY);
        udma_camera_csr.wo(utra::udma_camera::REG_CAM_VSYNC_POLARITY, foo);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_VSYNC_POLARITY);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_VSYNC_POLARITY, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_VSYNC_POLARITY, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_VSYNC_POLARITY, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_VSYNC_POLARITY, baz);
        let bar = udma_camera_csr.rf(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_HSYNC_POLARITY);
        udma_camera_csr.rmwf(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_HSYNC_POLARITY, bar);
        let mut baz = udma_camera_csr.zf(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_HSYNC_POLARITY, bar);
        baz |= udma_camera_csr.ms(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_HSYNC_POLARITY, 1);
        udma_camera_csr.wfo(utra::udma_camera::REG_CAM_VSYNC_POLARITY_R_CAM_HSYNC_POLARITY, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_filter_csr() {
        use super::*;
        let mut udma_filter_csr = CSR::new(HW_UDMA_FILTER_BASE as *mut u32);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH0_ADD);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH0_ADD, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH0_ADD_R_FILTER_TX_START_ADDR_0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH0_ADD_R_FILTER_TX_START_ADDR_0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH0_ADD_R_FILTER_TX_START_ADDR_0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH0_ADD_R_FILTER_TX_START_ADDR_0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH0_ADD_R_FILTER_TX_START_ADDR_0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH0_CFG);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH0_CFG, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_DATASIZE_0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_DATASIZE_0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_DATASIZE_0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_DATASIZE_0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_DATASIZE_0, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_MODE_0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_MODE_0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_MODE_0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_MODE_0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH0_CFG_R_FILTER_TX_MODE_0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH0_LEN0);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH0_LEN0, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH0_LEN0_R_FILTER_TX_LEN0_0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH0_LEN0_R_FILTER_TX_LEN0_0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH0_LEN0_R_FILTER_TX_LEN0_0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH0_LEN0_R_FILTER_TX_LEN0_0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH0_LEN0_R_FILTER_TX_LEN0_0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH0_LEN1);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH0_LEN1, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH0_LEN1_R_FILTER_TX_LEN1_0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH0_LEN1_R_FILTER_TX_LEN1_0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH0_LEN1_R_FILTER_TX_LEN1_0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH0_LEN1_R_FILTER_TX_LEN1_0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH0_LEN1_R_FILTER_TX_LEN1_0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH0_LEN2);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH0_LEN2, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH0_LEN2_R_FILTER_TX_LEN2_0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH0_LEN2_R_FILTER_TX_LEN2_0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH0_LEN2_R_FILTER_TX_LEN2_0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH0_LEN2_R_FILTER_TX_LEN2_0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH0_LEN2_R_FILTER_TX_LEN2_0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH1_ADD);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH1_ADD, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH1_ADD_R_FILTER_TX_START_ADDR_1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH1_ADD_R_FILTER_TX_START_ADDR_1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH1_ADD_R_FILTER_TX_START_ADDR_1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH1_ADD_R_FILTER_TX_START_ADDR_1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH1_ADD_R_FILTER_TX_START_ADDR_1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH1_CFG);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH1_CFG, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_DATASIZE_1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_DATASIZE_1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_DATASIZE_1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_DATASIZE_1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_DATASIZE_1, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_MODE_1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_MODE_1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_MODE_1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_MODE_1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH1_CFG_R_FILTER_TX_MODE_1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH1_LEN0);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH1_LEN0, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH1_LEN0_R_FILTER_TX_LEN0_1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH1_LEN0_R_FILTER_TX_LEN0_1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH1_LEN0_R_FILTER_TX_LEN0_1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH1_LEN0_R_FILTER_TX_LEN0_1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH1_LEN0_R_FILTER_TX_LEN0_1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH1_LEN1);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH1_LEN1, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH1_LEN1_R_FILTER_TX_LEN1_1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH1_LEN1_R_FILTER_TX_LEN1_1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH1_LEN1_R_FILTER_TX_LEN1_1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH1_LEN1_R_FILTER_TX_LEN1_1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH1_LEN1_R_FILTER_TX_LEN1_1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_TX_CH1_LEN2);
        udma_filter_csr.wo(utra::udma_filter::REG_TX_CH1_LEN2, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_TX_CH1_LEN2_R_FILTER_TX_LEN2_1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_TX_CH1_LEN2_R_FILTER_TX_LEN2_1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_TX_CH1_LEN2_R_FILTER_TX_LEN2_1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_TX_CH1_LEN2_R_FILTER_TX_LEN2_1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_TX_CH1_LEN2_R_FILTER_TX_LEN2_1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_RX_CH_ADD);
        udma_filter_csr.wo(utra::udma_filter::REG_RX_CH_ADD, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_RX_CH_ADD_R_FILTER_RX_START_ADDR);
        udma_filter_csr.rmwf(utra::udma_filter::REG_RX_CH_ADD_R_FILTER_RX_START_ADDR, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_RX_CH_ADD_R_FILTER_RX_START_ADDR, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_RX_CH_ADD_R_FILTER_RX_START_ADDR, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_RX_CH_ADD_R_FILTER_RX_START_ADDR, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_RX_CH_CFG);
        udma_filter_csr.wo(utra::udma_filter::REG_RX_CH_CFG, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_DATASIZE);
        udma_filter_csr.rmwf(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_DATASIZE, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_DATASIZE, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_DATASIZE, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_DATASIZE, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_MODE);
        udma_filter_csr.rmwf(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_MODE, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_MODE, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_MODE, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_RX_CH_CFG_R_FILTER_RX_MODE, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_RX_CH_LEN0);
        udma_filter_csr.wo(utra::udma_filter::REG_RX_CH_LEN0, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_RX_CH_LEN0_R_FILTER_RX_LEN0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_RX_CH_LEN0_R_FILTER_RX_LEN0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_RX_CH_LEN0_R_FILTER_RX_LEN0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_RX_CH_LEN0_R_FILTER_RX_LEN0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_RX_CH_LEN0_R_FILTER_RX_LEN0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_RX_CH_LEN1);
        udma_filter_csr.wo(utra::udma_filter::REG_RX_CH_LEN1, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_RX_CH_LEN1_R_FILTER_RX_LEN1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_RX_CH_LEN1_R_FILTER_RX_LEN1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_RX_CH_LEN1_R_FILTER_RX_LEN1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_RX_CH_LEN1_R_FILTER_RX_LEN1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_RX_CH_LEN1_R_FILTER_RX_LEN1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_RX_CH_LEN2);
        udma_filter_csr.wo(utra::udma_filter::REG_RX_CH_LEN2, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_RX_CH_LEN2_R_FILTER_RX_LEN2);
        udma_filter_csr.rmwf(utra::udma_filter::REG_RX_CH_LEN2_R_FILTER_RX_LEN2, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_RX_CH_LEN2_R_FILTER_RX_LEN2, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_RX_CH_LEN2_R_FILTER_RX_LEN2, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_RX_CH_LEN2_R_FILTER_RX_LEN2, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_AU_CFG);
        udma_filter_csr.wo(utra::udma_filter::REG_AU_CFG, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_AU_CFG_R_AU_USE_SIGNED);
        udma_filter_csr.rmwf(utra::udma_filter::REG_AU_CFG_R_AU_USE_SIGNED, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_AU_CFG_R_AU_USE_SIGNED, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_AU_CFG_R_AU_USE_SIGNED, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_AU_CFG_R_AU_USE_SIGNED, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_AU_CFG_R_AU_BYPASS);
        udma_filter_csr.rmwf(utra::udma_filter::REG_AU_CFG_R_AU_BYPASS, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_AU_CFG_R_AU_BYPASS, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_AU_CFG_R_AU_BYPASS, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_AU_CFG_R_AU_BYPASS, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_AU_CFG_R_AU_MODE);
        udma_filter_csr.rmwf(utra::udma_filter::REG_AU_CFG_R_AU_MODE, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_AU_CFG_R_AU_MODE, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_AU_CFG_R_AU_MODE, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_AU_CFG_R_AU_MODE, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_AU_CFG_R_AU_SHIFT);
        udma_filter_csr.rmwf(utra::udma_filter::REG_AU_CFG_R_AU_SHIFT, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_AU_CFG_R_AU_SHIFT, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_AU_CFG_R_AU_SHIFT, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_AU_CFG_R_AU_SHIFT, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_AU_REG0);
        udma_filter_csr.wo(utra::udma_filter::REG_AU_REG0, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_AU_REG0_R_COMMIT_AU_REG0);
        udma_filter_csr.rmwf(utra::udma_filter::REG_AU_REG0_R_COMMIT_AU_REG0, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_AU_REG0_R_COMMIT_AU_REG0, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_AU_REG0_R_COMMIT_AU_REG0, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_AU_REG0_R_COMMIT_AU_REG0, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_AU_REG1);
        udma_filter_csr.wo(utra::udma_filter::REG_AU_REG1, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_AU_REG1_R_COMMIT_AU_REG1);
        udma_filter_csr.rmwf(utra::udma_filter::REG_AU_REG1_R_COMMIT_AU_REG1, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_AU_REG1_R_COMMIT_AU_REG1, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_AU_REG1_R_COMMIT_AU_REG1, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_AU_REG1_R_COMMIT_AU_REG1, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_BINCU_TH);
        udma_filter_csr.wo(utra::udma_filter::REG_BINCU_TH, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_BINCU_TH_R_COMMIT_BINCU_THRESHOLD);
        udma_filter_csr.rmwf(utra::udma_filter::REG_BINCU_TH_R_COMMIT_BINCU_THRESHOLD, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_BINCU_TH_R_COMMIT_BINCU_THRESHOLD, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_BINCU_TH_R_COMMIT_BINCU_THRESHOLD, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_BINCU_TH_R_COMMIT_BINCU_THRESHOLD, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_BINCU_CNT);
        udma_filter_csr.wo(utra::udma_filter::REG_BINCU_CNT, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_BINCU_CNT_R_BINCU_COUNTER);
        udma_filter_csr.rmwf(utra::udma_filter::REG_BINCU_CNT_R_BINCU_COUNTER, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_BINCU_CNT_R_BINCU_COUNTER, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_BINCU_CNT_R_BINCU_COUNTER, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_BINCU_CNT_R_BINCU_COUNTER, baz);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_BINCU_CNT_R_BINCU_EN_COUNTER);
        udma_filter_csr.rmwf(utra::udma_filter::REG_BINCU_CNT_R_BINCU_EN_COUNTER, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_BINCU_CNT_R_BINCU_EN_COUNTER, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_BINCU_CNT_R_BINCU_EN_COUNTER, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_BINCU_CNT_R_BINCU_EN_COUNTER, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_BINCU_SETUP);
        udma_filter_csr.wo(utra::udma_filter::REG_BINCU_SETUP, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_BINCU_SETUP_R_BINCU_DATASIZE);
        udma_filter_csr.rmwf(utra::udma_filter::REG_BINCU_SETUP_R_BINCU_DATASIZE, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_BINCU_SETUP_R_BINCU_DATASIZE, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_BINCU_SETUP_R_BINCU_DATASIZE, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_BINCU_SETUP_R_BINCU_DATASIZE, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_BINCU_VAL);
        udma_filter_csr.wo(utra::udma_filter::REG_BINCU_VAL, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_BINCU_VAL_BINCU_COUNTER_I);
        udma_filter_csr.rmwf(utra::udma_filter::REG_BINCU_VAL_BINCU_COUNTER_I, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_BINCU_VAL_BINCU_COUNTER_I, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_BINCU_VAL_BINCU_COUNTER_I, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_BINCU_VAL_BINCU_COUNTER_I, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_FILT);
        udma_filter_csr.wo(utra::udma_filter::REG_FILT, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_FILT_R_FILTER_MODE);
        udma_filter_csr.rmwf(utra::udma_filter::REG_FILT_R_FILTER_MODE, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_FILT_R_FILTER_MODE, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_FILT_R_FILTER_MODE, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_FILT_R_FILTER_MODE, baz);

        let foo = udma_filter_csr.r(utra::udma_filter::REG_STATUS);
        udma_filter_csr.wo(utra::udma_filter::REG_STATUS, foo);
        let bar = udma_filter_csr.rf(utra::udma_filter::REG_STATUS_R_FILTER_DONE);
        udma_filter_csr.rmwf(utra::udma_filter::REG_STATUS_R_FILTER_DONE, bar);
        let mut baz = udma_filter_csr.zf(utra::udma_filter::REG_STATUS_R_FILTER_DONE, bar);
        baz |= udma_filter_csr.ms(utra::udma_filter::REG_STATUS_R_FILTER_DONE, 1);
        udma_filter_csr.wfo(utra::udma_filter::REG_STATUS_R_FILTER_DONE, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_scif_csr() {
        use super::*;
        let mut udma_scif_csr = CSR::new(HW_UDMA_SCIF_BASE as *mut u32);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_RX_SADDR);
        udma_scif_csr.wo(utra::udma_scif::REG_RX_SADDR, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_RX_SADDR_R_RX_STARTADDR);
        udma_scif_csr.rmwf(utra::udma_scif::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_RX_SIZE);
        udma_scif_csr.wo(utra::udma_scif::REG_RX_SIZE, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_RX_SIZE_R_RX_SIZE);
        udma_scif_csr.rmwf(utra::udma_scif::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_RX_CFG);
        udma_scif_csr.wo(utra::udma_scif::REG_RX_CFG, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_scif_csr.rmwf(utra::udma_scif::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_RX_CFG_R_RX_CONTINUOUS, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_TX_SADDR);
        udma_scif_csr.wo(utra::udma_scif::REG_TX_SADDR, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_TX_SADDR_R_TX_STARTADDR);
        udma_scif_csr.rmwf(utra::udma_scif::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_TX_SIZE);
        udma_scif_csr.wo(utra::udma_scif::REG_TX_SIZE, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_TX_SIZE_R_TX_SIZE);
        udma_scif_csr.rmwf(utra::udma_scif::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_TX_CFG);
        udma_scif_csr.wo(utra::udma_scif::REG_TX_CFG, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_scif_csr.rmwf(utra::udma_scif::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_TX_CFG_R_TX_CONTINUOUS, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_STATUS);
        udma_scif_csr.wo(utra::udma_scif::REG_STATUS, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_STATUS_STATUS_I);
        udma_scif_csr.rmwf(utra::udma_scif::REG_STATUS_STATUS_I, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_STATUS_STATUS_I, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_STATUS_STATUS_I, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_STATUS_STATUS_I, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_SCIF_SETUP);
        udma_scif_csr.wo(utra::udma_scif::REG_SCIF_SETUP, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_PARITY_EN);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_PARITY_EN, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_PARITY_EN, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_PARITY_EN, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_PARITY_EN, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_BITS);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_BITS, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_BITS, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_BITS, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_BITS, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_STOP_BITS);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_STOP_BITS, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_STOP_BITS, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_STOP_BITS, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_STOP_BITS, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_POLLING_EN);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_POLLING_EN, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_POLLING_EN, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_POLLING_EN, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_POLLING_EN, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_CLEAN_FIFO);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_CLEAN_FIFO, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_CLEAN_FIFO, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_CLEAN_FIFO, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_RX_CLEAN_FIFO, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_TX);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_TX, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_TX, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_TX, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_TX, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_RX);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_RX, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_RX, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_RX, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_EN_RX, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_CLKSEL);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_CLKSEL, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_CLKSEL, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_CLKSEL, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_CLKSEL, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_DIV);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_DIV, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_DIV, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_DIV, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_SETUP_R_SCIF_DIV, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_ERROR);
        udma_scif_csr.wo(utra::udma_scif::REG_ERROR, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_ERROR_R_ERR_OVERFLOW);
        udma_scif_csr.rmwf(utra::udma_scif::REG_ERROR_R_ERR_OVERFLOW, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_ERROR_R_ERR_OVERFLOW, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_ERROR_R_ERR_OVERFLOW, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_ERROR_R_ERR_OVERFLOW, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_ERROR_R_ERR_PARITY);
        udma_scif_csr.rmwf(utra::udma_scif::REG_ERROR_R_ERR_PARITY, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_ERROR_R_ERR_PARITY, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_ERROR_R_ERR_PARITY, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_ERROR_R_ERR_PARITY, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_IRQ_EN);
        udma_scif_csr.wo(utra::udma_scif::REG_IRQ_EN, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_IRQ_EN_R_SCIF_RX_IRQ_EN);
        udma_scif_csr.rmwf(utra::udma_scif::REG_IRQ_EN_R_SCIF_RX_IRQ_EN, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_IRQ_EN_R_SCIF_RX_IRQ_EN, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_IRQ_EN_R_SCIF_RX_IRQ_EN, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_IRQ_EN_R_SCIF_RX_IRQ_EN, baz);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_IRQ_EN_R_SCIF_ERR_IRQ_EN);
        udma_scif_csr.rmwf(utra::udma_scif::REG_IRQ_EN_R_SCIF_ERR_IRQ_EN, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_IRQ_EN_R_SCIF_ERR_IRQ_EN, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_IRQ_EN_R_SCIF_ERR_IRQ_EN, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_IRQ_EN_R_SCIF_ERR_IRQ_EN, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_VALID);
        udma_scif_csr.wo(utra::udma_scif::REG_VALID, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_VALID_R_SCIF_RX_DATA_VALID);
        udma_scif_csr.rmwf(utra::udma_scif::REG_VALID_R_SCIF_RX_DATA_VALID, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_VALID_R_SCIF_RX_DATA_VALID, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_VALID_R_SCIF_RX_DATA_VALID, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_VALID_R_SCIF_RX_DATA_VALID, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_DATA);
        udma_scif_csr.wo(utra::udma_scif::REG_DATA, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_DATA_R_SCIF_RX_DATA);
        udma_scif_csr.rmwf(utra::udma_scif::REG_DATA_R_SCIF_RX_DATA, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_DATA_R_SCIF_RX_DATA, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_DATA_R_SCIF_RX_DATA, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_DATA_R_SCIF_RX_DATA, baz);

        let foo = udma_scif_csr.r(utra::udma_scif::REG_SCIF_ETU);
        udma_scif_csr.wo(utra::udma_scif::REG_SCIF_ETU, foo);
        let bar = udma_scif_csr.rf(utra::udma_scif::REG_SCIF_ETU_R_SCIF_ETU);
        udma_scif_csr.rmwf(utra::udma_scif::REG_SCIF_ETU_R_SCIF_ETU, bar);
        let mut baz = udma_scif_csr.zf(utra::udma_scif::REG_SCIF_ETU_R_SCIF_ETU, bar);
        baz |= udma_scif_csr.ms(utra::udma_scif::REG_SCIF_ETU_R_SCIF_ETU, 1);
        udma_scif_csr.wfo(utra::udma_scif::REG_SCIF_ETU_R_SCIF_ETU, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_spis_0_csr() {
        use super::*;
        let mut udma_spis_0_csr = CSR::new(HW_UDMA_SPIS_0_BASE as *mut u32);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_RX_SADDR);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_RX_SADDR, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_RX_SADDR_R_RX_STARTADDR);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_RX_SIZE);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_RX_SIZE, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_RX_SIZE_R_RX_SIZE);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_RX_CFG);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_RX_CFG, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_RX_CFG_R_RX_EN);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_RX_CFG_R_RX_EN, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_RX_CFG_R_RX_CLR);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_RX_CFG_R_RX_CLR, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_TX_SADDR);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_TX_SADDR, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_TX_SADDR_R_TX_STARTADDR);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_TX_SIZE);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_TX_SIZE, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_TX_SIZE_R_TX_SIZE);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_TX_CFG);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_TX_CFG, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_TX_CFG_R_TX_EN);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_TX_CFG_R_TX_EN, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_TX_CFG_R_TX_CLR);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_TX_CFG_R_TX_CLR, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_SPIS_SETUP);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_SPIS_SETUP, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPOL);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPOL, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPOL, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPOL, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPOL, baz);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPHA);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPHA, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPHA, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPHA, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SPIS_SETUP_CFGCPHA, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_SEOT_CNT);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_SEOT_CNT, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SEOT_CNT_SR_SEOT_CNT);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SEOT_CNT_SR_SEOT_CNT, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SEOT_CNT_SR_SEOT_CNT, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SEOT_CNT_SR_SEOT_CNT, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SEOT_CNT_SR_SEOT_CNT, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_SPIS_IRQ_EN);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_SPIS_IRQ_EN, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SPIS_IRQ_EN_SEOT_IRQ_EN);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_SPIS_RXCNT);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_SPIS_RXCNT, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SPIS_RXCNT_CFGRXCNT);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SPIS_RXCNT_CFGRXCNT, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SPIS_RXCNT_CFGRXCNT, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SPIS_RXCNT_CFGRXCNT, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SPIS_RXCNT_CFGRXCNT, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_SPIS_TXCNT);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_SPIS_TXCNT, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SPIS_TXCNT_CFGTXCNT);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SPIS_TXCNT_CFGTXCNT, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SPIS_TXCNT_CFGTXCNT, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SPIS_TXCNT_CFGTXCNT, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SPIS_TXCNT_CFGTXCNT, baz);

        let foo = udma_spis_0_csr.r(utra::udma_spis_0::REG_SPIS_DMCNT);
        udma_spis_0_csr.wo(utra::udma_spis_0::REG_SPIS_DMCNT, foo);
        let bar = udma_spis_0_csr.rf(utra::udma_spis_0::REG_SPIS_DMCNT_CFGDMCNT);
        udma_spis_0_csr.rmwf(utra::udma_spis_0::REG_SPIS_DMCNT_CFGDMCNT, bar);
        let mut baz = udma_spis_0_csr.zf(utra::udma_spis_0::REG_SPIS_DMCNT_CFGDMCNT, bar);
        baz |= udma_spis_0_csr.ms(utra::udma_spis_0::REG_SPIS_DMCNT_CFGDMCNT, 1);
        udma_spis_0_csr.wfo(utra::udma_spis_0::REG_SPIS_DMCNT_CFGDMCNT, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_spis_1_csr() {
        use super::*;
        let mut udma_spis_1_csr = CSR::new(HW_UDMA_SPIS_1_BASE as *mut u32);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_RX_SADDR);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_RX_SADDR, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_RX_SADDR_R_RX_STARTADDR);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_RX_SADDR_R_RX_STARTADDR, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_RX_SADDR_R_RX_STARTADDR, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_RX_SADDR_R_RX_STARTADDR, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_RX_SIZE);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_RX_SIZE, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_RX_SIZE_R_RX_SIZE);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_RX_SIZE_R_RX_SIZE, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_RX_SIZE_R_RX_SIZE, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_RX_SIZE_R_RX_SIZE, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_RX_SIZE_R_RX_SIZE, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_RX_CFG);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_RX_CFG, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_RX_CFG_R_RX_CONTINUOUS);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_RX_CFG_R_RX_CONTINUOUS, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_RX_CFG_R_RX_CONTINUOUS, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_RX_CFG_R_RX_CONTINUOUS, baz);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_RX_CFG_R_RX_EN);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_RX_CFG_R_RX_EN, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_RX_CFG_R_RX_EN, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_RX_CFG_R_RX_EN, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_RX_CFG_R_RX_EN, baz);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_RX_CFG_R_RX_CLR);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_RX_CFG_R_RX_CLR, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_RX_CFG_R_RX_CLR, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_RX_CFG_R_RX_CLR, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_RX_CFG_R_RX_CLR, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_TX_SADDR);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_TX_SADDR, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_TX_SADDR_R_TX_STARTADDR);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_TX_SADDR_R_TX_STARTADDR, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_TX_SADDR_R_TX_STARTADDR, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_TX_SADDR_R_TX_STARTADDR, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_TX_SIZE);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_TX_SIZE, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_TX_SIZE_R_TX_SIZE);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_TX_SIZE_R_TX_SIZE, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_TX_SIZE_R_TX_SIZE, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_TX_SIZE_R_TX_SIZE, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_TX_SIZE_R_TX_SIZE, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_TX_CFG);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_TX_CFG, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_TX_CFG_R_TX_CONTINUOUS);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_TX_CFG_R_TX_CONTINUOUS, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_TX_CFG_R_TX_CONTINUOUS, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_TX_CFG_R_TX_CONTINUOUS, baz);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_TX_CFG_R_TX_EN);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_TX_CFG_R_TX_EN, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_TX_CFG_R_TX_EN, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_TX_CFG_R_TX_EN, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_TX_CFG_R_TX_EN, baz);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_TX_CFG_R_TX_CLR);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_TX_CFG_R_TX_CLR, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_TX_CFG_R_TX_CLR, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_TX_CFG_R_TX_CLR, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_TX_CFG_R_TX_CLR, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_SPIS_SETUP);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_SPIS_SETUP, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPOL);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPOL, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPOL, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPOL, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPOL, baz);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPHA);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPHA, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPHA, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPHA, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SPIS_SETUP_CFGCPHA, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_SEOT_CNT);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_SEOT_CNT, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SEOT_CNT_SR_SEOT_CNT);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SEOT_CNT_SR_SEOT_CNT, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SEOT_CNT_SR_SEOT_CNT, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SEOT_CNT_SR_SEOT_CNT, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SEOT_CNT_SR_SEOT_CNT, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_SPIS_IRQ_EN);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_SPIS_IRQ_EN, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SPIS_IRQ_EN_SEOT_IRQ_EN);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SPIS_IRQ_EN_SEOT_IRQ_EN, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_SPIS_RXCNT);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_SPIS_RXCNT, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SPIS_RXCNT_CFGRXCNT);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SPIS_RXCNT_CFGRXCNT, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SPIS_RXCNT_CFGRXCNT, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SPIS_RXCNT_CFGRXCNT, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SPIS_RXCNT_CFGRXCNT, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_SPIS_TXCNT);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_SPIS_TXCNT, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SPIS_TXCNT_CFGTXCNT);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SPIS_TXCNT_CFGTXCNT, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SPIS_TXCNT_CFGTXCNT, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SPIS_TXCNT_CFGTXCNT, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SPIS_TXCNT_CFGTXCNT, baz);

        let foo = udma_spis_1_csr.r(utra::udma_spis_1::REG_SPIS_DMCNT);
        udma_spis_1_csr.wo(utra::udma_spis_1::REG_SPIS_DMCNT, foo);
        let bar = udma_spis_1_csr.rf(utra::udma_spis_1::REG_SPIS_DMCNT_CFGDMCNT);
        udma_spis_1_csr.rmwf(utra::udma_spis_1::REG_SPIS_DMCNT_CFGDMCNT, bar);
        let mut baz = udma_spis_1_csr.zf(utra::udma_spis_1::REG_SPIS_DMCNT_CFGDMCNT, bar);
        baz |= udma_spis_1_csr.ms(utra::udma_spis_1::REG_SPIS_DMCNT_CFGDMCNT, 1);
        udma_spis_1_csr.wfo(utra::udma_spis_1::REG_SPIS_DMCNT_CFGDMCNT, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_udma_adc_csr() {
        use super::*;
        let mut udma_adc_csr = CSR::new(HW_UDMA_ADC_BASE as *mut u32);
  }

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

        let foo = aes_csr.r(utra::aes::SFR_OPTLTX);
        aes_csr.wo(utra::aes::SFR_OPTLTX, foo);
        let bar = aes_csr.rf(utra::aes::SFR_OPTLTX_SFR_OPTLTX);
        aes_csr.rmwf(utra::aes::SFR_OPTLTX_SFR_OPTLTX, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_OPTLTX_SFR_OPTLTX, bar);
        baz |= aes_csr.ms(utra::aes::SFR_OPTLTX_SFR_OPTLTX, 1);
        aes_csr.wfo(utra::aes::SFR_OPTLTX_SFR_OPTLTX, baz);

        let foo = aes_csr.r(utra::aes::SFR_MASKSEED);
        aes_csr.wo(utra::aes::SFR_MASKSEED, foo);
        let bar = aes_csr.rf(utra::aes::SFR_MASKSEED_SFR_MASKSEED);
        aes_csr.rmwf(utra::aes::SFR_MASKSEED_SFR_MASKSEED, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_MASKSEED_SFR_MASKSEED, bar);
        baz |= aes_csr.ms(utra::aes::SFR_MASKSEED_SFR_MASKSEED, 1);
        aes_csr.wfo(utra::aes::SFR_MASKSEED_SFR_MASKSEED, baz);

        let foo = aes_csr.r(utra::aes::SFR_MASKSEEDAR);
        aes_csr.wo(utra::aes::SFR_MASKSEEDAR, foo);
        let bar = aes_csr.rf(utra::aes::SFR_MASKSEEDAR_SFR_MASKSEEDAR);
        aes_csr.rmwf(utra::aes::SFR_MASKSEEDAR_SFR_MASKSEEDAR, bar);
        let mut baz = aes_csr.zf(utra::aes::SFR_MASKSEEDAR_SFR_MASKSEEDAR, bar);
        baz |= aes_csr.ms(utra::aes::SFR_MASKSEEDAR_SFR_MASKSEEDAR, 1);
        aes_csr.wfo(utra::aes::SFR_MASKSEEDAR_SFR_MASKSEEDAR, baz);

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
        let bar = combohash_csr.rf(utra::combohash::SFR_FR_CHKDONE);
        combohash_csr.rmwf(utra::combohash::SFR_FR_CHKDONE, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_FR_CHKDONE, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_FR_CHKDONE, 1);
        combohash_csr.wfo(utra::combohash::SFR_FR_CHKDONE, baz);
        let bar = combohash_csr.rf(utra::combohash::SFR_FR_CHKPASS);
        combohash_csr.rmwf(utra::combohash::SFR_FR_CHKPASS, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_FR_CHKPASS, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_FR_CHKPASS, 1);
        combohash_csr.wfo(utra::combohash::SFR_FR_CHKPASS, baz);

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

        let foo = combohash_csr.r(utra::combohash::SFR_OPT3);
        combohash_csr.wo(utra::combohash::SFR_OPT3, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_OPT3_SFR_OPT3);
        combohash_csr.rmwf(utra::combohash::SFR_OPT3_SFR_OPT3, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_OPT3_SFR_OPT3, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_OPT3_SFR_OPT3, 1);
        combohash_csr.wfo(utra::combohash::SFR_OPT3_SFR_OPT3, baz);

        let foo = combohash_csr.r(utra::combohash::SFR_BLKT0);
        combohash_csr.wo(utra::combohash::SFR_BLKT0, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_BLKT0_SFR_BLKT0);
        combohash_csr.rmwf(utra::combohash::SFR_BLKT0_SFR_BLKT0, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_BLKT0_SFR_BLKT0, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_BLKT0_SFR_BLKT0, 1);
        combohash_csr.wfo(utra::combohash::SFR_BLKT0_SFR_BLKT0, baz);

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

        let foo = combohash_csr.r(utra::combohash::SFR_SEGPTR_SEGID_HOUT2);
        combohash_csr.wo(utra::combohash::SFR_SEGPTR_SEGID_HOUT2, foo);
        let bar = combohash_csr.rf(utra::combohash::SFR_SEGPTR_SEGID_HOUT2_SEGID_HOUT2);
        combohash_csr.rmwf(utra::combohash::SFR_SEGPTR_SEGID_HOUT2_SEGID_HOUT2, bar);
        let mut baz = combohash_csr.zf(utra::combohash::SFR_SEGPTR_SEGID_HOUT2_SEGID_HOUT2, bar);
        baz |= combohash_csr.ms(utra::combohash::SFR_SEGPTR_SEGID_HOUT2_SEGID_HOUT2, 1);
        combohash_csr.wfo(utra::combohash::SFR_SEGPTR_SEGID_HOUT2_SEGID_HOUT2, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_pke_csr() {
        use super::*;
        let mut pke_csr = CSR::new(HW_PKE_BASE as *mut u32);

        let foo = pke_csr.r(utra::pke::SFR_CRFUNC);
        pke_csr.wo(utra::pke::SFR_CRFUNC, foo);
        let bar = pke_csr.rf(utra::pke::SFR_CRFUNC_CR_FUNC);
        pke_csr.rmwf(utra::pke::SFR_CRFUNC_CR_FUNC, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_CRFUNC_CR_FUNC, bar);
        baz |= pke_csr.ms(utra::pke::SFR_CRFUNC_CR_FUNC, 1);
        pke_csr.wfo(utra::pke::SFR_CRFUNC_CR_FUNC, baz);
        let bar = pke_csr.rf(utra::pke::SFR_CRFUNC_CR_PCOREIR);
        pke_csr.rmwf(utra::pke::SFR_CRFUNC_CR_PCOREIR, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_CRFUNC_CR_PCOREIR, bar);
        baz |= pke_csr.ms(utra::pke::SFR_CRFUNC_CR_PCOREIR, 1);
        pke_csr.wfo(utra::pke::SFR_CRFUNC_CR_PCOREIR, baz);

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

        let foo = pke_csr.r(utra::pke::SFR_OPTRW);
        pke_csr.wo(utra::pke::SFR_OPTRW, foo);
        let bar = pke_csr.rf(utra::pke::SFR_OPTRW_SFR_OPTRW);
        pke_csr.rmwf(utra::pke::SFR_OPTRW_SFR_OPTRW, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_OPTRW_SFR_OPTRW, bar);
        baz |= pke_csr.ms(utra::pke::SFR_OPTRW_SFR_OPTRW, 1);
        pke_csr.wfo(utra::pke::SFR_OPTRW_SFR_OPTRW, baz);

        let foo = pke_csr.r(utra::pke::SFR_OPTLTX);
        pke_csr.wo(utra::pke::SFR_OPTLTX, foo);
        let bar = pke_csr.rf(utra::pke::SFR_OPTLTX_SFR_OPTLTX);
        pke_csr.rmwf(utra::pke::SFR_OPTLTX_SFR_OPTLTX, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_OPTLTX_SFR_OPTLTX, bar);
        baz |= pke_csr.ms(utra::pke::SFR_OPTLTX_SFR_OPTLTX, 1);
        pke_csr.wfo(utra::pke::SFR_OPTLTX_SFR_OPTLTX, baz);

        let foo = pke_csr.r(utra::pke::SFR_OPTMASK);
        pke_csr.wo(utra::pke::SFR_OPTMASK, foo);
        let bar = pke_csr.rf(utra::pke::SFR_OPTMASK_SFR_OPTMASK);
        pke_csr.rmwf(utra::pke::SFR_OPTMASK_SFR_OPTMASK, bar);
        let mut baz = pke_csr.zf(utra::pke::SFR_OPTMASK_SFR_OPTMASK, bar);
        baz |= pke_csr.ms(utra::pke::SFR_OPTMASK_SFR_OPTMASK, 1);
        pke_csr.wfo(utra::pke::SFR_OPTMASK_SFR_OPTMASK, baz);

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

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_FRACERR);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_FRACERR, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_FRACERR_FR_ACERR);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_FRACERR_FR_ACERR, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_FRACERR_FR_ACERR, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_FRACERR_FR_ACERR, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_FRACERR_FR_ACERR, baz);

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_TICKCNT);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_TICKCNT, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_TICKCNT_SFR_TICKCNT, baz);

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

        let foo = sce_glbsfr_csr.r(utra::sce_glbsfr::SFR_TS);
        sce_glbsfr_csr.wo(utra::sce_glbsfr::SFR_TS, foo);
        let bar = sce_glbsfr_csr.rf(utra::sce_glbsfr::SFR_TS_CR_TS);
        sce_glbsfr_csr.rmwf(utra::sce_glbsfr::SFR_TS_CR_TS, bar);
        let mut baz = sce_glbsfr_csr.zf(utra::sce_glbsfr::SFR_TS_CR_TS, bar);
        baz |= sce_glbsfr_csr.ms(utra::sce_glbsfr::SFR_TS_CR_TS, 1);
        sce_glbsfr_csr.wfo(utra::sce_glbsfr::SFR_TS_CR_TS, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_trng_csr() {
        use super::*;
        let mut trng_csr = CSR::new(HW_TRNG_BASE as *mut u32);

        let foo = trng_csr.r(utra::trng::SFR_CRSRC);
        trng_csr.wo(utra::trng::SFR_CRSRC, foo);
        let bar = trng_csr.rf(utra::trng::SFR_CRSRC_SFR_CRSRC);
        trng_csr.rmwf(utra::trng::SFR_CRSRC_SFR_CRSRC, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_CRSRC_SFR_CRSRC, bar);
        baz |= trng_csr.ms(utra::trng::SFR_CRSRC_SFR_CRSRC, 1);
        trng_csr.wfo(utra::trng::SFR_CRSRC_SFR_CRSRC, baz);

        let foo = trng_csr.r(utra::trng::SFR_CRANA);
        trng_csr.wo(utra::trng::SFR_CRANA, foo);
        let bar = trng_csr.rf(utra::trng::SFR_CRANA_SFR_CRANA);
        trng_csr.rmwf(utra::trng::SFR_CRANA_SFR_CRANA, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_CRANA_SFR_CRANA, bar);
        baz |= trng_csr.ms(utra::trng::SFR_CRANA_SFR_CRANA, 1);
        trng_csr.wfo(utra::trng::SFR_CRANA_SFR_CRANA, baz);

        let foo = trng_csr.r(utra::trng::SFR_PP);
        trng_csr.wo(utra::trng::SFR_PP, foo);
        let bar = trng_csr.rf(utra::trng::SFR_PP_SFR_PP);
        trng_csr.rmwf(utra::trng::SFR_PP_SFR_PP, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_PP_SFR_PP, bar);
        baz |= trng_csr.ms(utra::trng::SFR_PP_SFR_PP, 1);
        trng_csr.wfo(utra::trng::SFR_PP_SFR_PP, baz);

        let foo = trng_csr.r(utra::trng::SFR_OPT);
        trng_csr.wo(utra::trng::SFR_OPT, foo);
        let bar = trng_csr.rf(utra::trng::SFR_OPT_SFR_OPT);
        trng_csr.rmwf(utra::trng::SFR_OPT_SFR_OPT, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_OPT_SFR_OPT, bar);
        baz |= trng_csr.ms(utra::trng::SFR_OPT_SFR_OPT, 1);
        trng_csr.wfo(utra::trng::SFR_OPT_SFR_OPT, baz);

        let foo = trng_csr.r(utra::trng::SFR_SR);
        trng_csr.wo(utra::trng::SFR_SR, foo);
        let bar = trng_csr.rf(utra::trng::SFR_SR_SR_RNG);
        trng_csr.rmwf(utra::trng::SFR_SR_SR_RNG, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_SR_SR_RNG, bar);
        baz |= trng_csr.ms(utra::trng::SFR_SR_SR_RNG, 1);
        trng_csr.wfo(utra::trng::SFR_SR_SR_RNG, baz);

        let foo = trng_csr.r(utra::trng::SFR_AR_GEN);
        trng_csr.wo(utra::trng::SFR_AR_GEN, foo);
        let bar = trng_csr.rf(utra::trng::SFR_AR_GEN_SFR_AR_GEN);
        trng_csr.rmwf(utra::trng::SFR_AR_GEN_SFR_AR_GEN, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_AR_GEN_SFR_AR_GEN, bar);
        baz |= trng_csr.ms(utra::trng::SFR_AR_GEN_SFR_AR_GEN, 1);
        trng_csr.wfo(utra::trng::SFR_AR_GEN_SFR_AR_GEN, baz);

        let foo = trng_csr.r(utra::trng::SFR_FR);
        trng_csr.wo(utra::trng::SFR_FR, foo);
        let bar = trng_csr.rf(utra::trng::SFR_FR_SFR_FR);
        trng_csr.rmwf(utra::trng::SFR_FR_SFR_FR, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_FR_SFR_FR, bar);
        baz |= trng_csr.ms(utra::trng::SFR_FR_SFR_FR, 1);
        trng_csr.wfo(utra::trng::SFR_FR_SFR_FR, baz);

        let foo = trng_csr.r(utra::trng::SFR_DRPSZ);
        trng_csr.wo(utra::trng::SFR_DRPSZ, foo);
        let bar = trng_csr.rf(utra::trng::SFR_DRPSZ_SFR_DRPSZ);
        trng_csr.rmwf(utra::trng::SFR_DRPSZ_SFR_DRPSZ, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_DRPSZ_SFR_DRPSZ, bar);
        baz |= trng_csr.ms(utra::trng::SFR_DRPSZ_SFR_DRPSZ, 1);
        trng_csr.wfo(utra::trng::SFR_DRPSZ_SFR_DRPSZ, baz);

        let foo = trng_csr.r(utra::trng::SFR_DRGEN);
        trng_csr.wo(utra::trng::SFR_DRGEN, foo);
        let bar = trng_csr.rf(utra::trng::SFR_DRGEN_SFR_DRGEN);
        trng_csr.rmwf(utra::trng::SFR_DRGEN_SFR_DRGEN, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_DRGEN_SFR_DRGEN, bar);
        baz |= trng_csr.ms(utra::trng::SFR_DRGEN_SFR_DRGEN, 1);
        trng_csr.wfo(utra::trng::SFR_DRGEN_SFR_DRGEN, baz);

        let foo = trng_csr.r(utra::trng::SFR_DRRESEED);
        trng_csr.wo(utra::trng::SFR_DRRESEED, foo);
        let bar = trng_csr.rf(utra::trng::SFR_DRRESEED_SFR_DRRESEED);
        trng_csr.rmwf(utra::trng::SFR_DRRESEED_SFR_DRRESEED, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_DRRESEED_SFR_DRRESEED, bar);
        baz |= trng_csr.ms(utra::trng::SFR_DRRESEED_SFR_DRRESEED, 1);
        trng_csr.wfo(utra::trng::SFR_DRRESEED_SFR_DRRESEED, baz);

        let foo = trng_csr.r(utra::trng::SFR_BUF);
        trng_csr.wo(utra::trng::SFR_BUF, foo);
        let bar = trng_csr.rf(utra::trng::SFR_BUF_SFR_BUF);
        trng_csr.rmwf(utra::trng::SFR_BUF_SFR_BUF, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_BUF_SFR_BUF, bar);
        baz |= trng_csr.ms(utra::trng::SFR_BUF_SFR_BUF, 1);
        trng_csr.wfo(utra::trng::SFR_BUF_SFR_BUF, baz);

        let foo = trng_csr.r(utra::trng::SFR_CHAIN_RNGCHAINEN0);
        trng_csr.wo(utra::trng::SFR_CHAIN_RNGCHAINEN0, foo);
        let bar = trng_csr.rf(utra::trng::SFR_CHAIN_RNGCHAINEN0_RNGCHAINEN0);
        trng_csr.rmwf(utra::trng::SFR_CHAIN_RNGCHAINEN0_RNGCHAINEN0, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_CHAIN_RNGCHAINEN0_RNGCHAINEN0, bar);
        baz |= trng_csr.ms(utra::trng::SFR_CHAIN_RNGCHAINEN0_RNGCHAINEN0, 1);
        trng_csr.wfo(utra::trng::SFR_CHAIN_RNGCHAINEN0_RNGCHAINEN0, baz);

        let foo = trng_csr.r(utra::trng::SFR_CHAIN_RNGCHAINEN1);
        trng_csr.wo(utra::trng::SFR_CHAIN_RNGCHAINEN1, foo);
        let bar = trng_csr.rf(utra::trng::SFR_CHAIN_RNGCHAINEN1_RNGCHAINEN1);
        trng_csr.rmwf(utra::trng::SFR_CHAIN_RNGCHAINEN1_RNGCHAINEN1, bar);
        let mut baz = trng_csr.zf(utra::trng::SFR_CHAIN_RNGCHAINEN1_RNGCHAINEN1, bar);
        baz |= trng_csr.ms(utra::trng::SFR_CHAIN_RNGCHAINEN1_RNGCHAINEN1, 1);
        trng_csr.wfo(utra::trng::SFR_CHAIN_RNGCHAINEN1_RNGCHAINEN1, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_alu_csr() {
        use super::*;
        let mut alu_csr = CSR::new(HW_ALU_BASE as *mut u32);
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
  }

    #[test]
    #[ignore]
    fn compile_check_timer_intf_csr() {
        use super::*;
        let mut timer_intf_csr = CSR::new(HW_TIMER_INTF_BASE as *mut u32);
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

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_SEED);
        sysctrl_csr.wo(utra::sysctrl::SFR_SEED, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_SEED_SFR_SEED);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_SEED_SFR_SEED, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_SEED_SFR_SEED, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_SEED_SFR_SEED, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_SEED_SFR_SEED, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_SEEDAR);
        sysctrl_csr.wo(utra::sysctrl::SFR_SEEDAR, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_SEEDAR_SFR_SEEDAR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_SEEDAR_SFR_SEEDAR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_SEEDAR_SFR_SEEDAR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_SEEDAR_SFR_SEEDAR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_SEEDAR_SFR_SEEDAR, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUSEL0);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUSEL0, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUSEL0_SFR_CGUSEL0, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0_CFGFDCR_0_4_0);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0_CFGFDCR_0_4_0, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0_CFGFDCR_0_4_0, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0_CFGFDCR_0_4_0, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_0_CFGFDCR_0_4_0, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1_CFGFDCR_0_4_1);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1_CFGFDCR_0_4_1, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1_CFGFDCR_0_4_1, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1_CFGFDCR_0_4_1, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_1_CFGFDCR_0_4_1, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2_CFGFDCR_0_4_2);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2_CFGFDCR_0_4_2, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2_CFGFDCR_0_4_2, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2_CFGFDCR_0_4_2, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_2_CFGFDCR_0_4_2, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3_CFGFDCR_0_4_3);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3_CFGFDCR_0_4_3, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3_CFGFDCR_0_4_3, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3_CFGFDCR_0_4_3, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_3_CFGFDCR_0_4_3, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4_CFGFDCR_0_4_4);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4_CFGFDCR_0_4_4, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4_CFGFDCR_0_4_4, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4_CFGFDCR_0_4_4, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFD_CFGFDCR_0_4_4_CFGFDCR_0_4_4, baz);

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFDAO);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFDAO, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFDAO_CFGFDCR);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFDAO_CFGFDCR, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFDAO_CFGFDCR, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFDAO_CFGFDCR, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFDAO_CFGFDCR, baz);

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

        let foo = sysctrl_csr.r(utra::sysctrl::SFR_CGUFDPKE);
        sysctrl_csr.wo(utra::sysctrl::SFR_CGUFDPKE, foo);
        let bar = sysctrl_csr.rf(utra::sysctrl::SFR_CGUFDPKE_SFR_CGUFDPKE);
        sysctrl_csr.rmwf(utra::sysctrl::SFR_CGUFDPKE_SFR_CGUFDPKE, bar);
        let mut baz = sysctrl_csr.zf(utra::sysctrl::SFR_CGUFDPKE_SFR_CGUFDPKE, bar);
        baz |= sysctrl_csr.ms(utra::sysctrl::SFR_CGUFDPKE_SFR_CGUFDPKE, 1);
        sysctrl_csr.wfo(utra::sysctrl::SFR_CGUFDPKE_SFR_CGUFDPKE, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL8);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL8, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL8_CRAFSEL8);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL8_CRAFSEL8, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL8_CRAFSEL8, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL8_CRAFSEL8, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL8_CRAFSEL8, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL9);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL9, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL9_CRAFSEL9);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL9_CRAFSEL9, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL9_CRAFSEL9, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL9_CRAFSEL9, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL9_CRAFSEL9, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL10);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL10, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL10_CRAFSEL10);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL10_CRAFSEL10, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL10_CRAFSEL10, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL10_CRAFSEL10, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL10_CRAFSEL10, baz);

        let foo = iox_csr.r(utra::iox::SFR_AFSEL_CRAFSEL11);
        iox_csr.wo(utra::iox::SFR_AFSEL_CRAFSEL11, foo);
        let bar = iox_csr.rf(utra::iox::SFR_AFSEL_CRAFSEL11_CRAFSEL11);
        iox_csr.rmwf(utra::iox::SFR_AFSEL_CRAFSEL11_CRAFSEL11, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_AFSEL_CRAFSEL11_CRAFSEL11, bar);
        baz |= iox_csr.ms(utra::iox::SFR_AFSEL_CRAFSEL11_CRAFSEL11, 1);
        iox_csr.wfo(utra::iox::SFR_AFSEL_CRAFSEL11_CRAFSEL11, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_GPIOOUT_CRGO4);
        iox_csr.wo(utra::iox::SFR_GPIOOUT_CRGO4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOUT_CRGO4_CRGO4);
        iox_csr.rmwf(utra::iox::SFR_GPIOOUT_CRGO4_CRGO4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOUT_CRGO4_CRGO4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOUT_CRGO4_CRGO4, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOUT_CRGO4_CRGO4, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOUT_CRGO5);
        iox_csr.wo(utra::iox::SFR_GPIOOUT_CRGO5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOUT_CRGO5_CRGO5);
        iox_csr.rmwf(utra::iox::SFR_GPIOOUT_CRGO5_CRGO5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOUT_CRGO5_CRGO5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOUT_CRGO5_CRGO5, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOUT_CRGO5_CRGO5, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_GPIOOE_CRGOE4);
        iox_csr.wo(utra::iox::SFR_GPIOOE_CRGOE4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOE_CRGOE4_CRGOE4);
        iox_csr.rmwf(utra::iox::SFR_GPIOOE_CRGOE4_CRGOE4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOE_CRGOE4_CRGOE4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOE_CRGOE4_CRGOE4, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOE_CRGOE4_CRGOE4, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOOE_CRGOE5);
        iox_csr.wo(utra::iox::SFR_GPIOOE_CRGOE5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOOE_CRGOE5_CRGOE5);
        iox_csr.rmwf(utra::iox::SFR_GPIOOE_CRGOE5_CRGOE5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOOE_CRGOE5_CRGOE5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOOE_CRGOE5_CRGOE5, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOOE_CRGOE5_CRGOE5, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_GPIOPU_CRGPU4);
        iox_csr.wo(utra::iox::SFR_GPIOPU_CRGPU4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOPU_CRGPU4_CRGPU4);
        iox_csr.rmwf(utra::iox::SFR_GPIOPU_CRGPU4_CRGPU4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOPU_CRGPU4_CRGPU4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOPU_CRGPU4_CRGPU4, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOPU_CRGPU4_CRGPU4, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOPU_CRGPU5);
        iox_csr.wo(utra::iox::SFR_GPIOPU_CRGPU5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOPU_CRGPU5_CRGPU5);
        iox_csr.rmwf(utra::iox::SFR_GPIOPU_CRGPU5_CRGPU5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOPU_CRGPU5_CRGPU5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOPU_CRGPU5_CRGPU5, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOPU_CRGPU5_CRGPU5, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_GPIOIN_SRGI4);
        iox_csr.wo(utra::iox::SFR_GPIOIN_SRGI4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOIN_SRGI4_SRGI4);
        iox_csr.rmwf(utra::iox::SFR_GPIOIN_SRGI4_SRGI4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOIN_SRGI4_SRGI4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOIN_SRGI4_SRGI4, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOIN_SRGI4_SRGI4, baz);

        let foo = iox_csr.r(utra::iox::SFR_GPIOIN_SRGI5);
        iox_csr.wo(utra::iox::SFR_GPIOIN_SRGI5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_GPIOIN_SRGI5_SRGI5);
        iox_csr.rmwf(utra::iox::SFR_GPIOIN_SRGI5_SRGI5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_GPIOIN_SRGI5_SRGI5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_GPIOIN_SRGI5_SRGI5, 1);
        iox_csr.wfo(utra::iox::SFR_GPIOIN_SRGI5_SRGI5, baz);

        let foo = iox_csr.r(utra::iox::SFR_PIOSEL);
        iox_csr.wo(utra::iox::SFR_PIOSEL, foo);
        let bar = iox_csr.rf(utra::iox::SFR_PIOSEL_PIOSEL);
        iox_csr.rmwf(utra::iox::SFR_PIOSEL_PIOSEL, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_PIOSEL_PIOSEL, bar);
        baz |= iox_csr.ms(utra::iox::SFR_PIOSEL_PIOSEL, 1);
        iox_csr.wfo(utra::iox::SFR_PIOSEL_PIOSEL, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4);
        iox_csr.wo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4_CR_CFG_SCHMSEL4);
        iox_csr.rmwf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4_CR_CFG_SCHMSEL4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4_CR_CFG_SCHMSEL4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4_CR_CFG_SCHMSEL4, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL4_CR_CFG_SCHMSEL4, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5);
        iox_csr.wo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5_CR_CFG_SCHMSEL5);
        iox_csr.rmwf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5_CR_CFG_SCHMSEL5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5_CR_CFG_SCHMSEL5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5_CR_CFG_SCHMSEL5, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SCHM_CR_CFG_SCHMSEL5_CR_CFG_SCHMSEL5, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4);
        iox_csr.wo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4_CR_CFG_SLEWSLOW4);
        iox_csr.rmwf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4_CR_CFG_SLEWSLOW4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4_CR_CFG_SLEWSLOW4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4_CR_CFG_SLEWSLOW4, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW4_CR_CFG_SLEWSLOW4, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5);
        iox_csr.wo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5_CR_CFG_SLEWSLOW5);
        iox_csr.rmwf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5_CR_CFG_SLEWSLOW5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5_CR_CFG_SLEWSLOW5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5_CR_CFG_SLEWSLOW5, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_SLEW_CR_CFG_SLEWSLOW5_CR_CFG_SLEWSLOW5, baz);

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

        let foo = iox_csr.r(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4);
        iox_csr.wo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4_CR_CFG_DRVSEL4);
        iox_csr.rmwf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4_CR_CFG_DRVSEL4, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4_CR_CFG_DRVSEL4, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4_CR_CFG_DRVSEL4, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL4_CR_CFG_DRVSEL4, baz);

        let foo = iox_csr.r(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5);
        iox_csr.wo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5, foo);
        let bar = iox_csr.rf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5_CR_CFG_DRVSEL5);
        iox_csr.rmwf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5_CR_CFG_DRVSEL5, bar);
        let mut baz = iox_csr.zf(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5_CR_CFG_DRVSEL5, bar);
        baz |= iox_csr.ms(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5_CR_CFG_DRVSEL5, 1);
        iox_csr.wfo(utra::iox::SFR_CFG_DRVSEL_CR_CFG_DRVSEL5_CR_CFG_DRVSEL5, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_pwm_csr() {
        use super::*;
        let mut pwm_csr = CSR::new(HW_PWM_BASE as *mut u32);
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
    fn compile_check_rp_pio_csr() {
        use super::*;
        let mut rp_pio_csr = CSR::new(HW_RP_PIO_BASE as *mut u32);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_CTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_CTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_CTRL_EN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_CTRL_EN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_CTRL_EN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_CTRL_EN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_CTRL_EN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_CTRL_RESTART);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_CTRL_RESTART, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_CTRL_RESTART, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_CTRL_RESTART, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_CTRL_RESTART, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_CTRL_CLKDIV_RESTART);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_CTRL_CLKDIV_RESTART, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_CTRL_CLKDIV_RESTART, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_CTRL_CLKDIV_RESTART, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_CTRL_CLKDIV_RESTART, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_FSTAT);
        rp_pio_csr.wo(utra::rp_pio::SFR_FSTAT, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_RX_FULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_RX_FULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_RX_FULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_RX_FULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_RX_FULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_CONSTANT0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_CONSTANT0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_CONSTANT0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_CONSTANT0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_CONSTANT0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_RX_EMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_RX_EMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_RX_EMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_RX_EMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_RX_EMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_CONSTANT1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_CONSTANT1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_CONSTANT1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_CONSTANT1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_CONSTANT1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_TX_FULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_TX_FULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_TX_FULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_TX_FULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_TX_FULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_CONSTANT2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_CONSTANT2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_CONSTANT2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_CONSTANT2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_CONSTANT2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_TX_EMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_TX_EMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_TX_EMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_TX_EMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_TX_EMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FSTAT_CONSTANT3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FSTAT_CONSTANT3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FSTAT_CONSTANT3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FSTAT_CONSTANT3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FSTAT_CONSTANT3, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_FDEBUG);
        rp_pio_csr.wo(utra::rp_pio::SFR_FDEBUG, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_RXSTALL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_RXSTALL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_RXSTALL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_RXSTALL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_RXSTALL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_NC_DBG3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_NC_DBG3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_NC_DBG3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_NC_DBG3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_NC_DBG3, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_RXUNDER);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_RXUNDER, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_RXUNDER, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_RXUNDER, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_RXUNDER, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_NC_DBG2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_NC_DBG2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_NC_DBG2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_NC_DBG2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_NC_DBG2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_TXOVER);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_TXOVER, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_TXOVER, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_TXOVER, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_TXOVER, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_NC_DBG1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_NC_DBG1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_NC_DBG1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_NC_DBG1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_NC_DBG1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_TXSTALL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_TXSTALL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_TXSTALL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_TXSTALL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_TXSTALL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FDEBUG_NC_DBG0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FDEBUG_NC_DBG0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FDEBUG_NC_DBG0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FDEBUG_NC_DBG0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FDEBUG_NC_DBG0, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_FLEVEL);
        rp_pio_csr.wo(utra::rp_pio::SFR_FLEVEL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_TX_LEVEL0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_TX_LEVEL0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_RX_LEVEL0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_RX_LEVEL0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_TX_LEVEL1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_TX_LEVEL1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_RX_LEVEL1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_RX_LEVEL1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT3, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_TX_LEVEL2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_TX_LEVEL2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT4);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT4, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT4, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT4, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT4, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_RX_LEVEL2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_RX_LEVEL2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT5);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT5, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT5, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT5, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT5, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_TX_LEVEL3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_TX_LEVEL3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_TX_LEVEL3, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT6);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT6, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT6, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT6, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT6, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_RX_LEVEL3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_RX_LEVEL3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_RX_LEVEL3, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FLEVEL_CONSTANT7);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FLEVEL_CONSTANT7, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FLEVEL_CONSTANT7, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FLEVEL_CONSTANT7, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FLEVEL_CONSTANT7, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_TXF0);
        rp_pio_csr.wo(utra::rp_pio::SFR_TXF0, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_TXF0_FDIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_TXF0_FDIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_TXF0_FDIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_TXF0_FDIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_TXF0_FDIN, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_TXF1);
        rp_pio_csr.wo(utra::rp_pio::SFR_TXF1, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_TXF1_FDIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_TXF1_FDIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_TXF1_FDIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_TXF1_FDIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_TXF1_FDIN, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_TXF2);
        rp_pio_csr.wo(utra::rp_pio::SFR_TXF2, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_TXF2_FDIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_TXF2_FDIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_TXF2_FDIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_TXF2_FDIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_TXF2_FDIN, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_TXF3);
        rp_pio_csr.wo(utra::rp_pio::SFR_TXF3, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_TXF3_FDIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_TXF3_FDIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_TXF3_FDIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_TXF3_FDIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_TXF3_FDIN, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_RXF0);
        rp_pio_csr.wo(utra::rp_pio::SFR_RXF0, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_RXF0_PDOUT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_RXF0_PDOUT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_RXF0_PDOUT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_RXF0_PDOUT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_RXF0_PDOUT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_RXF1);
        rp_pio_csr.wo(utra::rp_pio::SFR_RXF1, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_RXF1_PDOUT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_RXF1_PDOUT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_RXF1_PDOUT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_RXF1_PDOUT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_RXF1_PDOUT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_RXF2);
        rp_pio_csr.wo(utra::rp_pio::SFR_RXF2, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_RXF2_PDOUT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_RXF2_PDOUT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_RXF2_PDOUT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_RXF2_PDOUT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_RXF2_PDOUT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_RXF3);
        rp_pio_csr.wo(utra::rp_pio::SFR_RXF3, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_RXF3_PDOUT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_RXF3_PDOUT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_RXF3_PDOUT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_RXF3_PDOUT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_RXF3_PDOUT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ_SFR_IRQ);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ_SFR_IRQ, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ_SFR_IRQ, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ_SFR_IRQ, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ_SFR_IRQ, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ_FORCE);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ_FORCE, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ_FORCE_SFR_IRQ_FORCE, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SYNC_BYPASS);
        rp_pio_csr.wo(utra::rp_pio::SFR_SYNC_BYPASS, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_DBG_PADOUT);
        rp_pio_csr.wo(utra::rp_pio::SFR_DBG_PADOUT, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_DBG_PADOUT_SFR_DBG_PADOUT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_DBG_PADOE);
        rp_pio_csr.wo(utra::rp_pio::SFR_DBG_PADOE, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_DBG_PADOE_SFR_DBG_PADOE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_DBG_PADOE_SFR_DBG_PADOE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_DBG_PADOE_SFR_DBG_PADOE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_DBG_PADOE_SFR_DBG_PADOE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_DBG_PADOE_SFR_DBG_PADOE, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_DBG_CFGINFO);
        rp_pio_csr.wo(utra::rp_pio::SFR_DBG_CFGINFO, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_DBG_CFGINFO_CONSTANT2, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM0);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM0, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM0_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM0_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM0_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM0_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM0_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM1);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM1, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM1_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM1_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM1_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM1_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM1_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM2);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM2, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM2_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM2_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM2_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM2_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM2_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM3);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM3, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM3_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM3_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM3_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM3_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM3_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM4);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM4, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM4_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM4_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM4_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM4_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM4_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM5);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM5, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM5_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM5_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM5_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM5_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM5_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM6);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM6, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM6_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM6_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM6_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM6_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM6_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM7);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM7, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM7_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM7_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM7_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM7_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM7_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM8);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM8, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM8_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM8_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM8_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM8_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM8_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM9);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM9, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM9_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM9_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM9_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM9_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM9_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM10);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM10, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM10_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM10_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM10_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM10_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM10_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM11);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM11, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM11_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM11_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM11_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM11_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM11_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM12);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM12, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM12_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM12_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM12_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM12_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM12_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM13);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM13, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM13_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM13_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM13_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM13_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM13_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM14);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM14, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM14_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM14_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM14_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM14_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM14_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM15);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM15, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM15_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM15_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM15_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM15_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM15_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM16);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM16, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM16_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM16_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM16_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM16_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM16_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM17);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM17, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM17_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM17_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM17_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM17_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM17_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM18);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM18, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM18_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM18_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM18_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM18_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM18_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM19);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM19, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM19_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM19_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM19_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM19_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM19_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM20);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM20, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM20_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM20_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM20_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM20_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM20_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM21);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM21, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM21_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM21_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM21_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM21_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM21_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM22);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM22, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM22_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM22_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM22_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM22_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM22_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM23);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM23, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM23_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM23_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM23_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM23_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM23_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM24);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM24, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM24_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM24_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM24_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM24_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM24_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM25);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM25, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM25_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM25_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM25_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM25_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM25_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM26);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM26, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM26_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM26_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM26_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM26_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM26_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM27);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM27, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM27_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM27_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM27_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM27_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM27_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM28);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM28, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM28_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM28_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM28_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM28_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM28_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM29);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM29, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM29_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM29_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM29_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM29_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM29_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM30);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM30, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM30_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM30_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM30_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM30_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM30_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INSTR_MEM31);
        rp_pio_csr.wo(utra::rp_pio::SFR_INSTR_MEM31, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INSTR_MEM31_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INSTR_MEM31_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INSTR_MEM31_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INSTR_MEM31_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INSTR_MEM31_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM0_CLKDIV);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM0_CLKDIV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_CLKDIV_UNUSED_DIV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_CLKDIV_UNUSED_DIV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_CLKDIV_UNUSED_DIV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_CLKDIV_UNUSED_DIV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_CLKDIV_UNUSED_DIV, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_CLKDIV_DIV_FRAC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_CLKDIV_DIV_FRAC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_CLKDIV_DIV_FRAC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_CLKDIV_DIV_FRAC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_CLKDIV_DIV_FRAC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_CLKDIV_DIV_INT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_CLKDIV_DIV_INT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_CLKDIV_DIV_INT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_CLKDIV_DIV_INT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_CLKDIV_DIV_INT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM0_EXECCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM0_EXECCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_N);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_N, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_N, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_N, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_N, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_STATUS_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_RESVD_EXEC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_RESVD_EXEC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_RESVD_EXEC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_RESVD_EXEC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_PEND);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_PEND, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_PEND, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_PEND, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_PEND, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_EN_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_OUT_EN_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_JMP_PIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_JMP_PIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_JMP_PIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_JMP_PIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_JMP_PIN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM0_SHIFTCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM0_SHIFTCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_TX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_TX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_TX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_TX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_TX, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_RX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_RX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_RX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_RX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_SHIFTCTRL_JOIN_RX, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM0_ADDR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM0_ADDR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_ADDR_PC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_ADDR_PC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_ADDR_PC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_ADDR_PC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_ADDR_PC, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM0_INSTR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM0_INSTR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_INSTR_IMM_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_INSTR_IMM_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_INSTR_IMM_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_INSTR_IMM_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_INSTR_IMM_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM0_PINCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM0_PINCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM1_CLKDIV);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM1_CLKDIV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_CLKDIV_UNUSED_DIV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_CLKDIV_UNUSED_DIV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_CLKDIV_UNUSED_DIV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_CLKDIV_UNUSED_DIV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_CLKDIV_UNUSED_DIV, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_CLKDIV_DIV_FRAC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_CLKDIV_DIV_FRAC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_CLKDIV_DIV_FRAC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_CLKDIV_DIV_FRAC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_CLKDIV_DIV_FRAC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_CLKDIV_DIV_INT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_CLKDIV_DIV_INT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_CLKDIV_DIV_INT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_CLKDIV_DIV_INT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_CLKDIV_DIV_INT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM1_EXECCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM1_EXECCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_N);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_N, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_N, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_N, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_N, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_STATUS_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_RESVD_EXEC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_RESVD_EXEC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_RESVD_EXEC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_RESVD_EXEC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_WRAP_TARGET);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_WRAP_TARGET, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_WRAP_TARGET, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_WRAP_TARGET, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_PEND);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_PEND, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_PEND, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_PEND, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_PEND, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_STICKY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_STICKY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_STICKY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_STICKY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_STICKY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_EN_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_OUT_EN_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_JMP_PIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_JMP_PIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_JMP_PIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_JMP_PIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_JMP_PIN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_SIDE_PINDIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_SIDE_PINDIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_EXECCTRL_EXEC_STALLED_RO1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_EXECCTRL_EXEC_STALLED_RO1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_EXECCTRL_EXEC_STALLED_RO1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_EXECCTRL_EXEC_STALLED_RO1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_EXECCTRL_EXEC_STALLED_RO1, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM1_SHIFTCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM1_SHIFTCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_AUTO_PULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_TX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_TX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_TX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_TX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_TX, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_RX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_RX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_RX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_RX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_SHIFTCTRL_JOIN_RX, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM1_ADDR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM1_ADDR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_ADDR_PC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_ADDR_PC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_ADDR_PC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_ADDR_PC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_ADDR_PC, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM1_INSTR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM1_INSTR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_INSTR_IMM_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_INSTR_IMM_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_INSTR_IMM_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_INSTR_IMM_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_INSTR_IMM_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM1_PINCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM1_PINCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_IN_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_IN_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_IN_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_IN_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SET_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM1_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM2_CLKDIV);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM2_CLKDIV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_CLKDIV_UNUSED_DIV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_CLKDIV_UNUSED_DIV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_CLKDIV_UNUSED_DIV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_CLKDIV_UNUSED_DIV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_CLKDIV_UNUSED_DIV, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_CLKDIV_DIV_FRAC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_CLKDIV_DIV_FRAC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_CLKDIV_DIV_FRAC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_CLKDIV_DIV_FRAC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_CLKDIV_DIV_FRAC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_CLKDIV_DIV_INT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_CLKDIV_DIV_INT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_CLKDIV_DIV_INT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_CLKDIV_DIV_INT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_CLKDIV_DIV_INT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM2_EXECCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM2_EXECCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_N);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_N, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_N, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_N, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_N, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_STATUS_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_RESVD_EXEC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_RESVD_EXEC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_RESVD_EXEC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_RESVD_EXEC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_WRAP_TARGET);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_WRAP_TARGET, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_WRAP_TARGET, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_WRAP_TARGET, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_PEND);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_PEND, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_PEND, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_PEND, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_PEND, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_STICKY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_STICKY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_STICKY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_STICKY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_STICKY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_EN_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_OUT_EN_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_JMP_PIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_JMP_PIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_JMP_PIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_JMP_PIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_JMP_PIN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_SIDE_PINDIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_SIDE_PINDIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_EXECCTRL_EXEC_STALLED_RO2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_EXECCTRL_EXEC_STALLED_RO2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_EXECCTRL_EXEC_STALLED_RO2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_EXECCTRL_EXEC_STALLED_RO2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_EXECCTRL_EXEC_STALLED_RO2, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM2_SHIFTCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM2_SHIFTCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_AUTO_PULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_TX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_TX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_TX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_TX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_TX, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_RX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_RX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_RX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_RX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_SHIFTCTRL_JOIN_RX, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM2_ADDR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM2_ADDR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_ADDR_PC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_ADDR_PC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_ADDR_PC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_ADDR_PC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_ADDR_PC, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM2_INSTR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM2_INSTR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_INSTR_IMM_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_INSTR_IMM_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_INSTR_IMM_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_INSTR_IMM_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_INSTR_IMM_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM2_PINCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM2_PINCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_IN_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_IN_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_IN_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_IN_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SET_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM2_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM3_CLKDIV);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM3_CLKDIV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_CLKDIV_UNUSED_DIV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_CLKDIV_UNUSED_DIV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_CLKDIV_UNUSED_DIV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_CLKDIV_UNUSED_DIV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_CLKDIV_UNUSED_DIV, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_CLKDIV_DIV_FRAC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_CLKDIV_DIV_FRAC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_CLKDIV_DIV_FRAC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_CLKDIV_DIV_FRAC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_CLKDIV_DIV_FRAC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_CLKDIV_DIV_INT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_CLKDIV_DIV_INT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_CLKDIV_DIV_INT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_CLKDIV_DIV_INT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_CLKDIV_DIV_INT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM3_EXECCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM3_EXECCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_N);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_N, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_N, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_N, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_N, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_STATUS_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_RESVD_EXEC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_RESVD_EXEC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_RESVD_EXEC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_RESVD_EXEC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_RESVD_EXEC, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_WRAP_TARGET);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_WRAP_TARGET, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_WRAP_TARGET, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_WRAP_TARGET, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_WRAP_TARGET, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_PEND);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_PEND, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_PEND, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_PEND, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_PEND, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_STICKY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_STICKY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_STICKY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_STICKY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_STICKY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_INLINE_OUT_EN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_EN_SEL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_OUT_EN_SEL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_JMP_PIN);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_JMP_PIN, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_JMP_PIN, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_JMP_PIN, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_JMP_PIN, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_SIDE_PINDIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_SIDE_PINDIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_SIDESET_ENABLE_BIT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_EXECCTRL_EXEC_STALLED_RO3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_EXECCTRL_EXEC_STALLED_RO3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_EXECCTRL_EXEC_STALLED_RO3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_EXECCTRL_EXEC_STALLED_RO3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_EXECCTRL_EXEC_STALLED_RO3, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM3_SHIFTCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM3_SHIFTCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_RESVD_SHIFT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PUSH, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_AUTO_PULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_IN_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_OUT_SHIFT_DIR, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_ISR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_OSR_THRESHOLD, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_TX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_TX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_TX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_TX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_TX, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_RX);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_RX, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_RX, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_RX, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_SHIFTCTRL_JOIN_RX, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM3_ADDR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM3_ADDR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_ADDR_PC);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_ADDR_PC, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_ADDR_PC, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_ADDR_PC, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_ADDR_PC, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM3_INSTR);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM3_INSTR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_INSTR_IMM_INSTR);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_INSTR_IMM_INSTR, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_INSTR_IMM_INSTR, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_INSTR_IMM_INSTR, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_INSTR_IMM_INSTR, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_SM3_PINCTRL);
        rp_pio_csr.wo(utra::rp_pio::SFR_SM3_PINCTRL, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_IN_BASE);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_IN_BASE, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_IN_BASE, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_IN_BASE, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_IN_BASE, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_OUT_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SET_COUNT, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_SM3_PINCTRL_PINS_SIDE_COUNT, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_INTR);
        rp_pio_csr.wo(utra::rp_pio::SFR_INTR, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INTR_INTR_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INTR_INTR_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INTR_INTR_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INTR_INTR_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INTR_INTR_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INTR_INTR_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INTR_INTR_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INTR_INTR_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INTR_INTR_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INTR_INTR_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_INTR_INTR_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_INTR_INTR_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_INTR_INTR_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_INTR_INTR_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_INTR_INTR_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ0_INTE);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ0_INTE, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTE_IRQ0_INTE_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ0_INTF);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ0_INTF, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTF_IRQ0_INTF_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ0_INTS);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ0_INTS, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ0_INTS_IRQ0_INTS_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ1_INTE);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ1_INTE, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTE_IRQ1_INTE_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ1_INTF);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ1_INTF, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTF_IRQ1_INTF_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IRQ1_INTS);
        rp_pio_csr.wo(utra::rp_pio::SFR_IRQ1_INTS, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_RXNEMPTY, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_TXNFULL, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_SM);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IRQ1_INTS_IRQ1_INTS_SM, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IO_OE_INV);
        rp_pio_csr.wo(utra::rp_pio::SFR_IO_OE_INV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IO_OE_INV_SFR_IO_OE_INV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IO_OE_INV_SFR_IO_OE_INV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IO_OE_INV_SFR_IO_OE_INV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IO_OE_INV_SFR_IO_OE_INV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IO_OE_INV_SFR_IO_OE_INV, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IO_O_INV);
        rp_pio_csr.wo(utra::rp_pio::SFR_IO_O_INV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IO_O_INV_SFR_IO_O_INV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IO_O_INV_SFR_IO_O_INV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IO_O_INV_SFR_IO_O_INV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IO_O_INV_SFR_IO_O_INV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IO_O_INV_SFR_IO_O_INV, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_IO_I_INV);
        rp_pio_csr.wo(utra::rp_pio::SFR_IO_I_INV, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_IO_I_INV_SFR_IO_I_INV);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_IO_I_INV_SFR_IO_I_INV, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_IO_I_INV_SFR_IO_I_INV, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_IO_I_INV_SFR_IO_I_INV, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_IO_I_INV_SFR_IO_I_INV, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_FIFO_MARGIN);
        rp_pio_csr.wo(utra::rp_pio::SFR_FIFO_MARGIN, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN0, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN1, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN2, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_TX_MARGIN3, baz);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_FIFO_MARGIN_FIFO_RX_MARGIN3, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_ZERO0);
        rp_pio_csr.wo(utra::rp_pio::SFR_ZERO0, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_ZERO0_SFR_ZERO0);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_ZERO0_SFR_ZERO0, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_ZERO0_SFR_ZERO0, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_ZERO0_SFR_ZERO0, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_ZERO0_SFR_ZERO0, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_ZERO1);
        rp_pio_csr.wo(utra::rp_pio::SFR_ZERO1, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_ZERO1_SFR_ZERO1);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_ZERO1_SFR_ZERO1, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_ZERO1_SFR_ZERO1, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_ZERO1_SFR_ZERO1, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_ZERO1_SFR_ZERO1, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_ZERO2);
        rp_pio_csr.wo(utra::rp_pio::SFR_ZERO2, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_ZERO2_SFR_ZERO2);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_ZERO2_SFR_ZERO2, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_ZERO2_SFR_ZERO2, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_ZERO2_SFR_ZERO2, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_ZERO2_SFR_ZERO2, baz);

        let foo = rp_pio_csr.r(utra::rp_pio::SFR_ZERO3);
        rp_pio_csr.wo(utra::rp_pio::SFR_ZERO3, foo);
        let bar = rp_pio_csr.rf(utra::rp_pio::SFR_ZERO3_SFR_ZERO3);
        rp_pio_csr.rmwf(utra::rp_pio::SFR_ZERO3_SFR_ZERO3, bar);
        let mut baz = rp_pio_csr.zf(utra::rp_pio::SFR_ZERO3_SFR_ZERO3, bar);
        baz |= rp_pio_csr.ms(utra::rp_pio::SFR_ZERO3_SFR_ZERO3, 1);
        rp_pio_csr.wfo(utra::rp_pio::SFR_ZERO3_SFR_ZERO3, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_bio_csr() {
        use super::*;
        let mut bio_csr = CSR::new(HW_BIO_BASE as *mut u32);

        let foo = bio_csr.r(utra::bio::SFR_CTRL);
        bio_csr.wo(utra::bio::SFR_CTRL, foo);
        let bar = bio_csr.rf(utra::bio::SFR_CTRL_EN);
        bio_csr.rmwf(utra::bio::SFR_CTRL_EN, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_CTRL_EN, bar);
        baz |= bio_csr.ms(utra::bio::SFR_CTRL_EN, 1);
        bio_csr.wfo(utra::bio::SFR_CTRL_EN, baz);
        let bar = bio_csr.rf(utra::bio::SFR_CTRL_RESTART);
        bio_csr.rmwf(utra::bio::SFR_CTRL_RESTART, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_CTRL_RESTART, bar);
        baz |= bio_csr.ms(utra::bio::SFR_CTRL_RESTART, 1);
        bio_csr.wfo(utra::bio::SFR_CTRL_RESTART, baz);
        let bar = bio_csr.rf(utra::bio::SFR_CTRL_CLKDIV_RESTART);
        bio_csr.rmwf(utra::bio::SFR_CTRL_CLKDIV_RESTART, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_CTRL_CLKDIV_RESTART, bar);
        baz |= bio_csr.ms(utra::bio::SFR_CTRL_CLKDIV_RESTART, 1);
        bio_csr.wfo(utra::bio::SFR_CTRL_CLKDIV_RESTART, baz);

        let foo = bio_csr.r(utra::bio::SFR_CFGINFO);
        bio_csr.wo(utra::bio::SFR_CFGINFO, foo);
        let bar = bio_csr.rf(utra::bio::SFR_CFGINFO_CONSTANT0);
        bio_csr.rmwf(utra::bio::SFR_CFGINFO_CONSTANT0, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_CFGINFO_CONSTANT0, bar);
        baz |= bio_csr.ms(utra::bio::SFR_CFGINFO_CONSTANT0, 1);
        bio_csr.wfo(utra::bio::SFR_CFGINFO_CONSTANT0, baz);
        let bar = bio_csr.rf(utra::bio::SFR_CFGINFO_CONSTANT1);
        bio_csr.rmwf(utra::bio::SFR_CFGINFO_CONSTANT1, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_CFGINFO_CONSTANT1, bar);
        baz |= bio_csr.ms(utra::bio::SFR_CFGINFO_CONSTANT1, 1);
        bio_csr.wfo(utra::bio::SFR_CFGINFO_CONSTANT1, baz);
        let bar = bio_csr.rf(utra::bio::SFR_CFGINFO_CONSTANT2);
        bio_csr.rmwf(utra::bio::SFR_CFGINFO_CONSTANT2, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_CFGINFO_CONSTANT2, bar);
        baz |= bio_csr.ms(utra::bio::SFR_CFGINFO_CONSTANT2, 1);
        bio_csr.wfo(utra::bio::SFR_CFGINFO_CONSTANT2, baz);

        let foo = bio_csr.r(utra::bio::SFR_FLEVEL);
        bio_csr.wo(utra::bio::SFR_FLEVEL, foo);
        let bar = bio_csr.rf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL0);
        bio_csr.rmwf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL0, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL0, bar);
        baz |= bio_csr.ms(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL0, 1);
        bio_csr.wfo(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL0, baz);
        let bar = bio_csr.rf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1);
        bio_csr.rmwf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1, bar);
        baz |= bio_csr.ms(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1, 1);
        bio_csr.wfo(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1, baz);
        let bar = bio_csr.rf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2);
        bio_csr.rmwf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2, bar);
        baz |= bio_csr.ms(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2, 1);
        bio_csr.wfo(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2, baz);
        let bar = bio_csr.rf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL3);
        bio_csr.rmwf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL3, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL3, bar);
        baz |= bio_csr.ms(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL3, 1);
        bio_csr.wfo(utra::bio::SFR_FLEVEL_PCLK_REGFIFO_LEVEL3, baz);

        let foo = bio_csr.r(utra::bio::SFR_TXF0);
        bio_csr.wo(utra::bio::SFR_TXF0, foo);
        let bar = bio_csr.rf(utra::bio::SFR_TXF0_FDIN);
        bio_csr.rmwf(utra::bio::SFR_TXF0_FDIN, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_TXF0_FDIN, bar);
        baz |= bio_csr.ms(utra::bio::SFR_TXF0_FDIN, 1);
        bio_csr.wfo(utra::bio::SFR_TXF0_FDIN, baz);

        let foo = bio_csr.r(utra::bio::SFR_TXF1);
        bio_csr.wo(utra::bio::SFR_TXF1, foo);
        let bar = bio_csr.rf(utra::bio::SFR_TXF1_FDIN);
        bio_csr.rmwf(utra::bio::SFR_TXF1_FDIN, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_TXF1_FDIN, bar);
        baz |= bio_csr.ms(utra::bio::SFR_TXF1_FDIN, 1);
        bio_csr.wfo(utra::bio::SFR_TXF1_FDIN, baz);

        let foo = bio_csr.r(utra::bio::SFR_TXF2);
        bio_csr.wo(utra::bio::SFR_TXF2, foo);
        let bar = bio_csr.rf(utra::bio::SFR_TXF2_FDIN);
        bio_csr.rmwf(utra::bio::SFR_TXF2_FDIN, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_TXF2_FDIN, bar);
        baz |= bio_csr.ms(utra::bio::SFR_TXF2_FDIN, 1);
        bio_csr.wfo(utra::bio::SFR_TXF2_FDIN, baz);

        let foo = bio_csr.r(utra::bio::SFR_TXF3);
        bio_csr.wo(utra::bio::SFR_TXF3, foo);
        let bar = bio_csr.rf(utra::bio::SFR_TXF3_FDIN);
        bio_csr.rmwf(utra::bio::SFR_TXF3_FDIN, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_TXF3_FDIN, bar);
        baz |= bio_csr.ms(utra::bio::SFR_TXF3_FDIN, 1);
        bio_csr.wfo(utra::bio::SFR_TXF3_FDIN, baz);

        let foo = bio_csr.r(utra::bio::SFR_RXF0);
        bio_csr.wo(utra::bio::SFR_RXF0, foo);
        let bar = bio_csr.rf(utra::bio::SFR_RXF0_FDOUT);
        bio_csr.rmwf(utra::bio::SFR_RXF0_FDOUT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_RXF0_FDOUT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_RXF0_FDOUT, 1);
        bio_csr.wfo(utra::bio::SFR_RXF0_FDOUT, baz);

        let foo = bio_csr.r(utra::bio::SFR_RXF1);
        bio_csr.wo(utra::bio::SFR_RXF1, foo);
        let bar = bio_csr.rf(utra::bio::SFR_RXF1_FDOUT);
        bio_csr.rmwf(utra::bio::SFR_RXF1_FDOUT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_RXF1_FDOUT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_RXF1_FDOUT, 1);
        bio_csr.wfo(utra::bio::SFR_RXF1_FDOUT, baz);

        let foo = bio_csr.r(utra::bio::SFR_RXF2);
        bio_csr.wo(utra::bio::SFR_RXF2, foo);
        let bar = bio_csr.rf(utra::bio::SFR_RXF2_FDOUT);
        bio_csr.rmwf(utra::bio::SFR_RXF2_FDOUT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_RXF2_FDOUT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_RXF2_FDOUT, 1);
        bio_csr.wfo(utra::bio::SFR_RXF2_FDOUT, baz);

        let foo = bio_csr.r(utra::bio::SFR_RXF3);
        bio_csr.wo(utra::bio::SFR_RXF3, foo);
        let bar = bio_csr.rf(utra::bio::SFR_RXF3_FDOUT);
        bio_csr.rmwf(utra::bio::SFR_RXF3_FDOUT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_RXF3_FDOUT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_RXF3_FDOUT, 1);
        bio_csr.wfo(utra::bio::SFR_RXF3_FDOUT, baz);

        let foo = bio_csr.r(utra::bio::SFR_ELEVEL0);
        bio_csr.wo(utra::bio::SFR_ELEVEL0, foo);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL0);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL0, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL0, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL0, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL0, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL1);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL1, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL1, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL1, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL1, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL2);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL2, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL2, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL2, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL2, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL3);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL3, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL3, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL3, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL0_PCLK_FIFO_EVENT_LEVEL3, baz);

        let foo = bio_csr.r(utra::bio::SFR_ELEVEL1);
        bio_csr.wo(utra::bio::SFR_ELEVEL1, foo);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL4);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL4, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL4, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL4, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL4, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL5);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL5, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL5, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL5, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL5, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL6);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL6, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL6, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL6, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL6, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL7);
        bio_csr.rmwf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL7, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL7, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL7, 1);
        bio_csr.wfo(utra::bio::SFR_ELEVEL1_PCLK_FIFO_EVENT_LEVEL7, baz);

        let foo = bio_csr.r(utra::bio::SFR_ETYPE);
        bio_csr.wo(utra::bio::SFR_ETYPE, foo);
        let bar = bio_csr.rf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_LT_MASK);
        bio_csr.rmwf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_LT_MASK, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_LT_MASK, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_LT_MASK, 1);
        bio_csr.wfo(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_LT_MASK, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_EQ_MASK);
        bio_csr.rmwf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_EQ_MASK, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_EQ_MASK, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_EQ_MASK, 1);
        bio_csr.wfo(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_EQ_MASK, baz);
        let bar = bio_csr.rf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_GT_MASK);
        bio_csr.rmwf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_GT_MASK, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_GT_MASK, bar);
        baz |= bio_csr.ms(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_GT_MASK, 1);
        bio_csr.wfo(utra::bio::SFR_ETYPE_PCLK_FIFO_EVENT_GT_MASK, baz);

        let foo = bio_csr.r(utra::bio::SFR_EVENT_SET);
        bio_csr.wo(utra::bio::SFR_EVENT_SET, foo);
        let bar = bio_csr.rf(utra::bio::SFR_EVENT_SET_SFR_EVENT_SET);
        bio_csr.rmwf(utra::bio::SFR_EVENT_SET_SFR_EVENT_SET, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_EVENT_SET_SFR_EVENT_SET, bar);
        baz |= bio_csr.ms(utra::bio::SFR_EVENT_SET_SFR_EVENT_SET, 1);
        bio_csr.wfo(utra::bio::SFR_EVENT_SET_SFR_EVENT_SET, baz);

        let foo = bio_csr.r(utra::bio::SFR_EVENT_CLR);
        bio_csr.wo(utra::bio::SFR_EVENT_CLR, foo);
        let bar = bio_csr.rf(utra::bio::SFR_EVENT_CLR_SFR_EVENT_CLR);
        bio_csr.rmwf(utra::bio::SFR_EVENT_CLR_SFR_EVENT_CLR, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_EVENT_CLR_SFR_EVENT_CLR, bar);
        baz |= bio_csr.ms(utra::bio::SFR_EVENT_CLR_SFR_EVENT_CLR, 1);
        bio_csr.wfo(utra::bio::SFR_EVENT_CLR_SFR_EVENT_CLR, baz);

        let foo = bio_csr.r(utra::bio::SFR_EVENT_STATUS);
        bio_csr.wo(utra::bio::SFR_EVENT_STATUS, foo);
        let bar = bio_csr.rf(utra::bio::SFR_EVENT_STATUS_SFR_EVENT_STATUS);
        bio_csr.rmwf(utra::bio::SFR_EVENT_STATUS_SFR_EVENT_STATUS, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_EVENT_STATUS_SFR_EVENT_STATUS, bar);
        baz |= bio_csr.ms(utra::bio::SFR_EVENT_STATUS_SFR_EVENT_STATUS, 1);
        bio_csr.wfo(utra::bio::SFR_EVENT_STATUS_SFR_EVENT_STATUS, baz);

        let foo = bio_csr.r(utra::bio::SFR_QDIV0);
        bio_csr.wo(utra::bio::SFR_QDIV0, foo);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV0_UNUSED_DIV);
        bio_csr.rmwf(utra::bio::SFR_QDIV0_UNUSED_DIV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV0_UNUSED_DIV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV0_UNUSED_DIV, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV0_UNUSED_DIV, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV0_DIV_FRAC);
        bio_csr.rmwf(utra::bio::SFR_QDIV0_DIV_FRAC, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV0_DIV_FRAC, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV0_DIV_FRAC, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV0_DIV_FRAC, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV0_DIV_INT);
        bio_csr.rmwf(utra::bio::SFR_QDIV0_DIV_INT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV0_DIV_INT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV0_DIV_INT, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV0_DIV_INT, baz);

        let foo = bio_csr.r(utra::bio::SFR_QDIV1);
        bio_csr.wo(utra::bio::SFR_QDIV1, foo);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV1_UNUSED_DIV);
        bio_csr.rmwf(utra::bio::SFR_QDIV1_UNUSED_DIV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV1_UNUSED_DIV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV1_UNUSED_DIV, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV1_UNUSED_DIV, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV1_DIV_FRAC);
        bio_csr.rmwf(utra::bio::SFR_QDIV1_DIV_FRAC, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV1_DIV_FRAC, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV1_DIV_FRAC, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV1_DIV_FRAC, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV1_DIV_INT);
        bio_csr.rmwf(utra::bio::SFR_QDIV1_DIV_INT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV1_DIV_INT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV1_DIV_INT, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV1_DIV_INT, baz);

        let foo = bio_csr.r(utra::bio::SFR_QDIV2);
        bio_csr.wo(utra::bio::SFR_QDIV2, foo);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV2_UNUSED_DIV);
        bio_csr.rmwf(utra::bio::SFR_QDIV2_UNUSED_DIV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV2_UNUSED_DIV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV2_UNUSED_DIV, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV2_UNUSED_DIV, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV2_DIV_FRAC);
        bio_csr.rmwf(utra::bio::SFR_QDIV2_DIV_FRAC, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV2_DIV_FRAC, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV2_DIV_FRAC, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV2_DIV_FRAC, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV2_DIV_INT);
        bio_csr.rmwf(utra::bio::SFR_QDIV2_DIV_INT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV2_DIV_INT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV2_DIV_INT, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV2_DIV_INT, baz);

        let foo = bio_csr.r(utra::bio::SFR_QDIV3);
        bio_csr.wo(utra::bio::SFR_QDIV3, foo);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV3_UNUSED_DIV);
        bio_csr.rmwf(utra::bio::SFR_QDIV3_UNUSED_DIV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV3_UNUSED_DIV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV3_UNUSED_DIV, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV3_UNUSED_DIV, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV3_DIV_FRAC);
        bio_csr.rmwf(utra::bio::SFR_QDIV3_DIV_FRAC, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV3_DIV_FRAC, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV3_DIV_FRAC, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV3_DIV_FRAC, baz);
        let bar = bio_csr.rf(utra::bio::SFR_QDIV3_DIV_INT);
        bio_csr.rmwf(utra::bio::SFR_QDIV3_DIV_INT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_QDIV3_DIV_INT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_QDIV3_DIV_INT, 1);
        bio_csr.wfo(utra::bio::SFR_QDIV3_DIV_INT, baz);

        let foo = bio_csr.r(utra::bio::SFR_SYNC_BYPASS);
        bio_csr.wo(utra::bio::SFR_SYNC_BYPASS, foo);
        let bar = bio_csr.rf(utra::bio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS);
        bio_csr.rmwf(utra::bio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, bar);
        baz |= bio_csr.ms(utra::bio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, 1);
        bio_csr.wfo(utra::bio::SFR_SYNC_BYPASS_SFR_SYNC_BYPASS, baz);

        let foo = bio_csr.r(utra::bio::SFR_IO_OE_INV);
        bio_csr.wo(utra::bio::SFR_IO_OE_INV, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IO_OE_INV_SFR_IO_OE_INV);
        bio_csr.rmwf(utra::bio::SFR_IO_OE_INV_SFR_IO_OE_INV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IO_OE_INV_SFR_IO_OE_INV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IO_OE_INV_SFR_IO_OE_INV, 1);
        bio_csr.wfo(utra::bio::SFR_IO_OE_INV_SFR_IO_OE_INV, baz);

        let foo = bio_csr.r(utra::bio::SFR_IO_O_INV);
        bio_csr.wo(utra::bio::SFR_IO_O_INV, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IO_O_INV_SFR_IO_O_INV);
        bio_csr.rmwf(utra::bio::SFR_IO_O_INV_SFR_IO_O_INV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IO_O_INV_SFR_IO_O_INV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IO_O_INV_SFR_IO_O_INV, 1);
        bio_csr.wfo(utra::bio::SFR_IO_O_INV_SFR_IO_O_INV, baz);

        let foo = bio_csr.r(utra::bio::SFR_IO_I_INV);
        bio_csr.wo(utra::bio::SFR_IO_I_INV, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IO_I_INV_SFR_IO_I_INV);
        bio_csr.rmwf(utra::bio::SFR_IO_I_INV_SFR_IO_I_INV, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IO_I_INV_SFR_IO_I_INV, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IO_I_INV_SFR_IO_I_INV, 1);
        bio_csr.wfo(utra::bio::SFR_IO_I_INV_SFR_IO_I_INV, baz);

        let foo = bio_csr.r(utra::bio::SFR_IRQMASK_0);
        bio_csr.wo(utra::bio::SFR_IRQMASK_0, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IRQMASK_0_SFR_IRQMASK_0);
        bio_csr.rmwf(utra::bio::SFR_IRQMASK_0_SFR_IRQMASK_0, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IRQMASK_0_SFR_IRQMASK_0, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IRQMASK_0_SFR_IRQMASK_0, 1);
        bio_csr.wfo(utra::bio::SFR_IRQMASK_0_SFR_IRQMASK_0, baz);

        let foo = bio_csr.r(utra::bio::SFR_IRQMASK_1);
        bio_csr.wo(utra::bio::SFR_IRQMASK_1, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IRQMASK_1_SFR_IRQMASK_1);
        bio_csr.rmwf(utra::bio::SFR_IRQMASK_1_SFR_IRQMASK_1, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IRQMASK_1_SFR_IRQMASK_1, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IRQMASK_1_SFR_IRQMASK_1, 1);
        bio_csr.wfo(utra::bio::SFR_IRQMASK_1_SFR_IRQMASK_1, baz);

        let foo = bio_csr.r(utra::bio::SFR_IRQMASK_2);
        bio_csr.wo(utra::bio::SFR_IRQMASK_2, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IRQMASK_2_SFR_IRQMASK_2);
        bio_csr.rmwf(utra::bio::SFR_IRQMASK_2_SFR_IRQMASK_2, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IRQMASK_2_SFR_IRQMASK_2, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IRQMASK_2_SFR_IRQMASK_2, 1);
        bio_csr.wfo(utra::bio::SFR_IRQMASK_2_SFR_IRQMASK_2, baz);

        let foo = bio_csr.r(utra::bio::SFR_IRQMASK_3);
        bio_csr.wo(utra::bio::SFR_IRQMASK_3, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IRQMASK_3_SFR_IRQMASK_3);
        bio_csr.rmwf(utra::bio::SFR_IRQMASK_3_SFR_IRQMASK_3, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IRQMASK_3_SFR_IRQMASK_3, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IRQMASK_3_SFR_IRQMASK_3, 1);
        bio_csr.wfo(utra::bio::SFR_IRQMASK_3_SFR_IRQMASK_3, baz);

        let foo = bio_csr.r(utra::bio::SFR_IRQ_EDGE);
        bio_csr.wo(utra::bio::SFR_IRQ_EDGE, foo);
        let bar = bio_csr.rf(utra::bio::SFR_IRQ_EDGE_SFR_IRQ_EDGE);
        bio_csr.rmwf(utra::bio::SFR_IRQ_EDGE_SFR_IRQ_EDGE, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_IRQ_EDGE_SFR_IRQ_EDGE, bar);
        baz |= bio_csr.ms(utra::bio::SFR_IRQ_EDGE_SFR_IRQ_EDGE, 1);
        bio_csr.wfo(utra::bio::SFR_IRQ_EDGE_SFR_IRQ_EDGE, baz);

        let foo = bio_csr.r(utra::bio::SFR_DBG_PADOUT);
        bio_csr.wo(utra::bio::SFR_DBG_PADOUT, foo);
        let bar = bio_csr.rf(utra::bio::SFR_DBG_PADOUT_SFR_DBG_PADOUT);
        bio_csr.rmwf(utra::bio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, bar);
        baz |= bio_csr.ms(utra::bio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, 1);
        bio_csr.wfo(utra::bio::SFR_DBG_PADOUT_SFR_DBG_PADOUT, baz);

        let foo = bio_csr.r(utra::bio::SFR_DBG_PADOE);
        bio_csr.wo(utra::bio::SFR_DBG_PADOE, foo);
        let bar = bio_csr.rf(utra::bio::SFR_DBG_PADOE_SFR_DBG_PADOE);
        bio_csr.rmwf(utra::bio::SFR_DBG_PADOE_SFR_DBG_PADOE, bar);
        let mut baz = bio_csr.zf(utra::bio::SFR_DBG_PADOE_SFR_DBG_PADOE, bar);
        baz |= bio_csr.ms(utra::bio::SFR_DBG_PADOE_SFR_DBG_PADOE, 1);
        bio_csr.wfo(utra::bio::SFR_DBG_PADOE_SFR_DBG_PADOE, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_coresub_sramtrm_csr() {
        use super::*;
        let mut coresub_sramtrm_csr = CSR::new(HW_CORESUB_SRAMTRM_BASE as *mut u32);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_CACHE);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_CACHE, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_CACHE_SFR_CACHE);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_CACHE_SFR_CACHE, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_CACHE_SFR_CACHE, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_CACHE_SFR_CACHE, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_CACHE_SFR_CACHE, baz);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_ITCM);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_ITCM, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_ITCM_SFR_ITCM);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_ITCM_SFR_ITCM, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_ITCM_SFR_ITCM, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_ITCM_SFR_ITCM, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_ITCM_SFR_ITCM, baz);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_DTCM);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_DTCM, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_DTCM_SFR_DTCM);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_DTCM_SFR_DTCM, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_DTCM_SFR_DTCM, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_DTCM_SFR_DTCM, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_DTCM_SFR_DTCM, baz);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_SRAM0);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_SRAM0, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_SRAM0_SFR_SRAM0);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_SRAM0_SFR_SRAM0, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_SRAM0_SFR_SRAM0, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_SRAM0_SFR_SRAM0, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_SRAM0_SFR_SRAM0, baz);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_SRAM1);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_SRAM1, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_SRAM1_SFR_SRAM1);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_SRAM1_SFR_SRAM1, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_SRAM1_SFR_SRAM1, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_SRAM1_SFR_SRAM1, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_SRAM1_SFR_SRAM1, baz);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_VEXRAM);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_VEXRAM, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_VEXRAM_SFR_VEXRAM);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_VEXRAM_SFR_VEXRAM, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_VEXRAM_SFR_VEXRAM, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_VEXRAM_SFR_VEXRAM, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_VEXRAM_SFR_VEXRAM, baz);

        let foo = coresub_sramtrm_csr.r(utra::coresub_sramtrm::SFR_SRAMERR);
        coresub_sramtrm_csr.wo(utra::coresub_sramtrm::SFR_SRAMERR, foo);
        let bar = coresub_sramtrm_csr.rf(utra::coresub_sramtrm::SFR_SRAMERR_SRAMBANKERR);
        coresub_sramtrm_csr.rmwf(utra::coresub_sramtrm::SFR_SRAMERR_SRAMBANKERR, bar);
        let mut baz = coresub_sramtrm_csr.zf(utra::coresub_sramtrm::SFR_SRAMERR_SRAMBANKERR, bar);
        baz |= coresub_sramtrm_csr.ms(utra::coresub_sramtrm::SFR_SRAMERR_SRAMBANKERR, 1);
        coresub_sramtrm_csr.wfo(utra::coresub_sramtrm::SFR_SRAMERR_SRAMBANKERR, baz);
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

        let foo = qfc_csr.r(utra::qfc::CR_AESKEY_AESKEYIN0);
        qfc_csr.wo(utra::qfc::CR_AESKEY_AESKEYIN0, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_AESKEY_AESKEYIN0_AESKEYIN0);
        qfc_csr.rmwf(utra::qfc::CR_AESKEY_AESKEYIN0_AESKEYIN0, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_AESKEY_AESKEYIN0_AESKEYIN0, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_AESKEY_AESKEYIN0_AESKEYIN0, 1);
        qfc_csr.wfo(utra::qfc::CR_AESKEY_AESKEYIN0_AESKEYIN0, baz);

        let foo = qfc_csr.r(utra::qfc::CR_AESKEY_AESKEYIN1);
        qfc_csr.wo(utra::qfc::CR_AESKEY_AESKEYIN1, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_AESKEY_AESKEYIN1_AESKEYIN1);
        qfc_csr.rmwf(utra::qfc::CR_AESKEY_AESKEYIN1_AESKEYIN1, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_AESKEY_AESKEYIN1_AESKEYIN1, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_AESKEY_AESKEYIN1_AESKEYIN1, 1);
        qfc_csr.wfo(utra::qfc::CR_AESKEY_AESKEYIN1_AESKEYIN1, baz);

        let foo = qfc_csr.r(utra::qfc::CR_AESKEY_AESKEYIN2);
        qfc_csr.wo(utra::qfc::CR_AESKEY_AESKEYIN2, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_AESKEY_AESKEYIN2_AESKEYIN2);
        qfc_csr.rmwf(utra::qfc::CR_AESKEY_AESKEYIN2_AESKEYIN2, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_AESKEY_AESKEYIN2_AESKEYIN2, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_AESKEY_AESKEYIN2_AESKEYIN2, 1);
        qfc_csr.wfo(utra::qfc::CR_AESKEY_AESKEYIN2_AESKEYIN2, baz);

        let foo = qfc_csr.r(utra::qfc::CR_AESKEY_AESKEYIN3);
        qfc_csr.wo(utra::qfc::CR_AESKEY_AESKEYIN3, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_AESKEY_AESKEYIN3_AESKEYIN3);
        qfc_csr.rmwf(utra::qfc::CR_AESKEY_AESKEYIN3_AESKEYIN3, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_AESKEY_AESKEYIN3_AESKEYIN3, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_AESKEY_AESKEYIN3_AESKEYIN3, 1);
        qfc_csr.wfo(utra::qfc::CR_AESKEY_AESKEYIN3_AESKEYIN3, baz);

        let foo = qfc_csr.r(utra::qfc::CR_AESENA);
        qfc_csr.wo(utra::qfc::CR_AESENA, foo);
        let bar = qfc_csr.rf(utra::qfc::CR_AESENA_CR_AESENA);
        qfc_csr.rmwf(utra::qfc::CR_AESENA_CR_AESENA, bar);
        let mut baz = qfc_csr.zf(utra::qfc::CR_AESENA_CR_AESENA, bar);
        baz |= qfc_csr.ms(utra::qfc::CR_AESENA_CR_AESENA, 1);
        qfc_csr.wfo(utra::qfc::CR_AESENA_CR_AESENA, baz);
  }

    #[test]
    #[ignore]
    fn compile_check_mbox_apb_csr() {
        use super::*;
        let mut mbox_apb_csr = CSR::new(HW_MBOX_APB_BASE as *mut u32);

        let foo = mbox_apb_csr.r(utra::mbox_apb::SFR_WDATA);
        mbox_apb_csr.wo(utra::mbox_apb::SFR_WDATA, foo);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_WDATA_SFR_WDATA);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_WDATA_SFR_WDATA, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_WDATA_SFR_WDATA, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_WDATA_SFR_WDATA, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_WDATA_SFR_WDATA, baz);

        let foo = mbox_apb_csr.r(utra::mbox_apb::SFR_RDATA);
        mbox_apb_csr.wo(utra::mbox_apb::SFR_RDATA, foo);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_RDATA_SFR_RDATA);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_RDATA_SFR_RDATA, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_RDATA_SFR_RDATA, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_RDATA_SFR_RDATA, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_RDATA_SFR_RDATA, baz);

        let foo = mbox_apb_csr.r(utra::mbox_apb::SFR_STATUS);
        mbox_apb_csr.wo(utra::mbox_apb::SFR_STATUS, foo);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_STATUS_RX_AVAIL);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_STATUS_RX_AVAIL, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_STATUS_RX_AVAIL, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_STATUS_RX_AVAIL, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_STATUS_RX_AVAIL, baz);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_STATUS_TX_FREE);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_STATUS_TX_FREE, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_STATUS_TX_FREE, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_STATUS_TX_FREE, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_STATUS_TX_FREE, baz);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_STATUS_ABORT_IN_PROGRESS);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_STATUS_ABORT_IN_PROGRESS, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_STATUS_ABORT_IN_PROGRESS, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_STATUS_ABORT_IN_PROGRESS, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_STATUS_ABORT_IN_PROGRESS, baz);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_STATUS_ABORT_ACK);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_STATUS_ABORT_ACK, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_STATUS_ABORT_ACK, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_STATUS_ABORT_ACK, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_STATUS_ABORT_ACK, baz);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_STATUS_TX_ERR);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_STATUS_TX_ERR, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_STATUS_TX_ERR, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_STATUS_TX_ERR, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_STATUS_TX_ERR, baz);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_STATUS_RX_ERR);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_STATUS_RX_ERR, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_STATUS_RX_ERR, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_STATUS_RX_ERR, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_STATUS_RX_ERR, baz);

        let foo = mbox_apb_csr.r(utra::mbox_apb::SFR_ABORT);
        mbox_apb_csr.wo(utra::mbox_apb::SFR_ABORT, foo);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_ABORT_SFR_ABORT);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_ABORT_SFR_ABORT, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_ABORT_SFR_ABORT, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_ABORT_SFR_ABORT, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_ABORT_SFR_ABORT, baz);

        let foo = mbox_apb_csr.r(utra::mbox_apb::SFR_DONE);
        mbox_apb_csr.wo(utra::mbox_apb::SFR_DONE, foo);
        let bar = mbox_apb_csr.rf(utra::mbox_apb::SFR_DONE_SFR_DONE);
        mbox_apb_csr.rmwf(utra::mbox_apb::SFR_DONE_SFR_DONE, bar);
        let mut baz = mbox_apb_csr.zf(utra::mbox_apb::SFR_DONE_SFR_DONE, bar);
        baz |= mbox_apb_csr.ms(utra::mbox_apb::SFR_DONE_SFR_DONE, 1);
        mbox_apb_csr.wfo(utra::mbox_apb::SFR_DONE_SFR_DONE, baz);
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
}
