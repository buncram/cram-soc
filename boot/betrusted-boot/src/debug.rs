use utralib::generated::*;
pub struct Uart {
    // pub base: *mut u32,
}

#[allow(dead_code)]
#[cfg(feature="daric")]
pub mod duart {
    pub const UART_DOUT: utralib::Register = utralib::Register::new(0, 0xff);
    pub const UART_DOUT_DOUT: utralib::Field = utralib::Field::new(8, 0, UART_DOUT);
    pub const UART_CTL: utralib::Register = utralib::Register::new(1, 1);
    pub const UART_CTL_EN: utralib::Field = utralib::Field::new(1, 0, UART_CTL);
    pub const UART_BUSY: utralib::Register = utralib::Register::new(2, 1);
    pub const UART_BUSY_BUSY: utralib::Field = utralib::Field::new(1, 0, UART_BUSY);

    pub const HW_DUART_BASE: usize = 0x4004_2000;
}

#[allow(dead_code)]
impl Uart {
    fn put_digit(&mut self, d: u8) {
        let nyb = d & 0xF;
        if nyb < 10 {
            self.putc(nyb + 0x30);
        } else {
            self.putc(nyb + 0x61 - 10);
        }
    }
    pub fn put_hex(&mut self, c: u8) {
        self.put_digit(c >> 4);
        self.put_digit(c & 0xF);
    }
    pub fn newline(&mut self) {
        self.putc(0xa);
        self.putc(0xd);
    }
    pub fn print_hex_word(&mut self, word: u32) {
        for &byte in word.to_be_bytes().iter() {
            self.put_hex(byte);
        }
    }

    #[cfg(not(feature="daric"))]
    pub fn putc(&self, c: u8) {
        self.putc_litex(c);
    }

    pub fn putc_litex(&self, c: u8) {
        let base = utra::uart::HW_UART_BASE as *mut u32;
        let mut uart = CSR::new(base);
        // Wait until TXFULL is `0`
        while uart.r(utra::uart::TXFULL) != 0 {}
        uart.wo(utra::uart::RXTX, c as u32)
    }

    #[cfg(not(feature="daric"))]
    pub fn getc(&self) -> Option<u8> {
        self.getc_litex()
    }

    pub fn getc_litex(&self) -> Option<u8> {
        let base = utra::uart::HW_UART_BASE as *mut u32;
        let mut uart = CSR::new(base);
        match uart.rf(utra::uart::EV_PENDING_RX) {
            0 => None,
            ack => {
                let c = Some(uart.rf(utra::uart::RXTX_RXTX) as u8);
                uart.wfo(utra::uart::EV_PENDING_RX, ack);
                c
            }
        }
    }

    #[cfg(feature="daric")]
    pub fn putc(&self, c: u8) {
        let base = duart::HW_DUART_BASE as *mut u32;
        let mut uart = CSR::new(base);

        if uart.rf(duart::UART_CTL_EN) == 0 {
            uart.wfo(duart::UART_CTL_EN, 1);
        }
        while uart.rf(duart::UART_BUSY_BUSY) != 0 {
            // spin wait
        }
        uart.wfo(duart::UART_DOUT_DOUT, c as u32);

        #[cfg(feature="arty")]
        self.putc_litex(c);
    }

    #[cfg(feature="daric")]
    pub fn getc(&self) -> Option<u8> {
        #[cfg(not(feature="arty"))]
        unimplemented!();
        #[cfg(feature="arty")]
        self.getc_litex()
    }

    pub fn tiny_write_str(&mut self, s: &str) {
        for c in s.bytes() {
            self.putc(c);
        }
    }

}

use core::fmt::{Error, Write};
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.putc(c);
        }
        Ok(())
    }
}

#[macro_use]
#[cfg(all(not(test), feature = "debug-print"))]
pub mod debug_print_hardware {
    #[macro_export]
    macro_rules! print
    {
        ($($args:tt)+) => ({
                use core::fmt::Write;
                let _ = write!(crate::debug::Uart {}, $($args)+);
        });
    }
}

#[macro_use]
#[cfg(all(not(test), not(feature = "debug-print")))]
mod debug_print_hardware {
    #[macro_export]
    #[allow(unused_variables)]
    macro_rules! print {
        ($($args:tt)+) => ({
            ()
        });
    }
}

#[macro_use]
#[cfg(test)]
mod debug_print_hardware {
    #[macro_export]
    #[allow(unused_variables)]
    macro_rules! print {
        ($($args:tt)+) => ({
            std::print!($($args)+)
        });
    }
}

#[macro_export]
macro_rules! println
{
    () => ({
        $crate::print!("\r\n")
    });
    ($fmt:expr) => ({
        $crate::print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!($fmt, "\r\n"), $($args)+)
    });
}
