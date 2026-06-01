#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::{
    pac::USART0,
    port::{
        D0, D1, D13, Pin,
        mode::{Input, Output},
    },
    usart::{UsartReader, UsartWriter},
};
use avr_device::interrupt;
use http_over_morse::{
    interrupts::{INT0, InterruptStorage, TIMER0_COMPA},
    strategies,
};
use panic_halt as _;

type App = http_over_morse::App<
    UsartReader<USART0, Pin<Input, D0>, Pin<Output, D1>>,
    UsartWriter<USART0, Pin<Input, D0>, Pin<Output, D1>>,
    Pin<Output, D13>,
    strategies::Base32,
>;

type Interrupts = http_over_morse::Interrupts<Pin<Output, D13>>;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let (serial_reader, serial_writer) = arduino_hal::default_serial!(dp, pins, 9600).split();
    let external_interrupts = dp.EXINT;
    let rx = pins.d2;
    let tx = pins.d13.into_output();
    let timer = dp.TC0;

    let (mut app, interrupts) = App::new(
        serial_reader,
        serial_writer,
        external_interrupts,
        rx,
        tx,
        timer,
    );

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

#[interrupt(atmega328p)]
fn INT0() {
    unsafe {
        INTERRUPTS.trigger::<INT0>();
    }
}
