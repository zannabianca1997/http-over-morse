#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]

use core::convert::Infallible;

use concurrency::{
    process::ProcessExt,
    ready::Ready,
    slots::{Slot, Slots},
    stream::{BufferStream, Stream, StreamExt},
};
use envparse::parse_env;
use hal::{Peripherals, Serial, led::DebugLed};
use serial::{ReadSerial, WriteSerial};

const BUFFER_SIZE: usize = parse_env!("BUFFER_SIZE" as usize else 128);

/// Interpret single byte commands read from the input stream to control the
/// led, answering with the state after
struct LedControl<S> {
    input: S,
    led: DebugLed,
}

struct UnknownCommand(u8);

impl<S> Stream for LedControl<S>
where
    S: BufferStream<Item = u8, Err = Infallible>,
{
    type Item = u8;
    type Err = UnknownCommand;

    fn fill<T>(&mut self, slots: &mut T) -> nb::Result<(), Self::Err>
    where
        T: Slots<Item = Self::Item>,
    {
        self.input
            .with_ready(|ready| {
                let mut work_done = false;

                while let Some(slot) = slots.slot()
                    && let Some(command) = ready.pop()
                {
                    let mut state = None;
                    match command {
                        // \1 - turn off
                        0x01 => {
                            self.led.off();
                            state = Some(false);
                        }
                        // \2 - turn on
                        0x02 => {
                            self.led.on();
                            state = Some(true);
                        }
                        // \3 - toggle
                        0x03 => {
                            let new_state = !self.led.state();
                            if new_state {
                                self.led.on();
                            } else {
                                self.led.off();
                            }
                            state = Some(new_state);
                        }
                        // \4 - no-op, effectively a read
                        0x04 => state = Some(self.led.state()),
                        // unknown command: ignore it silently. Line noise reads
                        // as `\0`, and answering it would only feed the storm
                        _ => {}
                    };

                    if let Some(state) = state {
                        work_done = true;
                        // the state is encoded as `\1` when off and `\2` when
                        // on: `\0` is never a valid answer, so line noise is
                        // always distinguishable from one
                        slot.fill(1 + state as u8);
                    }
                }

                if work_done {
                    Ok(())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            })
            .unwrap_or_else(|err| match err {})
    }
}

#[hal::entry]
fn main() -> ! {
    let Peripherals {
        serial: Serial { reader, writer },
        debug_led: led,
        ..
    } = unsafe {
        // SAFETY: main just started
        Peripherals::new_unchecked()
    };

    LedControl {
        input: ReadSerial::new(reader).buffer::<BUFFER_SIZE, usize>(),
        led,
    }
    .buffer::<BUFFER_SIZE, usize>()
    .sink_into(WriteSerial::new(writer))
    .handle_errors(|err| {
        let UnknownCommand(_command) = err.left_or_else(|err| match err {});
        // can't do much
    })
    .block_on()
}
