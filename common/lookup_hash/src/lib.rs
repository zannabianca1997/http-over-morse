#![doc = include_str!("../README.md")]
#![no_std]

use core::mem::MaybeUninit;

/// Shared hash function
const fn hash(multiplier: u16, modulus: u16, size: u8, value: u8) -> u8 {
    let value = (value as u16)
        .wrapping_mul(multiplier)
        .wrapping_rem_euclid(modulus)
        .wrapping_rem(size as u16);

    unsafe {
        core::hint::assert_unchecked(value < size as u16);
    }
    value as u8
}

/// A u8 to u8 lookup table
pub struct LookupTable<T, const MULTIPLIER: u16, const MODULUS: u16, const SIZE: usize>(
    [MaybeUninit<T>; SIZE],
);

impl<T, const MULTIPLIER: u16, const MODULUS: u16, const SIZE: usize>
    LookupTable<T, MULTIPLIER, MODULUS, SIZE>
{
    /// Constructor
    ///
    /// # Safety
    ///
    /// Only safe to call from the generated code in [`search`]
    #[doc(hidden)]
    pub const unsafe fn new(_0: [MaybeUninit<T>; SIZE]) -> Self {
        Self(_0)
    }

    /// Lookup the corresponding value
    ///
    /// # Safety
    ///
    /// The index must be one of the ones accepted by the function used to create the table
    pub const unsafe fn lookup(&self, idx: u8) -> &T {
        let idx = if SIZE <= u8::MAX as usize {
            hash(MULTIPLIER, MODULUS, SIZE as u8, idx) as usize
        } else {
            idx as usize
        };

        let value = &self.0[idx];

        unsafe {
            // Safety: function contract requires this is one of the populated ones.
            value.assume_init_ref()
        }
    }
}

#[cfg(feature = "search")]
pub mod search;
