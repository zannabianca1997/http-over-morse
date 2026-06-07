use core::convert::Infallible;

use crate::{process::Process, sink::Sink, stream::Stream};

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

impl<I, H> Stream for HandleErrors<I, H>
where
    I: Stream,
    H: FnMut(I::Err),
{
    type Err = Infallible;
    type Item = I::Item;
    type Ready = I::Ready;

    fn ready(&mut self) -> nb::Result<Self::Ready, Self::Err> {
        loop {
            match self.inner.ready() {
                Ok(r) => return Ok(r),
                Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
                Err(nb::Error::Other(err)) => (self.handler)(err),
            }
        }
    }

    fn stream(&mut self, ready: &mut Self::Ready) -> Result<Option<Self::Item>, Self::Err> {
        loop {
            match self.inner.stream(ready) {
                Ok(r) => return Ok(r),
                Err(err) => (self.handler)(err),
            }
        }
    }
}

impl<I, H> Sink for HandleErrors<I, H>
where
    I: Sink,
    H: FnMut(I::Err),
{
    type Err = Infallible;

    type Item = I::Item;

    type Slots = I::Slots;

    type Slot = I::Slot;

    fn slots(&mut self) -> nb::Result<Self::Slots, Self::Err> {
        loop {
            match self.inner.slots() {
                Ok(r) => return Ok(r),
                Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
                Err(nb::Error::Other(err)) => (self.handler)(err),
            }
        }
    }

    fn slot(&mut self, slots: &mut Self::Slots) -> Result<Option<Self::Slot>, Self::Err> {
        loop {
            match self.inner.slot(slots) {
                Ok(r) => return Ok(r),
                Err(err) => (self.handler)(err),
            }
        }
    }

    fn sink(&mut self, slot: Self::Slot, item: Self::Item) {
        self.inner.sink(slot, item)
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
