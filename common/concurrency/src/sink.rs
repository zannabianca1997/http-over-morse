//! Sink of items
//!
//! Basis of push-based concurrency

use crate::{process::pipe::Pipe, stream::Stream};

/// A sink of items
pub trait Sink {
    /// Items sunk
    type Item;

    /// Error in this stream
    type Err;

    /// Non blocking writer of consumed items
    type Slots<'s>: Slots<'s, Self>
    where
        Self: 's;

    /// Get a non blocking writer of consumed items
    fn slots(&mut self) -> nb::Result<Self::Slots<'_>, Self::Err>;
}

/// Non blocking writer of consumed items
pub trait Slots<'s, S>
where
    S: Sink + ?Sized + 's,
{
    type Slot<'a>: Slot<'a, S>
    where
        Self: 'a,
        S: 'a;

    /// Get a slot if still available
    fn next(&mut self) -> Option<Self::Slot<'_>>;
}

/// Consumer of a single item
pub trait Slot<'s, S>
where
    S: Sink + ?Sized + 's,
{
    /// Get a slot if still available
    fn sink(self, item: S::Item);
}

pub mod bottomless;

impl<T: Sink> SinkExt for T {}
pub trait SinkExt: Sink + Sized {
    /// Stream items into this sink
    fn stream<S>(self, stream: S) -> Pipe<S, Self>
    where
        S: Stream<Item = Self::Item>,
    {
        Pipe { stream, sink: self }
    }
}
