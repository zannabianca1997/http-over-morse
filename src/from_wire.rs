//! Processes for reading from the wire

use core::convert::Infallible;

use letters_to_bytes::LettersToBytes;
use segments_to_signals::SegmentsToSignals;
use signals_to_letters::SignalsToLetters;
pub use wire_to_segments::ListeningPinCallback;
use wire_to_segments::WireToSegments;

use crate::{listening_pin::ListeningPin, process::Process, time::stopwatch::StopwatchOverflow};

pub mod letters_to_bytes;
pub mod segments_to_signals;
pub mod signals_to_letters;
pub mod wire_to_segments;

pub struct BytesFromWire<Strategy: crate::strategies::Strategy>(
    LettersToBytes<SignalsToLetters<SegmentsToSignals<WireToSegments>>, Strategy>,
);

impl<Strategy: crate::strategies::Strategy> BytesFromWire<Strategy> {
    pub fn new(
        listening_pin: ListeningPin<ListeningPinCallback>,
        overflow: StopwatchOverflow,
    ) -> Self {
        Self(LettersToBytes::new(SignalsToLetters::new(
            SegmentsToSignals::new(WireToSegments::new(listening_pin, overflow)),
        )))
    }
}

impl<Strategy> Process for BytesFromWire<Strategy>
where
    Strategy: crate::strategies::Strategy,
{
    type Ok = u8;
    type Err = Infallible;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.0.poll()
    }
}
