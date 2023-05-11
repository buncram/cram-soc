pub mod i2c;
pub mod spi;
pub mod units;

pub fn pio_tests() {
    units::restart_imm_test();
    units::fifo_join_test();
    units::sticky_test();
    i2c::i2c_test();
    spi::spi_test();
}
