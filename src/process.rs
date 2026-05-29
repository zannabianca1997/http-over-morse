/// A running process
pub trait Process {
    /// The type of item produced
    type Ok;

    /// Error type
    type Err;

    /// Do some work without blocking
    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err>;
}

impl<T> Process for T
where
    T: arduino_hal::prelude::_embedded_hal_serial_Read<u8>,
{
    type Ok = u8;
    type Err = T::Error;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.read()
    }
}
