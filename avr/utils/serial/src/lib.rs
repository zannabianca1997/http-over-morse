#![no_std]

use core::convert::Infallible;

use arduino_hal::{
    pac::USART0,
    port::{
        D0, D1, Pin,
        mode::{Input, Output},
    },
    prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write},
    usart::{UsartReader, UsartWriter},
};
use concurrency::{sink::Sink, stream::Stream};

/// Read bytes from the serial
pub struct ReadSerial {
    serial: UsartReader<USART0, Pin<Input, D0>, Pin<Output, D1>>,
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
    serial: UsartWriter<USART0, Pin<Input, D0>, Pin<Output, D1>>,
    writing: Option<u8>,
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
}

pub struct Slots(Option<Slot>);
pub struct Slot(());
