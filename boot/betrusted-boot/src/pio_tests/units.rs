use pio::RP2040_MAX_PROGRAM_SIZE;
use utralib::generated::*;
use crate::pio_generated::utra::rp_pio::{self, SFR_FDEBUG, SFR_FLEVEL, SFR_FSTAT, SFR_DBG_CFGINFO};
use crate::pio::*;

/// Test the sticky out bits
pub fn sticky_test() {
    /* Test case from https://forums.raspberrypi.com/viewtopic.php?t=313962

    Reading the waveforms: bits 24-26 correspond to the number in the test result
    Bit 27 indicates A/B: 0 means A, 1 means B
    Bit 28 is the side-set enable bit

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

    let mut pio_ss = PioSharedState::new();
    let mut sm_a = unsafe{pio_ss.force_alloc_sm(1).unwrap()};
    let mut sm_b = unsafe{pio_ss.force_alloc_sm(2).unwrap()};

    let a_code = pio_proc::pio_asm!(
        "set pins, 1",
        "set pins, 2",
        "set pins, 3",
        "set pins, 4",
        "set pins, 5",
        "nop"
    );
    let a_prog = LoadedProg::load(a_code.program, &mut pio_ss).unwrap();
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
    let b_prog = LoadedProg::load(b_code.program, &mut pio_ss).unwrap();

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
        sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_a.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_a.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_b.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_b.sm_bitmask())
    );
    // now set both running at the same time
    report.wfo(utra::main::REPORT_REPORT, 0x51C2_1111);
    sm_a.pio.wo(
        rp_pio::SFR_CTRL,
        sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_b.sm_bitmask())
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
        sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_a.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_a.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_CLKDIV_RESTART, sm_b.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_RESTART, sm_b.sm_bitmask())
    );
    // now set both running at the same time
    report.wfo(utra::main::REPORT_REPORT, 0x51C2_3333);
    sm_a.pio.wo(
        rp_pio::SFR_CTRL,
        sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask())
        | sm_a.pio.ms(rp_pio::SFR_CTRL_EN, sm_b.sm_bitmask())
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
    pio_ss.clear_instruction_memory();

    // NOTE: this test requires manual inspection of the output waveforms for pass/fail.
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

    let mut pio_ss = PioSharedState::new();
    let mut sm_a = pio_ss.alloc_sm().unwrap();
    let a_code = pio_proc::pio_asm!(
        "set pins, 1",
        "set pins, 2",
        "set pins, 3",
        "set pins, 4",
        "set pins, 5",
        "nop"
    );
    let a_prog = LoadedProg::load(a_code.program, &mut pio_ss).unwrap();
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
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
    delay(50);
    assert!(sm_a.pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 0);

    let mut a = pio::Assembler::<32>::new();
    a.out(pio::OutDestination::PINS, 16);
    let p= a.assemble_program();

    // this should stall the state machine
    sm_a.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_2222);
    delay(50);
    assert!(sm_a.pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 1);

    // this should clear the stall
    sm_a.pio.rmwf(rp_pio::SFR_CTRL_RESTART, sm_a.sm_bitmask());
    report.wfo(utra::main::REPORT_REPORT, 0x0133_3333);
    delay(50);
    assert!(sm_a.pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 0);

    // this should stall the state machine again
    sm_a.sm_exec(p.code[p.origin.unwrap_or(0) as usize]);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_4444);
    delay(50);
    assert!(sm_a.pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 1);

    // this should also clear the stall by resolving the halt condition with a tx_fifo push
    sm_a.sm_txfifo_push_u16_msb(0xFFFF);
    report.wfo(utra::main::REPORT_REPORT, 0x0133_5555);
    delay(50);
    assert!(sm_a.pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 0);

    sm_a.pio.wo(rp_pio::SFR_CTRL, 0);
    pio_ss.clear_instruction_memory();
    report.wfo(utra::main::REPORT_REPORT, 0x0133_600d);
}

pub fn fifo_join_test() -> bool {
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_0000);

    let mut pio_ss = PioSharedState::new();
    // test TX fifo with non-join. Simple program that just copies the TX fifo content to pins, then stalls.
    let mut sm_a = pio_ss.alloc_sm().unwrap();
    let a_code = pio_proc::pio_asm!(
        "out pins, 32",
    );
    let a_prog = LoadedProg::load(a_code.program, &mut pio_ss).unwrap();
    sm_a.sm_set_enabled(false);
    pio_ss.pio.wo(rp_pio::SFR_IRQ0_INTE, 0); // clear these in case a previous test set them
    pio_ss.pio.wo(rp_pio::SFR_IRQ1_INTE, 0);
    a_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_out_pins(0, 32);
    sm_a.config_set_clkdiv(192.0); // slow down the machine so we can read out the values after writing them...
    sm_a.config_set_out_shift(false, true, 0);
    sm_a.sm_init(a_prog.entry());
    sm_a.sm_irq0_source_enabled(PioIntSource::TxNotFull, true);

    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_1111);
    // load up the TX fifo, count how many entries it takes until it is full
    // note: full test requires manual inspection of waveform to confirm GPIO out has the expected report value.
    let mut entries = 0;
    while !sm_a.sm_txfifo_is_full() {
        entries += 1;
        sm_a.sm_txfifo_push_u32(0xF1F0_0000 + entries);
    }
    assert!(entries == 4);
    let mut passing = true;
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_1000 + entries);
    // push the FIFO data out, and try to compare using PIO capture (clkdiv set slow so we can do this...)
    let mut last_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
    let mut detected = 0;
    // run the machine
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
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
    assert!(entries == 8);
    // should push the FIFO out
    last_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
    detected = 0;
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
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

    // this should reset join TX and also halt the engine
    sm_a.config_set_fifo_join(PioFifoJoin::None);
    sm_a.sm_init(a_prog.entry());

    // now test with "margin" on the FIFOs.
    assert!(sm_a.sm_get_tx_fifo_margin() == 0);
    sm_a.sm_set_tx_fifo_margin(1);
    assert!(sm_a.sm_get_tx_fifo_margin() == 1);
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::TxNotFull)));

    // repeat, this time measuring the depth of the FIFO with margin
    let mut entries = 0;
    // loop looks at the raw interrupt value, the asserts look at the feedback INTS value, so we have coverage of both views
    while (pio_ss.pio.rf(rp_pio::SFR_INTR_INTR_TXNFULL) & sm_a.sm_bitmask()) != 0 {
        entries += 1;
        sm_a.sm_txfifo_push_u32(0xF1F0_0000 + entries);
    }
    assert!(entries == 3);
    assert!(sm_a.sm_txfifo_level() == 3); // should have space for one more item.
    assert!(sm_a.sm_txfifo_is_full() == false); // the actual "full" signal should not be asserted.
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::TxNotFull)) == false);
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_2100 + entries);
    // push the FIFO data out, and try to compare using PIO capture (clkdiv set slow so we can do this...)
    let mut last_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
    let mut detected = 0;
    // run the machine
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
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
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_2100 + if passing {1} else {0});
    sm_a.sm_set_tx_fifo_margin(0);

    // this should reset join TX and also halt the engine
    sm_a.config_set_fifo_join(PioFifoJoin::JoinTx);
    sm_a.sm_init(a_prog.entry());

    // now test with "margin" on the FIFOs.
    assert!(sm_a.sm_get_tx_fifo_margin() == 0);
    sm_a.sm_set_tx_fifo_margin(1);
    assert!(sm_a.sm_get_tx_fifo_margin() == 1);
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::TxNotFull)));

    // repeat, this time measuring the depth of the FIFO with margin
    let mut entries = 0;
    // loop looks at the raw interrupt value, the asserts look at the feedback INTS value, so we have coverage of both views
    while (sm_a.pio.rf(rp_pio::SFR_INTR_INTR_TXNFULL) & sm_a.sm_bitmask()) != 0 {
        entries += 1;
        sm_a.sm_txfifo_push_u32(0xF1F0_0000 + entries);
    }
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_2200 + entries);
    assert!(entries == 7);
    assert!(sm_a.sm_rxfifo_level() == 3); // should have space for one more item.
    assert!(sm_a.sm_txfifo_level() == 4); // this one should be full
    assert!(sm_a.sm_txfifo_is_full() == false); // the actual "full" signal should not be asserted.
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::TxNotFull)) == false);
    // push the FIFO data out, and try to compare using PIO capture (clkdiv set slow so we can do this...)
    let mut last_val = sm_a.pio.r(rp_pio::SFR_DBG_PADOUT);
    let mut detected = 0;
    // run the machine
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
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
    sm_a.sm_irq0_source_enabled(PioIntSource::TxNotFull, false);
    sm_a.sm_set_tx_fifo_margin(0);
    sm_a.sm_irq0_source_enabled(PioIntSource::RxNotEmpty, true);

    // a program for testing IN
    let b_code = pio_proc::pio_asm!(
        "   set x, 16",
        "loop: ",
        "   in x, 32",
        "   push block",
        "   jmp x--, loop",
    );
    let b_prog = LoadedProg::load(b_code.program, &mut pio_ss).unwrap();

    // setup for rx test
    sm_a.sm_set_enabled(false);
    b_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_fifo_join(PioFifoJoin::None);
    sm_a.config_set_clkdiv(16.0);

    sm_a.sm_init(b_prog.entry());
    // start the program running
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_3333);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
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
    assert!(entries == 4);

    // now join
    sm_a.config_set_fifo_join(PioFifoJoin::JoinRx);
    sm_a.sm_init(b_prog.entry());
    // start the program running
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4444);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
    while !sm_a.sm_rxfifo_is_full() {
        // just wait until the rx fifo fills up
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
    assert!(entries == 8);
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4000 + entries);

    // no join, but with margin
    sm_a.config_set_fifo_join(PioFifoJoin::None);
    sm_a.sm_init(b_prog.entry());

    // now test with "margin" on the FIFOs.
    assert!(sm_a.sm_get_rx_fifo_margin() == 0);
    sm_a.sm_set_rx_fifo_margin(1);
    assert!(sm_a.sm_get_rx_fifo_margin() == 1);
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::RxNotEmpty)) == false);

    // start the program running
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4555);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
    while !sm_a.sm_rxfifo_is_full() {
        // just wait until the rx fifo fills up
    }
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::RxNotEmpty)) == true);
    // stop filling it
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, 0);
    entries = 0;
    expected = 16;
    while (pio_ss.pio.rf(rp_pio::SFR_INTR_INTR_RXNEMPTY) & sm_a.sm_bitmask()) != 0  {
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
    assert!(entries == 3);
    assert!(sm_a.sm_rxfifo_level() == 1); // should be exactly one entry left
    assert!(sm_a.sm_rxfifo_is_empty() == false); // the actual "empty" signal should not be asserted.
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::RxNotEmpty)) == false);
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4100 + entries);
    sm_a.sm_set_rx_fifo_margin(0);

    // join, but with margin
    sm_a.config_set_fifo_join(PioFifoJoin::JoinRx);
    sm_a.sm_init(b_prog.entry());

    // now test with "margin" on the FIFOs.
    assert!(sm_a.sm_get_rx_fifo_margin() == 0);
    sm_a.sm_set_rx_fifo_margin(1);
    assert!(sm_a.sm_get_rx_fifo_margin() == 1);
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::RxNotEmpty)) == false);

    // start the program running
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4666);
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, sm_a.sm_bitmask());
    while !sm_a.sm_rxfifo_is_full() {
        // just wait until the rx fifo fills up
    }
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::RxNotEmpty)) == true);
    // stop filling it
    sm_a.pio.wfo(rp_pio::SFR_CTRL_EN, 0);
    entries = 0;
    expected = 16;
    while (pio_ss.pio.rf(rp_pio::SFR_INTR_INTR_RXNEMPTY) & sm_a.sm_bitmask()) != 0  {
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
    report.wfo(utra::main::REPORT_REPORT, 0xF1F0_4200 + entries);
    assert!(entries == 7);
    assert!(sm_a.sm_rxfifo_level() == 1); // this one should have one entry left
    assert!(sm_a.sm_txfifo_level() == 0); // should be empty
    assert!(sm_a.sm_rxfifo_is_empty() == false); // the actual "empty" signal should not be asserted.
    assert!(sm_a.sm_irq0_status(Some(PioIntSource::RxNotEmpty)) == false);

    // clean up
    sm_a.sm_irq0_source_enabled(PioIntSource::RxNotEmpty, false);
    sm_a.sm_set_rx_fifo_margin(0);
    pio_ss.clear_instruction_memory();

    if passing {
        report.wfo(utra::main::REPORT_REPORT, 0xF1F0_600D);
    } else {
        report.wfo(utra::main::REPORT_REPORT, 0xF1F0_DEAD);
    }
    assert!(passing); // stop the test bench if there was a failure
    passing
}

/// A test designed to exercise as much of the APB register interface as we can.
///
/// The test sets up four SM's to run simultaneously. SM3 does the "master sync"
/// with an IRQ instruction. After that point, all four should update their respective
/// GPIO pins simultaneously, and then wait for a `1` on GPIO 31.
///
/// The value sent to the GPIO pins should be pre-loaded into the TX fifo before
/// the test runs. The loop also puts the value of a loop counter into the RX fifo
/// as the loop runs, so these can be read out and checked for correctness. The loop
/// counter for each test starts at a different offset, so, we can be sure there is
/// no cross-wiring of registers or FIFOS by checking the offsets.
///
/// The interlocking of the test means we can also read the FIFO empty/full bits and
/// do asserts on them throughout the test.
///
/// The input/output registers have to be configured carefully, because each of the
/// code loops is constructed slightly differently to exercise different corner cases
/// of the input/output configurations. The pin mapping is as follows:
///
/// GPIO#   Input    Output
/// 0..4             SM0 TX fifo readout LSBs
/// 4..8             SM1 TX fifo readout LSBs
/// 8..12            SM2 TX fifo readout LSBs
/// 12..16           SM3 TX fifo readout LSBs
/// 14..16           SM3 sideset -- deliberately conflicting with SM3 TX fifo readout to test sideset > out
/// 16..18           SM0 sideset
/// 18..20           SM1 sideset via pindirs
/// 20..22           SM2 sideset
/// 31               synchronizing GPIO input
///

pub fn register_tests() {
    const REGTEST_DIV: f32 = 2.5;
    let mut report = CSR::new(utra::main::HW_MAIN_BASE as *mut u32);
    report.wfo(utra::main::REPORT_REPORT, 0x1336_0000);

    let mut pio_ss = PioSharedState::new();

    let mut sm_a = pio_ss.alloc_sm().unwrap();
    sm_a.pio.wo(rp_pio::SFR_CTRL, 0xFF0); // reset all state machines to a known state.
    sm_a.pio.wo(rp_pio::SFR_FDEBUG, 0xFFFF_FFFF); // clear all the FIFO debug registers

    let a_code = pio_proc::pio_asm!(
        ".side_set 2 opt",
        "   set x, 24",                 // 18
        "loop: ",
        "   in x, 0        side 3 [2]", // 19 0 maps to 32
        "   push block            [1]", // 1A puts X into the output FIFO
        "   wait 1 irq 1   side 2",     // 1B wait until IRQ0 is set to 1
        "   out pins, 32   side 1",     // 1C now push OSR onto the GPIO pins
        "   wait 1 gpio 31 side 0",     // 1D wait until GPIO 31 is 1
        "   wait 0 gpio 31 side 1",     // 1E wait until GPIO 31 is 0
        "   jmp x--, loop",             // 1F
    );
    report.wfo(utra::main::REPORT_REPORT, a_code.program.side_set.bits() as u32);
    let a_prog = LoadedProg::load(a_code.program, &mut pio_ss).unwrap();
    sm_a.sm_set_enabled(false);
    a_prog.setup_default_config(&mut sm_a);
    sm_a.config_set_out_pins(0, 4);
    sm_a.config_set_in_pins(16); // should have no impact on "wait" because it is absolutely specified
    sm_a.config_set_out_shift(false, true, 0);
    sm_a.config_set_sideset_pins(16);
    sm_a.config_set_clkdiv(REGTEST_DIV);
    sm_a.config_set_set_pins(31, 1); // special case, A is used to set GPIO 31 to resume machines on wait
    sm_a.sm_init(a_prog.entry());

    let b_code = pio_proc::pio_asm!(
        ".side_set 2 opt pindirs",
        "   set y, 16 [1]",             // 10 start with some variable delay to prove syncing works
        "loop: ",
        "   in y, 0        side 3 [1]", // 11 0 maps to 32
        "   push block            [1]", // 12 puts Y into the output FIFO
        "   wait 1 irq 1   side 2",     // 13 wait until IRQ0 is set to 1
        "   out pins, 32   side 1",     // 14 now push OSR onto the GPIO pins
        "   wait 1 pin 0   side 0",     // 15 wait until the mapped input pin is 1. Map this to GPIO 31.
        "   wait 0 pin 0   side 1",     // 16 wait until the mapped input pin is 0. Map this to GPIO 31.
        "   jmp y--, loop",             // 17
    );
    let mut sm_b = pio_ss.alloc_sm().unwrap();
    let b_prog = LoadedProg::load(b_code.program, &mut pio_ss).unwrap();
    sm_b.sm_set_enabled(false);
    b_prog.setup_default_config(&mut sm_b);
    sm_b.config_set_out_pins(4, 4);
    sm_b.config_set_in_pins(31); // maps pin 0 to GPIO 31
    sm_b.config_set_out_shift(false, true, 0);
    sm_b.config_set_sideset_pins(18);
    sm_b.config_set_clkdiv(REGTEST_DIV);
    sm_b.sm_init(b_prog.entry());

    let c_code = pio_proc::pio_asm!(
        ".side_set 2 opt",
        "   set x, 8 [2]",
        "loop: ",
        "   in x, 0        side 3 [1]", // 0 maps to 32
        "   push block            [2]", // puts X into the output FIFO
        "   wait 1 irq 1   side 2",     // wait until IRQ0 is set to 1
        "   out pins, 32   side 1",     // now push OSR onto the GPIO pins.
        "   wait 1 gpio 31 side 0",     // wait until GPIO 31 is 1
        "   wait 0 gpio 31 side 1",     // wait until GPIO 31 is 0
        "   jmp x--, loop",
    );
    let mut sm_c =  pio_ss.alloc_sm().unwrap();
    let c_prog = LoadedProg::load(c_code.program, &mut pio_ss).unwrap();
    sm_c.sm_set_enabled(false);
    c_prog.setup_default_config(&mut sm_c);
    sm_c.config_set_out_pins(8, 4);
    sm_c.config_set_in_pins(24); // should not matter because absolute GPIO is used
    sm_c.config_set_out_shift(false, true, 0);
    sm_c.config_set_sideset_pins(20);
    sm_c.config_set_clkdiv(REGTEST_DIV);
    sm_c.sm_init(c_prog.entry());

    let d_code = pio_proc::pio_asm!(
        ".side_set 2 opt",
        "   set y, 2  [3]",
        "loop: ",
        "   in y, 0        side 3 [2]", // 0 maps to 32
        "   push block",                // puts Y into the output FIFO
        "   irq set 1      side 2",     // set the IRQ, so all the machines sync here
        "   out pins, 32   side 1",     // now push OSR onto the GPIO pins. This one's side set interferes with GPIO mappings deliberately.
        "   wait 1 pin 7   side 0",     // wait until the mapped input pin is 1. Map this to GPIO 31.
        "   wait 0 pin 7   side 0",     // wait until the mapped input pin is 0. Map this to GPIO 31.
        "   jmp y--, loop",
    );
    let mut sm_d =  pio_ss.alloc_sm().unwrap();
    let d_prog = LoadedProg::load(d_code.program, &mut pio_ss).unwrap();
    sm_d.sm_set_enabled(false);
    d_prog.setup_default_config(&mut sm_d);
    sm_d.config_set_out_pins(12, 4);
    sm_d.config_set_in_pins(24); // maps pin 0 to GPIO 31
    sm_d.config_set_out_shift(false, true, 0);
    sm_d.config_set_sideset_pins(14); // deliberate conflict with out_pins
    sm_d.config_set_clkdiv(REGTEST_DIV);
    sm_d.sm_init(d_prog.entry());

    // enable interrupts for readback on IRQ0
    sm_a.pio.wo(rp_pio::SFR_IRQ0_INTE, 0xFFF);

    report.wfo(utra::main::REPORT_REPORT, 0x1336_0001);

    // confirm that the FIFOs are all in expected states. Hard-coded as expected values for efficiency.
    assert!(sm_a.pio.r(SFR_FDEBUG) == 0);
    assert!(sm_a.pio.r(SFR_FLEVEL) == 0);
    assert!(sm_a.pio.r(SFR_FSTAT) == 0x0F00_0F00);
    assert!(sm_a.pio.r(SFR_DBG_CFGINFO) == 0x0020_0404); // hard-coded number, check that it's correct

    // dump the loaded instructions. Tests all the INSTR registers for readback.
    // must be re-generated every time programs are updated
    let expected_instrs: [u16; 32] = [
        0xe342,
        0x5e40,
        0x8020,
        0xd801,
        0x7400,
        0x30a7,
        0x3027,
        0x0081,
        0xe228,
        0x5d20,
        0x8220,
        0x38c1,
        0x7400,
        0x309f,
        0x341f,
        0x0049,
        0xe150,
        0x5d40,
        0x8120,
        0x38c1,
        0x7400,
        0x30a0,
        0x3420,
        0x0091,
        0xe038,
        0x5e20,
        0x8120,
        0x38c1,
        0x7400,
        0x309f,
        0x341f,
        0x0059,
    ];
    for i in 0..RP2040_MAX_PROGRAM_SIZE {
        let rbk = unsafe{sm_a.pio.base.add(rp_pio::SFR_INSTR_MEM0.offset() + i).read_volatile()};
        report.wfo(utra::main::REPORT_REPORT, rbk + ((i as u32) << 24));
        assert!(rbk as u16 == expected_instrs[i]);
    }
    report.wfo(utra::main::REPORT_REPORT, 0x1336_0002);

    // load the TX fifos with output data we expect to see on the GPIO pins
    let tx_vals: [[u32; 4]; 4] =
    [
        [0x3, 0xC, 0x6, 0x0],
        [0xA, 0x5, 0x0, 0xF],
        [0xC, 0x0, 0x1, 0x2],
        [0x3, 0x2, 0x1, 0x0],
    ];
    // load the FIFO values, and check that the levels & flags change as we expected.
    let mut sm_array = [sm_a, sm_b, sm_c, sm_d];
    for (sm_index, sm) in sm_array.iter_mut().enumerate() {
        /* report.wfo(utra::main::REPORT_REPORT,
            (sm_index as u32) << 16 |
            if sm.sm_txfifo_is_empty() {0x8000} else {0x0} |
            sm.sm_txfifo_level() as u32
        ); */
        assert!(sm.sm_txfifo_is_empty() == true);
        assert!(sm.sm_txfifo_level() == 0);
        // TXNFULL should be asserted
        assert!((sm.pio.r(rp_pio::SFR_IRQ0_INTS) >> 4) & sm.sm_bitmask() != 0);
        // RXNEMPTY should be de-asserted
        assert!((sm.pio.r(rp_pio::SFR_IRQ0_INTS) >> 0) & sm.sm_bitmask() == 0);
        for (index, &word) in tx_vals[sm_index].iter().enumerate() {
            sm.sm_txfifo_push_u32(word);
            // report.wfo(utra::main::REPORT_REPORT, 0x1336_0031);
            assert!(sm.sm_txfifo_is_empty() == false);
            // report.wfo(utra::main::REPORT_REPORT, 0x1336_0000 + sm.sm_txfifo_level() as u32);
            assert!(sm.sm_txfifo_level() == index + 1);
            // report.wfo(utra::main::REPORT_REPORT, 0x1336_0033);
        }
        // report.wfo(utra::main::REPORT_REPORT, 0x1336_0004);
        assert!(sm.sm_txfifo_is_full() == true);
        // TXNFULL should be de-asserted
        assert!((sm.pio.r(rp_pio::SFR_IRQ0_INTS) >> 4) & sm.sm_bitmask() == 0);
        // push an extra value and confirm that we cause an overflow
        sm.sm_txfifo_push_u8_msb(0x7); // Note: this number does not appear in the loaded set
        // confirm that we see the overflow flag; then clear it, and confirm it's cleared.
        report.wfo(utra::main::REPORT_REPORT, 0x1336_0005);
        assert!(sm.pio.rf(rp_pio::SFR_FDEBUG_TXOVER) == sm.sm_bitmask());
        sm.pio.wfo(rp_pio::SFR_FDEBUG_TXOVER, sm.sm_bitmask());
        report.wfo(utra::main::REPORT_REPORT, 0x1336_0006);
        assert!(sm.pio.rf(rp_pio::SFR_FDEBUG_TXOVER) == 0);
    }

    // prepare instruction to flip OE on bit 31 to move the state machine forward
    // requires testbench to wire that bit back in as an input on GPIO 31 for the test to complete!!
    let mut a = pio::Assembler::<RP2040_MAX_PROGRAM_SIZE>::new();
    a.set(pio::SetDestination::PINDIRS, 1);
    let p= a.assemble_program();
    let set_bit31_oe: u16 = p.code[p.origin.unwrap_or(0) as usize];
    // program that clears the same bit
    let mut c = pio::Assembler::<RP2040_MAX_PROGRAM_SIZE>::new();
    c.set(pio::SetDestination::PINDIRS, 0);
    let p2= c.assemble_program();
    let clear_bit31_oe: u16 = p2.code[p2.origin.unwrap_or(0) as usize];
    // prepare an instruction that stalls
    let mut b = pio::Assembler::<RP2040_MAX_PROGRAM_SIZE>::new();
    b.wait(1, pio::WaitSource::IRQ, 0, false);
    let p_wait = b.assemble_program();

    // check that the RX FIFOs have the correct levels
    report.wfo(utra::main::REPORT_REPORT, 0x1336_0007);
    for sm in sm_array.iter_mut() {
        assert!(sm.sm_rxfifo_is_empty());
        assert!(sm.sm_rxfifo_level() == 0);
    }

    // start the machines running
    sm_array[0].pio.wfo(rp_pio::SFR_CTRL_CLKDIV_RESTART, 0xF); // sync the clocks; the clock free-runs after the div is setup, and the divs are set up at arbitrary points in time
    sm_array[0].pio.wfo(rp_pio::SFR_CTRL_EN, 0xF);
    report.wfo(utra::main::REPORT_REPORT, 0x1336_0008);

    let mut waiting_for = 0;
    loop {
        if waiting_for >= tx_vals[0].len() {
            report.wfo(utra::main::REPORT_REPORT, 0x1336_000B);
            break;
        }
        // assembled the expected value
        let mut expected = 0;
        for (index, vals) in tx_vals.iter().enumerate() {
            expected |= (vals[waiting_for] & 0xF) << (index as u32 * 4);
        }
        // compensate for sideset override on SM3 (doesn't apply, because the sideset is out of phase with out)
        // expected &= 0x3FFF;
        // expected |= 0x4000; // "side 1" should be executed on SM3 on bits 14-16, overriding any TX fifo value

        let outputs = sm_array[0].pio.r(rp_pio::SFR_DBG_PADOUT);
        report.wfo(utra::main::REPORT_REPORT, 0x1336_0000 | (outputs & 0xFFFF));
        if expected == (outputs & 0xFFFF) {
            // got it, moving forward
            waiting_for += 1;
            report.wfo(utra::main::REPORT_REPORT, 0x0000_1336 | ((waiting_for as u32) << 16)); // report waiting_for

            // check that RX fifos have the right number of entries
            for sm in sm_array.iter_mut() {
                assert!(sm.sm_rxfifo_level() == waiting_for);
            }
            // no "exec" is in progress, so the stall bit should not be set
            report.wfo(utra::main::REPORT_REPORT, 0x1336_0007);
            assert!(sm_array[0].pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 0);
            assert!(sm_array[1].pio.rf(rp_pio::SFR_SM1_EXECCTRL_EXEC_STALLED_RO1) == 0);
            assert!(sm_array[2].pio.rf(rp_pio::SFR_SM2_EXECCTRL_EXEC_STALLED_RO2) == 0);
            assert!(sm_array[3].pio.rf(rp_pio::SFR_SM3_EXECCTRL_EXEC_STALLED_RO3) == 0);

            // read the address of the PC, and confirm the instruction is correct
            report.wfo(utra::main::REPORT_REPORT, 0x1336_0008);
            assert!(expected_instrs[sm_array[0].pio.rf(rp_pio::SFR_SM0_ADDR_PC) as usize] ==
                    sm_array[0].pio.rf(rp_pio::SFR_SM0_INSTR_IMM_INSTR) as u16);
            assert!(expected_instrs[sm_array[1].pio.rf(rp_pio::SFR_SM1_ADDR_PC) as usize] ==
                    sm_array[1].pio.rf(rp_pio::SFR_SM1_INSTR_IMM_INSTR) as u16);
            assert!(expected_instrs[sm_array[2].pio.rf(rp_pio::SFR_SM2_ADDR_PC) as usize] ==
                    sm_array[2].pio.rf(rp_pio::SFR_SM2_INSTR_IMM_INSTR) as u16);
            assert!(expected_instrs[sm_array[3].pio.rf(rp_pio::SFR_SM3_ADDR_PC) as usize] ==
                    sm_array[3].pio.rf(rp_pio::SFR_SM3_INSTR_IMM_INSTR) as u16);

            // execute the "program" that flips the OE bit, which should get us to the next iteration
            report.wfo(utra::main::REPORT_REPORT, 0x1336_0009);
            // exec an instruction that can't complete
            sm_array[0].sm_exec(p_wait.code[p_wait.origin.unwrap_or(0) as usize]);
            // confirm that the stall bit is set
            report.wfo(utra::main::REPORT_REPORT, 0x1336_000A);
            assert!(sm_array[0].pio.rf(rp_pio::SFR_SM0_EXECCTRL_EXEC_STALLED_RO0) == 1);

            // now exec the instruction that should clear the wait condition on an input pin by flipping bit 31 via a side-set operation
            // this requires the testbench to reflect that bit back correctly!
            sm_array[0].sm_exec(set_bit31_oe);
            sm_array[0].sm_exec(clear_bit31_oe);
            delay(4); // give some time for the state machines to run to the stall point (necessary for fast pclk case)
        }
    }
    // stop the machine from running, so we can test RX fifo underflow, etc.
    report.wfo(utra::main::REPORT_REPORT, 0x1336_000C);
    sm_array[0].pio.wfo(rp_pio::SFR_CTRL_EN, 0);

    // since the program ran one extra iteration out the bottom of the loop, we should have overflowed the RX fifo, etc.
    report.wfo(utra::main::REPORT_REPORT,
        sm_array[0].pio.r(rp_pio::SFR_FDEBUG)
    );
    assert!(sm_array[0].pio.r(rp_pio::SFR_FDEBUG) == 0xF); // stalls should be asserted
    sm_array[0].pio.wfo(rp_pio::SFR_FDEBUG_RXSTALL, 0xF); // clear the stall
    assert!(sm_array[0].pio.r(rp_pio::SFR_FDEBUG) == 0); // confirm it is cleared

    // read back the FIFOs and check that the correct values were committed
    report.wfo(utra::main::REPORT_REPORT, 0x1336_000D);
    let loop_ivs = [24u8, 16u8, 8u8, 2u8];
    let mut loop_counters = [0u8; 4];
    loop_counters.copy_from_slice(&loop_ivs);
    for expected_fifo_level in (1..=4).rev() {
        report.wfo(utra::main::REPORT_REPORT, 0x1336_001D | (expected_fifo_level as u32) << 8);
        for (sm_index, sm) in sm_array.iter_mut().enumerate() {
            // RXNEMPTY should be asserted
            assert!((sm.pio.r(rp_pio::SFR_IRQ0_INTS) >> 0) & sm.sm_bitmask() != 0);

            // check that the fifo level is correct
            assert!(sm.sm_rxfifo_level() == expected_fifo_level);
            // check that the index matched
            let rxval = sm.sm_rxfifo_pull_u8_lsb();
            report.wfo(utra::main::REPORT_REPORT, 0x1336_001D | (rxval as u32) << 8);
            assert!(rxval == loop_counters[sm_index]);
        }
        // update the expected indices
        report.wfo(utra::main::REPORT_REPORT, 0x1336_002D | (expected_fifo_level as u32) << 8);
        for (index, loop_counter) in loop_counters.iter_mut().enumerate() {
            if *loop_counter != 0 {
                *loop_counter -= 1;
            } else {
                *loop_counter = loop_ivs[index];
            }
        }
    }
    // check that the RX fifos are empty and no underflow, we should be "just nice"
    report.wfo(utra::main::REPORT_REPORT, 0x1336_000E);
    assert!(sm_array[0].pio.rf(rp_pio::SFR_FDEBUG_RXUNDER) == 0);
    let mut expected_underflows = 0;
    for sm in sm_array.iter_mut() {
        assert!(sm.sm_rxfifo_level() == 0);
        // now do an extra pull
        let _ = sm.sm_rxfifo_pull_u8_lsb();
        assert!(sm.sm_rxfifo_level() == 0);
        expected_underflows |= sm.sm_bitmask();
        assert!(sm.pio.rf(rp_pio::SFR_FDEBUG_RXUNDER) == expected_underflows);
    }
    // clear all the FIFOs and check default states
    // we also clear the RXUNDER bit progressively and confirm that it can be incrementally cleared
    // (this checks the action register implementation is bit-wise and not register-wide)
    report.wfo(utra::main::REPORT_REPORT, 0x1336_000F);
    for sm in sm_array.iter_mut() {
        sm.sm_clear_fifos();
        assert!(sm.sm_rxfifo_level() == 0);
        assert!(sm.sm_txfifo_level() == 0);
        assert!(sm.sm_rxfifo_is_empty());
        assert!(sm.sm_txfifo_is_empty());
        sm.pio.wfo(rp_pio::SFR_FDEBUG_RXUNDER, sm.sm_bitmask());
        expected_underflows &= !(sm.sm_bitmask());
        assert!(sm.pio.rf(rp_pio::SFR_FDEBUG_RXUNDER) == expected_underflows);
    }

    report.wfo(utra::main::REPORT_REPORT, 0x1336_600d);
}