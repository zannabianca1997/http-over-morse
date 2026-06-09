//! Implementations on the `sync` accessors

use core::convert::Infallible;

use ringbuf::sync::{ReadRegion, Reader, Synchronization, WriteRegion, WriteRegionSlot, Writer};

use crate::{
    ready::Ready,
    sink::BufferSink,
    slots::{Slot, Slots},
    stream::BufferStream,
};

impl<T, const CAP: usize, BS, S> BufferStream for Reader<'_, T, CAP, BS, S>
where
    BS: Synchronization,
    S: Synchronization<Usize = BS::Usize>,
{
    type Item = T;
    type Err = Infallible;
    type Ready<'s> = ReadRegion<'s, T, S>;

    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R,
    {
        Ok(self.read(fun))
    }
}

impl<T, S> Ready for ReadRegion<'_, T, S>
where
    S: Synchronization,
{
    type Item = T;

    fn pop(&mut self) -> Option<Self::Item> {
        self.pop()
    }

    fn peek(&self) -> Option<&Self::Item> {
        self.as_slice().first()
    }
}

impl<T, const CAP: usize, BS, S> BufferSink for Writer<'_, T, CAP, BS, S>
where
    BS: Synchronization,
    S: Synchronization<Usize = BS::Usize>,
{
    type Item = T;
    type Err = Infallible;
    type Slots<'s> = WriteRegion<'s, T, S>;

    fn with_slots<R>(
        &mut self,
        fun: impl FnOnce(&mut Self::Slots<'_>) -> R,
    ) -> Result<R, Self::Err> {
        Ok(self.write(fun))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        if self.available() < CAP {
            // We cannot do much, just wait for buffer empty
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

impl<'b, T, S> Slots for WriteRegion<'b, T, S>
where
    S: Synchronization,
{
    type Item = T;

    type Slot<'r>
        = WriteRegionSlot<'r, T>
    where
        Self: 'r;

    fn slot(&mut self) -> Option<Self::Slot<'_>> {
        WriteRegion::slot(self)
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'r, T> Slot for WriteRegionSlot<'r, T> {
    type Item = T;

    fn fill(self, item: Self::Item) {
        WriteRegionSlot::fill(self, item);
    }
}
