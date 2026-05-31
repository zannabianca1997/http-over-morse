//! Read segments from the wire

use core::{cell::Cell, convert::Infallible, num::NonZero};

use avr_device::interrupt::{CriticalSection, Mutex, free};
use embedded_hal::digital::PinState;

use crate::{
    encoding::{SYMBOL_SPACE_LENGTH, WORD_SPACE_LENGTH, threshold},
    listening_pin::{Edge, ListeningPin},
    process::Process,
    segments::Segment,
    time::{
        Time,
        stopwatch::{StopwatchOverflow, StopwatchReset},
    },
    utils::ringbuf::Buffer,
};

pub struct ListeningPinCallback {
    stopwatch: StopwatchReset,
}

impl ListeningPinCallback {
    pub fn new(stopwatch: StopwatchReset) -> Self {
        if free(|cs| BUFFER_WRITE_RIGHTS_TAKEN.borrow(cs).replace(true)) {
            panic!("Cannot create two callbacks")
        }

        Self { stopwatch }
    }
}

impl crate::listening_pin::ListeningPinCallback for ListeningPinCallback {
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

static BUFFER_WRITE_RIGHTS_TAKEN: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

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
