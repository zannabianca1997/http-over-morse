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
    } = Peripherals::new().unwrap();

    ReadSerial::new(reader)
        .sink(WriteSerial::new(writer))
        .handle_errors(|err| match err {})
        .block_on()
}
