//! Read bytes from a letter source

use process::Process;

use crate::{
    encoding::LetterOrSpace,
    strategies::{Strategy, ToBytes},
};

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
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.data.process(|| self.inner.poll())
    }
}
