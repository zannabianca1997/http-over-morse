//! Write all data from a stream to a sink

use either::{Either, Left, Right};

use crate::{
    sink::{BufferSink, Sink},
    stream::{BufferStream, Stream},
};

use super::Process;

/// Pipe from a sink to a stream
#[derive(Debug, Clone, Copy, Default)]
pub struct Pipe<Src, Dst, const STREAM_BUFFERS: bool> {
    pub stream: Src,
    pub sink: Dst,
}

impl<Src, Dst, const STREAM_BUFFERS: bool> Pipe<Src, Dst, STREAM_BUFFERS> {
    pub fn new(stream: Src, sink: Dst) -> Self {
        Self { stream, sink }
    }
}

impl<Src, Dst> Process for Pipe<Src, Dst, true>
where
    Src: BufferStream,
    Dst: Sink<Item = Src::Item>,
    Src::Item:,
{
    type Err = Either<Src::Err, Dst::Err>;

    fn run(&mut self) -> nb::Error<Self::Err> {
        loop {
            match self
                .stream
                .with_ready(|mut ready| self.sink.sink(&mut ready))
            {
                Ok(Ok(())) => (),
                Ok(Err(err)) => return err.map(Right),
                Err(err) => return nb::Error::Other(Left(err)),
            };
        }
    }
}

impl<Src, Dst> Process for Pipe<Src, Dst, false>
where
    Src: Stream,
    Dst: BufferSink<Item = Src::Item>,
    Src::Item:,
{
    type Err = Either<Src::Err, Dst::Err>;

    fn run(&mut self) -> nb::Error<Self::Err> {
        loop {
            match self
                .sink
                .with_slots(|mut slots| self.stream.fill(&mut slots))
            {
                Ok(Ok(())) => (),
                Ok(Err(err)) => return err.map(Left),
                Err(err) => return nb::Error::Other(Right(err)),
            };
        }
    }
}
