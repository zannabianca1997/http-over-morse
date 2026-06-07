//! Infinite sink

use core::{convert::Infallible, marker::PhantomData};

use super::Sink;

/// Infinite sink
pub struct Bottomless<I>(PhantomData<I>);

impl<I> Bottomless<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Sink for Bottomless<I> {
    type Item = I;
    type Err = Infallible;

    type Slots = ();
    type Slot = ();

    fn slots(&mut self) -> nb::Result<Self::Slots, Self::Err> {
        Ok(())
    }

    fn slot(&mut self, (): &mut Self::Slots) -> Result<Option<Self::Slot>, Self::Err> {
        Ok(Some(()))
    }

    fn sink(&mut self, (): Self::Slot, item: I) {
        drop(item)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        Ok(())
    }
}

impl<I> Default for Bottomless<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> Clone for Bottomless<I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<I> Copy for Bottomless<I> {}
