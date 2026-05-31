//! Timer
//!
//! Used to set events in the future. Has a callback to call once it ended

use core::{cell::Cell, marker::PhantomData, num::NonZero};

use avr_device::interrupt::{CriticalSection, Mutex, free};

use super::{Time, tc0::InitializedForTimer};

/// Callback to trigger when the timer reached zero
pub trait TimerCallback {
    /// Timer has reached 0.
    ///
    /// If the callback returns [`Some`], the timer is reset.
    fn triggered(&mut self, cs: CriticalSection) -> Option<NonZero<Time>>;
}

pub struct Timer<CB> {
    _cb: PhantomData<CB>,
}

impl<CB> Timer<CB> {
    /// Initialize the timer
    pub(super) fn new(_tc0: InitializedForTimer, callback: CB) -> (Self, Interrupts<CB>) {
        (Self { _cb: PhantomData }, Interrupts { _tc0, callback })
    }

    /// Set the timer if not set
    pub fn try_set(&mut self, time: NonZero<Time>) -> bool {
        free(|cs| {
            let timer = TIMER.borrow(cs);
            if timer.get().is_some() {
                false
            } else {
                timer.set(Some(time));
                true
            }
        })
    }

    /// Trigger the timer at the next tick if it is not set
    pub fn trigger_if_not_set(&mut self) -> bool {
        self.try_set(NonZero::new(1).unwrap())
    }
}

pub struct Interrupts<CB> {
    _tc0: InitializedForTimer,
    callback: CB,
}

/// Timer state
///
/// As only one [`arduino_hal::pac::TC0`] exists, only one [`Interrupts`] can
/// exists at a time.
static TIMER: Mutex<Cell<Option<NonZero<Time>>>> = Mutex::new(Cell::new(None));

impl<CB> crate::interrupts::Interrupts for Interrupts<CB>
where
    CB: TimerCallback,
{
    fn timer0compa(&mut self, cs: CriticalSection) {
        let timer = TIMER.borrow(cs);

        let Some(ticks) = timer.get() else {
            // Timer is not set
            return;
        };

        // Decrease, if reached zero call the callback and set the timer to that
        let ticks = NonZero::new(ticks.get() - 1).or_else(|| self.callback.triggered(cs));

        // put back ticks on the timer
        timer.set(ticks);
    }
}
