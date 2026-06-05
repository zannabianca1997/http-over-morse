//! Read packet framing

use core::{
    hint::cold_path,
    mem::{MaybeUninit, offset_of},
};

use either::{Either, Left, Right};
use frames::header::{FrameHeader, Read};
use net::ip_address::Address;
use parity::Checked;
use process::Process;

use crate::{
    events::{FrameEvent, FramingProsigns},
    traits::{SkipCurrentFrame, WithHeader},
};

pub struct BytesToFrameEvents<Inner> {
    inner: Inner,
    state: State,
}

const FRAME_HEADER_SIZE: u8 = size_of::<Read>() as u8;

#[derive(Clone, Copy)]
enum State {
    Idle,
    ReadingHeader {
        len: u8,
        read: [MaybeUninit<u8>; FRAME_HEADER_SIZE as usize],
    },
    HeaderRead {
        // TODO: check if it is truly useful to add this padding to avoid the shifting of the header in memory
        _pad: MaybeUninit<u8>,
        header: FrameHeader<Address>,
    },
    PassingPayload {
        remaining: u16,
    },
}

const _: () = const {
    let offset_1 = offset_of!(State, ReadingHeader.read);
    let offset_2 = offset_of!(State, HeaderRead.header);
    assert!(offset_1 == offset_2)
};

impl<Inner> BytesToFrameEvents<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            state: State::Idle,
        }
    }
}

impl<Inner> Process for BytesToFrameEvents<Inner>
where
    Inner: Process<Ok = Either<u8, FramingProsigns>>,
{
    type Ok = FrameEvent;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            match (&mut self.state, self.inner.poll()?) {
                // == Waiting for frame

                // If idle, ignore all except [KA]
                (State::Idle, Right(FramingProsigns::FrameStart)) => {
                    self.state = State::ReadingHeader {
                        len: 0,
                        read: [MaybeUninit::uninit(); _],
                    }
                }
                (State::Idle, _) => (),

                // == Reading the header

                // Collect as many bytes as the header size, then end the header
                (State::ReadingHeader { len, read }, Left(b)) if *len < FRAME_HEADER_SIZE => {
                    read[*len as usize].write(b);
                    *len += 1;

                    if *len == FRAME_HEADER_SIZE {
                        let bytes: &mut [u8; FRAME_HEADER_SIZE as usize] = unsafe {
                            // Safety: we read all the bytes
                            read.assume_init_mut()
                        }
                        .as_mut_array()
                        // TODO: check the compiler can remove this unwrap
                        .unwrap();

                        let header: &mut Read = zerocopy::transmute_mut!(bytes);

                        let Some(header) = header.check_mut() else {
                            // Frame header did not pass parity check

                            // TODO: Error handling?

                            cold_path();
                            // Ignoring the entire defective frame
                            self.skip_current_frame();

                            continue;
                        };

                        // Discard the parity byte: even if we pass this header
                        // forward, we need to change the time to live and so
                        // recalculate parity
                        let mut header = Checked::unwrap(*header);

                        // Reduce time to live as the frame did a hop
                        header.ttl -= 1;

                        self.state = State::HeaderRead {
                            header,
                            _pad: MaybeUninit::uninit(),
                        };
                        return Ok(FrameEvent::HeaderReceived);
                    }
                }

                // Pass through other bytes until the right prosign
                (State::HeaderRead { .. }, Left(_)) => {}
                // Start of payload
                (State::HeaderRead { header, .. }, Right(FramingProsigns::PayloadStart)) => {
                    self.state = State::PassingPayload {
                        remaining: header.payload_size(),
                    };
                    return Ok(FrameEvent::PayloadStart);
                }

                // == Reading the body

                // If any remaining, count down and pass it forward
                (
                    State::PassingPayload {
                        remaining: remaining @ 1..,
                    },
                    Left(b),
                ) => {
                    *remaining -= 1;
                    return Ok(FrameEvent::Payload(b));
                }
                // Concluding either at the [AR] or when payload size is finished
                (State::PassingPayload { .. }, Right(FramingProsigns::PayloadEnd))
                | (State::PassingPayload { remaining: 0 }, _) => {
                    self.state = State::Idle;
                    return Ok(FrameEvent::PayloadEnd);
                }

                // Error path
                _ => {
                    // TODO: Error handling?

                    cold_path();
                    // Ignoring the entire defective frame
                    self.skip_current_frame()
                }
            }
        }
    }
}

impl<Inner> SkipCurrentFrame for BytesToFrameEvents<Inner>
where
    Inner: Process<Ok = Either<u8, FramingProsigns>>,
{
    fn skip_current_frame(&mut self) {
        self.state = State::Idle
    }
}

impl<Inner> WithHeader for BytesToFrameEvents<Inner>
where
    Inner: Process<Ok = Either<u8, FramingProsigns>>,
{
    type Header = FrameHeader<Address>;

    fn header(&self) -> Option<&Self::Header> {
        if let State::HeaderRead { header, .. } = &self.state {
            Some(header)
        } else {
            None
        }
    }
}
