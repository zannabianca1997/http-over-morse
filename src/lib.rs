#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
// Needed for slicing the static storage in a const way
#![feature(const_trait_impl)]
#![feature(const_index)]

use core::convert::Infallible;

use arduino_hal::{
    pac::{EXINT, TC0},
    prelude::{_embedded_hal_serial_Read as Read, _embedded_hal_serial_Write as Write},
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
    time::Stopwatch,
    wire_to_serial::{
        letters_to_bytes::LettersToBytes,
        segments_to_signals::SegmentsToSignals,
        signals_to_letters::SignalsToLetters,
        wire_to_segments::{ListeningPinCallback, WireToSegments},
    },
};

mod encoding;
pub mod interrupts;
mod listening_pin;
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
    pub mod wire_to_segments;
}

/// Firmware
pub struct App<SerialReader, SerialWriter, WireTx> {
    serial_to_wire: SegmentsToWire<
        SignalsToSegments<LettersToSignals<BytesToLetters<SerialReader, Discard>>>,
        WireTx,
    >,
    wire_to_serial: PipeToWriter<
        LettersToBytes<SignalsToLetters<SegmentsToSignals<WireToSegments>>, Discard>,
        SerialWriter,
    >,
}

/// Interrupts of the application
pub type Interrupts<WireTx> = (
    time::Interrupts<TimerCallback<WireTx>>,
    listening_pin::Interrupts<ListeningPinCallback>,
);

impl<SerialReader, SerialWriter, WireTx> App<SerialReader, SerialWriter, WireTx> {
    /// Initialize the application
    pub fn new(
        serial_reader: SerialReader,
        serial_writer: SerialWriter,
        external_interrupts: EXINT,
        rx: listening_pin::Pin,
        tx: WireTx,
        timer: TC0,
    ) -> (Self, Interrupts<WireTx>) {
        // Setup the clock
        let (
            time::Clock {
                timer,
                stopwatch: Stopwatch { reset, overflow },
            },
            clock_interrupts,
        ) = time::Clock::new(timer, TimerCallback::new(tx));

        // Setup the pin to listen
        let (listening_pin, listening_pin_interrupts) = listening_pin::ListeningPin::new(
            external_interrupts,
            rx,
            ListeningPinCallback::new(reset),
        );

        // Wire to serial
        let wire_to_serial = PipeToWriter::new(
            serial_writer,
            LettersToBytes::new(SignalsToLetters::new(SegmentsToSignals::new(
                WireToSegments::new(listening_pin, overflow),
            ))),
        );

        // Serial to wire
        let serial_to_wire = SegmentsToWire::new(
            timer,
            SignalsToSegments::new(LettersToSignals::new(BytesToLetters::new(serial_reader))),
        );

        (
            Self {
                serial_to_wire,
                wire_to_serial,
            },
            (clock_interrupts, listening_pin_interrupts),
        )
    }

    pub fn main_loop(&mut self) -> !
    where
        WireTx: OutputPin<Error = Infallible>,
        SerialReader: Read<u8, Error = Infallible>,
        SerialWriter: Write<u8, Error = Infallible>,
    {
        loop {
            match self.serial_to_wire.poll() {
                Err(nb::Error::WouldBlock) => (),
            }
            match self.wire_to_serial.poll() {
                Err(nb::Error::WouldBlock) => (),
                Err(nb::Error::Other(err)) => {
                    // TODO: handle the errors somehow

                    drop(err);
                }
            }
        }
    }
}
