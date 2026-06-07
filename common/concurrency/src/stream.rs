//! Streams of items
//!
//! Basis of pull-based concurrency

use crate::{handle_errors::HandleErrors, process::pipe::Pipe, sink::Sink};

/// A stream of items
pub trait Stream {
    /// Items produced
    type Item;

    /// Error in this stream
    type Err;

    /// Items that can be produced without blocking
    type Ready;

    /// Get a non blocking list of ready items
    fn ready(&mut self) -> nb::Result<Self::Ready, Self::Err>;

    /// Get the next ready item
    fn stream(&mut self, ready: &mut Self::Ready) -> Result<Option<Self::Item>, Self::Err>;
}

pub mod empty;

impl<T: Stream> StreamExt for T {}
pub trait StreamExt: Stream {
    /// Sink this stream
    fn sink<S>(self, sink: S) -> Pipe<Self, S>
    where
        Self: Sized,
        S: Sink<Item = Self::Item>,
    {
        Pipe { stream: self, sink }
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
