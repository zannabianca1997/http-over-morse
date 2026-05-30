#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(cold_path)]

use core::num::NonZero;

use arduino_hal::{
    hal::{
        Atmega, Usart,
        usart::{UsartReader, UsartWriter},
    },
    pac::TC0,
    usart::UsartOps,
};

use crate::{
    process::{PipeToWriter, Process},
    serial_to_wire::{
        bytes_to_letters::BytesToLetters, letters_to_signals::LettersToSignals,
        signals_to_wire::TimerCallback,
    },
    strategies::Discard,
    wire_to_serial::{letters_to_bytes::LettersToBytes, signals_to_letters::SignalsToLetters},
};

mod encoding;
pub mod interrupts;
mod process;
mod time;
mod utils {}

mod strategies;

mod serial_to_wire {
    //! Processes for writing to wire

    pub mod bytes_to_letters;
    pub mod letters_to_signals;
    pub mod signals_to_wire;
}
mod wire_to_serial {
    //! Processes for reading from the wire

    pub mod letters_to_bytes;
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
    process: PipeToWriter<
        LettersToBytes<
            SignalsToLetters<
                LettersToSignals<
                    BytesToLetters<UsartReader<Serial, SerialRx, SerialTx, Clock>, Discard>,
                >,
            >,
            Discard,
        >,
        UsartWriter<Serial, SerialRx, SerialTx, Clock>,
    >,
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

        let read = LettersToSignals::new(BytesToLetters::new(read));

        let process = PipeToWriter::new(write, LettersToBytes::new(SignalsToLetters::new(read)));

        let (clock, clock_interrupts) = time::Clock::new(timer, TimerCallback::new(tx));

        (Self { process, rx, clock }, (clock_interrupts,))
    }

    pub fn main_loop(&mut self) -> ! {
        self.clock.timer.try_set(NonZero::new(1).unwrap());

        loop {
            match self.process.poll() {
                Ok(()) => (),
                Err(nb::Error::Other(err)) => {
                    // TODO: error handling

                    drop(err);
                }
                Err(nb::Error::WouldBlock) => (),
            }
        }
    }
}
