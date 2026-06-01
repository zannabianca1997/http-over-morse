#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod buffer_output;
pub mod pipe_to_writer;
mod read_as_process;

/// A running process
pub trait Process {
    /// The type of item produced
    type Ok;

    /// Error type
    type Err;

    /// Do some work without blocking
    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err>;

    /// Block on this process
    fn block_on(&mut self) -> Result<Self::Ok, Self::Err> {
        loop {
            return match self.poll() {
                Ok(ok) => Ok(ok),
                Err(nb::Error::Other(err)) => Err(err),
                Err(nb::Error::WouldBlock) => continue,
            };
        }
    }
}

/// Uninhabited type for processes that does not produce anything
pub enum NoItemProduced {}
