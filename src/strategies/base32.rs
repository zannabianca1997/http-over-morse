//! Encode all bytes using base32
//!
//! Alphabet is: A-Z,0

use super::{Strategy, ToBytes, ToLetters, sealed};

use crate::encoding::{Letter, LetterOrSpace};

pub enum Base32 {}
impl sealed::Sealed for Base32 {}
impl Strategy for Base32 {
    type ToLetters = Encode;
    type ToBytes = Decode;
}

pub struct Decode {
    len: u8,
    bits: u16,
}

impl Decode {
    /// Decode a letter into the last five bits of a u8
    const fn letter_to_u5(letter: Letter) -> Option<u8> {
        Some(match letter.char() {
            ch @ b'A'..=b'Z' => ch - b'A',
            dg @ b'0'..=b'5' => dg - (b'0' + b'A' - b'Z' - 1),
            _ => return None,
        })
    }
}

impl ToBytes for Decode {
    fn new() -> Self {
        Self {
            len: 0,
            bits: 0b0000_0000_0000_0000,
        }
    }

    fn process<E>(
        &mut self,
        mut feed: impl FnMut() -> nb::Result<LetterOrSpace, E>,
    ) -> nb::Result<u8, E> {
        while self.len < 8 {
            // Pull in a new letter

            let letter_or_space = feed()?;

            if let Some(letter) = letter_or_space.letter() {
                let Some(u5) = Self::letter_to_u5(letter) else {
                    // TODO: handle errors what to do when we receive a letter
                    // that's not a b32 letter?

                    // Skipping for now
                    continue;
                };

                self.bits |= (u5 as u16) << self.len;
                self.len += 5;
            } else {
                // Space: flushing. len should be zero if the connection was good
                if self.len == 0 {
                    // Good. trying to fetch a new letter to start the new stream
                    continue;
                } else {
                    // Ouch. Additional bits...
                    // TODO: handle errors. What to do when we received additional bytes?
                    // Ignoring them for now
                    self.bits = 0;
                    self.len = 0;
                }
            }
        }

        let ch = (self.bits & 0x00_ff) as u8;
        self.bits >>= 8;
        self.len -= 8;

        Ok(ch)
    }
}

pub struct Encode {
    len: u8,
    bits: u16,
    flushing: bool,
}

impl Encode {
    /// Encode the last five bits to a letter
    const fn u5_to_letter(u5: u8) -> Letter {
        let u5 = u5 & 0b0001_1111;
        let letter = if u5 <= b'Z' - b'A' {
            Letter::new(u5 + b'A')
        } else {
            Letter::new(u5 + (b'0' + b'A' - b'Z' - 1))
        };
        unsafe {
            // Safety: Between the mask and the `if` all the u8 must end up to
            // valid letters. This is also checked by the const evaluation
            // under.
            letter.unwrap_unchecked()
        }
    }
}

impl ToLetters for Encode {
    fn new() -> Self {
        Self {
            len: 0,
            bits: 0b0000_0000_0000_0000,
            flushing: false,
        }
    }

    fn process<E>(
        &mut self,
        mut feed: impl FnMut() -> nb::Result<u8, E>,
    ) -> nb::Result<LetterOrSpace, E> {
        if self.flushing {
            self.flushing = false;
            return Ok(LetterOrSpace::SPACE);
        }

        if self.len < 5 {
            // Pull in a new character

            match feed() {
                Ok(ch) => {
                    // Push the new byte
                    self.bits |= (ch as u16) << self.len;
                    self.len += 8;
                }
                Err(nb::Error::WouldBlock) if self.len != 0 => {
                    // Take the occasion to flush. Pad with 0 and add a space
                    self.flushing = true;
                    self.len = 5;
                }
                Err(err) => return Err(err),
            };
        }

        let u5 = (self.bits & 0b0000_0000_0001_1111) as u8;
        self.bits >>= 5;
        self.len -= 5;

        Ok(Self::u5_to_letter(u5).into())
    }
}

/// Did yet not set no test on the AVR.
///
/// This is a const evaluation that will panic if `letter_to_u5` does not
/// represent a one to one mapping from letter to 0..32, and if `u5_to_letter`
/// is not the inverse function.
const _: () = const {
    let mut mapped = [false; 32];

    // Check all the u8
    let mut next = Some(u8::MIN);
    while let Some(ch) = {
        let ch = next;
        if let Some(n) = next {
            next = n.checked_add(1);
        }
        ch
    } {
        // Only the letters
        let Some(letter) = Letter::new(ch) else {
            continue;
        };

        // And the one that converts to bytes
        let Some(byte) = Decode::letter_to_u5(letter) else {
            continue;
        };

        let recovered = Encode::u5_to_letter(byte);

        assert!(
            letter.char() == recovered.char(),
            "A letter did not roundtrip"
        );

        assert!(
            byte & 0b0001_1111 == byte,
            "A letter mapped over the five byte range"
        );

        assert!(!mapped[byte as usize], "A byte was mapped twice");

        mapped[byte as usize] = true;
    }

    let mut i = 0;
    while i < mapped.len() {
        assert!(
            mapped[i],
            "There is a unmapped space in the five byte range"
        );
        i += 1;
    }
};
