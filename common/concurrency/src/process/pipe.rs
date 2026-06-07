//! Write all data from a stream to a sink

use either::{Either, Left, Right};

use crate::{sink::Sink, stream::Stream};

use super::Process;

/// Pipe from a sink to a stream
#[derive(Debug, Clone, Copy, Default)]
pub struct Pipe<Src, Dst> {
    pub stream: Src,
    pub sink: Dst,
}

impl<Src, Dst> Pipe<Src, Dst> {
    pub fn new(stream: Src, sink: Dst) -> Self {
        Self { stream, sink }
    }
}

impl<Src, Dst> Process for Pipe<Src, Dst>
where
    Src: Stream,
    Dst: Sink<Item = Src::Item>,
    Src::Item:,
{
    type Err = Either<Src::Err, Dst::Err>;

    fn run(&mut self) -> nb::Error<Self::Err> {
        loop {
            // Get the non blocking ready and slots
            let mut ready = match self.stream.ready() {
                Ok(r) => r,
                Err(err) => return err.map(Left),
            };
            let mut slots = match self.sink.slots() {
                Ok(r) => r,
                Err(err) => return err.map(Right),
            };

            // Pipe all items we can
            //
            // Notice the order: get a slot, then get a item. There is no way to
            // drop an item here!
            while let Some(slot) = match self.sink.slot(&mut slots) {
                Ok(v) => v,
                Err(err) => return nb::Error::Other(Right(err)),
            } && let Some(item) = match self.stream.stream(&mut ready) {
                Ok(v) => v,
                Err(err) => return nb::Error::Other(Left(err)),
            } {
                self.sink.sink(slot, item)
            }
        }
    }
}
