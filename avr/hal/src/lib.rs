#![doc = include_str!("../README.md")]
#![no_std]

pub use arduino_hal::entry;

use pins::Pins;

pub use crate::led::DebugLed;
pub use serial::Serial;
pub use wire::Wire;

pub mod led;
mod panic;
mod pins;
pub mod serial;
pub mod wire;

/// Peripherals used in the project
pub struct Peripherals {
    pub serial: Serial,
    pub wires: (Wire<wire::A>, Wire<wire::B>),
    pub debug_led: DebugLed,
}

impl Peripherals {
    /// Initialize the peripherals
    pub fn new() -> Option<Self> {
        arduino_hal::Peripherals::take().map(Self::init)
    }

    /// Initialize the peripherals without checking if they were already
    /// initialized
    ///
    /// # SAFETY
    ///
    /// The user must not have initialized the peripherals before, nor have
    /// interacted with the hardware in other ways.
    pub unsafe fn new_unchecked() -> Self {
        Self::init(unsafe {
            // Safety: see the contract above
            arduino_hal::Peripherals::steal()
        })
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
            pin_2,
            pin_13,
        } = Pins::new(PORTB, PORTC, PORTD);

        let serial = Serial::new(USART0, pin_0, pin_1);

        let wire_a = Wire::new(pin_2);
        let wire_b = Wire::new(pin_3);

        let debug_led = DebugLed::new(pin_13);

        Self {
            serial,
            wires: (wire_a, wire_b),
            debug_led,
        }
    }
}
