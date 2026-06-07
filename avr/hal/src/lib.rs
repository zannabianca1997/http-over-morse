#![doc = include_str!("../README.md")]
#![no_std]

pub use arduino_hal::entry;

use pins::Pins;
pub use serial::Serial;

mod panic;
mod pins;
pub mod serial;

/// Peripherals used in the project
pub struct Peripherals {
    pub serial: Serial,
}

impl Peripherals {
    /// Initialize the peripherals
    pub fn new() -> Option<Self> {
        arduino_hal::Peripherals::take().map(Self::init)
    }

    /// Unchecked version of [`Peripherals::new`].
    ///
    /// # Safety
    ///
    /// Exactly one between this and [`Peripherals::new`] must be ever called
    pub unsafe fn new_unchecked() -> Self {
        Self::init(unsafe { arduino_hal::Peripherals::steal() })
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
        let Pins { pin_0, pin_1 } = Pins::new(PORTB, PORTC, PORTD);

        let serial = Serial::new(USART0, pin_0, pin_1);

        Self { serial }
    }
}
