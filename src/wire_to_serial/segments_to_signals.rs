//! Converts segments to signals
//!
//! Also remove repeated spaces

use core::num::NonZero;

use embedded_hal::digital::PinState;

use crate::{
    encoding::{
        DOT_LENGTH, LETTER_SPACE, LETTER_SPACE_LENGTH, LINE_LENGTH, SYMBOL_SPACE_LENGTH,
        Signal::{self, Space, Symbol},
        Symbol::{DOT, LINE},
        WORD_SPACE, WORD_SPACE_LENGTH,
    },
    process::Process,
    segments::Segment,
    time::Time,
};

const fn threshold(a: NonZero<Time>, b: NonZero<Time>) -> NonZero<Time> {
    let mean = a
        .checked_add(b.get())
        .unwrap()
        .div_ceil(NonZero::new(2).unwrap());

    if mean.get() == a.get() || mean.get() == b.get() {
        panic!("Cannot calculate threshold, range too tight")
    }

    mean
}

const DOT_LINE_THRESHOLD: NonZero<Time> = threshold(DOT_LENGTH, LINE_LENGTH);

const SYMBOL_LETTER_SPACE_THRESHOLD: NonZero<Time> =
    threshold(SYMBOL_SPACE_LENGTH, LETTER_SPACE_LENGTH);
const LETTER_WORD_SPACE_THRESHOLD: NonZero<Time> =
    threshold(LETTER_SPACE_LENGTH, WORD_SPACE_LENGTH);

pub struct SegmentsToSignals<Inner> {
    inner: Inner,
    last_signal_was_space: bool,
}

impl<Inner> SegmentsToSignals<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            last_signal_was_space: true,
        }
    }
}

impl<Inner> Process for SegmentsToSignals<Inner>
where
    Inner: Process<Ok = Segment>,
{
    type Ok = Signal;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        Ok(loop {
            let (state, len) = self.inner.poll()?.unpack();
            break match state {
                PinState::Low => Space({
                    if self.last_signal_was_space || len < SYMBOL_LETTER_SPACE_THRESHOLD {
                        // Ignoring the spaces between symbols, and repeated spaces
                        continue;
                    } else {
                        self.last_signal_was_space = true;
                        if len < LETTER_WORD_SPACE_THRESHOLD {
                            LETTER_SPACE
                        } else {
                            WORD_SPACE
                        }
                    }
                }),
                PinState::High => Symbol({
                    self.last_signal_was_space = false;
                    if len < DOT_LINE_THRESHOLD { DOT } else { LINE }
                }),
            };
        })
    }
}
