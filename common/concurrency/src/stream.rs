//! Streams of items
//!
//! Basis of pull-based concurrency

use crate::{process::pipe::Pipe, sink::Sink};

/// A stream of items
pub trait Stream {
    /// Items produced
    type Item;

    /// Error in this stream
    type Err;

    /// Produced items
    type Ready<'s>: Ready<'s, Self>
    where
        Self: 's;

    /// Get a non blocking list of ready items
    fn ready(&mut self) -> nb::Result<Self::Ready<'_>, Self::Err>;
}

pub trait Ready<'s, S>
where
    S: Stream + ?Sized + 's,
{
    /// Get the next item
    fn next(&mut self) -> Option<S::Item>;
}

pub mod empty;

impl<T: Stream> StreamExt for T {}
pub trait StreamExt: Stream + Sized {
    /// Sink this stream
    fn sink<S>(self, sink: S) -> Pipe<Self, S>
    where
        S: Sink<Item = Self::Item>,
    {
        Pipe { stream: self, sink }
    }
}
