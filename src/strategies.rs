//! Strategies to convert bytes to morse letters

use crate::encoding::LetterOrSpace;

pub use base32::Base32;
pub use discard::Discard;

mod base32;
mod discard;

mod sealed {
    pub trait Sealed {}
}

/// Strategy
///
/// This trait is not implementable outside as ToLetters and ToBytes need to
/// guarantee roundtrip
pub trait Strategy: sealed::Sealed {
    type ToLetters: ToLetters;
    type ToBytes: ToBytes;
}

pub trait ToLetters {
    fn new() -> Self;
    fn process<E>(
        &mut self,
        feed: impl FnMut() -> nb::Result<u8, E>,
    ) -> nb::Result<LetterOrSpace, E>;
}

pub trait ToBytes {
    fn new() -> Self;
    fn process<E>(
        &mut self,
        feed: impl FnMut() -> nb::Result<LetterOrSpace, E>,
    ) -> nb::Result<u8, E>;
}
