use bitflags::*;
use volatile::Volatile;

bitflags! {
    pub struct Sha512_256Config: u32 {
        const NONE        = 0b0000_0000;
        const Sha512_25656_EN   = 0b0000_0001;
        const ENDIAN_SWAP = 0b0000_0010;
        const DIGEST_SWAP = 0b0000_0100;
        const HMAC_EN     = 0b0000_1000;
    }
}

bitflags! {
    pub struct Sha512_256Command: u32 {
        const HASH_START  = 0b0000_0001;
        const HASH_DIGEST = 0b0000_0010;
    }
}

bitflags! {
    pub struct Sha512_256Status: u32 {
        const DONE = 0b0000_0001;
    }
}

bitflags! {
    pub struct Sha512_256Fifo: u32 {
        const READ_COUNT_MASK  = 0b0000_0000_0000_0011_1111;
        const WRITE_COUNT_MASK = 0b0000_1111_1111_1100_0000;
        const READ_ERROR       = 0b0001_0000_0000_0000_0000;
        const WRITE_ERROR      = 0b0010_0000_0000_0000_0000;
        const ALMOST_FULL      = 0b0100_0000_0000_0000_0000;
        const ALMOST_EMPTY     = 0b1000_0000_0000_0000_0000;
    }
}

bitflags! {
    pub struct Sha512_256Event: u32 {
        const ERROR       = 0b0001;
        const FIFO_FULL   = 0b0010;
        const HMAC_DONE   = 0b0100;
        const Sha512_25656_DONE = 0b1000;
    }
}

pub struct BtSha512_256 {
    p: betrusted_pac::Peripherals,
    pub config: Sha512_256Config,
    pub keys: [u32; 8],
}

impl BtSha512_256 {
    pub fn init(&mut self) -> bool {
        unsafe{ self.p.Sha512_256.config.write(|w|{ w.bits(self.config.bits()) }); }
        for reg in 0..8 {
            match reg {
                0 => unsafe{ self.p.Sha512_256.key0.write(|w|{ w.bits(self.keys[0]) }) },
                1 => unsafe{ self.p.Sha512_256.key1.write(|w|{ w.bits(self.keys[1]) }) },
                2 => unsafe{ self.p.Sha512_256.key2.write(|w|{ w.bits(self.keys[2]) }) },
                3 => unsafe{ self.p.Sha512_256.key3.write(|w|{ w.bits(self.keys[3]) }) },
                4 => unsafe{ self.p.Sha512_256.key4.write(|w|{ w.bits(self.keys[4]) }) },
                5 => unsafe{ self.p.Sha512_256.key5.write(|w|{ w.bits(self.keys[5]) }) },
                6 => unsafe{ self.p.Sha512_256.key6.write(|w|{ w.bits(self.keys[6]) }) },
                7 => unsafe{ self.p.Sha512_256.key7.write(|w|{ w.bits(self.keys[7]) }) },
                _ => assert!(false),
            }
        }
        self.p.Sha512_256.command.write(|w|{ w.hash_start().set_bit() });
        true
    }

    pub fn digest(&mut self, digest: &mut [u32; 8]) {
        self.p.Sha512_256.command.write(|w|{ w.hash_process().set_bit()});
        while (self.p.Sha512_256.ev_pending.read().bits() & (Sha512_256Event::Sha512_25656_DONE | Sha512_256Event::HMAC_DONE).bits()) == 0 {}
        unsafe{ self.p.Sha512_256.ev_pending.write(|w| w.bits((Sha512_256Event::Sha512_25656_DONE | Sha512_256Event::HMAC_DONE).bits()) ); }

        for reg in 0..8 {
            match reg {
                0 => digest[reg] = self.p.Sha512_256.digest0.read().bits(),
                1 => digest[reg] = self.p.Sha512_256.digest1.read().bits(),
                2 => digest[reg] = self.p.Sha512_256.digest2.read().bits(),
                3 => digest[reg] = self.p.Sha512_256.digest3.read().bits(),
                4 => digest[reg] = self.p.Sha512_256.digest4.read().bits(),
                5 => digest[reg] = self.p.Sha512_256.digest5.read().bits(),
                6 => digest[reg] = self.p.Sha512_256.digest6.read().bits(),
                7 => digest[reg] = self.p.Sha512_256.digest7.read().bits(),
                _ => assert!(false),
            }
        }
    }
}

use digest::{BlockInput, FixedOutputDirty, Reset, Update};
use digest::generic_array::GenericArray;

impl Digest for BtSha512_256 {
    fn new() -> Self {
        unsafe {
            BtSha512_256 {
                p: betrusted_pac::Peripherals::steal(),
                config: Sha512_256Config::NONE,
                keys: [0; 8],
            }
        }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        let sha_ptr: *mut u32 = 0xe0001000 as *mut u32;
        let sha = sha_ptr as *mut Volatile<u32>;
        let sha_byte_ptr: *mut u8 = 0xe0001000 as *mut u8;
        let sha_byte = sha_byte_ptr as *mut Volatile<u8>;

        for (_reg, chunk) in data.chunks(4).enumerate() {
            let mut temp: [u8; 4] = Default::default();
            if chunk.len() == 4 {
                temp.copy_from_slice(chunk);
                let dword: u32 = u32::from_le_bytes(temp);

                while self.p.Sha512_256.fifo.read().almost_full().bit() {}
                unsafe { (*sha).write(dword); }
            } else {
                for index in 0..chunk.len() {
                    while self.p.Sha512_256.fifo.read().almost_full().bit() {}
                    unsafe{ (*sha_byte).write(chunk[index]); }
                }
            }
        }
    }

    pub fn finalize(&mut self, digest: &mut [u32; 8]) {
        self.p.Sha512_256.command.write(|w|{ w.hash_process().set_bit()});
        while (self.p.Sha512_256.ev_pending.read().bits() & (Sha512_256Event::Sha512_25656_DONE | Sha512_256Event::HMAC_DONE).bits()) == 0 {}
        unsafe{ self.p.Sha512_256.ev_pending.write(|w| w.bits((Sha512_256Event::Sha512_25656_DONE | Sha512_256Event::HMAC_DONE).bits()) ); }

        for reg in 0..8 {
            match reg {
                0 => digest[reg] = self.p.Sha512_256.digest0.read().bits(),
                1 => digest[reg] = self.p.Sha512_256.digest1.read().bits(),
                2 => digest[reg] = self.p.Sha512_256.digest2.read().bits(),
                3 => digest[reg] = self.p.Sha512_256.digest3.read().bits(),
                4 => digest[reg] = self.p.Sha512_256.digest4.read().bits(),
                5 => digest[reg] = self.p.Sha512_256.digest5.read().bits(),
                6 => digest[reg] = self.p.Sha512_256.digest6.read().bits(),
                7 => digest[reg] = self.p.Sha512_256.digest7.read().bits(),
                _ => assert!(false),
            }
        }
    }

}