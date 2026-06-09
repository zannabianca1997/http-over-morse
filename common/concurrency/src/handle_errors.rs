use core::convert::Infallible;

use crate::{process::Process, ready::Ready, sink::Sink, slots::Slots, stream::Stream};

pub struct HandleErrors<I, H> {
    pub inner: I,
    pub handler: H,
}

impl<P, H> Process for HandleErrors<P, H>
where
    P: Process,
    H: FnMut(P::Err),
{
    type Err = Infallible;

    fn run(&mut self) -> nb::Error<Self::Err> {
        loop {
            match self.inner.run() {
                nb::Error::Other(err) => (self.handler)(err),
                nb::Error::WouldBlock => return nb::Error::WouldBlock,
            }
        }
    }
}

impl<P, H> Stream for HandleErrors<P, H>
where
    P: Stream,
    H: FnMut(P::Err),
{
    type Err = Infallible;

    type Item = P::Item;

    fn fill<S>(&mut self, slots: &mut S) -> nb::Result<(), Self::Err>
    where
        S: Slots<Item = Self::Item>,
    {
        loop {
            match self.inner.fill(slots) {
                Ok(()) => return Ok(()),
                Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
                Err(nb::Error::Other(err)) => (self.handler)(err),
            }
        }
    }
}

impl<P, H> Sink for HandleErrors<P, H>
where
    P: Sink,
    H: FnMut(P::Err),
{
    type Err = Infallible;

    type Item = P::Item;

    fn sink<R>(&mut self, items: &mut R) -> nb::Result<(), Self::Err>
    where
        R: Ready<Item = Self::Item>,
    {
        loop {
            match self.inner.sink(items) {
                Ok(()) => return Ok(()),
                Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
                Err(nb::Error::Other(err)) => (self.handler)(err),
            }
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        loop {
            match self.inner.flush() {
                Ok(()) => return Ok(()),
                Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
                Err(nb::Error::Other(err)) => (self.handler)(err),
            }
        }
    }
}
