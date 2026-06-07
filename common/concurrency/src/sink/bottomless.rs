//! Infinite sink

use core::{convert::Infallible, marker::PhantomData};

use super::{Sink, Slot, Slots};

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

    type Slots<'s>
        = InfiniteSlots<'s, I>
    where
        Self: 's;

    fn slots(&mut self) -> nb::Result<Self::Slots<'_>, Self::Err> {
        Ok(InfiniteSlots(PhantomData))
    }
}

/// Unlimited slots in a infinite sink
pub struct InfiniteSlots<'s, I>(PhantomData<&'s mut Bottomless<I>>);

impl<'s, I> Slots<'s, Bottomless<I>> for InfiniteSlots<'s, I> {
    type Slot<'a>
        = BottomlessSlot<'a, I>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Slot<'_>> {
        Some(BottomlessSlot(PhantomData))
    }
}

/// Slot in a bottomless sink
pub struct BottomlessSlot<'s, I>(PhantomData<&'s mut Bottomless<I>>);

impl<'s, I> Slot<'s, Bottomless<I>> for BottomlessSlot<'s, I> {
    fn sink(self, item: <Bottomless<I> as Sink>::Item) {
        drop(item)
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
