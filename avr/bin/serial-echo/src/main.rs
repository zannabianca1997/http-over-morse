#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]

use concurrency::{process::ProcessExt, stream::StreamExt};
use hal::{Peripherals, Serial};
use serial::{ReadSerial, WriteSerial};

const BUFFER_SIZE: usize = parse_env!("BUFFER_SIZE" as usize in 1..);

#[hal::entry]
fn main() -> ! {
    let Peripherals {
        serial: Serial { reader, writer },
    } = Peripherals::new().unwrap();

    ReadSerial::new(reader)
        .buffer::<BUFFER_SIZE, usize>()
        .sink_into(WriteSerial::new(writer))
        .handle_errors(|err| match err {})
        .block_on()
}
