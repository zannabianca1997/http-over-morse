//! Read bytes from a letter source

use crate::{
    encoding::LetterOrSpace,
    process::Process,
    strategies::{Discard, Strategy},
};

pub struct LettersToBytes<Inner, S: Strategy> {
    inner: Inner,
    data: S::WriteData,
}

impl<Inner, S: Strategy> LettersToBytes<Inner, S> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            data: S::new_write(),
        }
    }
}

impl<Inner> Process for LettersToBytes<Inner, Discard>
where
    Inner: Process<Ok = LetterOrSpace>,
{
    type Ok = u8;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.inner.poll().map(LetterOrSpace::char)
    }
}
