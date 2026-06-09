//! Infinite sink

use core::{convert::Infallible, marker::PhantomData};

use crate::{
    sink::BufferSink,
    slots::{Slot, Slots},
};

/// Infinite sink
pub struct Bottomless<I>(PhantomData<I>);

impl<I> Bottomless<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> BufferSink for Bottomless<I> {
    type Item = I;
    type Err = Infallible;
    type Slots<'s> = Self;

    fn with_slots<R>(
        &mut self,
        fun: impl FnOnce(&mut Self::Slots<'_>) -> R,
    ) -> Result<R, Self::Err> {
        Ok(fun(self))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        Ok(())
    }
}

impl<I> Slots for Bottomless<I> {
    type Item = I;

    type Slot<'s>
        = Self
    where
        Self: 's;

    fn slot(&mut self) -> Option<Self::Slot<'_>> {
        Some(*self)
    }

    fn is_empty(&self) -> bool {
        false
    }
}

impl<I> Slot for Bottomless<I> {
    type Item = I;

    fn fill(self, item: Self::Item) {
        drop(item);
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
