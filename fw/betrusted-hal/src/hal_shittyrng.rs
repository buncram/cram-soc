#![allow(dead_code)]
use rand_core::{RngCore, Error, impls};
use super::hal_xadc::*;

/// This is a "beginner's" RNG just to get API integration working with Rust.
/// Do not use this for anything serious, unless you want to be sad.

pub struct ShittyRng {
    p: betrusted_pac::Peripherals,
    xadc: BtXadc,
    bucket: u64,
    count: u64,  // count of bits of randomness generated since created
}

impl ShittyRng {
    pub fn new() -> Self {
        unsafe { ShittyRng { p: betrusted_pac::Peripherals::steal(), xadc: BtXadc::new(), bucket: 0, count: 0 }}
    }
}

impl ShittyRng {
    pub fn get_bits_generated(&mut self) -> u64 {
        self.count
    }
}

// allow this to be use for testing the crypto functions...
impl rand_core::CryptoRng for ShittyRng {}

impl RngCore for ShittyRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        // make sure the noise bias is on for the avalanche TRNG
        unsafe{ self.p.POWER.power.write(|w| w.noisebias().bit(true).noise().bits(3).self_().bit(true).state().bits(3) ); }
        // TODO: need to add some mechanism to confirm when the TRNG has powered on

        // start loading the ring osc trng
        self.p.TRNG_OSC.ctl.write(|w|{ w.ena().bit(true)});

        self.xadc.noise_only(true); // cut out other round-robin sensor readings
        for _ in 0..8 {
            self.xadc.wait_update();
            self.bucket <<= 8;
            self.bucket ^= (self.xadc.noise0() ^ self.xadc.noise1()) as u64;
        }
        self.xadc.noise_only(false); // bring them back

        for i in 0..1 {
            while self.p.TRNG_OSC.status.read().fresh().bit_is_clear() {}
            if i == 0 {
                self.bucket ^= self.p.TRNG_OSC.rand.read().rand().bits() as u64;
            } else {
                self.bucket ^= (self.p.TRNG_OSC.rand.read().rand().bits() as u64) << 32;
            }
        }
        self.p.TRNG_OSC.ctl.write(|w|{ w.ena().bit(false)});

        self.count += 64;

        self.bucket
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}