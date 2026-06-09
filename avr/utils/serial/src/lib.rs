#![doc = include_str!("../README.md")]
#![no_std]

use core::convert::Infallible;

use concurrency::{
    ready::Ready,
    sink::Sink,
    slots::{Slot, Slots},
    stream::Stream,
};
use hal::serial::{SerialRead, SerialReader, SerialWrite, SerialWriter};

/// Read bytes from the serial
pub struct ReadSerial {
    serial: SerialReader,
}

impl ReadSerial {
    pub fn new(serial: SerialReader) -> Self {
        Self { serial }
    }
}

impl Stream for ReadSerial {
    type Item = u8;
    type Err = Infallible;

    fn fill<S>(&mut self, slots: &mut S) -> nb::Result<(), Self::Err>
    where
        S: Slots<Item = Self::Item>,
    {
        while let Some(slot) = slots.slot() {
            let ch = self.serial.read()?;
            slot.fill(ch);
        }
        Ok(())
    }
}

/// Write bytes into the serial
pub struct WriteSerial {
    serial: SerialWriter,
}

impl WriteSerial {
    pub fn new(serial: SerialWriter) -> Self {
        Self { serial }
    }
}

impl Sink for WriteSerial {
    type Item = u8;
    type Err = Infallible;

    fn sink<R>(&mut self, items: &mut R) -> nb::Result<(), Self::Err>
    where
        R: Ready<Item = Self::Item>,
    {
        while let Some(&next) = items.peek() {
            self.serial.write(next)?;
            items.pop();
        }
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        self.serial.flush()
    }
}
