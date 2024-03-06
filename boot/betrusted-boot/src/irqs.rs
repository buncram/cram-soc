use utralib::generated::*;
use riscv::register::{scause, sepc, stval, vexriscv::sim, vexriscv::sip, sie};
use crate::*;

#[cfg(feature="quanta-test")]
pub fn setup_quantum_timer() {
    let mut pio_ss = xous_pio::PioSharedState::new();
    let mut sm_a = pio_ss.alloc_sm().unwrap();

    pio_ss.clear_instruction_memory();
    pio_ss.pio.rmwf(utra::rp_pio::SFR_CTRL_EN, 0);
    #[rustfmt::skip]
    let timer_code = pio_proc::pio_asm!(
        "restart:",
        "set x, 6",  // 4 cycles overhead gets us to 10 iterations per pulse
        "waitloop:",
        "jmp x-- waitloop",
        "irq set 0",
        "jmp restart",
    );
    let a_prog = LoadedProg::load(timer_code.program, &mut pio_ss).unwrap();
    sm_a.sm_set_enabled(false);
    a_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_clkdiv(5000.0f32);
    sm_a.sm_init(a_prog.entry());
    sm_a.sm_irq0_source_enabled(PioIntSource::Sm, true);
    sm_a.sm_set_enabled(true);
}

pub fn enable_irq(irq_no: usize) {
    // Note that the vexriscv "IRQ Mask" register is inverse-logic --
    // that is, setting a bit in the "mask" register unmasks (i.e. enables) it.
    sim::write(sim::read() | (1 << irq_no));
}

// TODO:
//   - test basic interrupt behaviors (in progress)
//   - test WFI instruction behavior (not at all tested)
pub fn irq_setup() {
    unsafe {
        core::arch::asm!(
            // Set trap handler, which will be called
            // on interrupts and cpu faults
            "la   t0, _start_trap", // this first one forces the nop sled symbol to be generated
            "la   t0, _start_trap_aligned", // this is the actual target
            "csrw stvec, t0",
        );
    }

    report_api(0x1dcd_0000);

    let mut irqarray18 = CSR::new(utra::irqarray18::HW_IRQARRAY18_BASE as *mut u32);
    let mut irqarray19 = CSR::new(utra::irqarray19::HW_IRQARRAY19_BASE as *mut u32);
    // unmask interrupt sources
    irqarray18.wo(utra::irqarray18::EV_ENABLE, 0x4); // don't allow PIO IROQs to trigger us
    #[cfg(feature="quanta-test")]
    {
        irqarray18.rmwf(utra::irqarray18::EV_ENABLE_PIOIRQ0_DUPE, 1);
        setup_quantum_timer();
    }
    irqarray19.wo(utra::irqarray19::EV_ENABLE, 0x80); // narrow this down because mdma currently maps to this and causes troubles if we don't handle it
    // enable IRQ handling
    sim::write(0x0); // first make sure everything is disabled, so we aren't OR'ing in garbage
    enable_irq(utra::irqarray18::IRQARRAY18_IRQ);
    enable_irq(utra::irqarray19::IRQARRAY19_IRQ);
    // for wfi testing
    enable_irq(utra::ticktimer::TICKTIMER_IRQ);

    // must enable external interrupts on the CPU for any of the above to matter
    unsafe{sie::set_sext()};

    report_api(0x1dcd_600d);
}

pub fn irq_test() {
    // trigger an interrupt
    report_api(0x3dcd_0000);

    let mut main = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    // simulate hw trigger from IRQ0
    report_api(0x3dcd_0001);
    main.wfo(utra::main::IRQTEST0_TRIGGER, 4);
    // software-only trigger from IRQ2
    report_api(0x3dcd_0003);
    let mut irqarray19 = CSR::new(utra::irqarray19::HW_IRQARRAY19_BASE as *mut u32);
    irqarray19.wfo(utra::irqarray19::EV_SOFT_TRIGGER, 0x80);
    report_api(0x3dcd_600d);
}

pub fn wfi_test() {
    report_api(0x03f1_0000);
    let mut tt = CSR::new(utra::ticktimer::HW_TICKTIMER_BASE as *mut u32);
    tt.wo(utra::ticktimer::CLOCKS_PER_TICK, 10000); // short-ish time to wake up
    tt.wfo(utra::ticktimer::CONTROL_RESET, 1);
    tt.wo(utra::ticktimer::MSLEEP_TARGET1, 0);
    tt.wo(utra::ticktimer::MSLEEP_TARGET0, 2);
    tt.wfo(utra::ticktimer::EV_ENABLE_ALARM, 1);
    unsafe { core::arch::asm!(
        "wfi",
    ); }
    tt.wo(utra::ticktimer::MSLEEP_TARGET0, 0xffff_ffff); // sometime way out there so we don't see it again during this test.
    report_api(0x03f1_600d);
}

// Notes: 403 CPU cycles to enter the handler (~4us wall-clock @ 100MHz).
// ~half of the time is burned storing the registers via the write-through cache.
// 713 cycles to fully return after doing some small amount of handling code,
// ~7.14us total entry/exit overhead
#[export_name = "_start_trap"]
// #[repr(align(4))] // can't do this yet.
#[inline(never)]
pub unsafe extern "C" fn _start_trap() -> ! {
    loop {
        // install a NOP sled before _start_trap() until https://github.com/rust-lang/rust/issues/82232 is stable
        core::arch::asm!(
            "nop",
            "nop",
        );
        #[export_name = "_start_trap_aligned"]
        pub unsafe extern "C" fn _start_trap_aligned() {
            core::arch::asm!(
                "csrw        sscratch, sp",
                "li          sp, 0x61008000", // crate::satp::SCRATCH_PAGE

                "sw       x1, 0*4(sp)",
                // Skip SP for now
                "sw       x3, 2*4(sp)",
                "sw       x4, 3*4(sp)",
                "sw       x5, 4*4(sp)",
                "sw       x6, 5*4(sp)",
                "sw       x7, 6*4(sp)",
                "sw       x8, 7*4(sp)",
                "sw       x9, 8*4(sp)",
                "sw       x10, 9*4(sp)",
                "sw       x11, 10*4(sp)",
                "sw       x12, 11*4(sp)",
                "sw       x13, 12*4(sp)",
                "sw       x14, 13*4(sp)",
                "sw       x15, 14*4(sp)",
                "sw       x16, 15*4(sp)",
                "sw       x17, 16*4(sp)",
                "sw       x18, 17*4(sp)",
                "sw       x19, 18*4(sp)",
                "sw       x20, 19*4(sp)",
                "sw       x21, 20*4(sp)",
                "sw       x22, 21*4(sp)",
                "sw       x23, 22*4(sp)",
                "sw       x24, 23*4(sp)",
                "sw       x25, 24*4(sp)",
                "sw       x26, 25*4(sp)",
                "sw       x27, 26*4(sp)",
                "sw       x28, 27*4(sp)",
                "sw       x29, 28*4(sp)",
                "sw       x30, 29*4(sp)",
                "sw       x31, 30*4(sp)",

                // Save SEPC
                "csrr        t0, sepc",
                "sw       t0, 31*4(sp)",

                // Save x1, which was used to calculate the offset.  Prior to
                // calculating, it was stashed at 0x61006000.
                //"li          t0, 0x61006000",
                //"lw        t1, 0*4(t0)",
                //"sw       t1, 0*4(sp)",

                // Finally, save SP
                "csrr        t0, sscratch",
                "sw          t0, 1*4(sp)",

                // Restore a default stack pointer
                "li          sp, 0x6100A000", // start from the page above the base: (crate::satp::EXCEPTION_STACK_LIMIT + 0x1000)

                // Note that registers $a0-$a7 still contain the arguments
                "j           _start_trap_rust",

                // Note to self: trying to assign the scratch and default pages using in(reg) syntax
                // clobbers the `a0` register and places the initialization outside of the handler loop
                // and there seems to be no way to refer directly to a symbol? the `sym` directive wants
                // to refer to an address, not a constant.
            );
        }
        _start_trap_aligned();
        core::arch::asm!(
            "nop",
            "nop",
        );
    }
}

#[export_name = "_resume_context"]
#[inline(never)]
pub unsafe extern "C" fn _resume_context(registers: u32) -> ! {
    core::arch::asm!(
        "move        sp, {registers}",

        "lw        x1, 0*4(sp)",
        // Skip SP for now
        "lw        x3, 2*4(sp)",
        "lw        x4, 3*4(sp)",
        "lw        x5, 4*4(sp)",
        "lw        x6, 5*4(sp)",
        "lw        x7, 6*4(sp)",
        "lw        x8, 7*4(sp)",
        "lw        x9, 8*4(sp)",
        "lw        x10, 9*4(sp)",
        "lw        x11, 10*4(sp)",
        "lw        x12, 11*4(sp)",
        "lw        x13, 12*4(sp)",
        "lw        x14, 13*4(sp)",
        "lw        x15, 14*4(sp)",
        "lw        x16, 15*4(sp)",
        "lw        x17, 16*4(sp)",
        "lw        x18, 17*4(sp)",
        "lw        x19, 18*4(sp)",
        "lw        x20, 19*4(sp)",
        "lw        x21, 20*4(sp)",
        "lw        x22, 21*4(sp)",
        "lw        x23, 22*4(sp)",
        "lw        x24, 23*4(sp)",
        "lw        x25, 24*4(sp)",
        "lw        x26, 25*4(sp)",
        "lw        x27, 26*4(sp)",
        "lw        x28, 27*4(sp)",
        "lw        x29, 28*4(sp)",
        "lw        x30, 29*4(sp)",
        "lw        x31, 30*4(sp)",

        // Restore SP
        "lw        x2, 1*4(sp)",
        "sret",
        registers = in(reg) registers,
    );
    loop {}
}

/// Just handles specific traps for testing CPU interactions. Doesn't do anything useful with the traps.
#[export_name = "_start_trap_rust"]
pub extern "C" fn trap_handler(
    _a0: usize,
    _a1: usize,
    _a2: usize,
    _a3: usize,
    _a4: usize,
    _a5: usize,
    _a6: usize,
    _a7: usize,
) -> ! {
    let mut main = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report_api(0x2dcd_0000);

    let sc: scause::Scause = scause::read();
    report_api(sc.bits() as u32);
    // 2 is illegal instruction
    if sc.bits() == 2 {
        // skip past the illegal instruction, since we are just testing that they trigger exceptions.
        unsafe {
            core::arch::asm!(
                "csrr        t0, sepc",
                "addi        t0, t0, 4",
                "csrw        sepc, t0",
            );
        }
    } else if sc.bits() == 0x8000_0009 {
        // external interrupt. find out which ones triggered it, and clear the source.
        let irqs_pending = sip::read();
        report_api(irqs_pending as u32);
        if (irqs_pending & (1 << 18)) != 0 {
            let mut irqarray18 = CSR::new(utra::irqarray18::HW_IRQARRAY18_BASE as *mut u32);
            #[cfg(feature="quanta-test")]
            {
                if irqarray18.rf(utra::irqarray18::EV_PENDING_PIOIRQ0_DUPE) != 0 {
                    report_api(0x51C2_1111);
                    let mut pio_ss = xous_pio::PioSharedState::new();
                    pio_ss.pio.wo(utra::rp_pio::SFR_IRQ, 1 << 0); // clear irq bit 0
                }
            }
            // handle irq18 hw test
            main.wfo(utra::main::IRQTEST0_TRIGGER, 0);
            let pending = irqarray18.r(utra::irqarray18::EV_PENDING);
            report_api(pending << 16 | 18); // encode the irq bank number and bit number as [bit | bank]
            irqarray18.wo(utra::irqarray18::EV_PENDING, pending);
        }
        if (irqs_pending & (1 << 19)) != 0 {
            // handle irq19 sw trigger test
            let mut irqarray19 = CSR::new(utra::irqarray19::HW_IRQARRAY19_BASE as *mut u32);
            let pending = irqarray19.r(utra::irqarray19::EV_PENDING);
            report_api(pending << 16 | 19); // encode the irq bank number and bit number as [bit | bank]
            irqarray19.wo(utra::irqarray19::EV_PENDING, pending);
            // software interrupt should not require a 0-write to reset it
        }
        if (irqs_pending & (1 << utra::ticktimer::TICKTIMER_IRQ)) != 0 {
            let mut tt = CSR::new(utra::ticktimer::HW_TICKTIMER_BASE as *mut u32);
            report_api(utra::ticktimer::TICKTIMER_IRQ as u32); // encode the irq bank number and bit number as [bit | bank]
            tt.wfo(utra::ticktimer::EV_PENDING_ALARM, 1); // clear the interrupt
            tt.wfo(utra::ticktimer::EV_ENABLE_ALARM, 0); // mask out the wakeup alarm
        }
    }

    // report interrupt status
    report_api(sepc::read() as u32);
    report_api(stval::read() as u32);
    report_api(sim::read() as u32);

    // re-enable interrupts
    let status: u32;
    unsafe {
        core::arch::asm!(
            "csrr        t0, sstatus",
            "ori         t0, t0, 3",
            "csrw        sstatus, t0",
            "csrr        {status}, sstatus",
            status = out(reg) status,
        )
    }
    unsafe{sie::set_sext()};
    report_api(status);

    // drop us back to user mode
    report_api(0x2dcd_600d);
    unsafe {_resume_context(crate::satp::SCRATCH_PAGE as u32)};
}