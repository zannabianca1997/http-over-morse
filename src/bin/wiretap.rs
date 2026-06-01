//! Direct to wire communication
//!
//! This binary will pipe the content of the serial port directly to the wire,
//! and read back the wire content to the serial port.
//!
//! Configuration is possible during compilation with environment variables:
//! - BAUDRATE: set the baudrate for the serial port
//! - BASE32: consider the values on the wire as base32 encoded bytes
//!
//! # Caveat
//! Even at the slowest possible baud rate of 250 the encoding is yet not fast
//! enough to empty the serial buffer in time. Do not pipe things straight to
//! this serial port

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::port::{D13, Pin, mode::Output};
use avr_device::interrupt;
use panic_halt as _;

use interrupts::{INT0, InterruptStorage, TIMER0_COMPA};
use listening_pin::{self, ListeningPin};
use process::{Process as _, pipe_to_writer::PipeToWriter};
use time::{self, Clock, Stopwatch};

use http_over_morse::{
    from_wire::{BytesFromWire, ListeningPinCallback},
    strategies,
    to_wire::{BytesToWire, TimerCallback},
};

type Strategy = cfg_select! {
    base32 => strategies::Base32,
    _ => strategies::Discard,
};

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

    // Setup the clock
    let (
        Clock {
            timer,
            stopwatch: Stopwatch { reset, overflow },
        },
        clock_interrupts,
    ) = Clock::new(timer, unsafe {
        // Safety: we instantiate this only here
        TimerCallback::new(tx)
    });

    // Setup the pin to listen
    let (listening_pin, listening_pin_interrupts) =
        ListeningPin::new(external_interrupts, rx, ListeningPinCallback::new(reset));

    // Wire to serial
    let mut wire_to_serial = PipeToWriter::new(
        serial_writer,
        BytesFromWire::<Strategy>::new(listening_pin, overflow),
    );

    // Serial to wire
    let mut serial_to_wire = BytesToWire::<_, Strategy, _>::new(serial_reader, timer);

    unsafe {
        // Safety: interrupts are off at the start
        INTERRUPTS.init((clock_interrupts, listening_pin_interrupts));
        avr_device::interrupt::enable()
    };

    loop {
        match serial_to_wire.poll() {
            Err(nb::Error::WouldBlock) => (),
        }
        match wire_to_serial.poll() {
            Err(nb::Error::WouldBlock) => (),
        }
    }
}

static INTERRUPTS: InterruptStorage<(
    time::Interrupts<TimerCallback<Pin<Output, D13>>>,
    listening_pin::Interrupts<ListeningPinCallback>,
)> = InterruptStorage::new();

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
