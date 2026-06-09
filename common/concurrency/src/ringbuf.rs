use core::convert::Infallible;

use ringbuf::{
    Buffer, ReadRegion, Reader, WriteRegion, WriteRegionSlot, Writer,
    sync::{DefaultSynchronization, Synchronization},
};

use crate::{
    ready::Ready,
    sink::{BufferSink, Sink},
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

impl<Inner, const CAP: usize, Sync: Synchronization + Default> BufferStream
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

        // First fill itself from the inner, until either buffer is full or inner would block
        while writer.available() > 0 {
            match writer.write(|region| self.inner.fill(region)) {
                Ok(()) => (),
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(err)) => return Err(err),
            }
        }

        Ok(reader.read(|region| fun(region)))
    }
}

impl<Inner, const CAP: usize, Sync: Synchronization + Default> BufferSink
    for Buffered<Inner, Inner::Item, CAP, Sync>
where
    Inner: Sink,
{
    type Item = Inner::Item;
    type Err = Inner::Err;
    type Slots<'s> = WriteRegion<'s, Inner::Item, Sync>;

    fn with_slots<R>(
        &mut self,
        fun: impl FnOnce(&mut Self::Slots<'_>) -> R,
    ) -> Result<R, Self::Err> {
        let (mut writer, mut reader) = self.buffer.split();

        // First sink all item available, until buffer is empty or inner would block
        while reader.available() > 0 {
            match reader.read(|region| self.inner.sink(region)) {
                Ok(()) => (),
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(err)) => return Err(err),
            }
        }

        Ok(writer.write(|region| fun(region)))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        let (_, mut reader) = self.buffer.split();

        // Sink all item available, until buffer is empty or inner would block
        while reader.available() > 0 {
            reader.read(|region| self.inner.sink(region))?;
        }

        Ok(())
    }
}

// == Implementations on the `ringbuf` types ==

impl<T, const CAP: usize, BS, S> BufferStream for Reader<'_, T, CAP, BS, S>
where
    BS:Synchronization,
    S: Synchronization<Usize = BS::Usize>
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
    S: Synchronization
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
    BS:Synchronization,
    S: Synchronization<Usize = BS::Usize>
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
