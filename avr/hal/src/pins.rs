use arduino_hal::{
    pac::{PORTB, PORTC, PORTD},
    port::{
        D0, D1, Pin,
        mode::{Floating, Input, Output},
    },
};

pub(crate) type Pin0 = Pin<Input<Floating>, D0>;
pub(crate) type Pin1 = Pin<Output, D1>;

/// Pins used in the project
pub(crate) struct Pins {
    pub pin_0: Pin0,
    pub pin_1: Pin<Output, D1>,
}

impl Pins {
    pub(crate) fn new(portb: PORTB, portc: PORTC, portd: PORTD) -> Self {
        // Construct the pins
        let arduino_hal::Pins { d0, d1, .. } =
            arduino_hal::Pins::with_mcu_pins(arduino_hal::hal::Pins::new(portb, portc, portd));

        // Setup the pins
        let pin_0 = d0;
        let pin_1 = d1.into_output();

        Self { pin_0, pin_1 }
    }
}
