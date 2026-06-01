//! Convert signals from a process into segments to put on the wire
//!
//! Other than direct conversion, space is inserted between the symbols

use embedded_hal::digital::PinState;
use process::Process;

use crate::{
    encoding::{
        self, DOT, DOT_LENGTH, LETTER_SPACE, LETTER_SPACE_LENGTH, LINE, LINE_LENGTH,
        SYMBOL_SPACE_LENGTH,
        Signal::{self, Space, Symbol},
        WORD_SPACE, WORD_SPACE_LENGTH,
    },
    segments::Segment,
};

pub struct SignalsToSegments<Inner> {
    inner: Inner,
    state: State,
}

#[derive(PartialEq, Eq)]
enum State {
    Idle,
    LastEmittedWasSymbol,
    SymbolPushBack(encoding::Symbol),
}

impl<Inner> SignalsToSegments<Inner> {
    pub const fn new(inner: Inner) -> Self {
        Self {
            inner,
            state: State::Idle,
        }
    }
}

impl<Inner> Process for SignalsToSegments<Inner>
where
    Inner: Process<Ok = Signal>,
{
    type Ok = Segment;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        let signal = if let State::SymbolPushBack(symbol) = self.state {
            self.state = State::Idle;
            Symbol(symbol)
        } else {
            self.inner.poll()?
        };

        Ok(match signal {
            Symbol(symbol) => {
                if self.state == State::LastEmittedWasSymbol {
                    // need to put a space between those two symbols
                    self.state = State::SymbolPushBack(symbol);
                    Segment::pack(PinState::Low, SYMBOL_SPACE_LENGTH)
                } else {
                    self.state = State::LastEmittedWasSymbol;
                    match symbol {
                        DOT => Segment::pack(PinState::High, DOT_LENGTH),
                        LINE => Segment::pack(PinState::High, LINE_LENGTH),
                    }
                }
            }
            Space(space) => {
                self.state = State::Idle;
                match space {
                    LETTER_SPACE => Segment::pack(PinState::Low, LETTER_SPACE_LENGTH),
                    WORD_SPACE => Segment::pack(PinState::Low, WORD_SPACE_LENGTH),
                }
            }
        })
    }
}
