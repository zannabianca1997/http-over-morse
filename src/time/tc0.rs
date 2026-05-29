use arduino_hal::pac::{TC0, tc0::tccr0b::CS0_A};

pub enum Prescale {
    ///1: Running, No Prescaling
    Direct = 1,
    ///2: Running, CLK/8
    Prescale8 = 2,
    ///3: Running, CLK/64
    Prescale64 = 3,
    ///4: Running, CLK/256
    Prescale256 = 4,
    ///5: Running, CLK/1024
    Prescale1024 = 5,
}
impl From<Prescale> for CS0_A {
    fn from(value: Prescale) -> Self {
        match value {
            Prescale::Direct => CS0_A::DIRECT,
            Prescale::Prescale8 => CS0_A::PRESCALE_8,
            Prescale::Prescale64 => CS0_A::PRESCALE_64,
            Prescale::Prescale256 => CS0_A::PRESCALE_256,
            Prescale::Prescale1024 => CS0_A::PRESCALE_1024,
        }
    }
}

/// Marker struct that the tc0 timer has been initialized
pub(super) struct InitializedForTimer {
    _priv: (),
}
/// Marker struct that the tc0 timer has been initialized
pub(super) struct InitializedForStopwatch {
    _priv: (),
}

pub(super) fn init(
    tc0: TC0,
    timer_counts: u8,
    prescaler: Prescale,
) -> (InitializedForTimer, InitializedForStopwatch) {
    // set the timer it ctc mode
    tc0.tccr0a().write(|w| w.wgm0().ctc());
    // set how many ticks before the interrupt is triggered
    tc0.ocr0a().write(|w| w.set(timer_counts));
    // set the prescale
    tc0.tccr0b().write(|w| w.cs0().variant(prescaler.into()));
    // enable the interrupt
    tc0.timsk0().write(|w| w.ocie0a().set_bit());

    drop(tc0);
    (
        InitializedForTimer { _priv: () },
        InitializedForStopwatch { _priv: () },
    )
}

pub const fn millis_per_tick(timer_counts: u8, prescaler: Prescale) -> u32 {
    (match prescaler {
        Prescale::Direct => 1,
        Prescale::Prescale8 => 8,
        Prescale::Prescale64 => 64,
        Prescale::Prescale256 => 256,
        Prescale::Prescale1024 => 1024,
    }) * timer_counts as u32
        / 16000
}
