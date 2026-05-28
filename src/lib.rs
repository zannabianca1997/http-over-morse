#![no_std]

use core::convert::Infallible;

use arduino_hal::{
    prelude::*,
    hal::{Atmega, Usart},
    pac::TC0,
    usart::UsartOps,
};
use embedded_hal::digital::OutputPin;

/// Firmware
pub struct App<
    Serial: UsartOps<Atmega, SerialRx, SerialTx>,
    SerialRx,
    SerialTx,
    Clock,
    WireRx,
    WireTx,
> {
    serial: Usart<Serial, SerialRx, SerialTx, Clock>,
    wire: (WireRx, WireTx),
}

/// Interrupts of the application
pub struct AppInterrupts {}

impl<Serial: UsartOps<Atmega, SerialRx, SerialTx>, SerialRx, SerialTx, Clock, WireRx, WireTx>
    App<Serial, SerialRx, SerialTx, Clock, WireRx, WireTx>
{
    /// Initialize the application
    pub fn new(
        serial: Usart<Serial, SerialRx, SerialTx, Clock>,
        wire: (WireRx, WireTx),
        _timer: TC0,
    ) -> (Self, AppInterrupts) {
        (Self { serial, wire }, AppInterrupts {})
    }

    pub fn main_loop(&mut self) -> !
    where
        WireTx: OutputPin<Error = Infallible>,
    {
        loop {
            self.wire.1.set_high().unwrap_infallible();
            arduino_hal::delay_ms(1000);
            self.wire.1.set_low().unwrap_infallible();
            arduino_hal::delay_ms(1000);
        }
    }
}
