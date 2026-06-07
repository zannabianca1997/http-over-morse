//! Process that's not doing anything

use core::convert::Infallible;

use super::Process;

/// Idle process
#[derive(Debug, Clone, Copy, Default)]
pub struct Idle;

impl Idle {
    pub fn new() -> Self {
        Idle
    }
}

impl Process for Idle {
    type Err = Infallible;

    fn run(&mut self) -> nb::Error<Self::Err> {
        nb::Error::WouldBlock
    }
}
