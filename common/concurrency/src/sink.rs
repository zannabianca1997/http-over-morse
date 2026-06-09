//! Sink of items
//!
//! Basis of push-based concurrency

use crate::{
    handle_errors::HandleErrors,
    process::pipe::Pipe,
    ready::Ready,
    slots::{Slot, Slots},
    stream::{BufferStream, Stream},
};

/// A sink of items
pub trait Sink {
    /// Items sunk
    type Item;
    /// Error in this stream
    type Err;

    /// Sink all item until blocks
    fn sink<R>(&mut self, items: &mut R) -> nb::Result<(), Self::Err>
    where
        R: Ready<Item = Self::Item>;

    /// Ensure all the item have been sunk
    fn flush(&mut self) -> nb::Result<(), Self::Err>;
}

pub trait BufferSink {
    /// Items sunk
    type Item;
    /// Error in this stream
    type Err;
    /// View into the buffer
    type Slots<'s>: Slots<Item = Self::Item>;

    /// Slots that can be sunk without blocking
    fn with_slots<R>(
        &mut self,
        fun: impl FnOnce(&mut Self::Slots<'_>) -> R,
    ) -> Result<R, Self::Err>;

    /// Ensure all the item have been sunk
    fn flush(&mut self) -> nb::Result<(), Self::Err>;
}

impl<T> Sink for T
where
    T: BufferSink,
{
    type Item = T::Item;

    type Err = T::Err;

    /// Sink all item until blocks
    fn sink<R>(&mut self, items: &mut R) -> nb::Result<(), Self::Err>
    where
        R: Ready<Item = Self::Item>,
    {
        self.with_slots(|slots| {
            while let Some(slot) = slots.slot()
                && let Some(item) = items.pop()
            {
                slot.fill(item);
            }

            if slots.is_empty() {
                Err(nb::Error::WouldBlock)
            } else {
                Ok(())
            }
        })?
    }

    fn flush(&mut self) -> nb::Result<(), Self::Err> {
        BufferSink::flush(self)
    }
}

pub mod bottomless;

impl<T: Sink> SinkExt for T {}
pub trait SinkExt: Sink {
    /// Stream items into this sink
    fn stream_from_buffered<S>(self, stream: S) -> Pipe<S, Self, true>
    where
        Self: Sized,
        S: BufferStream<Item = Self::Item>,
    {
        Pipe::new(stream, self)
    }

    /// Stream items into this buffered sink
    fn stream<S>(self, stream: S) -> Pipe<S, Self, false>
    where
        Self: Sized + BufferSink,
        S: Stream<Item = <Self as Sink>::Item>,
    {
        Pipe::new(stream, self)
    }

    /// Handle the errors incoming
    fn handle_errors<H>(self, handler: H) -> HandleErrors<Self, H>
    where
        H: FnMut(Self::Err),
        Self: Sized,
    {
        HandleErrors {
            inner: self,
            handler,
        }
    }
}
