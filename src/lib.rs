#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use core::num::NonZero;

use arduino_hal::{
    hal::{
        Atmega, Usart,
        usart::{UsartReader, UsartWriter},
    },
    pac::TC0,
    prelude::_embedded_hal_serial_Write,
    usart::UsartOps,
};

use crate::{
    process::Process,
    strategies::Discard,
    write::{letters::ReadLetters, wire::TimerCallback},
};

mod encoding;
pub mod interrupts;
mod process;
mod time;

mod strategies;

mod write {
    //! Processes for writing to wire

    pub mod letters;
    pub mod wire;
}
mod read {
    //! Processes for reading from the wire

    pub mod serial {
        use arduino_hal::prelude::_embedded_hal_serial_Write;

        use crate::process::Process;

        pub struct WriteToSerial<Inner, Serial> {
            serial: Serial,
            inner: Inner,
            writing: Option<u8>,
        }

        impl<Inner, Serial> WriteToSerial<Inner, Serial> {
            pub fn new(serial: Serial, inner: Inner) -> Self {
                Self {
                    serial,
                    inner,
                    writing: None,
                }
            }
        }

        impl<Inner, Serial> Process for WriteToSerial<Inner, Serial>
        where
            Inner: Process<Ok = u8>,
            Serial: _embedded_hal_serial_Write<u8>,
        {
            type Ok = ();
            type Err = Error<Inner::Err, Serial::Error>;

            fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
                loop {
                    // Ensure the writing char is empty
                    while let Some(writing) = self.writing {
                        match self.serial.write(writing) {
                            Ok(()) => self.writing = None,
                            Err(err) => return Err(err.map(Error::Serial)),
                        }
                    }
                    // Get a new char to write
                    match self.inner.poll() {
                        Ok(ch) => self.writing = Some(ch),
                        Err(err) => return Err(err.map(Error::Inner)),
                    }
                }
            }
        }

        pub enum Error<I, S> {
            Inner(I),
            Serial(S),
        }
    }
}

/// Firmware
pub struct App<
    Serial: UsartOps<Atmega, SerialRx, SerialTx>,
    SerialRx,
    SerialTx,
    Clock,
    WireRx,
    WireTx,
> {
    write: ReadLetters<UsartReader<Serial, SerialRx, SerialTx, Clock>, Discard>,
    read: UsartWriter<Serial, SerialRx, SerialTx, Clock>,
    rx: WireRx,
    clock: time::Clock<TimerCallback<WireTx>>,
}

/// Interrupts of the application
pub type Interrupts<WireTx> = (time::Interrupts<TimerCallback<WireTx>>,);

impl<Serial: UsartOps<Atmega, SerialRx, SerialTx>, SerialRx, SerialTx, Clock, WireRx, WireTx>
    App<Serial, SerialRx, SerialTx, Clock, WireRx, WireTx>
{
    /// Initialize the application
    pub fn new(
        serial: Usart<Serial, SerialRx, SerialTx, Clock>,
        rx: WireRx,
        tx: WireTx,
        timer: TC0,
    ) -> (Self, Interrupts<WireTx>) {
        let (read, write) = serial.split();

        let read = ReadLetters::new(read);

        let (clock, clock_interrupts) = time::Clock::new(timer, TimerCallback::new(tx));

        (
            Self {
                write: read,
                read: write,
                rx,
                clock,
            },
            (clock_interrupts,),
        )
    }

    pub fn main_loop(&mut self) -> ! {
        self.clock.timer.try_set(NonZero::new(1).unwrap());

        loop {
            let ch = loop {
                match self.write.poll() {
                    Ok(ch) => break ch,
                    Err(nb::Error::WouldBlock) => (),
                    Err(nb::Error::Other(err)) => match err {},
                }
            };
            loop {
                match self.read.write(ch.char()) {
                    Ok(()) => break,
                    Err(nb::Error::WouldBlock) => (),
                    Err(nb::Error::Other(err)) => match err {},
                }
            }
        }
    }
}
