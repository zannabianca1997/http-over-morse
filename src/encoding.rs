//! General encoding stuff

use core::num::NonZeroU8;

use crate::time::MILLIS_FOR_TICK;
use crate::time::Time;

mod symbol;
pub use symbol::{DOT, LINE, Symbol};

mod perfect_hash;

pub const DOT_LENGTH: Time = (333 / MILLIS_FOR_TICK) as Time;
pub const LINE_LENGTH: Time = 3 * DOT_LENGTH;

/// Space on the wire
///
/// The `SYMBOL` variant is missing as it is implicit when two symbols are
/// consecutive
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Space {
    /// Space between letters
    LETTER,
    /// Space between words
    WORD,
}
pub use Space::LETTER as LETTER_SPACE;
pub use Space::WORD as WORD_SPACE;

pub const SYMBOL_SPACE_LENGTH: Time = 1 * DOT_LENGTH;
pub const LETTER_SPACE_LENGTH: Time = 3 * SYMBOL_SPACE_LENGTH;
pub const WORD_SPACE_LENGTH: Time = 7 * SYMBOL_SPACE_LENGTH;

/// Any significative signal on the wire
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Symbol(Symbol),
    Space(Space),
}

/// A valid morse letter
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Letter(NonZeroU8);

mod generated {
    //! Code generated from `morse.txt`
    use super::{perfect_hash::*, symbol::*};

    include! {env!("MORSE_GENERATED_RS")}
}

impl Letter {
    pub const fn new(ch: u8) -> Option<Self> {
        if generated::is_morse_letter(ch) {
            Some(Self(
                NonZeroU8::new(ch).expect("`\\0` is not a valid morse letter"),
            ))
        } else {
            None
        }
    }

    pub const fn char(self) -> u8 {
        self.0.get()
    }

    pub const fn symbols(self) -> &'static nunny::Slice<Symbol> {
        generated::encode(self.char())
    }
}

/// A valid morse letter, or a word space
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LetterOrSpace(NonZeroU8);

impl LetterOrSpace {
    pub const SPACE: LetterOrSpace = LetterOrSpace(NonZeroU8::new(b' ').unwrap());

    pub const fn new(ch: u8) -> Option<Self> {
        if ch == b' ' {
            Some(Self::SPACE)
        } else if let Some(letter) = Letter::new(ch) {
            Some(Self(letter.0))
        } else {
            None
        }
    }

    pub const fn letter(self) -> Option<Letter> {
        if self.0.get() == b' ' {
            None
        } else {
            Some(Letter(self.0))
        }
    }

    pub const fn char(self) -> u8 {
        self.0.get()
    }
}

impl From<Letter> for LetterOrSpace {
    fn from(value: Letter) -> Self {
        LetterOrSpace(value.0)
    }
}

/// Decoder state machine
#[derive(Clone, Copy)]
pub struct Decoder {
    inner: generated::DecodeState,
}

impl Decoder {
    /// Create a new state machine with no symbols inside
    pub const fn new() -> Self {
        Self {
            inner: generated::DecodeState::new(),
        }
    }

    /// If the state machine contains any symbol
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Drop all symbols read
    pub const fn reset(&mut self) {
        self.inner = generated::DecodeState::new();
    }

    /// Add another symbol
    ///
    /// If there are no letter starting with that symbols, leave the machine
    /// untouched and return an error
    pub const fn digest(&mut self, symbol: Symbol) -> Result<(), InvalidContinuation> {
        if let Some(next) = self.inner.digest(symbol) {
            self.inner = next;
            Ok(())
        } else {
            Err(InvalidContinuation {
                current: self.inner,
                symbol,
            })
        }
    }

    /// Finish and get the decoded letter
    ///
    /// This will reset the machine, even in the case of an error
    pub const fn finish(&mut self) -> Result<Letter, InvalidLetter> {
        let res = match self.inner.finish() {
            Some(value) => Ok({
                // we know this is okay, the state machine emits only valid morse letters
                Letter(NonZeroU8::new(value).unwrap())
            }),
            None => Err(InvalidLetter { state: self.inner }),
        };
        self.reset();
        res
    }
}

/// An invalid continuation has been submitted to the state machine
#[derive(Clone, Copy)]
pub struct InvalidContinuation {
    /// State at the failure
    current: generated::DecodeState,
    /// The invalid symbol
    pub symbol: Symbol,
}

impl InvalidContinuation {
    /// All symbols emitted before
    pub const fn current(&self) -> &'static [Symbol] {
        self.current.symbols()
    }
}

/// The state machine symbol sequence do not represent any letter
#[derive(Clone, Copy)]
pub struct InvalidLetter {
    /// State at the failure
    state: generated::DecodeState,
}

impl InvalidLetter {
    /// All symbols emitted
    pub const fn symbols(&self) -> &'static [Symbol] {
        self.state.symbols()
    }
}
