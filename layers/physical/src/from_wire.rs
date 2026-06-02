//! Processes for reading from the wire

use core::convert::Infallible;

use either::Either;
use encoding::{Error, ToBytes};
use listening_pin::ListeningPin;
use morse::Prosign;
use process::Process;
use time::StopwatchOverflow;

use letters_to_bytes::LettersToBytes;
use segments_to_signals::SegmentsToSignals;
use signals_to_letters::SignalsToLetters;
pub use wire_to_segments::ListeningPinCallback;
use wire_to_segments::WireToSegments;

pub mod letters_to_bytes;
pub mod segments_to_signals;
pub mod signals_to_letters;
pub mod wire_to_segments;

pub struct BytesFromWire<Strategy: encoding::Strategy>(
    LettersToBytes<SignalsToLetters<SegmentsToSignals<WireToSegments>>, Strategy>,
);

impl<Strategy: encoding::Strategy> BytesFromWire<Strategy> {
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
    Strategy: encoding::Strategy,
{
    type Ok = Either<u8, Prosign>;
    type Err = Error<Infallible, <Strategy::ToBytes as ToBytes>::Error>;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.0.poll()
    }
}
