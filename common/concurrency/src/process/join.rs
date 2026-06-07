//! Run multiple processes at once

use either::Either::{self, Left, Right};

use super::Process;

#[derive(Debug, Clone, Copy)]
pub struct Join<A, B>(pub A, pub B);

impl<A, B> Process for Join<A, B>
where
    A: Process,
    B: Process,
{
    type Err = Either<A::Err, B::Err>;

    fn run(&mut self) -> nb::Error<Self::Err> {
        // we do not care about fairness as the errors are considered sparse
        // this also make so `Join` is associative!

        match self.0.run() {
            nb::Error::WouldBlock => (),
            nb::Error::Other(err) => return nb::Error::Other(Left(err)),
        };

        match self.1.run() {
            nb::Error::WouldBlock => (),
            nb::Error::Other(err) => return nb::Error::Other(Right(err)),
        }

        nb::Error::WouldBlock
    }
}
