#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
// Needed for slicing the static storage in a const way
#![feature(const_trait_impl)]
#![feature(const_index)]

use core::convert::Infallible;

use arduino_hal::{
    hal::{
        Atmega, Usart,
        usart::{UsartReader, UsartWriter},
    },
    pac::TC0,
    usart::UsartOps,
};
use embedded_hal::digital::OutputPin;

use crate::{
    process::{PipeToWriter, Process},
    serial_to_wire::{
        bytes_to_letters::BytesToLetters,
        letters_to_signals::LettersToSignals,
        segments_to_wire::{SegmentsToWire, TimerCallback},
        signals_to_segments::SignalsToSegments,
    },
    strategies::Discard,
    wire_to_serial::{
        letters_to_bytes::LettersToBytes, segments_to_signals::SegmentsToSignals,
        signals_to_letters::SignalsToLetters,
    },
};

mod encoding;
pub mod interrupts;
mod process;
mod segments;
mod time;
mod utils {
    pub mod ringbuf;
}

mod strategies;

mod serial_to_wire {
    //! Processes for writing to wire

    pub mod bytes_to_letters;
    pub mod letters_to_signals;
    pub mod segments_to_wire;
    pub mod signals_to_segments;
}
mod wire_to_serial {
    //! Processes for reading from the wire

    pub mod letters_to_bytes;
    pub mod segments_to_signals;
    pub mod signals_to_letters;
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
    process: SegmentsToWire<
        SignalsToSegments<
            LettersToSignals<
                BytesToLetters<UsartReader<Serial, SerialRx, SerialTx, Clock>, Discard>,
            >,
        >,
        WireTx,
    >,
    _rx: WireRx,
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
        let (serial_read, _serial_write) = serial.split();
        let (time::Clock { timer, .. }, clock_interrupts) =
            time::Clock::new(timer, TimerCallback::new(tx));

        let process = SegmentsToWire::new(
            SignalsToSegments::new(LettersToSignals::new(BytesToLetters::new(serial_read))),
            timer,
        );

        (Self { process, _rx: rx }, (clock_interrupts,))
    }

    pub fn main_loop(&mut self) -> !
    where
        WireTx: OutputPin<Error = Infallible>,
    {
        loop {
            match self.process.poll() {
                Err(nb::Error::WouldBlock) => (),
            }
        }
    }
}
