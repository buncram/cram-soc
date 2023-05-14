pub mod i2c;
pub mod spi;
pub mod units;
pub mod adder;
pub mod nec;

pub fn pio_tests() {
    units::register_tests();
    units::restart_imm_test();
    units::fifo_join_test();
    units::sticky_test();
    adder::adder_test();
    nec::nec_ir_loopback_test();
    i2c::i2c_test();
    spi::spi_test();
}
