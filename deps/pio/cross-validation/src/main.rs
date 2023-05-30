//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use xous_pio;
use xous_pio::pio_tests::*;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let mut led_pin = pins.led.into_push_pull_output();

    unsafe {
        info!("ID readback: 0x{:x}", (0x5020_0044 as *mut u32).read_volatile());
    }

    // Reset the PIO block. This is necessary for the PIO block to work.
    pac.RESETS.reset.modify(|_, w| w.pio0().set_bit());
    pac.RESETS.reset.modify(|_, w| w.pio0().clear_bit());
    while pac.RESETS.reset_done.read().pio0().bit_is_clear() {}

    info!("ID: 0x{:x}", xous_pio::get_id());

    info!("FIFO test");
    units::fifo_join_test();
    info!("adder test");
    adder::adder_test();
    info!("restart immediate test");
    units::restart_imm_test();

    info!("setting up GPIOs for feedback");
    const GPIO_BASE: usize = 0x4001_4000;
    for i in 2..30 { // 0 & 1 are for serial output, 30 and 31 don't exist
        if i != 25 { // 25 is for the LED
            unsafe{((GPIO_BASE + 4 + i * 8) as *mut u32).write_volatile(6);}
        }
    }

    info!("doing corner cases");
    units::corner_cases();
    info!("doing instruction tests");
    units::instruction_tests();

    // note: an external pulldown on GPIO28 makes this test much more reliable
    info!("doing register tests");
    units::register_tests();

    info!("doing sticky tests");
    units::sticky_test();

    loop {
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        info!("off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}

