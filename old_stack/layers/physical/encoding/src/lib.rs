#![doc = include_str!("../README.md")]
#![no_std]

use either::Either;
use morse::{LetterOrProsignOrSpace, Prosign};

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
    type Error;
    fn new() -> Self;
    fn process<E>(
        &mut self,
        feed: impl FnMut() -> nb::Result<Either<u8, Prosign>, E>,
    ) -> nb::Result<LetterOrProsignOrSpace, Error<E, Self::Error>>;
}

pub trait ToBytes {
    type Error;
    fn new() -> Self;
    fn process<E>(
        &mut self,
        feed: impl FnMut() -> nb::Result<LetterOrProsignOrSpace, E>,
    ) -> nb::Result<Either<u8, Prosign>, Error<E, Self::Error>>;
}

pub enum Error<Inner, Strategy> {
    Inner(Inner),
    Strategy(Strategy),
}
