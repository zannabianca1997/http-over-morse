//! Convert bytes into morse letters

use process::Process;

use crate::{
    encoding::LetterOrSpace,
    strategies::{Strategy, ToLetters},
};

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
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.data.process(|| self.inner.poll())
    }
}
