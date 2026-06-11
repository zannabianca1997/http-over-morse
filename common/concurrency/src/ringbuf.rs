use ringbuf::{
    indices::Index,
    unsync::{Buffer, ReadRegion, WriteRegion},
};

use crate::{
    sink::{BufferSink, Sink},
    stream::{BufferStream, Stream},
};

mod sync;
mod unsync;

pub struct Buffered<Inner, T, const CAP: usize, Idx: Index> {
    inner: Inner,
    buffer: Buffer<T, CAP, Idx>,
}

impl<Inner, T, const CAP: usize, Idx: Index> Buffered<Inner, T, CAP, Idx> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            buffer: Buffer::new(),
        }
    }
}

impl<Inner, const CAP: usize, Idx: Index> BufferStream for Buffered<Inner, Inner::Item, CAP, Idx>
where
    Inner: Stream,
{
    type Item = Inner::Item;
    type Err = Inner::Err;
    type Ready<'s> = ReadRegion<'s, Inner::Item, Idx>;

    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R,
    {
        let mut writer = self.buffer.writer_mut();

        // First fill itself from the inner, until either buffer is full or inner would block
        while writer.available() > 0 {
            match writer.write(|region| self.inner.fill(region)) {
                Ok(()) => (),
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(err)) => return Err(err),
            }
        }

        Ok(self.buffer.reader_mut().read(|region| fun(region)))
    }
}

impl<Inner, const CAP: usize, Idx: Index> BufferSink for Buffered<Inner, Inner::Item, CAP, Idx>
where
    Inner: Sink,
{
    type Item = Inner::Item;
    type Err = Inner::Err;
    type Slots<'s> = WriteRegion<'s, Inner::Item, Idx>;

    fn with_slots<R>(
        &mut self,
        fun: impl FnOnce(&mut Self::Slots<'_>) -> R,
    ) -> Result<R, Self::Err> {
        let mut reader = self.buffer.reader_mut();

        // First sink all item available, until buffer is empty or inner would block
        while reader.available() > 0 {
            match reader.read(|region| self.inner.sink(region)) {
                Ok(()) => (),
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(err)) => return Err(err),
            }
        }

        Ok(self.buffer.writer_mut().write(|region| fun(region)))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        let mut reader = self.buffer.reader_mut();

        // Sink all item available, until buffer is empty or inner would block
        while reader.available() > 0 {
            reader.read(|region| self.inner.sink(region))?;
        }

        Ok(())
    }
}
