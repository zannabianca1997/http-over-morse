use core::convert::Infallible;

use ringbuf::{
    Buffer, ReadRegion, Reader, WriteRegion, WriteRegionSlot, Writer,
    sync::{DefaultSynchronization, MaybeCriticalSection, Synchronization},
};

use crate::{
    ready::Ready,
    sink::BufferSink,
    slots::{Slot, Slots},
    stream::{BufferStream, Stream},
};

pub struct Buffered<Inner, T, const CAP: usize, Sync: Synchronization = DefaultSynchronization> {
    inner: Inner,
    buffer: Buffer<T, CAP, Sync>,
}

impl<Inner, T, const CAP: usize, Sync: Synchronization> Buffered<Inner, T, CAP, Sync> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            buffer: Buffer::new(),
        }
    }
}

impl<Inner, const CAP: usize, Sync: Synchronization> BufferStream
    for Buffered<Inner, Inner::Item, CAP, Sync>
where
    Inner: Stream,
{
    type Item = Inner::Item;
    type Err = Inner::Err;
    type Ready<'s> = ReadRegion<'s, Inner::Item, Sync>;

    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R,
    {
        let (mut writer, mut reader) = self.buffer.split();

        // First fill itself from the inner, don't care if it manages or not to fill it
        match writer.write(|region| self.inner.fill(region)) {
            Ok(()) | Err(nb::Error::WouldBlock) => (),
            Err(nb::Error::Other(err)) => return Err(err),
        };

        Ok(reader.read(|region| fun(region)))
    }
}

// == Implementations on the `ringbuf` types ==

impl<T, const CAP: usize, S, CS> BufferStream for Reader<'_, T, CAP, S, CS>
where
    S: Synchronization,
    CS: MaybeCriticalSection<S>,
{
    type Item = T;
    type Err = Infallible;
    type Ready<'s> = ReadRegion<'s, T, S, CS>;

    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R,
    {
        Ok(self.read(fun))
    }
}

impl<T, S, CS> Ready for ReadRegion<'_, T, S, CS>
where
    S: Synchronization,
    CS: MaybeCriticalSection<S>,
{
    type Item = T;

    fn pop(&mut self) -> Option<Self::Item> {
        self.pop()
    }

    fn peek(&self) -> Option<&Self::Item> {
        self.as_slice().first()
    }
}

impl<T, const CAP: usize, S, CS> BufferSink for Writer<'_, T, CAP, S, CS>
where
    S: Synchronization,
    CS: MaybeCriticalSection<S>,
{
    type Item = T;
    type Err = Infallible;
    type Slots<'s> = WriteRegion<'s, T, S, CS>;

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

impl<'b, T, S, CS> Slots for WriteRegion<'b, T, S, CS>
where
    S: Synchronization,
    CS: MaybeCriticalSection<S>,
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
