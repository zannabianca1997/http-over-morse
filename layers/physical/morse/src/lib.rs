#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
// Needed to slice the static storage in a const way
#![feature(const_trait_impl)]
#![feature(const_index)]

use core::mem;
use core::num::NonZero;

use time::{MILLIS_FOR_TICK, Time};

mod generated {
    //! Code generated from `morse.txt`
    use super::{perfect_hash::*, symbol::*};

    include! {env!("MORSE_GENERATED_RS")}
}
pub use generated::{
    Letter, LetterOrProsign, LetterOrProsignOrSpace, LetterOrSpace, Prosign, WordSpace,
};

mod symbol;
pub use symbol::{DOT, LINE, Symbol};

mod perfect_hash;

/// Main knob for the encoding speed: number of millis in a single dot
pub const MILLIS_DOT_LENGTH: NonZero<u32> = NonZero::new(8).unwrap();

pub const DOT_LENGTH: NonZero<Time> = const {
    let value = MILLIS_DOT_LENGTH.div_ceil(MILLIS_FOR_TICK).get();
    if value > time::segments::MAX_LENGTH.get() as _ {
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

use crate::generated::DecodeState;

pub const SYMBOL_SPACE_LENGTH: NonZero<Time> = time_multiple(DOT_LENGTH, 1);
pub const LETTER_SPACE_LENGTH: NonZero<Time> = time_multiple(SYMBOL_SPACE_LENGTH, 3);
pub const WORD_SPACE_LENGTH: NonZero<Time> = time_multiple(SYMBOL_SPACE_LENGTH, 7);

const fn time_multiple(unit: NonZero<Time>, count: Time) -> NonZero<Time> {
    let total = unit.checked_mul(NonZero::new(count).unwrap()).unwrap();
    if total.get() > time::segments::MAX_LENGTH.get() {
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
    pub const fn finish(&mut self) -> Result<LetterOrProsign, InvalidLetter> {
        let res = match self.inner.finish() {
            Some(value) => Ok(unsafe {
                // Safety: we know this is okay, the state machine emits only
                // valid morse letters or prosigns, and the enums have the same
                // representation as the bytes
                mem::transmute(value)
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

// = Symbol accessors

impl Letter {
    pub const fn symbols(&self) -> &'static nunny::Slice<Symbol> {
        let state = unsafe {
            // Safety: all letters are valid states
            mem::transmute::<_, DecodeState>(*self)
        };

        unsafe {
            // Safety: all letters have at least a symbol
            nunny::Slice::new_unchecked(state.symbols())
        }
    }
}
impl Prosign {
    pub const fn symbols(&self) -> &'static nunny::Slice<Symbol> {
        let state = unsafe {
            // Safety: all prosigns are valid states
            mem::transmute::<_, DecodeState>(*self)
        };

        unsafe {
            // Safety: all prosigns have at least a symbol
            nunny::Slice::new_unchecked(state.symbols())
        }
    }
}
impl LetterOrProsign {
    pub const fn symbols(&self) -> &'static nunny::Slice<Symbol> {
        let state = unsafe {
            // Safety: all letter and prosigns are valid states
            mem::transmute::<_, DecodeState>(*self)
        };

        unsafe {
            // Safety: all letter and prosigns have at least a symbol
            nunny::Slice::new_unchecked(state.symbols())
        }
    }
}

// = Casts to the lower types

// SAFETY: as the enums variants are all aligned by the codegen, the transmute
// is always safe once the first group has been checked

impl TryFrom<LetterOrProsign> for Letter {
    type Error = Prosign;
    fn try_from(value: LetterOrProsign) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}
impl TryFrom<LetterOrProsign> for Prosign {
    type Error = Letter;
    fn try_from(value: LetterOrProsign) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}
impl TryFrom<LetterOrSpace> for Letter {
    type Error = WordSpace;
    fn try_from(value: LetterOrSpace) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}
impl TryFrom<LetterOrSpace> for WordSpace {
    type Error = Letter;
    fn try_from(value: LetterOrSpace) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}
impl TryFrom<LetterOrProsignOrSpace> for WordSpace {
    type Error = LetterOrProsign;
    fn try_from(value: LetterOrProsignOrSpace) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}
impl TryFrom<LetterOrProsignOrSpace> for LetterOrProsign {
    type Error = WordSpace;
    fn try_from(value: LetterOrProsignOrSpace) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}
impl TryFrom<LetterOrProsignOrSpace> for LetterOrSpace {
    type Error = Prosign;
    fn try_from(value: LetterOrProsignOrSpace) -> Result<Self, Self::Error> {
        Self::new(value as u8).ok_or_else(|| unsafe { mem::transmute(value) })
    }
}

// = Casts to the larger types

// SAFETY: as the enums variants are all aligned by the codegen, the transmute
// is always safe being a subset of the larger type

impl From<Letter> for LetterOrProsign {
    fn from(value: Letter) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<Prosign> for LetterOrProsign {
    fn from(value: Prosign) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<Letter> for LetterOrSpace {
    fn from(value: Letter) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<WordSpace> for LetterOrSpace {
    fn from(value: WordSpace) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<Letter> for LetterOrProsignOrSpace {
    fn from(value: Letter) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<LetterOrProsign> for LetterOrProsignOrSpace {
    fn from(value: LetterOrProsign) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<LetterOrSpace> for LetterOrProsignOrSpace {
    fn from(value: LetterOrSpace) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<Prosign> for LetterOrProsignOrSpace {
    fn from(value: Prosign) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<WordSpace> for LetterOrProsignOrSpace {
    fn from(value: WordSpace) -> Self {
        unsafe { mem::transmute(value) }
    }
}
