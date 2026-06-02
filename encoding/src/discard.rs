//! Discard non encodable bytes
//!
//! Simple test strategy for human input

use core::convert::Infallible;

use super::{Strategy, ToBytes, ToLetters, sealed};

use morse::LetterOrSpace;

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
        mut feed: impl FnMut() -> nb::Result<u8, E>,
    ) -> nb::Result<LetterOrSpace, super::Error<E, Infallible>> {
        loop {
            let mut ch = feed().map_err(|err| err.map(super::Error::Inner))?;

            if ch.is_ascii_whitespace() {
                ch = b' '
            } else {
                ch = ch.to_ascii_uppercase()
            }

            if let Some(ch) = LetterOrSpace::new(ch) {
                break Ok(ch);
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
        mut feed: impl FnMut() -> nb::Result<LetterOrSpace, E>,
    ) -> nb::Result<u8, super::Error<E, Infallible>> {
        feed()
            .map(LetterOrSpace::char)
            .map_err(|err| err.map(super::Error::Inner))
    }
}
