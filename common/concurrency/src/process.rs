use core::convert::Infallible;

use crate::process::handle_errors::HandleErrors;

/// A ongoing process
pub trait Process {
    /// Error in this process
    type Err;

    /// Execute work until it would block or an error happens
    fn run(&mut self) -> nb::Error<Self::Err>;
}

pub mod handle_errors;
pub mod idle;
pub mod join;
pub mod pipe;

impl<T: Process> ProcessExt for T {}
pub trait ProcessExt: Process + Sized {
    /// Block on this process forever
    fn block_on(mut self) -> !
    where
        Self: Process<Err = Infallible>,
    {
        loop {
            let nb::Error::WouldBlock = self.run();
        }
    }

    /// Handle the errors incoming
    fn handle_errors<H>(self, handler: H) -> HandleErrors<Self, H>
    where
        H: FnMut(Self::Err),
    {
        HandleErrors {
            process: self,
            handler,
        }
    }
}
