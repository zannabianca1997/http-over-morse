//! Read segments from the wire

use core::{convert::Infallible, num::NonZero};

use avr_device::interrupt::CriticalSection;
use embedded_hal::digital::PinState;

use morse::WORD_SPACE_LENGTH;
use process::Process;
use ringbuf::Buffer;
use time::{StopwatchOverflow, StopwatchReset, Time, segments::Segment};

use listening_pin::{Edge, ListeningPin};

/// Callback to the listening pin
///
/// Only one of these can exist at a time, guaranteed by the fact that only one
/// stopwatch exists
pub struct ListeningPinCallback {
    stopwatch: StopwatchReset,
}

impl ListeningPinCallback {
    /// Create a new callback
    pub fn new(stopwatch: StopwatchReset) -> Self {
        Self { stopwatch }
    }
}

impl listening_pin::ListeningPinCallback for ListeningPinCallback {
    fn triggered(&mut self, cs: CriticalSection, edge: Edge) {
        let mut buffer = unsafe {
            // Safety: Only place we take the writer, there is only one
            // ListeningPinCallback, and this function is not reentrant.
            BUFFER.writer_cs(cs)
        };

        // Length of this segment
        let len = self.stopwatch.reset(cs);
        let Some(len) = NonZero::new(len) else {
            // Too near the last trigger
            return;
        };

        // State the pin was during this segment
        let state = match edge {
            Edge::Falling => PinState::High,
            Edge::Rising => PinState::Low,
        };

        buffer.push(Segment::pack(state, len));
    }
}

static BUFFER: Buffer<Segment, 16> = Buffer::new();

pub struct WireToSegments {
    pin: ListeningPin<ListeningPinCallback>,
    stopwatch: StopwatchOverflow,
}

impl WireToSegments {
    pub fn new(pin: ListeningPin<ListeningPinCallback>, stopwatch: StopwatchOverflow) -> Self {
        Self { pin, stopwatch }
    }
}

/// Where a space is considered long enough to be sent anyway
const SPACE_OVERFLOW_THRESHOLD: NonZero<Time> = WORD_SPACE_LENGTH;

impl Process for WireToSegments {
    type Ok = Segment;
    type Err = Infallible;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        let mut buffer = unsafe {
            // Safety: Only place we take the reader, there is only one
            // ListeningPinCallback (that we possess), and this function is not
            // reentrant.
            BUFFER.reader()
        };

        if let Some(segment) = buffer.pop() {
            return Ok(segment);
        }

        // Send spaces to complete letters if the wire is down
        if let Some(time) = self
            .stopwatch
            .reset_if_greater(SPACE_OVERFLOW_THRESHOLD.get())
        {
            let segment = Segment::pack(self.pin.state(), NonZero::new(time).unwrap());

            return Ok(segment);
        }

        // There are no segment to read, need to wait for the pin or overflow
        Err(nb::Error::WouldBlock)
    }
}
