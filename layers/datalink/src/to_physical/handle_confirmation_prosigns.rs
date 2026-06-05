//! Handle confirmation prosigns
//!
//! In the future this will receive the confirmation signal for a well received frame

use either::Either;
use morse::Prosign;
use process::Process;

use crate::events::FramingProsigns;

pub struct HandleConfirmationProsigns<Inner> {
    inner: Inner,
}

impl<Inner> HandleConfirmationProsigns<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }
}

impl<T, Inner> Process for HandleConfirmationProsigns<Inner>
where
    Inner: Process<Ok = Either<T, FramingProsigns>>,
{
    type Ok = Either<T, Prosign>;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.inner.poll().map(|r| r.map_right(Prosign::from))
    }
}
