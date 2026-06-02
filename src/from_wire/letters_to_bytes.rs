//! Read bytes from a letter source

use encoding::{Error, Strategy, ToBytes};
use morse::LetterOrSpace;
use process::Process;

pub struct LettersToBytes<Inner, S: Strategy> {
    inner: Inner,
    data: S::ToBytes,
}

impl<Inner, S: Strategy> LettersToBytes<Inner, S> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            data: S::ToBytes::new(),
        }
    }
}

impl<Inner, S> Process for LettersToBytes<Inner, S>
where
    Inner: Process<Ok = LetterOrSpace>,
    S: Strategy,
{
    type Ok = u8;
    type Err = Error<Inner::Err, <S::ToBytes as ToBytes>::Error>;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.data.process(|| self.inner.poll())
    }
}
