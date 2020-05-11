#![no_std]

extern crate alloc;
extern crate bitflags;
extern crate volatile;

pub mod hal_i2c;
pub mod hal_time;
pub mod hal_lcd;
pub mod hal_com;
pub mod hal_kbd;
pub mod hal_uart;
pub mod hal_xadc;
pub mod hal_audio;
pub mod hal_rtc;
pub mod hal_aes;
pub mod hal_sha2;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
