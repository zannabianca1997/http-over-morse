#![doc = include_str!("../README.md")]
#![no_std]

use core::convert::Infallible;

use concurrency::{sink::Sink, stream::Stream};
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
    type Ready = Ready;

    fn ready(&mut self) -> nb::Result<Self::Ready, Self::Err> {
        self.serial.read().map(|v| Ready(Some(v)))
    }

    fn stream(&mut self, ready: &mut Self::Ready) -> Result<Option<Self::Item>, Self::Err> {
        Ok(ready.0.take())
    }
}

pub struct Ready(Option<u8>);

/// Write bytes into the serial
pub struct WriteSerial {
    serial: SerialWriter,
    writing: Option<u8>,
}

impl WriteSerial {
    pub fn new(serial: SerialWriter) -> Self {
        Self {
            serial,
            writing: None,
        }
    }
}

impl Sink for WriteSerial {
    type Item = u8;
    type Err = Infallible;
    type Slots = Slots;
    type Slot = Slot;

    fn slots(&mut self) -> nb::Result<Self::Slots, Self::Err> {
        if let Some(writing) = self.writing {
            self.serial.write(writing)?;
        };
        self.writing = None;
        Ok(Slots(Some(Slot(()))))
    }

    fn slot(&mut self, Slots(slot): &mut Self::Slots) -> Result<Option<Self::Slot>, Self::Err> {
        Ok(slot.take())
    }

    fn sink(&mut self, Slot(()): Self::Slot, item: Self::Item) {
        self.writing = Some(item)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        // Empty the buffer
        if let Some(writing) = self.writing {
            self.serial.write(writing)?;
        };
        // Flush the serial
        self.serial.flush()
    }
}

pub struct Slots(Option<Slot>);
pub struct Slot(());
