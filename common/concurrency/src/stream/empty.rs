//! Empty stream

use core::{convert::Infallible, marker::PhantomData};

use super::Stream;

/// Empty stream
pub struct Empty<I>(PhantomData<I>);

impl<I> Empty<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Stream for Empty<I> {
    type Item = I;
    type Err = Infallible;

    type Ready = NoItems;

    fn ready(&mut self) -> nb::Result<Self::Ready, Self::Err> {
        nb::Result::Err(nb::Error::WouldBlock)
    }

    fn stream(&mut self, ready: &mut Self::Ready) -> Result<Option<Self::Item>, Self::Err> {
        match *ready {}
    }
}

/// There are never ready items
pub enum NoItems {}

impl<I> Default for Empty<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> Clone for Empty<I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<I> Copy for Empty<I> {}
