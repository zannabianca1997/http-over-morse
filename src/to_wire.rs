//! Processes for writing to wire

use core::convert::Infallible;

use bytes_to_letters::BytesToLetters;
use embedded_hal::digital::OutputPin;
use letters_to_signals::LettersToSignals;
use segments_to_wire::SegmentsToWire;
pub use segments_to_wire::TimerCallback;
use signals_to_segments::SignalsToSegments;

use crate::{
    process::{NoItemProduced, Process},
    time::Timer,
};

pub mod bytes_to_letters;
pub mod letters_to_signals;
pub mod segments_to_wire;
pub mod signals_to_segments;

pub struct BytesToWire<Inner, Strategy: crate::strategies::Strategy, Tx>(
    SegmentsToWire<SignalsToSegments<LettersToSignals<BytesToLetters<Inner, Strategy>>>, Tx>,
);

impl<Inner, Strategy: crate::strategies::Strategy, Tx> BytesToWire<Inner, Strategy, Tx> {
    pub fn new(inner: Inner, timer: Timer<TimerCallback<Tx>>) -> Self {
        Self(SegmentsToWire::new(
            timer,
            SignalsToSegments::new(LettersToSignals::new(BytesToLetters::new(inner))),
        ))
    }
}

impl<Inner, Strategy, Tx> Process for BytesToWire<Inner, Strategy, Tx>
where
    Strategy: crate::strategies::Strategy,
    Inner: Process<Ok = u8>,
    Tx: OutputPin<Error = Infallible>,
{
    type Ok = NoItemProduced;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.0.poll()
    }
}
