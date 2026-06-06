#![no_std]
#![no_main]

use panic_halt as _;
use ufmt::uwriteln;

use test_config::Config;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

    let config = Config::get();

    loop {
        uwriteln!(serial, "{:?}", config);
        arduino_hal::delay_ms(800);
    }
}
