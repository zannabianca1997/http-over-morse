//! Led
//!
//! Usually the led is mapped to the things that are being written on the cable.
//! In some debug programs with no cable support (e.g. identify) it can be split.

use crate::pins::Pin13;

pub struct DebugLed(Pin13);

impl DebugLed {
    pub(crate) fn new(pin_13: Pin13) -> Self {
        Self(pin_13)
    }

    pub fn state(&self) -> bool {
        self.0.is_set_high()
    }

    pub fn on(&mut self) {
        self.0.set_high();
    }

    pub fn off(&mut self) {
        self.0.set_low();
    }
}
