//! Stopwatch
//!
//! Used to measure incoming signal, counts up and can be reset

use core::cell::Cell;

use arduino_hal::pac::TC0;
use avr_device::interrupt::{CriticalSection, Mutex, free};

use super::{Time, tc0::InitializedForStopwatch};

pub struct Stopwatch {
    pub reset: StopwatchReset,
    pub overflow: StopwatchOverflow,
}

impl Stopwatch {
    /// Initialize the stopwatch with an already initialized tc0
    pub(super) fn new_with_initialized(_tc0: InitializedForStopwatch) -> (Self, Interrupts) {
        (
            Self {
                reset: StopwatchReset { _priv: () },
                overflow: StopwatchOverflow { _priv: () },
            },
            Interrupts { _tc0 },
        )
    }

    /// Initialize the stopwatch
    pub fn new(timer: TC0) -> (Self, Interrupts) {
        let (_, _tc0) = super::tc0::init(timer, super::TIMER_COUNTS, super::PRESCALE);
        Self::new_with_initialized(_tc0)
    }
}

pub struct StopwatchReset {
    _priv: (),
}

impl StopwatchReset {
    /// Reset the timer
    pub fn reset(&mut self, cs: CriticalSection) -> Time {
        STOPWATCH.borrow(cs).replace(0)
    }
}

pub struct StopwatchOverflow {
    _priv: (),
}

impl StopwatchOverflow {
    /// Reset the timer if greater than some limit
    pub fn reset_if_greater(&mut self, threshold: Time) -> Option<Time> {
        free(|cs| {
            let stopwatch = STOPWATCH.borrow(cs);
            if stopwatch.get() > threshold {
                Some(stopwatch.replace(0))
            } else {
                None
            }
        })
    }
}

pub struct Interrupts {
    _tc0: InitializedForStopwatch,
}

/// Stopwatch state
///
/// As only one [`arduino_hal::pac::TC0`] exists, only one [`Interrupts`] can
/// exists at a time.
static STOPWATCH: Mutex<Cell<Time>> = Mutex::new(Cell::new(0));

impl interrupts::Interrupts for Interrupts {
    fn timer0compa(&mut self, cs: CriticalSection) {
        STOPWATCH.borrow(cs).update(|t| t.saturating_add(1));
    }
}
