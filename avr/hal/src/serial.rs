use arduino_hal::{
    DefaultClock,
    hal::usart::Usart0,
    pac::USART0,
    port::{
        D0, D1, Pin,
        mode::{Input, Output},
    },
    usart::{Baudrate, UsartReader, UsartWriter},
};

pub use arduino_hal::prelude::{
    _embedded_hal_serial_Read as SerialRead, _embedded_hal_serial_Write as SerialWrite,
};

use crate::pins::{Pin0, Pin1};

pub type SerialWriter = UsartWriter<USART0, Pin<Input, D0>, Pin<Output, D1>>;
pub type SerialReader = UsartReader<USART0, Pin<Input, D0>, Pin<Output, D1>>;

pub struct Serial {
    pub reader: SerialReader,
    pub writer: SerialWriter,
}
impl Serial {
    pub(crate) fn new(usart0: USART0, pin_0: Pin0, pin_1: Pin1) -> Self {
        let (reader, writer) =
            Usart0::new(usart0, pin_0, pin_1, Baudrate::<DefaultClock>::new(9600)).split();

        Self { reader, writer }
    }
}
