use utralib::generated::*;
use crate::pio_generated::utra::rp_pio;
use crate::pio::*;

/// Test the sticky out bits
pub fn sticky_test() {
    /* Test case from https://forums.raspberrypi.com/viewtopic.php?t=313962

    No sticky, but B using enable bit
    Cycle 1 :   A writes A1                                    : result = A1
    Cycle 2:    A writes A2, B writes B2 (without enable bit)  : result = A2
    Cycle 3:    A writes A3, B writes B3 (with enable bit)     : result = B3
    Cycle 4:    A writes A4                                    : result = A4
    Cycle 5:    A writes A5, B writes B5 (with enable bit)     : result = B5
    Cycle 5:    B writes B6 (without enable bit)               : result = B5

    With sticky set on both state machines (i.e. it is as if you did the OUT write on every cycle)
    Cycle 1 :   A writes A1                                    : result = A1
    Cycle 2:    A writes A2, B writes B2 (without enable bit)  : result = A2
    Cycle 3:    A writes A3, B writes B3 (with enable bit)     : result = B3
    Cycle 4:    A writes A4  (B rewrites B3)                   : result = B3
    Cycle 5:    A writes A5, B writes B5 (with enable bit)     : result = B5
    Cycle 5:   (A rewrites A5), B writes B6 (without enable bit) : result = A5
     */
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x51C2_0000);

    let mut sm_a = PioSm::new(1).unwrap();
    let mut sm_b = PioSm::new(2).unwrap();

    let a_code = pio_proc::pio_asm!(
        "set pins, 1",
        "set pins, 2",
        "set pins, 3",
        "set pins, 4",
        "set pins, 5",
        "nop"
    );
    let a_prog = LoadedProg::load(a_code.program, &mut sm_a).unwrap();
    let b_code = pio_proc::pio_asm!(
        // bit 4 indicates enable; bit 3 is the "B" machine flag. bits 2:0 are the payload
        "nop",
        "set pins, 0x0A", // without enable
        "set pins, 0x1B", // with enable
        "nop",
        "set pins, 0x1D", // with enable
        "set pins, 0x0E", // without enable
    );
    // note: this loads using sm_a so we can share the "used" vector state, but the code is global across all SM's
    let b_prog = LoadedProg::load(b_code.program, &mut sm_a).unwrap();

    sm_a.sm_set_enabled(false);
    sm_b.sm_set_enabled(false);

    a_prog.setup_default_config(&mut sm_a);
    b_prog.setup_default_config(&mut sm_b);

    sm_a.config_set_set_pins(24, 5);
    sm_b.config_set_set_pins(24, 5);

    sm_a.config_set_sideset(0, false, false);
    sm_b.config_set_sideset(0, false, false);

    sm_a.config_set_clkdiv(4.0);
    sm_b.config_set_clkdiv(4.0);

    sm_a.config_set_out_special(false, false, 0); // A has no special enabling
    sm_b.config_set_out_special(false, true, 28); // B uses output enable

    sm_a.sm_init(a_prog.entry());
    sm_b.sm_init(b_prog.entry());

    // use sm_a's PIO object to set state for both a & b here
    // restart dividers and machines so they are synchronized
    sm_a.pio.wo(
        rp_pio::SFR_CTRL,
        sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_a.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_a.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_b.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_b.sm as u32)
    );
    // now set both running at the same time
    report.wfo(utra::main::REPORT_REPORT, 0x51C2_1111);
    sm_a.pio.wo(
        rp_pio::SFR_CTRL,
        sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_a.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_b.sm as u32)
    );
    // wait for it to run
    for i in 0..16 {
        report.wfo(utra::main::REPORT_REPORT, 0x51C2_0000 + i as u32);
    }
    // disable the machines
    sm_a.pio.wo(rp_pio::SFR_CTRL, 0);

    report.wfo(utra::main::REPORT_REPORT, 0x51C2_2222);

    // now turn on the sticky bit
    sm_a.config_set_out_special(true, false, 0);
    sm_b.config_set_out_special(true, true, 28);
    // change clkdiv just to hit another corner case
    sm_a.config_set_clkdiv(1.0);
    sm_b.config_set_clkdiv(1.0);
    // commit config changes
    sm_a.sm_init(a_prog.entry());
    sm_b.sm_init(b_prog.entry());

    // restart dividers and machines so they are synchronized
    sm_a.pio.wo(
        rp_pio::SFR_CTRL,
        sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_a.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_a.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_b.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_b.sm as u32)
    );
    // now set both running at the same time
    report.wfo(utra::main::REPORT_REPORT, 0x51C2_3333);
    sm_a.pio.wo(
        rp_pio::SFR_CTRL,
        sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_a.sm as u32)
        | sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_b.sm as u32)
    );
    // wait for it to run
    for i in 0..16 {
        report.wfo(utra::main::REPORT_REPORT, 0x51C2_1000 + i as u32);
    }

    // disable the machines and cleanup
    sm_a.pio.wo(rp_pio::SFR_CTRL, 0);
    // clear the sticky bits
    sm_a.config_set_out_special(false, false, 0);
    sm_b.config_set_out_special(false, false, 0);
    sm_a.sm_init(a_prog.entry());
    sm_b.sm_init(b_prog.entry());
    // clear the instruction memory
    sm_a.clear_instruction_memory();

    report.wfo(utra::main::REPORT_REPORT, 0x51C2_600d);
}

pub fn delay(count: usize) {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    // dummy writes
    for i in 0..count {
        report.wo(utra::main::WDATA, i as u32);
    }
}
/// test that stalled imm instructions are restarted on restart
pub fn restart_imm_test() {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_0000);

    let mut sm_a = PioSm::new(0).unwrap();
    let a_code = pio_proc::pio_asm!(
        "set pins, 1",
        "set pins, 2",
        "set pins, 3",
        "set pins, 4",
        "set pins, 5",
        "nop"
    );
    let a_prog = LoadedProg::load(a_code.program, &mut sm_a).unwrap();
    sm_a.sm_set_enabled(false);
    a_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_set_pins(24, 5);
    sm_a.config_set_out_pins(24, 5);
    sm_a.config_set_sideset(0, false, false);
    sm_a.config_set_clkdiv(8.25);
    sm_a.config_set_out_shift(false, true, 16);
    sm_a.sm_init(a_prog.entry());
    // run the loop on A
    report.wfo(utra::main::REPORT_REPORT, 0x0133_1111);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm as u32);
    delay(50);

    let mut a = pio::Assembler::<32>::new();
    a.out(pio::OutDestination::PINS, 16);
    let p= a.assemble_program();

    // this should stall the state machine
    sm_a.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_2222);
    delay(50);

    // this should clear the stall
    sm_a.pio.rmwf(rp_pio::SFR_CTRL_RESTART, sm_a.sm as u32);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_3333);
    delay(50);

    // this should stall the state machine again
    sm_a.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_4444);
    delay(50);

    // this should also clear the stall by resolving the halt condition with a tx_fifo push
    sm_a.sm_txfifo_push_u16_msb(0xFFFF);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_5555);
    delay(50);

    sm_a.pio.wo(rp_pio::SFR_CTRL, 0);
    sm_a.clear_instruction_memory();
    report.wfo(utra::main::REPORT_REPORT, 0x0133_600d);
}

pub fn fifo_join_test() -> bool {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_0000);

    // test TX fifo with non-join. Simple program that just copies the TX fifo content to pins, then stalls.
    let mut sm_a = PioSm::new(0).unwrap();
    let a_code = pio_proc::pio_asm!(
        "out pins, 32",
    );
    let a_prog = LoadedProg::load(a_code.program, &mut sm_a).unwrap();
    sm_a.sm_set_enabled(false);
    a_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_out_pins(0, 32);
    sm_a.config_set_clkdiv(128.0); // could make as aggressive as 64.0 and have it still work with 1:1 bus timings...
    sm_a.config_set_out_shift(false, true, 0);
    sm_a.sm_init(a_prog.entry());

    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_1111);
    // load up the TX fifo, count how many entries it takes until it is full
    // note: full test requires manual inspection of waveform to confirm GPIO out has the expected report value.
    let mut entries = 0;
    while !sm_a.sm_txfifo_is_full() {
        entries += 1;
        sm_a.sm_txfifo_push_u32(0xF1F0_0000 + entries);
    }
    let mut passing = true;
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_1000 + entries);
    // push the FIFO data out, and try to compare using PIO capture (clkdiv set slow so we can do this...)
    let mut last_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
    let mut detected = 0;
    // run the machine
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm as u32);
    while detected < entries {
        let latest_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
        if latest_val != last_val {
            detected += 1;
            if latest_val != (0xF1F0_0000 + detected) {
                passing = false;
            }
            last_val = latest_val;
        }
    }

    // this should set Join TX and also halt the engine
    sm_a.config_set_fifo_join(PioFifoJoin::JoinTx);
    sm_a.sm_init(a_prog.entry());

    // repeat, this time measuring the depth of the FIFO with join
    let mut entries = 0;
    while !sm_a.sm_txfifo_is_full() {
        entries += 1;
        sm_a.sm_txfifo_push_u32(0xF1F0_0000 + entries);
    }
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_2000 + entries);
    // should push the FIFO out
    last_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
    detected = 0;
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm as u32);
    while detected < entries {
        let latest_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
        if latest_val != last_val {
            detected += 1;
            if latest_val != (0xF1F0_0000 + detected) {
                passing = false;
            }
            last_val = latest_val;
        }
    }

    // a program for testing IN
    let b_code = pio_proc::pio_asm!(
        "   set x, 16",
        "loop: ",
        "   in x, 32",
        "   push block",
        "   jmp x--, loop",
    );
    let b_prog = LoadedProg::load(b_code.program, &mut sm_a).unwrap();

    // setup for rx test
    sm_a.sm_set_enabled(false);
    b_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_fifo_join(PioFifoJoin::None);
    sm_a.config_set_clkdiv(16.0);

    sm_a.sm_init(b_prog.entry());
    // start the program running
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_3333);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm as u32);
    while !sm_a.sm_rxfifo_is_full() {
        // just wait until the rx fifo fill sup
    }
    // stop filling it
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, 0);
    entries = 0;
    let mut expected = 16;
    while !sm_a.sm_rxfifo_is_empty() {
        let val = sm_a.sm_rxfifo_pull_u32();
        if val != expected {
            passing = false;
        }
        report.wfo(utra::main::REPORT_REPORT,
            0xF1F0_0000 + val
        );
        entries += 1;
        expected -= 1;
    }
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_3000 + entries);

    // now join
    sm_a.config_set_fifo_join(PioFifoJoin::JoinRx);
    sm_a.sm_init(b_prog.entry());
    // start the program running
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4444);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm as u32);
    while !sm_a.sm_rxfifo_is_full() {
        // just wait until the rx fifo fill sup
    }
    // stop filling it
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, 0);
    entries = 0;
    expected = 16;
    while !sm_a.sm_rxfifo_is_empty() {
        let val = sm_a.sm_rxfifo_pull_u32();
        if val != expected {
            passing = false;
        }
        report.wfo(utra::main::REPORT_REPORT,
            0xF1F0_0000 + val
        );
        entries += 1;
        expected -= 1;
    }
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4000 + entries);

    sm_a.clear_instruction_memory();

    if passing {
        report.wfo(utra::main::REPORT_REPORT, 0xF1F0_600D);
    } else {
        report.wfo(utra::main::REPORT_REPORT, 0xF1F0_DEAD);
    }
    passing
}
