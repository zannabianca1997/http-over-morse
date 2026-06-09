use core::convert::Infallible;

use crate::{handle_errors::HandleErrors, process::join::Join};

/// A ongoing process
pub trait Process {
    /// Error in this process
    type Err;

    /// Execute work until it would block or an error happens
    fn run(&mut self) -> nb::Error<Self::Err>;
}

pub mod idle;
pub mod join;
pub mod pipe;

impl<T: Process> ProcessExt for T {}
pub trait ProcessExt: Process {
    /// Block on this process forever
    fn block_on(mut self) -> !
    where
        Self: Process<Err = Infallible>,
        Self: Sized,
    {
        loop {
            let nb::Error::WouldBlock = self.run();
        }
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

    /// Merge this and another process in a single one that runs both
    /// concurrently
    fn join<Other>(self, other: Other) -> Join<Self, Other>
    where
        Self: Sized,
        Other: Process,
    {
        Join(self, other)
    }
}
