//! Reader can be seen as processes emitting bytes

use arduino_hal::prelude::_embedded_hal_serial_Read as Read;

use crate::Process;

impl<T> Process for T
where
    T: Read<u8>,
{
    type Ok = u8;
    type Err = T::Error;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.read()
    }
}
