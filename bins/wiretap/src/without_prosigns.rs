#![cfg(feature = "write")]

use either::{Either, Left};
use morse::Prosign;
use process::Process;

pub struct WithoutProsigns<Inner> {
    inner: Inner,
}

impl<Inner> WithoutProsigns<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }
}

impl<Inner> Process for WithoutProsigns<Inner>
where
    Inner: Process,
{
    type Ok = Either<Inner::Ok, Prosign>;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.inner.poll().map(Left)
    }
}
