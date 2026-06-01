#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::{
    hal::usart::BaudrateExt,
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
    cfg_select! {
        base32 => strategies::Base32,
        _ => strategies::Discard,
    },
>;

type Interrupts = http_over_morse::Interrupts<Pin<Output, D13>>;

/// Baudrate of the serial. Set the `BAUDRATE` enviroment variable to control
/// its value
const BAUDRATE: u32 = const {
    if let Some(env) = option_env!("BAUDRATE") {
        match u32::from_str_radix(env, 10) {
            Ok(v) => v,
            Err(_) => panic!("Invalid baud rate"),
        }
    } else {
        9600
    }
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = unsafe {
        // Safety: main just started
        arduino_hal::Peripherals::steal()
    };
    let pins = arduino_hal::pins!(dp);
    let (serial_reader, serial_writer) = arduino_hal::default_serial!(dp, pins, BAUDRATE).split();
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
