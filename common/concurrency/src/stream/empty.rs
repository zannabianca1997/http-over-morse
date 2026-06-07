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

    type Ready<'s>
        = NoneReady<'s, I>
    where
        Self: 's;

    fn ready(&mut self) -> nb::Result<Self::Ready<'_>, Self::Err> {
        nb::Result::Err(nb::Error::WouldBlock)
    }
}

/// No ready items
pub struct NoneReady<'s, I> {
    stream: PhantomData<&'s mut Empty<I>>,
    /// Ensure the compiler does not bother with creating this type
    uninhabited: Uninhabited,
}
enum Uninhabited {}

impl<'s, I> super::Ready<'s, Empty<I>> for NoneReady<'s, I> {
    fn next(&mut self) -> Option<I> {
        match self.uninhabited {}
    }
}

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
