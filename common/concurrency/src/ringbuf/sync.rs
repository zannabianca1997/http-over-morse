//! Implementations on the `sync` accessors

use core::convert::Infallible;

use ringbuf::sync::{
    ReadRegion, Reader, SyncIndex, Synchronization, WriteRegion, WriteRegionSlot, Writer,
};

use crate::{
    ready::Ready,
    sink::BufferSink,
    slots::{Slot, Slots},
    stream::BufferStream,
};

impl<T, const CAP: usize, Idx: SyncIndex, Sync: Synchronization<Idx>> BufferStream
    for Reader<'_, T, CAP, Idx, Sync>
{
    type Item = T;
    type Err = Infallible;
    type Ready<'s> = ReadRegion<'s, T, Idx, Sync>;

    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R,
    {
        Ok(self.read(fun))
    }
}

impl<T, Idx: SyncIndex, Sync: Synchronization<Idx>> Ready for ReadRegion<'_, T, Idx, Sync> {
    type Item = T;

    fn pop(&mut self) -> Option<Self::Item> {
        self.pop()
    }

    fn peek(&self) -> Option<&Self::Item> {
        self.as_slice().first()
    }
}

impl<T, const CAP: usize, Idx: SyncIndex, Sync: Synchronization<Idx>> BufferSink
    for Writer<'_, T, CAP, Idx, Sync>
{
    type Item = T;
    type Err = Infallible;
    type Slots<'s> = WriteRegion<'s, T, Idx, Sync>;

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

impl<'b, T, Idx: SyncIndex, Sync: Synchronization<Idx>> Slots for WriteRegion<'b, T, Idx, Sync> {
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
