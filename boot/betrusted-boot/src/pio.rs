use pio::Program;
use pio::RP2040_MAX_PROGRAM_SIZE;

use utralib::generated::*;
use crate::pio_generated::utra::rp_pio;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum PioRawIntSource {
    Sm3         = 0b1000_0000_0000,
    Sm2         = 0b0100_0000_0000,
    Sm1         = 0b0010_0000_0000,
    Sm0         = 0b0001_0000_0000,
    TxNotFull3  = 0b0000_1000_0000,
    TxNotFull2  = 0b0000_0100_0000,
    TxNotFull1  = 0b0000_0010_0000,
    TxNotFull0  = 0b0000_0001_0000,
    RxNotEmpty3 = 0b0000_0000_1000,
    RxNotEmpty2 = 0b0000_0000_0100,
    RxNotEmpty1 = 0b0000_0000_0010,
    RxNotEmpty0 = 0b0000_0000_0001,
}
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum PioIntSource {
    Sm,
    TxNotFull,
    RxNotEmpty,
}

#[derive(Debug)]
pub enum PioError {
    /// specified state machine is not valid
    InvalidSm,
    /// program can't fit in memory, for one reason or another
    Oom,
}

#[derive(Debug)]
pub struct SmConfig {
    pub clkdiv: u32,
    pub execctl: u32,
    pub shiftctl: u32,
    pub pinctl: u32,
}
impl SmConfig {
    pub fn default() -> SmConfig {
        // FIXME: use "proper" getters and setters to create the default config.
        SmConfig {
            clkdiv: 0x1_0000,
            execctl: 31 << 12,
            shiftctl: 1 << 18 | 1 << 19 | 32 << 25 | 32 << 20,
            pinctl: 0,
        }
    }
}
#[derive(Debug)]
pub struct LoadedProg {
    program: Program::<RP2040_MAX_PROGRAM_SIZE>,
    offset: usize,
    entry_point: Option<usize>,
}
impl LoadedProg {
    pub fn load(program: Program::<RP2040_MAX_PROGRAM_SIZE>, pio_sm: &mut PioSm) -> Result<Self, PioError> {
        let offset = pio_sm.add_program(&program)?;
        Ok({
            LoadedProg {
                program,
                offset: offset as usize,
                entry_point: None,
            }
        })
    }
    pub fn load_with_entrypoint(program: Program::<RP2040_MAX_PROGRAM_SIZE>, entry_point: usize, pio_sm: &mut PioSm) -> Result<Self, PioError> {
        let offset = pio_sm.add_program(&program)?;
        Ok({
            LoadedProg {
                program,
                offset: offset as usize,
                entry_point: Some(entry_point)
            }
        })
    }
    pub fn entry(&self) -> usize {
        if let Some(ep) = self.entry_point {
            ep + self.offset
        } else {
            self.start()
        }
    }
    pub fn start(&self) -> usize {
        self.program.wrap.target as usize + self.offset
    }
    pub fn end(&self) -> usize {
        self.program.wrap.source as usize + self.offset
    }
    pub fn setup_default_config(&self, pio_sm: &mut PioSm) {
        pio_sm.config_set_defaults();
        pio_sm.config_set_wrap(self.start(), self.end());
        if self.program.side_set.bits() > 0 {
            pio_sm.config_set_sideset(
                self.program.side_set.bits() as usize,
                self.program.side_set.optional(),
                self.program.side_set.pindirs(),
            )
        }
    }
}
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum SmBit {
    Sm0 = 1,
    Sm1 = 2,
    Sm2 = 4,
    Sm3 = 8
}
#[derive(Debug)]
pub struct PioSm {
    pub pio: CSR<u32>,
    pub sm: SmBit,
    // using a 32-bit wide bitmask to track used locations pins this implementation
    // to a 32-instruction PIO memory. ¯\_(ツ)_/¯
    // 0 means unused; 1 means used. LSB is lowest address.
    used_mask: u32,
    config: SmConfig,
}
impl PioSm {
    pub fn new(sm: usize) -> Result<PioSm, PioError> {
        let sm = match sm {
            0 => SmBit::Sm0,
            1 => SmBit::Sm1,
            2 => SmBit::Sm2,
            3 => SmBit::Sm3,
            _ => return Err(PioError::InvalidSm),
        };
        Ok(PioSm {
            pio: CSR::new(rp_pio::HW_RP_PIO_BASE as *mut u32),
            sm,
            used_mask: 0,
            config: SmConfig::default(),
        })
    }
    pub fn sm_index(&self) -> usize {
        match self.sm {
            SmBit::Sm0 => 0,
            SmBit::Sm1 => 1,
            SmBit::Sm2 => 2,
            SmBit::Sm3 => 3,
        }
    }
    pub fn sm_txfifo_is_full(&self) -> bool {
        (self.pio.rf(rp_pio::SFR_FSTAT_TX_FULL) & (self.sm as u32)) != 0
    }
    pub fn sm_txfifo_is_empty(&self) -> bool {
        (self.pio.rf(rp_pio::SFR_FSTAT_TX_EMPTY) & (self.sm as u32)) != 0
    }
    pub fn sm_txfifo_level(&self) -> usize {
        match self.sm {
            SmBit::Sm0 => self.pio.rf(rp_pio::SFR_FLEVEL_TX_LEVEL0) as usize,
            SmBit::Sm1 => self.pio.rf(rp_pio::SFR_FLEVEL_TX_LEVEL1) as usize,
            SmBit::Sm2 => self.pio.rf(rp_pio::SFR_FLEVEL_TX_LEVEL2) as usize,
            SmBit::Sm3 => self.pio.rf(rp_pio::SFR_FLEVEL_TX_LEVEL3) as usize,
        }
    }
    #[allow(dead_code)]
    pub fn sm_txfifo_push_u32(&mut self, data: u32) {
        match self.sm {
            SmBit::Sm0 => self.pio.wo(rp_pio::SFR_TXF0, data),
            SmBit::Sm1 => self.pio.wo(rp_pio::SFR_TXF1, data),
            SmBit::Sm2 => self.pio.wo(rp_pio::SFR_TXF2, data),
            SmBit::Sm3 => self.pio.wo(rp_pio::SFR_TXF3, data),
        }
    }
    pub fn sm_txfifo_push_u16_msb(&mut self, data: u16) {
        match self.sm {
            SmBit::Sm0 => self.pio.wo(rp_pio::SFR_TXF0, (data as u32) << 16),
            SmBit::Sm1 => self.pio.wo(rp_pio::SFR_TXF1, (data as u32) << 16),
            SmBit::Sm2 => self.pio.wo(rp_pio::SFR_TXF2, (data as u32) << 16),
            SmBit::Sm3 => self.pio.wo(rp_pio::SFR_TXF3, (data as u32) << 16),
        }
    }
    pub fn sm_txfifo_push_u16_lsb(&mut self, data: u16) {
        match self.sm {
            SmBit::Sm0 => self.pio.wo(rp_pio::SFR_TXF0, data as u32),
            SmBit::Sm1 => self.pio.wo(rp_pio::SFR_TXF1, data as u32),
            SmBit::Sm2 => self.pio.wo(rp_pio::SFR_TXF2, data as u32),
            SmBit::Sm3 => self.pio.wo(rp_pio::SFR_TXF3, data as u32),
        }
    }
    pub fn sm_txfifo_push_u8_msb(&mut self, data: u8) {
        match self.sm {
            SmBit::Sm0 => self.pio.wo(rp_pio::SFR_TXF0, (data as u32) << 24),
            SmBit::Sm1 => self.pio.wo(rp_pio::SFR_TXF1, (data as u32) << 24),
            SmBit::Sm2 => self.pio.wo(rp_pio::SFR_TXF2, (data as u32) << 24),
            SmBit::Sm3 => self.pio.wo(rp_pio::SFR_TXF3, (data as u32) << 24),
        }
    }
    pub fn sm_rxfifo_is_empty(&self) -> bool {
        (self.pio.rf(rp_pio::SFR_FSTAT_RX_EMPTY) & (self.sm as u32)) != 0
    }
    pub fn sm_rxfifo_is_full(&self) -> bool {
        (self.pio.rf(rp_pio::SFR_FSTAT_RX_FULL) & (self.sm as u32)) != 0
    }
    #[allow(dead_code)]
    pub fn sm_rxfifo_pull_u32(&mut self) -> u32 {
        match self.sm {
            SmBit::Sm0 => self.pio.r(rp_pio::SFR_RXF0),
            SmBit::Sm1 => self.pio.r(rp_pio::SFR_RXF1),
            SmBit::Sm2 => self.pio.r(rp_pio::SFR_RXF2),
            SmBit::Sm3 => self.pio.r(rp_pio::SFR_RXF3),
        }
    }
    pub fn sm_rxfifo_pull_u8_lsb(&mut self) -> u8 {
        match self.sm {
            SmBit::Sm0 => self.pio.r(rp_pio::SFR_RXF0) as u8,
            SmBit::Sm1 => self.pio.r(rp_pio::SFR_RXF1) as u8,
            SmBit::Sm2 => self.pio.r(rp_pio::SFR_RXF2) as u8,
            SmBit::Sm3 => self.pio.r(rp_pio::SFR_RXF3) as u8,
        }
    }
    fn find_offset_for_program(&self, program: &Program<RP2040_MAX_PROGRAM_SIZE>) -> Option<usize> {
        let prog_mask = (1 << program.code.len() as u32) - 1;
        if let Some(origin) = program.origin {
            if origin as usize > RP2040_MAX_PROGRAM_SIZE - program.code.len() {
                None
            } else {
                if (self.used_mask & (prog_mask << origin as u32)) != 0 {
                    None
                } else {
                    Some(origin as usize)
                }
            }
        } else {
            for i in (0..(32 - program.code.len())).rev() {
                if (self.used_mask & (prog_mask << i)) == 0 {
                    return Some(i)
                }
            }
            None
        }
    }
    pub fn can_add_program(&self, program: &Program<RP2040_MAX_PROGRAM_SIZE>) -> bool {
        self.find_offset_for_program(program).is_some()
    }
    /// Write an instruction to program memory.
    fn write_progmem(&mut self, offset: usize, data: u16) {
        assert!(offset < 32);
        unsafe {
            self.pio.base.add(offset + rp_pio::SFR_INSTR_MEM0.offset()).write_volatile(data as _);
        }
    }
    /// returns the offset of the program once loaded
    pub fn add_program(
        &mut self,
        program: &Program<RP2040_MAX_PROGRAM_SIZE>,
    ) -> Result<usize, PioError> {
        if self.can_add_program(&program) {
            if let Some(origin) = self.find_offset_for_program(&program) {
                for (i, &instr) in program.code.iter().enumerate() {
                    // I feel like if I were somehow more clever I could find somewhere in one of these
                    // libraries a macro that defines the jump instruction coding. But I can't. So,
                    // this function literally just masks off the opcode (top 3 bits) and checks if
                    // it's a jump instrution (3b000).
                    let located_instr = if instr & 0xE000 != 0x0000 {
                        instr
                    } else {
                        // this works because the offset is the LSB, and, generally the code is
                        // assembled to address 0. Gross, but that's how the API is defined.
                        instr + origin as u16
                    };
                    self.write_progmem(origin + i, located_instr);
                }
                let prog_mask = (1 << program.code.len()) - 1;
                self.used_mask |= prog_mask << origin as u32;
                Ok(origin as usize)
            } else {
                Err(PioError::Oom)
            }
        } else {
            Err(PioError::Oom)
        }
    }
    /// This merely de-allocates the space but it does not actually change the contents.
    #[allow(dead_code)]
    pub fn remove_program(
        &mut self,
        program: &Program<RP2040_MAX_PROGRAM_SIZE>,
        loaded_offset: usize,
    ) {
        let prog_mask = (((1 << program.code.len()) - 1) << loaded_offset) as u32;
        self.used_mask &= !prog_mask;
    }
    /// Clears all allocations and fills program memory with a set of instructions
    /// that jump to themselves (this mirrors the pattern in the Pi SDK)
    #[allow(dead_code)]
    pub fn clear_instruction_memory(
        &mut self,
    ) {
        self.used_mask = 0;

        // write it to program memory
        for i in 0..RP2040_MAX_PROGRAM_SIZE {
            // small program that jumps to itself
            let mut a = pio::Assembler::<32>::new();
            let mut self_label = a.label_at_offset(i as u8);
            a.jmp(pio::JmpCondition::Always, &mut self_label);
            let mut p= a.assemble_program();
            p = p.set_origin(Some(i as u8));
            self.write_progmem(i, p.code[p.origin.unwrap_or(0) as usize]);
        }
    }
    fn sm_to_stride_offset(&self) -> usize {
        // derive the constant value of the stride between SMs
        const STRIDE: usize = rp_pio::SFR_SM1_EXECCTRL.offset() - rp_pio::SFR_SM0_EXECCTRL.offset();
        match self.sm {
            SmBit::Sm0 => STRIDE * 0,
            SmBit::Sm1 => STRIDE * 1,
            SmBit::Sm2 => STRIDE * 2,
            SmBit::Sm3 => STRIDE * 3,
        }
    }
    pub fn config_set_out_pins(&mut self, out_base: usize, out_count: usize) {
        assert!(out_base < 32);
        assert!(out_count <= 32);
        // note a feature of UTRA is that for multi-bank operations, you can
        // refer to the base bank (SM0) and add an offset to it. All the SMn
        // field macros (.zf(), .ms()) are identical, so we can just use the SM0 macro
        // without type conflict or error.
        self.config.pinctl =
            // zero the PINS_OUT_COUNT field...
            self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT,
                // ... and zero the PINS_OUT_BASE field ...
                self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE,
                    // ... from the existing value of PINCTL
                    self.config.pinctl
                )
            )
            // OR with the new values of the fields, masked and shifted
            | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_OUT_BASE, out_base as _)
            | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_OUT_COUNT, out_count as _);
    }
    #[allow(dead_code)]
    pub fn config_set_set_pins(&mut self, set_base: usize, set_count: usize) {
        assert!(set_base < 32);
        assert!(set_count <= 5);
        self.config.pinctl =
            self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT,
                self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE,
                    self.config.pinctl
                )
            )
            | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, set_base as _)
            | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, set_count as _);
    }
    pub fn config_set_in_pins(&mut self, in_base: usize) {
        assert!(in_base < 32);
        self.config.pinctl =
                self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE,
                    self.config.pinctl
                )
                | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_IN_BASE, in_base as _);
    }
    pub fn config_set_sideset_pins(&mut self, sideset_base: usize) {
        assert!(sideset_base < 32);
        self.config.pinctl =
            self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE,
                self.config.pinctl
            )
            | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_BASE, sideset_base as _);
    }
    #[allow(dead_code)]
    pub fn config_set_sideset(&mut self, bit_count: usize, optional: bool, pindirs: bool) {
        assert!(bit_count < 5);
        assert!(!optional || bit_count >= 1);
        self.config.pinctl =
            self.pio.zf(rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT,
                self.config.pinctl
            )
            | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SIDE_COUNT, bit_count as _);

        self.config.execctl =
            self.pio.zf(rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR,
                self.pio.zf(rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT,
                    self.config.execctl
                )
            )
            | self.pio.ms(rp_pio::SFR_SM0_EXECCTRL_SIDESET_ENABLE_BIT, if optional {1} else {0})
            | self.pio.ms(rp_pio::SFR_SM0_EXECCTRL_SIDE_PINDIR, if pindirs {1} else {0});
    }
    pub fn config_set_out_shift(&mut self, shift_right: bool, autopull: bool, pull_threshold: usize) {
        assert!(pull_threshold <= 32);
        self.config.shiftctl =
            self.pio.zf(rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD,
                self.pio.zf(rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL,
                    self.pio.zf(rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR,
                        self.config.shiftctl
                    )
                )
            )
            | self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_OSR_THRESHOLD, pull_threshold as _)
            | self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, if autopull {1} else {0})
            | self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_OUT_SHIFT_DIR, if shift_right {1} else {0});
    }
    pub fn config_set_in_shift(&mut self, shift_right: bool, autopush: bool, push_threshold: usize) {
        assert!(push_threshold <= 32);
        self.config.shiftctl =
            self.pio.zf(rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD,
                self.pio.zf(rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH,
                    self.pio.zf(rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR,
                        self.config.shiftctl
                    )
                )
            )
            | self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_ISR_THRESHOLD, push_threshold as _)
            | self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, if autopush {1} else {0})
            | self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_IN_SHIFT_DIR, if shift_right {1} else {0});
    }
    pub fn config_set_defaults(&mut self) {
        self.config = SmConfig::default();
    }
    pub fn config_set_jmp_pin(&mut self, pin: usize) {
        assert!(pin < 32);
        self.config.execctl =
            self.pio.zf(
                rp_pio::SFR_SM0_EXECCTRL_JMP_PIN,
                self.config.execctl
            )
            | self.pio.ms(rp_pio::SFR_SM0_EXECCTRL_JMP_PIN, pin as _);
    }

    /// returns tuple as (int, frac)
    pub fn clkdiv_from_float(&self, div: f32) -> (u16, u8) {
        assert!(div >= 1.0);
        assert!(div <= 65536.0);
        let div_int = div as u16;
        let div_frac = if div_int == 0 {
            0u8
        } else {
            ((div - div_int as f32) * (1 << 8) as f32) as u8
        };
        (div_int, div_frac)
    }
    pub fn config_set_clkdiv_int_frac(&mut self, div_int: u16, div_frac: u8) {
        assert!(!(div_int == 0 && (div_frac != 0)));
        self.config.clkdiv =
            self.pio.ms(rp_pio::SFR_SM0_CLKDIV_DIV_INT, div_int as _)
            | self.pio.ms(rp_pio::SFR_SM0_CLKDIV_DIV_FRAC, div_frac as _);
    }
    pub fn config_set_clkdiv(&mut self, div: f32) {
        let (div_int, div_frac) = self.clkdiv_from_float(div);
        self.config_set_clkdiv_int_frac(div_int, div_frac);
    }
    pub fn config_set_wrap(&mut self, start: usize, end: usize) {
        self.config.execctl =
            self.pio.zf(rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET,
                self.pio.zf(rp_pio::SFR_SM0_EXECCTRL_PEND, self.config.execctl)
            )
            | self.pio.ms(rp_pio::SFR_SM0_EXECCTRL_PEND, end as _)
            | self.pio.ms(rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET, start as _)
            ;
    }

    pub fn sm_exec(&mut self, instr: u16) {
        let sm_offset = self.sm_to_stride_offset();
        unsafe {
            self.pio.base.add(rp_pio::SFR_SM0_INSTR.offset() + sm_offset)
            .write_volatile(instr as u32);
        }
    }
    pub fn sm_set_pindirs_with_mask(&mut self, pindirs: usize, mut pin_mask: usize) {
        let sm_offset = self.sm_to_stride_offset();
        unsafe {
            let pinctrl_saved = self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).read_volatile();
            let exectrl_saved = self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).read_volatile();
            self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).write_volatile(
                self.pio.zf(
                    rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY,
                    self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).read_volatile()
                )
            );
            while pin_mask != 0 {
                let base = pin_mask.trailing_zeros();
                self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).write_volatile(
                    self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, 1)
                    | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, base as _)
                );
                let mut a = pio::Assembler::<32>::new();
                a.set(pio::SetDestination::PINDIRS, ((pindirs >> base) & 1) as u8);
                let p= a.assemble_program();
                self.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
                pin_mask &= pin_mask - 1;
            }
            self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).write_volatile(pinctrl_saved);
            self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).write_volatile(exectrl_saved);
        }
    }
    pub fn sm_set_pins_with_mask(&mut self, pinvals: usize, mut pin_mask: usize) {
        let sm_offset = self.sm_to_stride_offset();
        unsafe {
            let pinctrl_saved = self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).read_volatile();
            let exectrl_saved = self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).read_volatile();
            self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).write_volatile(
                self.pio.zf(
                    rp_pio::SFR_SM0_EXECCTRL_OUT_STICKY,
                    self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).read_volatile()
                )
            );
            while pin_mask != 0 {
                let base = pin_mask.trailing_zeros();
                self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).write_volatile(
                    self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SET_COUNT, 1)
                    | self.pio.ms(rp_pio::SFR_SM0_PINCTRL_PINS_SET_BASE, base as _)
                );
                let mut a = pio::Assembler::<32>::new();
                a.set(pio::SetDestination::PINS, ((pinvals >> base) & 1) as u8);
                let p= a.assemble_program();
                self.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
                pin_mask &= pin_mask - 1;
            }
            self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).write_volatile(pinctrl_saved);
            self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).write_volatile(exectrl_saved);
        }
    }
    #[allow(dead_code)]
    pub fn global_irq0_source_enabled(&mut self, source: PioRawIntSource, enabled: bool) {
        self.pio.wo(rp_pio::SFR_IRQ0_INTE,
            if enabled {source as u32} else {0}
            | self.pio.r(rp_pio::SFR_IRQ0_INTE) & !(source as u32)
        );
    }
    #[allow(dead_code)]
    pub fn global_irq1_source_enabled(&mut self, source: PioRawIntSource, enabled: bool) {
        self.pio.wo(rp_pio::SFR_IRQ1_INTE,
            if enabled {source as u32} else {0}
            | self.pio.r(rp_pio::SFR_IRQ1_INTE) & !(source as u32)
        );
    }
    pub fn sm_irq0_source_enabled(&mut self, source: PioIntSource, enabled: bool) {
        let mask = match source {
            PioIntSource::Sm => (self.sm as u32) << 8,
            PioIntSource::TxNotFull => (self.sm as u32) << 4,
            PioIntSource::RxNotEmpty => (self.sm as u32) << 0,
        };
        self.pio.wo(rp_pio::SFR_IRQ0_INTE,
            if enabled {mask} else {0}
            | self.pio.r(rp_pio::SFR_IRQ0_INTE) & !mask
        );
    }
    pub fn sm_irq1_source_enabled(&mut self, source: PioIntSource, enabled: bool) {
        let mask = match source {
            PioIntSource::Sm => (self.sm as u32) << 8,
            PioIntSource::TxNotFull => (self.sm as u32) << 4,
            PioIntSource::RxNotEmpty => (self.sm as u32) << 0,
        };
        self.pio.wo(rp_pio::SFR_IRQ1_INTE,
            if enabled{mask} else {0}
            | self.pio.r(rp_pio::SFR_IRQ1_INTE) & !mask
        );
    }

    pub fn sm_set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.pio.rmwf(rp_pio::SFR_CTRL_EN,
                self.pio.rf(rp_pio::SFR_CTRL_EN) | (self.sm as u32)
            )
        } else {
            self.pio.rmwf(rp_pio::SFR_CTRL_EN,
                self.pio.rf(rp_pio::SFR_CTRL_EN) & !(self.sm as u32)
            )
        }
    }

    pub fn sm_set_config(&mut self) {
        let sm_offset = self.sm_to_stride_offset();
        unsafe {
            self.pio.base.add(rp_pio::SFR_SM0_CLKDIV.offset() + sm_offset).write_volatile(self.config.clkdiv);
            self.pio.base.add(rp_pio::SFR_SM0_EXECCTRL.offset() + sm_offset).write_volatile(self.config.execctl);
            self.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).write_volatile(self.config.shiftctl);
            self.pio.base.add(rp_pio::SFR_SM0_PINCTRL.offset() + sm_offset).write_volatile(self.config.pinctl);
        }
    }
    /// Clears the FIFOs by flipping the RX join bit
    pub fn sm_clear_fifos(&mut self) {
        let sm_offset = self.sm_to_stride_offset();
        unsafe {
            let baseval = self.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).read_volatile();
            let bitval = self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_JOIN_RX, 1);
            self.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).write_volatile(
                baseval ^ bitval
            );
            self.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).write_volatile(
                baseval
            );
        }
    }
    pub fn sm_init(&mut self, initial_pc: usize) {
        self.sm_set_enabled(false);
        self.sm_set_config();

        self.sm_clear_fifos();

        // Clear FIFO debug flags
        self.pio.wo(
            rp_pio::SFR_FDEBUG,
            self.pio.ms(rp_pio::SFR_FDEBUG_TXSTALL, self.sm as u32)
            | self.pio.ms(rp_pio::SFR_FDEBUG_TXOVER, self.sm as u32)
            | self.pio.ms(rp_pio::SFR_FDEBUG_RXUNDER, self.sm as u32)
            | self.pio.ms(rp_pio::SFR_FDEBUG_RXSTALL, self.sm as u32)
        );

        // Finally, clear some internal SM state
        self.pio.rmwf(rp_pio::SFR_CTRL_RESTART, self.sm as u32);
        self.pio.rmwf(rp_pio::SFR_CTRL_CLKDIV_RESTART, self.sm as u32);

        let mut a = pio::Assembler::<32>::new();
        let mut initial_label = a.label_at_offset(initial_pc as u8);
        a.jmp(pio::JmpCondition::Always, &mut initial_label);
        let p= a.assemble_program();

        self.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
    }
    pub fn sm_interrupt_get(&self, int_number: usize) -> bool {
        assert!(int_number < 8);
        (self.pio.r(rp_pio::SFR_IRQ) & (1 << int_number)) != 0
    }
    pub fn sm_drain_tx_fifo(&mut self) {
        let sm_offset = self.sm_to_stride_offset();
        let instr = {
            if (unsafe { self.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).read_volatile() }
            & self.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PULL, 1)) != 0 {
                // autopull is true
                let mut a = pio::Assembler::<32>::new();
                a.out(pio::OutDestination::NULL, 32);
                let p= a.assemble_program();
                p.code[0]
            } else {
                // autopull is false
                let mut a = pio::Assembler::<32>::new();
                a.pull(false, false);
                let p= a.assemble_program();
                p.code[0]
            }
        };
        while !self.sm_txfifo_is_empty() {
            self.sm_exec(instr);
        }
    }
    /// Changes the PC to the lowest address of the wrap range.
    pub fn sm_jump_to_wrap_bottom(&mut self) {
        // HACK: a jump instruction is just the address of the location you want to run
        // so we can just extract the wrap target and "use that as an instruction".
        let instr = match self.sm {
            SmBit::Sm0 => self.pio.rf(rp_pio::SFR_SM0_EXECCTRL_WRAP_TARGET),
            SmBit::Sm1 => self.pio.rf(rp_pio::SFR_SM1_EXECCTRL_WRAP_TARGET),
            SmBit::Sm2 => self.pio.rf(rp_pio::SFR_SM2_EXECCTRL_WRAP_TARGET),
            SmBit::Sm3 => self.pio.rf(rp_pio::SFR_SM3_EXECCTRL_WRAP_TARGET),
        } as u16;
        self.sm_exec(instr);
    }

    pub fn gpio_reset_overrides(&mut self) {
        self.pio.wo(rp_pio::SFR_IO_O_INV, 0);
        self.pio.wo(rp_pio::SFR_IO_OE_INV, 0);
        self.pio.wo(rp_pio::SFR_IO_I_INV, 0);
    }
    pub fn gpio_set_outover(&mut self, pin: usize, value: bool) {
        self.pio.wo(rp_pio::SFR_IO_O_INV,
            (if value {1} else {0}) << pin
            | (self.pio.r(rp_pio::SFR_IO_O_INV) & !(1 << pin))
        );
    }
    #[allow(dead_code)]
    pub fn gpio_set_oeover(&mut self, pin: usize, value: bool) {
        self.pio.wo(rp_pio::SFR_IO_OE_INV,
            (if value {1} else {0}) << pin
            | (self.pio.r(rp_pio::SFR_IO_OE_INV) & !(1 << pin))
        );
    }
    #[allow(dead_code)]
    pub fn gpio_set_inover(&mut self, pin: usize, value: bool) {
        self.pio.wo(rp_pio::SFR_IO_I_INV,
            (if value {1} else {0}) << pin
            | (self.pio.r(rp_pio::SFR_IO_I_INV) & !(1 << pin))
        );
    }
}

#[inline(always)]
pub fn pio_spi_write8_read8_blocking (
    pio_sm: &mut PioSm,
    src: &[u8],
    dst: &mut [u8],
) {
    assert!(src.len() == dst.len(), "src and dst arrays are not the same length!");

    let mut src_iter = src.iter();
    let mut dst_iter_mut = dst.iter_mut().peekable();
    let mut tx_done = false;
    let mut rx_done = false;
    // this weirdness checks that the SPI machine stalls when RX FIFO is full, and no data is lost
    let mut rx_reached_full = false;
    loop {
        if !pio_sm.sm_txfifo_is_full() {
            if let Some(&s) = src_iter.next() {
                pio_sm.sm_txfifo_push_u8_msb(s);
            } else {
                tx_done = true;
            }
        }
        if !rx_reached_full && pio_sm.sm_rxfifo_is_full() {
            rx_reached_full = true;
            let level = pio_sm.sm_txfifo_level();
            while level != 0 && pio_sm.sm_txfifo_level() == level {
                // wait
            }
            for _ in 0..16 {
                // dummy reads to cause some delay to confirm RX full stalls the machine
                pio_sm.sm_txfifo_level();
            }
        }
        if rx_reached_full {
            if !pio_sm.sm_rxfifo_is_empty() {
                if let Some(d) = dst_iter_mut.next() {
                    *d = pio_sm.sm_rxfifo_pull_u8_lsb();
                }
            }
        }
        // always have to peek ahead at this, because
        // we won't ever reach this if we have to wait for the rxfifo
        // to be "not empty" before peeking at it (the last element
        // never generates a new pending element...
        if dst_iter_mut.peek().is_none() {
            rx_done = true;
        }
        if tx_done && rx_done {
            break
        }
    }
}

pub fn spi_test_core(pio_sm: &mut PioSm) -> bool {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x0D10_05D1);

    const BUF_SIZE: usize = 20;
    let mut state: u16 = 0xAA;
    let mut tx_buf = [0u8; BUF_SIZE];
    let mut rx_buf = [0u8; BUF_SIZE];
    // init the TX buf
    for d in tx_buf.iter_mut() {
        state = crate::lfsr_next(state);
        *d = state as u8;
        report.wfo(utra::main::REPORT_REPORT, *d as u32);
    }
    pio_spi_write8_read8_blocking(pio_sm, &tx_buf, &mut rx_buf);
    let mut pass = true;
    for (&s, &d) in tx_buf.iter().zip(rx_buf.iter()) {
        if s != d {
            report.wfo(utra::main::REPORT_REPORT, 0xDEAD_0000 | (s as u32) << 8 | ((d as u32) << 0));
            pass = false;
        }
    }
    report.wfo(utra::main::REPORT_REPORT, 0x600D_05D1);
    pass
}

#[inline(always)]
pub fn pio_spi_init(
    pio_sm: &mut PioSm,
    program: &LoadedProg,
    n_bits: usize,
    clkdiv: f32,
    cpol: bool,
    pin_sck: usize,
    pin_mosi: usize,
    pin_miso: usize
) {
    pio_sm.sm_set_enabled(false);
    // this applies a default config to the PioSm object that is relevant to the program
    program.setup_default_config(pio_sm);

    pio_sm.config_set_out_pins(pin_mosi, 1);
    pio_sm.config_set_in_pins(pin_miso);
    pio_sm.config_set_sideset_pins(pin_sck);
    pio_sm.config_set_out_shift(false, true, n_bits);
    pio_sm.config_set_in_shift(false, true, n_bits);
    pio_sm.config_set_clkdiv(clkdiv);

    // MOSI, SCK output are low, MISO is input
    pio_sm.sm_set_pins_with_mask(
        0,
        (1 << pin_sck) | (1 << pin_mosi)
    );
    pio_sm.sm_set_pindirs_with_mask(
        (1 << pin_sck) | (1 << pin_mosi),
        (1 << pin_sck) | (1 << pin_mosi) | (1 << pin_miso)
    );

    pio_sm.gpio_set_outover(pin_sck, cpol);

    // SPI is synchronous, so bypass input synchroniser to reduce input delay.
    pio_sm.pio.wo(rp_pio::SFR_SYNC_BYPASS, 1 << pin_miso);

    // program origin should already be set by the loader. sm_init() also disables the engine.
    pio_sm.sm_init(program.start());
    pio_sm.sm_set_enabled(true);
}

pub fn spi_test() -> bool {
    const PIN_SCK: usize = 18;
    const PIN_MOSI: usize = 16;
    const PIN_MISO: usize = 16; // loopback

    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x0D10_05D1);

    let mut pio_sm = PioSm::new(0).unwrap();

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
    let prog_cpha0 = LoadedProg::load(spi_cpha0_prog.program, &mut pio_sm).unwrap();
    report.wfo(utra::main::REPORT_REPORT, 0x05D1_0000);
    let prog_cpha1 = LoadedProg::load(spi_cpha1_prog.program, &mut pio_sm).unwrap();
    report.wfo(utra::main::REPORT_REPORT, 0x05D1_0001);

    let clkdiv: f32 = 37.25;
    let mut passing = true;
    let mut cpol = false;
    pio_sm.pio.wo(rp_pio::SFR_IRQ0_INTE, pio_sm.sm as u32);
    pio_sm.pio.wo(rp_pio::SFR_IRQ1_INTE, (pio_sm.sm as u32) << 4);
    loop {
        // pha = 1
        report.wfo(utra::main::REPORT_REPORT, 0x05D1_0002);
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
        report.wfo(utra::main::REPORT_REPORT, 0x05D1_0003);
        if spi_test_core(&mut pio_sm) == false {
            passing = false;
        };

        // pha = 0
        report.wfo(utra::main::REPORT_REPORT, 0x05D1_0004);
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
        report.wfo(utra::main::REPORT_REPORT, 0x05D1_0005);
        if spi_test_core(&mut pio_sm) == false {
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

    if passing {
        report.wfo(utra::main::REPORT_REPORT, 0x05D1_600D);
    } else {
        report.wfo(utra::main::REPORT_REPORT, 0x05D1_DEAD);
    }

    passing
}

pub fn i2c_init(
    pio_sm: &mut PioSm,
    program: &LoadedProg,
    pin_sda: usize,
    pin_scl: usize,
) {
    pio_sm.sm_set_enabled(false);
    program.setup_default_config(pio_sm);

    pio_sm.config_set_out_pins(pin_sda, 1);
    pio_sm.config_set_set_pins(pin_sda, 1);
    pio_sm.config_set_in_pins(pin_sda);
    pio_sm.config_set_sideset_pins(pin_scl);
    pio_sm.config_set_jmp_pin(pin_sda);

    pio_sm.config_set_out_shift(false, true, 16);
    pio_sm.config_set_in_shift(false, true, 8);
    let div: f32 = 800_000_000.0 / (32.0 * 1_000_000.0);
    pio_sm.config_set_clkdiv(div);
    // require: use external pull-up
    let both_pins = (1 << pin_sda) | (1 << pin_scl);
    pio_sm.sm_set_pins_with_mask(both_pins, both_pins);
    pio_sm.sm_set_pindirs_with_mask(both_pins, both_pins);
    pio_sm.gpio_set_oeover(pin_sda, true);
    pio_sm.gpio_set_oeover(pin_scl, true);
    pio_sm.sm_set_pins_with_mask(0, both_pins);

    pio_sm.sm_irq0_source_enabled(PioIntSource::Sm, false);
    pio_sm.sm_irq1_source_enabled(PioIntSource::Sm, false);

    pio_sm.sm_init(program.entry());
    pio_sm.sm_set_enabled(true);
}
pub fn i2c_check_error(pio_sm: &mut PioSm) -> bool {
    pio_sm.sm_interrupt_get(pio_sm.sm_index())
}
pub fn i2c_resume_after_error(pio_sm: &mut PioSm) {
    pio_sm.sm_drain_tx_fifo();
    pio_sm.sm_jump_to_wrap_bottom();
    // this will clear the IRQ set by the current SM, relying on the fact that sm's encoding is a bitmask
    pio_sm.pio.wfo(rp_pio::SFR_IRQ_SFR_IRQ, pio_sm.sm as u32);
}
pub fn i2c_rx_enable(pio_sm: &mut PioSm, en: bool) {
    let sm_offset = pio_sm.sm_to_stride_offset();
    unsafe {
        let baseval = pio_sm.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).read_volatile();
        let bitval = pio_sm.pio.ms(rp_pio::SFR_SM0_SHIFTCTRL_AUTO_PUSH, 1);
        pio_sm.pio.base.add(rp_pio::SFR_SM0_SHIFTCTRL.offset() + sm_offset).write_volatile(
            baseval & !bitval
            | if en {bitval} else {0}
        );
    }
}
pub fn i2c_put16(pio_sm: &mut PioSm, data: u16) {
    while pio_sm.sm_txfifo_is_full() {
        // wait
    }
    pio_sm.sm_txfifo_push_u16_msb(data);
}
pub fn i2c_put_or_err(pio_sm: &mut PioSm, data: u16) {
    while pio_sm.sm_txfifo_is_full() {
        if i2c_check_error(pio_sm) {
            return
        }
    }
    if i2c_check_error(pio_sm) {
        return;
    }
    pio_sm.sm_txfifo_push_u16_msb(data);
}
pub fn i2c_get(pio_sm: &mut PioSm) -> u8 {
    pio_sm.sm_rxfifo_pull_u8_lsb() as u8
}
const PIO_I2C_ICOUNT_LSB: u16 = 10;
const PIO_I2C_FINAL_LSB: u16  = 9;
#[allow(dead_code)]
const PIO_I2C_DATA_LSB: u16   = 1;
const PIO_I2C_NAK_LSB: u16    = 0;
const I2C_SC0_SD0: usize = 0;
#[allow(dead_code)]
const I2C_SC0_SD1: usize = 1;
const I2C_SC1_SD0: usize = 2;
const I2C_SC1_SD1: usize = 3;
pub fn i2c_start(pio_sm: &mut PioSm, set_scl_sda_program_instructions: &[u16; 4]) {
    i2c_put_or_err(pio_sm, 1 << PIO_I2C_ICOUNT_LSB);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC1_SD0]);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC0_SD0]);
}
pub fn i2c_stop(pio_sm: &mut PioSm, set_scl_sda_program_instructions: &[u16; 4]) {
    i2c_put_or_err(pio_sm, 2 << PIO_I2C_ICOUNT_LSB);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC0_SD0]);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC1_SD0]);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC1_SD1]);
}
#[allow(dead_code)]
pub fn i2c_repstart(pio_sm: &mut PioSm, set_scl_sda_program_instructions: &[u16; 4]) {
    i2c_put_or_err(pio_sm, 3 << PIO_I2C_ICOUNT_LSB);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC0_SD1]);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC1_SD1]);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC1_SD0]);
    i2c_put_or_err(pio_sm, set_scl_sda_program_instructions[I2C_SC0_SD0]);
}
pub fn i2c_wait_idle(pio_sm: &mut PioSm) {
    pio_sm.pio.wfo(rp_pio::SFR_FDEBUG_TXSTALL, pio_sm.sm as u32);
    while ((pio_sm.pio.rf(rp_pio::SFR_FDEBUG_TXSTALL) & pio_sm.sm as u32) == 0)
    || i2c_check_error(pio_sm) {
        // busy loop
    }
}
/// returns false if there is an error; true if no error
#[allow(dead_code)]
pub fn i2c_write_blocking(pio_sm: &mut PioSm, set_scl_sda_program_instructions: &[u16; 4], addr: u8, txbuf: &[u8]) -> bool {
    i2c_start(pio_sm, set_scl_sda_program_instructions);
    i2c_rx_enable(pio_sm, false);
    i2c_put16(pio_sm, ((addr as u16) << 2) | 1);
    let mut txbuf_iter = txbuf.iter().peekable();
    loop {
        if i2c_check_error(pio_sm) {
            break;
        }
        if !pio_sm.sm_txfifo_is_full() {
            if let Some(&d) = txbuf_iter.next() {
                i2c_put_or_err(pio_sm,
                    (d as u16) << PIO_I2C_DATA_LSB |
                    if txbuf_iter.peek().is_none() { 1 } else { 0 }
                );
            } else {
                break;
            }
        }
    }
    i2c_stop(pio_sm, set_scl_sda_program_instructions);
    i2c_wait_idle(pio_sm);
    if i2c_check_error(pio_sm) {
        i2c_resume_after_error(pio_sm);
        i2c_stop(pio_sm, set_scl_sda_program_instructions);
        false
    } else {
        true
    }
}
/// returns false if there is an error; true if no error
pub fn i2c_read_blocking(pio_sm: &mut PioSm, set_scl_sda_program_instructions: &[u16; 4], addr: u8, rxbuf: &mut [u8]) -> bool {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0000);

    i2c_start(pio_sm, set_scl_sda_program_instructions);
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0001);
    i2c_rx_enable(pio_sm, true);
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0002);
    while !pio_sm.sm_rxfifo_is_empty() {
        i2c_get(pio_sm);
    }
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0003);
    let addr_composed = ((addr as u16) << 2)  | 3;
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0000 | addr_composed as u32);
    i2c_put16(pio_sm, addr_composed);
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0004);
    let mut first = true;
    let mut tx_remain = rxbuf.len();
    let mut len = rxbuf.len();
    let mut i = 0;
    while (tx_remain != 0) || (len != 0) && !i2c_check_error(pio_sm) {  // here
        report.wfo(utra::main::REPORT_REPORT, 0x12C0_0000 + ((tx_remain as u32) << 8) | len as u32);
        if (tx_remain != 0) && !pio_sm.sm_txfifo_is_full() {
            tx_remain -= 1;
            i2c_put16(pio_sm,
                (0xff << 1)
                | if tx_remain != 0 { 0 } else {
                    1 << PIO_I2C_FINAL_LSB
                    | 1 << PIO_I2C_NAK_LSB
                }
            );
        }
        if !pio_sm.sm_rxfifo_is_empty() {
            if first {
                i2c_get(pio_sm);
                first = false;
            } else {
                len -= 1;
                rxbuf[i] = i2c_get(pio_sm);
                i += 1;
            }
        }
    }
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0020);
    i2c_stop(pio_sm, set_scl_sda_program_instructions);
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0021);
    i2c_wait_idle(pio_sm);
    report.wfo(utra::main::REPORT_REPORT, 0x12C0_0022);
    if i2c_check_error(pio_sm) {
        i2c_resume_after_error(pio_sm);
        i2c_stop(pio_sm, set_scl_sda_program_instructions);
        false
    } else {
        true
    }
}
pub fn i2c_test() -> bool {
    const PIN_SDA: usize = 2;
    const PIN_SCL: usize = 3;

    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x0D10_012C);

    let mut pio_sm = PioSm::new(0).unwrap();

    let i2c_prog = pio_proc::pio_asm!(
        ".side_set 1 opt pindirs",

        // TX Encoding:
        // | 15:10 | 9     | 8:1  | 0   |
        // | Instr | Final | Data | NAK |
        //
        // If Instr has a value n > 0, then this FIFO word has no
        // data payload, and the next n + 1 words will be executed as instructions.
        // Otherwise, shift out the 8 data bits, followed by the ACK bit.
        //
        // The Instr mechanism allows stop/start/repstart sequences to be programmed
        // by the processor, and then carried out by the state machine at defined points
        // in the datastream.
        //
        // The "Final" field should be set for the final byte in a transfer.
        // This tells the state machine to ignore a NAK: if this field is not
        // set, then any NAK will cause the state machine to halt and interrupt.
        //
        // Autopull should be enabled, with a threshold of 16.
        // Autopush should be enabled, with a threshold of 8.
        // The TX FIFO should be accessed with halfword writes, to ensure
        // the data is immediately available in the OSR.
        //
        // Pin mapping:
        // - Input pin 0 is SDA, 1 is SCL (if clock stretching used)
        // - Jump pin is SDA
        // - Side-set pin 0 is SCL
        // - Set pin 0 is SDA
        // - OUT pin 0 is SDA
        // - SCL must be SDA + 1 (for wait mapping)
        //
        // The OE outputs should be inverted in the system IO controls!
        // (It's possible for the inversion to be done in this program,
        // but costs 2 instructions: 1 for inversion, and one to cope
        // with the side effect of the MOV on TX shift counter.)

        "do_nack:",
        "    jmp y-- entry_point",        // 0D Continue if NAK was expected
        "    irq wait 0 rel",             // 0E Otherwise stop, ask for help

        "do_byte:",
        "    set x, 7",                   // 0F E027 Loop 8 times
        "bitloop:",
        "    out pindirs, 1         [7]", // 10 6781 Serialise write data (all-ones if reading)
        "    nop             side 1 [2]", // 11 BA42 SCL rising edge
        "    wait 1 pin, 1          [4]", // 12 24A1 Allow clock to be stretched
        "    in pins, 1             [7]", // 13 4701 Sample read data in middle of SCL pulse
        "    jmp x-- bitloop side 0 [7]", // 14 1750 SCL falling edge

        // Handle ACK pulse
        "    out pindirs, 1         [7]", // 15 6781 On reads, we provide the ACK.
        "    nop             side 1 [7]", // 16 BF42 SCL rising edge
        "    wait 1 pin, 1          [7]", // 17 27A1 Allow clock to be stretched
        "    jmp pin do_nack side 0 [2]", // 18 12CD Test SDA for ACK/NAK, fall through if ACK

        "public entry_point:",
        ".wrap_target",
        "    out x, 6                  ", // 19 6026 Unpack Instr count
        "    out y, 1                  ", // 1A 6041 Unpack the NAK ignore bit
        "    jmp !x do_byte            ", // 1B 002F Instr == 0, this is a data record.
        "    out null, 32              ", // 1C 6060 Instr > 0, remainder of this OSR is invalid
        "do_exec:                      ",
        "    out exec, 16              ", // 1D 60F0 Execute one instruction per FIFO word
        "    jmp x-- do_exec           ", // 1E 005D Repeat n + 1 times
        ".wrap",
    );
    let ep = i2c_prog.public_defines.entry_point as usize;
    let prog_i2c = LoadedProg::load_with_entrypoint(i2c_prog.program, ep, &mut pio_sm).unwrap();
    i2c_init(&mut pio_sm, &prog_i2c, PIN_SDA, PIN_SCL);
    report.wfo(utra::main::REPORT_REPORT, 0x012C_3333);

    let i2c_cmds_raw = pio_proc::pio_asm!(
        ".side_set 1 opt",
        // Assemble a table of instructions which software can select from, and pass
        // into the FIFO, to issue START/STOP/RSTART. This isn't intended to be run as
        // a complete program.
        "    set pindirs, 0 side 0 [7] ", // SCL = 0, SDA = 0"
        "    set pindirs, 1 side 0 [7] ", // SCL = 0, SDA = 1",
        "    set pindirs, 0 side 1 [7] ", // SCL = 1, SDA = 0",
        "    set pindirs, 1 side 1 [7] ", // SCL = 1, SDA = 1",
    ).program.code;
    let mut i2c_cmds = [0u16; 4];
    i2c_cmds.copy_from_slice(&i2c_cmds_raw[..4]);
    // print the compiled program for debug purposes
    for i in 0..4 {
        report.wfo(utra::main::REPORT_REPORT, 0x012C_0000 + i2c_cmds[i] as u32);
    }

    let mut rxbuf = [0u8];
    for addr in 10..14 {
        report.wfo(utra::main::REPORT_REPORT, 0x012C_0000 + addr as u32);
        i2c_read_blocking(&mut pio_sm, &i2c_cmds, addr, &mut rxbuf);
    }
    report.wfo(utra::main::REPORT_REPORT, 0x012C_1111);
    false
}

pub fn pio_tests() {
    spi_test();
    i2c_test();
}
