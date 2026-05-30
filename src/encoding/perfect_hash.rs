//! Perfect hash u8 -> u8 implementation

use core::num::NonZeroU16;

#[derive(Clone, Copy)]
pub struct Hasher {
    pub multiplier: u16,
    pub modulus: NonZeroU16,
}

impl Hasher {
    pub const fn hash(&self, value: u8) -> u8 {
        (value as u16)
            .wrapping_mul(self.multiplier)
            .wrapping_rem(self.modulus.get()) as u8
    }
}
