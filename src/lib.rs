#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use core::num::NonZero;

use arduino_hal::{
    hal::{Atmega, Usart},
    pac::TC0,
    usart::UsartOps,
};

use crate::write::wire::TimerCallback;

pub mod interrupts;
mod time;

mod write {
    pub mod wire {
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

        const TICKS: NonZero<Time> = NonZero::new((1000 / MILLIS_FOR_TICK) as Time).unwrap();

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
    }
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
    serial: Usart<Serial, SerialRx, SerialTx, Clock>,
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
        let (clock, clock_interrupts) = time::Clock::new(timer, TimerCallback::new(tx));

        (Self { serial, rx, clock }, (clock_interrupts,))
    }

    pub fn main_loop(&mut self) -> ! {
        self.clock.timer.try_set(NonZero::new(1).unwrap());
        loop {}
    }
}
