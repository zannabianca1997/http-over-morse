use core::convert::Infallible;

use super::Process;

pub struct HandleErrors<P, H> {
    pub process: P,
    pub handler: H,
}

impl<P, H> Process for HandleErrors<P, H>
where
    P: Process,
    H: FnMut(P::Err),
{
    type Err = Infallible;

    fn run(&mut self) -> nb::Error<Self::Err> {
        loop {
            match self.process.run() {
                nb::Error::Other(err) => (self.handler)(err),
                nb::Error::WouldBlock => return nb::Error::WouldBlock,
            }
        }
    }
}
