use arduino_hal::{
    pac::{PORTB, PORTC, PORTD},
    port::{
        D0, D1, D2, D3, D4, D13, Pin,
        mode::{Floating, Input, Output},
    },
};

pub(crate) type Pin0 = Pin<Input<Floating>, D0>;
pub(crate) type Pin1 = Pin<Output, D1>;

pub(crate) type Pin3 = Pin<Input<Floating>, D3>;
pub(crate) type Pin4 = Pin<Output, D4>;

pub(crate) type Pin2 = Pin<Input<Floating>, D2>;
pub(crate) type Pin13 = Pin<Output, D13>;

/// Pins used in the project
pub(crate) struct Pins {
    // serial UART
    pub pin_0: Pin0,
    pub pin_1: Pin<Output, D1>,

    // wire A
    pub pin_3: Pin3,
    pub pin_4: Pin4,

    // wire B
    pub pin_2: Pin2,
    pub pin_13: Pin13,
}

impl Pins {
    pub(crate) fn new(portb: PORTB, portc: PORTC, portd: PORTD) -> Self {
        // Construct the pins
        let arduino_hal::Pins {
            d0,
            d1,
            d2,
            d3,
            d4,
            d13,
            ..
        } = arduino_hal::Pins::with_mcu_pins(arduino_hal::hal::Pins::new(portb, portc, portd));

        // Setup the pins
        let pin_0 = d0;
        let pin_1 = d1.into_output();

        let pin_3 = d3;
        let pin_4 = d4.into_output();

        let pin_2 = d2;
        let pin_13 = d13.into_output();

        Self {
            pin_0,
            pin_1,
            pin_3,
            pin_4,
            pin_2,
            pin_13,
        }
    }
}
