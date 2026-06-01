//! Perfect hash u8 -> u8 implementation
//!
//! Split in it's own file to be shared with `build.rs`

use core::num::NonZeroU16;

#[derive(Clone, Copy)]
pub struct Hasher {
    pub multiplier: u16,
    pub modulus: NonZeroU16,
    pub size: NonZeroU16,
}

impl Hasher {
    #[inline(always)]
    pub const fn hash(&self, value: u8) -> u16 {
        (value as u16)
            .wrapping_mul(self.multiplier)
            .wrapping_rem(self.modulus.get())
            .wrapping_rem(self.size.get())
    }
}
