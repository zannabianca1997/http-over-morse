//! Streams of items
//!
//! Basis of pull-based concurrency

use crate::{
    handle_errors::HandleErrors,
    process::pipe::Pipe,
    ready::Ready,
    ringbuf::Buffered,
    sink::{BufferSink, Sink},
    slots::{Slot, Slots},
};

/// A stream of items
pub trait Stream {
    /// Items produced
    type Item;
    /// Error in this stream
    type Err;

    /// Fill the given slots until either full, or would block
    ///
    /// Returns [`Ok`] if the slot ended before a block.
    fn fill<S>(&mut self, slots: &mut S) -> nb::Result<(), Self::Err>
    where
        S: Slots<Item = Self::Item>;
}

/// A stream of items that can provide a view in its items
pub trait BufferStream {
    /// Items produced
    type Item;
    /// Error in this stream
    type Err;
    /// View in the ready items
    type Ready<'s>: Ready<Item = Self::Item>;

    /// Get all the items that are ready without blocking
    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R;
}

impl<T> Stream for T
where
    T: BufferStream,
{
    type Item = T::Item;
    type Err = T::Err;

    fn fill<S>(&mut self, slots: &mut S) -> nb::Result<(), Self::Err>
    where
        S: Slots<Item = Self::Item>,
    {
        self.with_ready(|items| {
            while let Some(slot) = slots.slot()
                && let Some(item) = items.pop()
            {
                slot.fill(item);
            }
        })?;
        if slots.is_empty() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

pub mod empty;

impl<T: Stream> StreamExt for T {}
pub trait StreamExt: Stream {
    /// Sink this stream
    fn sink_into_buffered<S>(self, sink: S) -> Pipe<Self, S, false>
    where
        Self: Sized,
        S: BufferSink<Item = Self::Item>,
    {
        Pipe::new(self, sink)
    }

    /// Sink this buffered stream
    fn sink_into<S>(self, sink: S) -> Pipe<Self, S, true>
    where
        Self: Sized + BufferStream,
        S: Sink<Item = <Self as Stream>::Item>,
    {
        Pipe::new(self, sink)
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

    /// Add a buffer in front of this stream
    ///
    /// The buffer will be eagerly filled until blocking, then items will be
    /// streamed from the buffer.
    fn buffer<const CAP: usize>(self) -> Buffered<Self, Self::Item, CAP>
    where
        Self: Sized,
    {
        Buffered::new(self)
    }
}
