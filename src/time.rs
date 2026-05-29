//! Time related stuff

use arduino_hal::pac::TC0;

pub use timer::TimerCallback;

/// Type for time calculations
pub type Time = usize;

mod tc0;

mod stopwatch;
mod timer;

const PRESCALE: tc0::Prescale = tc0::Prescale::Prescale1024;
const TIMER_COUNTS: u8 = 125;
pub const MILLIS_FOR_TICK: u32 = tc0::millis_per_tick(TIMER_COUNTS, PRESCALE);

pub struct Clock<TCb> {
    pub timer: timer::Timer<TCb>,
    pub stopwatch: stopwatch::Stopwatch,
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
