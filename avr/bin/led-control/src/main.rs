#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]

use core::convert::Infallible;

use concurrency::{
    process::ProcessExt,
    slots::{Slot, Slots},
    stream::{Stream, StreamExt},
};
use envparse::parse_env;
use hal::{
    led::DebugLed,
    serial::{SerialRead, SerialReader},
    Peripherals, Serial,
};
use serial::WriteSerial;

const BUFFER_SIZE: usize = parse_env!("BUFFER_SIZE" as usize else 128);

/// Interpret single byte commands to control the led, answering with the state
/// after
struct LedControl {
    reader: SerialReader,
    led: DebugLed,
}

impl Stream for LedControl {
    type Item = u8;
    type Err = Infallible;

    fn fill<S>(&mut self, slots: &mut S) -> nb::Result<(), Self::Err>
    where
        S: Slots<Item = Self::Item>,
    {
        while let Some(slot) = slots.slot() {
            let command = self.reader.read()?;

            let state = match command {
                // \0 - turn off
                0x00 => {
                    self.led.off();
                    false
                }
                // \1 - turn on
                0x01 => {
                    self.led.on();
                    true
                }
                // \2 - toggle
                0x02 => {
                    let state = !self.led.state();
                    if state {
                        self.led.on();
                    } else {
                        self.led.off();
                    }
                    state
                }
                // \4 and any unknown command are no-ops
                _ => self.led.state(),
            };

            slot.fill(state as u8);
        }
        Ok(())
    }
}

#[hal::entry]
fn main() -> ! {
    let Peripherals {
        serial: Serial { reader, writer },
        wires: (_, wire),
    } = Peripherals::new().unwrap();
    let led = wire.downgrade_to_led();

    LedControl { reader, led }
        .buffer::<BUFFER_SIZE, usize>()
        .sink_into(WriteSerial::new(writer))
        .handle_errors(|err| match err {})
        .block_on()
}
