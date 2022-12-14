use utralib::generated::*;
use riscv::register::{scause, sepc, stval, vexriscv::sim, vexriscv::sip, sie};

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
            "la t0, _start_trap", // this first one forces the nop sled symbol to be generated
            "la t0, _start_trap_aligned", // this is the actual target
            "csrw stvec, t0",
        );
    }

    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x1dcd_0000);

    let mut irqarray0 = CSR::new(utra::irqarray0::HW_IRQARRAY0_BASE as *mut u32);
    let mut irqarray1 = CSR::new(utra::irqarray1::HW_IRQARRAY1_BASE as *mut u32);
    let mut irqarray2 = CSR::new(utra::irqarray2::HW_IRQARRAY2_BASE as *mut u32);
    // unmask interrupt sources
    irqarray0.wo(utra::irqarray0::EV_ENABLE, 0x7);
    irqarray1.wo(utra::irqarray1::EV_ENABLE, 0xF);
    irqarray2.wo(utra::irqarray2::EV_ENABLE, 0xFFFF_FFFF);
    // enable IRQ handling
    sim::write(0x0); // first make sure everything is disabled, so we aren't OR'ing in garbage
    enable_irq(utra::irqarray0::IRQARRAY0_IRQ);
    enable_irq(utra::irqarray1::IRQARRAY1_IRQ);
    enable_irq(utra::irqarray2::IRQARRAY2_IRQ);
    // for wfi testing
    let mut irqarray19 = CSR::new(utra::irqarray19::HW_IRQARRAY19_BASE as *mut u32);
    irqarray19.wo(utra::irqarray19::EV_ENABLE, 0xFFFF_FFFF);
    enable_irq(utra::irqarray19::IRQARRAY19_IRQ);

    // must enable external interrupts on the CPU for any of the above to matter
    unsafe{sie::set_sext()};

    report.wfo(utra::main::REPORT_REPORT, 0x1dcd_600d);
}

pub fn irq_test() {
    // trigger an interrupt
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x3dcd_0000);

    let mut main = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    // simulate hw trigger from IRQ0
    report.wfo(utra::main::REPORT_REPORT, 0x3dcd_0001);
    main.wfo(utra::main::IRQTEST0_TRIGGER, 4);
    // simulate hw trigger from IRQ1
    report.wfo(utra::main::REPORT_REPORT, 0x3dcd_0002);
    main.wfo(utra::main::IRQTEST1_TRIGGER, 1);
    // software-only trigger from IRQ2
    report.wfo(utra::main::REPORT_REPORT, 0x3dcd_0003);
    let mut irqarray2 = CSR::new(utra::irqarray2::HW_IRQARRAY2_BASE as *mut u32);
    irqarray2.wfo(utra::irqarray2::EV_SOFT_TRIGGER, 0x80);
    report.wfo(utra::main::REPORT_REPORT, 0x3dcd_600d);
}

pub fn wfi_test() {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x03f1_0000);
    unsafe { core::arch::asm!(
        "wfi",
    ); }
    report.wfo(utra::main::REPORT_REPORT, 0x03f1_600d);
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
                "li          sp, 0x61006000",
                //"sw          x1, 0*4(sp)",  // Store x1 in the scratch field
                //"lw          x1, 1*4(sp)",  // Load current context number
                //"slli        x1, x1, 7",           // Multiply current context number by 32
                //"add         sp, sp, x1",          // Set $sp to 0x61005000 + (current_context * 32)

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
                "sw       t0, 1*4(sp)",

                // Restore a default stack pointer
                "li          sp, 0x61008000", // SP goes from 6100_7000-6100_7FFF; place at top of range

                // Note that registers $a0-$a7 still contain the arguments
                "j           _start_trap_rust",
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
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    let mut main = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x2dcd_0000);

    let sc = scause::read();
    report.wfo(utra::main::REPORT_REPORT, sc.bits() as u32);
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
        report.wfo(utra::main::REPORT_REPORT, irqs_pending as u32);
        if (irqs_pending & 0x1) != 0 {
            // handle irq0 hw test
            main.wfo(utra::main::IRQTEST0_TRIGGER, 0);
            let mut irqarray0 = CSR::new(utra::irqarray0::HW_IRQARRAY0_BASE as *mut u32);
            let pending = irqarray0.r(utra::irqarray0::EV_PENDING);
            report.wfo(utra::main::REPORT_REPORT, pending << 16 | 0); // encode the irq bank number and bit number as [bit | bank]
            irqarray0.wo(utra::irqarray0::EV_PENDING, pending);
        }
        if (irqs_pending & 0x2) != 0 {
            // handle irq1 hw test
            main.wfo(utra::main::IRQTEST1_TRIGGER, 0);
            let mut irqarray1 = CSR::new(utra::irqarray1::HW_IRQARRAY1_BASE as *mut u32);
            let pending = irqarray1.r(utra::irqarray1::EV_PENDING);
            report.wfo(utra::main::REPORT_REPORT, pending << 16 | 1); // encode the irq bank number and bit number as [bit | bank]
            irqarray1.wo(utra::irqarray1::EV_PENDING, pending);
        }
        if (irqs_pending & 4) != 0 {
            // handle irq2 sw trigger test
            let mut irqarray2 = CSR::new(utra::irqarray2::HW_IRQARRAY2_BASE as *mut u32);
            let pending = irqarray2.r(utra::irqarray2::EV_PENDING);
            report.wfo(utra::main::REPORT_REPORT, pending << 16 | 2); // encode the irq bank number and bit number as [bit | bank]
            irqarray2.wo(utra::irqarray2::EV_PENDING, pending);
            // software interrupt should not require a 0-write to reset it
        }
        if (irqs_pending & (1 << 19)) != 0 {
            // handle wfi wakeup signal
            let mut irqarray19 = CSR::new(utra::irqarray19::HW_IRQARRAY19_BASE as *mut u32);
            let pending = irqarray19.r(utra::irqarray19::EV_PENDING);
            report.wfo(utra::main::REPORT_REPORT, pending << 16 | 19); // encode the irq bank number and bit number as [bit | bank]
            irqarray19.wo(utra::irqarray19::EV_PENDING, pending);
        }
    }

    // report interrupt status
    report.wfo(utra::main::REPORT_REPORT, sepc::read() as u32);
    report.wfo(utra::main::REPORT_REPORT, stval::read() as u32);
    report.wfo(utra::main::REPORT_REPORT, sim::read() as u32);

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
    report.wfo(utra::main::REPORT_REPORT, status);

    // drop us back to user mode
    report.wfo(utra::main::REPORT_REPORT, 0x2dcd_600d);
    unsafe {_resume_context(0x61006000)};
}