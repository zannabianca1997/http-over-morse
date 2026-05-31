//! Write segments to the wire

use core::{convert::Infallible, num::NonZero};

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use embedded_hal::digital::{OutputPin, PinState};

use crate::time::{MILLIS_FOR_TICK, Time};

pub struct TimerCallback<Tx> {
    pin: Tx,
    state: PinState,
}

impl<Tx> TimerCallback<Tx> {
    pub fn new(pin: Tx) -> Self {
        Self {
            pin,
            state: PinState::Low,
        }
    }
}

const TICKS: NonZero<Time> = NonZero::new(
    NonZero::new(1000u32)
        .unwrap()
        .div_ceil(MILLIS_FOR_TICK)
        .get() as Time,
)
.unwrap();

impl<Tx> crate::time::TimerCallback for TimerCallback<Tx>
where
    Tx: OutputPin<Error = Infallible>,
{
    fn triggered(
        &mut self,
        _cs: avr_device::interrupt::CriticalSection,
    ) -> Option<core::num::NonZero<crate::time::Time>> {
        self.state = !self.state;
        self.pin.set_state(self.state).unwrap_infallible();
        Some(TICKS)
    }
}
