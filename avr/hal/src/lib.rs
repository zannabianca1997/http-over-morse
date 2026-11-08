#![doc = include_str!("../README.md")]
#![no_std]

pub use arduino_hal::entry;

use pins::Pins;
pub use serial::Serial;
pub use wire::{WireA, WireB};

pub mod led;
mod panic;
mod pins;
pub mod serial;
pub mod wire;

/// Peripherals used in the project
pub struct Peripherals {
    pub serial: Serial,
    pub wires: (WireA, WireB),
}

impl Peripherals {
    /// Initialize the peripherals
    pub fn new() -> Option<Self> {
        arduino_hal::Peripherals::take().map(Self::init)
    }

    fn init(
        arduino_hal::Peripherals {
            PORTB,
            PORTC,
            PORTD,
            USART0,
            ..
        }: arduino_hal::Peripherals,
    ) -> Self {
        let Pins {
            pin_0,
            pin_1,
            pin_3,
            pin_4,
            pin_2,
            pin_13,
        } = Pins::new(PORTB, PORTC, PORTD);

        let serial = Serial::new(USART0, pin_0, pin_1);

        let wire_a = WireA::new(pin_3, pin_4);
        let wire_b = WireB::new(pin_2, pin_13);

        Self {
            serial,
            wires: (wire_a, wire_b),
        }
    }
}
