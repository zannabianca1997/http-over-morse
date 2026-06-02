#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use core::num::NonZero;

use arduino_hal::pac::TC0;

pub use stopwatch::{Stopwatch, StopwatchOverflow, StopwatchReset};
pub use timer::{Timer, TimerCallback};

/// Type for time calculations
// TODO: pretty sure we can get away with a u8, and that would speed up
// calculations quite some. But we need to check every point for assumptions on
// the size of this type
pub type Time = usize;

mod tc0;

mod stopwatch;
mod timer;

pub mod segments;

const PRESCALE: tc0::Prescale = tc0::Prescale::Prescale1024;
const TIMER_COUNTS: u8 = 125;
pub const MILLIS_FOR_TICK: NonZero<u32> = tc0::millis_per_tick(TIMER_COUNTS, PRESCALE);

pub struct Clock<TCb> {
    pub timer: Timer<TCb>,
    pub stopwatch: Stopwatch,
}

pub type Interrupts<TCb> = (timer::Interrupts<TCb>, stopwatch::Interrupts);

impl<TCb> Clock<TCb> {
    pub fn new(timer: TC0, callback: TCb) -> (Self, Interrupts<TCb>) {
        let (t, s) = tc0::init(timer, TIMER_COUNTS, PRESCALE);

        let (timer, timer_int) = timer::Timer::new(t, callback);
        let (stopwatch, stopwatch_int) = stopwatch::Stopwatch::new(s);

        (Self { timer, stopwatch }, (timer_int, stopwatch_int))
    }
}
