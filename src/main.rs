#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use http_over_morse::{App, AppInterrupts};
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let wire = (pins.d2, pins.d13.into_output());
    let timer = dp.TC0;

    let (mut app, AppInterrupts {}) = App::new(serial, wire, timer);

    unsafe { avr_device::interrupt::enable() };

    app.main_loop()
}
