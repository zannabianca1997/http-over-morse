#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

#[cfg(feature = "write")]
use arduino_hal::port::{D13, Pin, mode::Output};
use avr_device::interrupt;
use panic_halt as _;

use encoding;
use interrupts::{INT0, InterruptStorage, TIMER0_COMPA};
use listening_pin::{self, ListeningPin};
use process::{Process as _, pipe_to_writer::PipeToWriter};
#[cfg(feature = "write")]
use time::Clock;
use time::{self, Stopwatch};

use layer_physical::from_wire::{BytesFromWire, ListeningPinCallback};
#[cfg(feature = "write")]
use layer_physical::to_wire::{BytesToWire, TimerCallback};

use crate::print_prosigns::PrintProsigns;
#[cfg(feature = "write")]
use crate::read_prosigns::ReadProsigns;

type Strategy = cfg_select! {
    feature = "base32" => encoding::Base32,
    feature = "discard" => encoding::Discard,
};

mod print_prosigns;
mod read_prosigns;

/// Baudrate of the serial. Set the `BAUDRATE` environment variable to control
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
    #[cfg(feature = "write")]
    let tx = pins.d13.into_output();
    let timer = dp.TC0;

    // Setup the clock
    #[cfg(feature = "write")]
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
    #[cfg(not(feature = "write"))]
    let (Stopwatch { reset, overflow }, clock_interrupts) = Stopwatch::new(timer);

    // Setup the pin to listen
    let (listening_pin, listening_pin_interrupts) =
        ListeningPin::new(external_interrupts, rx, ListeningPinCallback::new(reset));

    // Wire to serial
    let mut wire_to_serial = PipeToWriter::new(
        serial_writer,
        PrintProsigns::new(BytesFromWire::<Strategy>::new(listening_pin, overflow)),
    );

    // Serial to wire
    #[cfg(feature = "write")]
    let mut serial_to_wire =
        BytesToWire::<_, Strategy, _>::new(ReadProsigns::new(serial_reader), timer);
    #[cfg(not(feature = "write"))]
    let _ = serial_reader;

    unsafe {
        // Safety: interrupts are off at the start
        INTERRUPTS.init((clock_interrupts, listening_pin_interrupts));
        avr_device::interrupt::enable()
    };

    loop {
        match wire_to_serial.poll() {
            Err(nb::Error::WouldBlock) => (),
        }
        #[cfg(feature = "write")]
        match serial_to_wire.poll() {
            Err(nb::Error::WouldBlock) => (),
        }
    }
}

#[cfg(feature = "write")]
type ClockInterrupts = time::Interrupts<TimerCallback<Pin<Output, D13>>>;
#[cfg(not(feature = "write"))]
type ClockInterrupts = time::stopwatch::Interrupts;

static INTERRUPTS: InterruptStorage<(
    ClockInterrupts,
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
