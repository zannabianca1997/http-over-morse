//! Discard non encodable bytes
//!
//! Simple test strategy for human input

use core::convert::Infallible;

use super::{Strategy, ToBytes, ToLetters, sealed};

use either::{Either, Left, Right};
use morse::{LetterOrProsignOrSpace, LetterOrSpace, Prosign};

/// Discard all bytes that are not encodable
///
/// To help encoding more, change all ascii whitespaces in spaces and all
/// letters to uppercase
pub struct Discard;

impl sealed::Sealed for Discard {}
impl Strategy for Discard {
    type ToLetters = Discard;
    type ToBytes = Discard;
}

impl ToLetters for Discard {
    type Error = Infallible;
    fn new() -> Self {
        Self
    }
    fn process<E>(
        &mut self,
        mut feed: impl FnMut() -> nb::Result<Either<u8, Prosign>, E>,
    ) -> nb::Result<LetterOrProsignOrSpace, super::Error<E, Infallible>> {
        loop {
            match feed().map_err(|err| err.map(super::Error::Inner))? {
                Left(mut ch) => {
                    if ch.is_ascii_whitespace() {
                        ch = b' '
                    } else {
                        ch = ch.to_ascii_uppercase()
                    }

                    if let Some(ch) = LetterOrSpace::new(ch) {
                        break Ok(ch.into());
                    }
                }
                Right(prosign) => {
                    // Prosign are untouched
                    break Ok(prosign.into());
                }
            }
        }
    }
}

impl ToBytes for Discard {
    type Error = Infallible;
    fn new() -> Self {
        Self
    }
    fn process<E>(
        &mut self,
        mut feed: impl FnMut() -> nb::Result<LetterOrProsignOrSpace, E>,
    ) -> nb::Result<Either<u8, Prosign>, super::Error<E, Infallible>> {
        feed()
            .map(|l| Either::from(LetterOrSpace::try_from(l).map(|l| l as u8)).flip())
            .map_err(|err| err.map(super::Error::Inner))
    }
}
