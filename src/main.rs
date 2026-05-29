#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::{
    clock::MHz16,
    pac::USART0,
    port::{
        D0, D1, D2, D13, Pin,
        mode::{Floating, Input, Output},
    },
};
use avr_device::interrupt;
use http_over_morse::interrupts::{InterruptStorage, TIMER0_COMPA};
use panic_halt as _;

type App = http_over_morse::App<
    USART0,
    Pin<Input, D0>,
    Pin<Output, D1>,
    MHz16,
    Pin<Input<Floating>, D2>,
    Pin<Output, D13>,
>;

type Interrupts = http_over_morse::Interrupts<Pin<Output, D13>>;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let rx = pins.d2;
    let tx = pins.d13.into_output();
    let timer = dp.TC0;

    let (mut app, interrupts) = App::new(serial, rx, tx, timer);

    unsafe {
        // Safety: interrupts are off at the start
        INTERRUPTS.init(interrupts);
        avr_device::interrupt::enable()
    };

    app.main_loop()
}

static INTERRUPTS: InterruptStorage<Interrupts> = InterruptStorage::new();

#[interrupt(atmega328p)]
fn TIMER0_COMPA() {
    unsafe {
        INTERRUPTS.trigger::<TIMER0_COMPA>();
    }
}
