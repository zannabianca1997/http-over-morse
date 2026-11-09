use arduino_hal::{
    pac::{PORTB, PORTC, PORTD},
    port::{
        D0, D1, D2, D3, D13, Pin,
        mode::{Input, OpenDrain, Output, PullUp},
    },
};

pub(crate) type Pin0 = Pin<Input<PullUp>, D0>;
pub(crate) type Pin1 = Pin<Output, D1>;

pub(crate) type Pin2 = Pin<OpenDrain, D2>;
pub(crate) type Pin3 = Pin<OpenDrain, D3>;

pub(crate) type Pin13 = Pin<Output, D13>;

/// Pins used in the project
pub(crate) struct Pins {
    // serial UART
    pub pin_0: Pin0,
    pub pin_1: Pin1,

    // wire A/B
    pub pin_2: Pin2,
    pub pin_3: Pin3,

    // debug led
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
            d13,
            ..
        } = arduino_hal::Pins::with_mcu_pins(arduino_hal::hal::Pins::new(portb, portc, portd));

        // Setup the pins
        // The serial RX pin is pulled up: a floating antenna picks up noise
        // that the usart would read as phantom bytes (e.g. 0x00)
        let pin_0 = d0.into_pull_up_input();
        let pin_1 = d1.into_output();

        let pin_3 = d3.into_opendrain_high();
        let pin_2 = d2.into_opendrain_high();

        let pin_13 = d13.into_output();

        Self {
            pin_0,
            pin_1,
            pin_2,
            pin_3,
            pin_13,
        }
    }
}
