//! Write bytes from a process to the serial interface

use arduino_hal::prelude::_embedded_hal_serial_Write;

use crate::process::Process;

pub struct BytesToSerial<Inner, Serial> {
    serial: Serial,
    inner: Inner,
    writing: Option<u8>,
}

impl<Inner, Serial> BytesToSerial<Inner, Serial> {
    pub fn new(serial: Serial, inner: Inner) -> Self {
        Self {
            serial,
            inner,
            writing: None,
        }
    }
}

impl<Inner, Serial> Process for BytesToSerial<Inner, Serial>
where
    Inner: Process<Ok = u8>,
    Serial: _embedded_hal_serial_Write<u8>,
{
    type Ok = ();
    type Err = Error<Inner::Err, Serial::Error>;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            // Ensure the writing char is empty
            while let Some(writing) = self.writing {
                match self.serial.write(writing) {
                    Ok(()) => self.writing = None,
                    Err(err) => return Err(err.map(Error::Serial)),
                }
            }
            // Get a new char to write
            match self.inner.poll() {
                Ok(ch) => self.writing = Some(ch),
                Err(err) => return Err(err.map(Error::Inner)),
            }
        }
    }
}

pub enum Error<I, S> {
    Inner(I),
    Serial(S),
}
