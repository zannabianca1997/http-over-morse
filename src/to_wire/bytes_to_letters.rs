//! Convert bytes into morse letters

use encoding::{Error, Strategy, ToLetters};
use morse::LetterOrSpace;
use process::Process;

/// Read morse letters from a byte source
pub struct BytesToLetters<Inner, S: Strategy> {
    inner: Inner,
    data: S::ToLetters,
}

impl<Inner, S: Strategy> BytesToLetters<Inner, S> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            data: S::ToLetters::new(),
        }
    }
}

impl<Inner, S> Process for BytesToLetters<Inner, S>
where
    Inner: Process<Ok = u8>,
    S: Strategy,
{
    type Ok = LetterOrSpace;
    type Err = Error<Inner::Err, <S::ToLetters as ToLetters>::Error>;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.data.process(|| self.inner.poll())
    }
}
