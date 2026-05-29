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

use crate::{process::Process, write::wire::TimerCallback};

mod encoding;
pub mod interrupts;
mod process;
mod time;

mod write {
    //! Writing processes

    pub mod wire;
}
mod read {
    //! Reading processes
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
    read: UsartReader<Serial, SerialRx, SerialTx, Clock>,
    write: UsartWriter<Serial, SerialRx, SerialTx, Clock>,
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

        let (clock, clock_interrupts) = time::Clock::new(timer, TimerCallback::new(tx));

        (
            Self {
                read,
                write,
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
                match self.read.poll() {
                    Ok(ch) => break ch,
                    Err(nb::Error::WouldBlock) => (),
                    Err(nb::Error::Other(err)) => match err {},
                }
            };
            loop {
                match self.write.write(ch) {
                    Ok(()) => break,
                    Err(nb::Error::WouldBlock) => (),
                    Err(nb::Error::Other(err)) => match err {},
                }
            }
        }
    }
}
