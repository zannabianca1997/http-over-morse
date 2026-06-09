#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]

use concurrency::{process::ProcessExt, stream::StreamExt};
use hal::{Peripherals, Serial};
use serial::{ReadSerial, WriteSerial};

#[hal::entry]
fn main() -> ! {
    let Peripherals {
        serial: Serial { reader, writer },
    } = unsafe {
        // Safety: main just started
        Peripherals::new_unchecked()
    };

    ReadSerial::new(reader)
        .buffer::<128>()
        .sink_into(WriteSerial::new(writer))
        .handle_errors(|err| match err {})
        .block_on()
}
