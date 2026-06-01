//! Write segments to the wire

use core::{convert::Infallible, num::NonZero};

use avr_device::interrupt::CriticalSection;
use embedded_hal::digital::OutputPin;
use ringbuf::Buffer;

use crate::{
    process::{NoItemProduced, Process},
    segments::Segment,
    time::{self, Time, Timer},
};

pub struct TimerCallback<Tx> {
    pin: Tx,
}

impl<Tx> TimerCallback<Tx> {
    /// Create a new callback
    ///
    /// Safety: the caller must ensure only one callback exists, as it has write
    /// rights on the global buffer.
    pub unsafe fn new(pin: Tx) -> Self {
        Self { pin }
    }
}

static BUFFER: Buffer<Segment, 16> = Buffer::new();

impl<Tx> time::TimerCallback for TimerCallback<Tx>
where
    Tx: OutputPin<Error = Infallible>,
{
    fn triggered(&mut self, cs: CriticalSection) -> Option<NonZero<Time>> {
        let mut buffer = unsafe {
            // Safety: Only place we take the reader, there is only one
            // TimerCallback, and this function is not reentrant.
            BUFFER.reader_cs(cs)
        };

        let Some(segment) = buffer.pop() else {
            // No more segments to read, putting the wire at rest
            self.pin.set_low();
            return None;
        };

        let (state, time) = segment.unpack();

        self.pin.set_state(state);
        Some(time)
    }
}

pub struct SegmentsToWire<Inner, Tx> {
    inner: Inner,
    timer: Timer<TimerCallback<Tx>>,
}

impl<Inner, Tx> SegmentsToWire<Inner, Tx> {
    pub const fn new(timer: Timer<TimerCallback<Tx>>, inner: Inner) -> Self {
        Self { inner, timer }
    }
}

impl<Inner, Tx> Process for SegmentsToWire<Inner, Tx>
where
    Inner: Process<Ok = Segment>,
    Tx: OutputPin<Error = Infallible>,
{
    type Ok = NoItemProduced;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        let mut buffer = unsafe {
            // Safety: Only place we take the writer, there is only one
            // TimerCallback (that we possess), and this function is not
            // reentrant.
            BUFFER.writer()
        };

        // Read all segments until either the inner process blocks, or the
        // buffer is full
        loop {
            // TODO: using a more general region api from fring would enable us
            // to check for the buffer fullness only once.
            let Some(mut slot) = buffer.slot() else {
                // Need to wait for the callback to empty it.
                return Err(nb::Error::WouldBlock);
            };

            let segment = self.inner.poll()?;

            slot.push(segment);

            // Trigger the timer now
            self.timer.trigger_if_not_set();
        }
    }
}
