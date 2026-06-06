//! Handle confirmation prosigns
//!
//! In the future this will send back the confirmation signal for a well
//! received frame

use either::{Either, Left, Right};
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
    Inner: Process<Ok = Either<T, Prosign>>,
{
    type Ok = Either<T, FramingProsigns>;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            match self.inner.poll()? {
                Either::Left(t) => return Ok(Left(t)),
                Either::Right(prosign) => match FramingProsigns::try_from(prosign) {
                    Ok(framing_prosing) => return Ok(Right(framing_prosing)),
                    Err(confirmation_prosign) => {
                        // TODO: handle confirmation prosigns

                        let _ = confirmation_prosign;
                    }
                },
            }
        }
    }
}
