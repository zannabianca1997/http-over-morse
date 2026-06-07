//! Sink of items
//!
//! Basis of push-based concurrency

use crate::{handle_errors::HandleErrors, process::pipe::Pipe, stream::Stream};

/// A sink of items
pub trait Sink {
    /// Items sunk
    type Item;

    /// Error in this stream
    type Err;

    /// Slots for items to be consumed without blocking
    type Slots;

    /// Slot for a single item to be consumed
    type Slot;

    /// Get a non blocking writer of consumed items
    fn slots(&mut self) -> nb::Result<Self::Slots, Self::Err>;

    /// Get the next slot, if available
    fn slot(&mut self, slots: &mut Self::Slots) -> Result<Option<Self::Slot>, Self::Err>;

    /// Get the next slot
    fn sink(&mut self, slot: Self::Slot, item: Self::Item);
}

pub mod bottomless;

impl<T: Sink> SinkExt for T {}
pub trait SinkExt: Sink {
    /// Stream items into this sink
    fn stream<S>(self, stream: S) -> Pipe<S, Self>
    where
        Self: Sized,
        S: Stream<Item = Self::Item>,
    {
        Pipe { stream, sink: self }
    }

    /// Handle the errors incoming
    fn handle_errors<H>(self, handler: H) -> HandleErrors<Self, H>
    where
        Self: Sized,
        H: FnMut(Self::Err),
    {
        HandleErrors {
            inner: self,
            handler,
        }
    }
}
