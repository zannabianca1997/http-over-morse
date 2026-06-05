//! Frame events to bytes

use either::Either::{self, Left, Right};
use morse::Prosign;
use process::Process;
use zerocopy::{Immutable, IntoBytes};

use crate::{events::FrameEvent, traits::WithHeader};

pub struct FrameEventsToBytes<Inner> {
    inner: Inner,
    state: State,
}

enum State {
    Idle,
    WritingHeader { pos: u8 },
}

impl<Inner> FrameEventsToBytes<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            state: State::Idle,
        }
    }
}

impl<Inner> Process for FrameEventsToBytes<Inner>
where
    Inner: Process<Ok = FrameEvent> + WithHeader,
    Inner::Header: Immutable + IntoBytes,
{
    type Ok = Either<u8, Prosign>;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        // Finish writing the header
        if let State::WritingHeader { pos } = &mut self.state {
            // Get the byte string of the header
            // TODO: Is the compiler able to remove that unwrap?
            let header: &[u8] = self.inner.header().unwrap().as_bytes();
            // Write a byte
            if let Some(b) = header.get(*pos as usize) {
                *pos += 1;
                return Ok(Left(*b));
            } else {
                // Header ended, ask for continuation
                self.state = State::Idle;
            }
        }

        match self.inner.poll()? {
            FrameEvent::HeaderReceived => {
                self.state = State::WritingHeader { pos: 0 };
                Ok(Right(Prosign::Attention))
            }
            FrameEvent::PayloadStart => Ok(Right(Prosign::Break)),
            FrameEvent::Payload(b) => Ok(Left(b)),
            FrameEvent::PayloadEnd => Ok(Right(Prosign::Out)),
        }
    }
}
