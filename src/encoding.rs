//! General encoding stuff

use core::mem;
use core::num::NonZero;
use core::num::NonZeroU8;

use crate::encoding::generated::DecodeState;
use crate::time::MILLIS_FOR_TICK;
use crate::time::Time;

mod symbol;
pub use symbol::{DOT, LINE, Symbol};

mod perfect_hash;

/// Main knob for the encoding speed: number of millis in a single dot
pub const MILLIS_DOT_LENGTH: NonZero<u32> = NonZero::new(64).unwrap();

pub const DOT_LENGTH: NonZero<Time> = const {
    let value = MILLIS_DOT_LENGTH.div_ceil(MILLIS_FOR_TICK).get();
    if value > crate::segments::MAX_LENGTH.get() as _ {
        panic!("Exceeded segment max length!")
    }
    NonZero::new(value as Time).unwrap()
};
pub const LINE_LENGTH: NonZero<Time> = time_multiple(DOT_LENGTH, 3);

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

pub const SYMBOL_SPACE_LENGTH: NonZero<Time> = time_multiple(DOT_LENGTH, 1);
pub const LETTER_SPACE_LENGTH: NonZero<Time> = time_multiple(SYMBOL_SPACE_LENGTH, 3);
pub const WORD_SPACE_LENGTH: NonZero<Time> = time_multiple(SYMBOL_SPACE_LENGTH, 7);

const fn time_multiple(unit: NonZero<Time>, count: Time) -> NonZero<Time> {
    let total = unit.checked_mul(NonZero::new(count).unwrap()).unwrap();
    if total.get() > crate::segments::MAX_LENGTH.get() {
        panic!("Exceeded segment max length!")
    }
    total
}

pub const fn threshold(a: NonZero<Time>, b: NonZero<Time>) -> NonZero<Time> {
    let mean = a
        .checked_add(b.get())
        .unwrap()
        .div_ceil(NonZero::new(2).unwrap());

    if mean.get() == a.get() || mean.get() == b.get() {
        panic!("Cannot calculate threshold, range too tight")
    }

    mean
}

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
        let state: DecodeState = unsafe {
            // Safety: DecodeState has `#[repr(u8)]` on it, and all variants
            // that corresponds to complete chars have their variant set to the
            // right value
            mem::transmute(self.char())
        };
        unsafe {
            // Safety: all letters have a non-empty list of symbols
            nunny::Slice::new_unchecked(state.symbols())
        }
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
