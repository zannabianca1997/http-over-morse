#![cfg(feature = "write")]

use core::mem::MaybeUninit;

use either::{
    Either::{self, Right},
    Left,
};
use morse::Prosign;
use process::Process;

const ESCAPE: u8 = 27;

pub struct ReadProsigns<Inner> {
    inner: Inner,
    state: State,
}

#[derive(Clone, Copy)]
enum State {
    Idle,
    ReadingProsign {
        len: u8,
        read: [MaybeUninit<u8>; MAX_PROSIGN_LEN],
    },
}

impl<Inner> ReadProsigns<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            state: State::Idle,
        }
    }
}

impl<Inner> Process for ReadProsigns<Inner>
where
    Inner: Process<Ok = u8>,
{
    type Ok = Either<u8, Prosign>;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            match (&mut self.state, self.inner.poll()?) {
                (State::Idle, ESCAPE) => {
                    self.state = State::ReadingProsign {
                        len: 0,
                        read: [MaybeUninit::uninit(); MAX_PROSIGN_LEN],
                    }
                }
                (State::Idle, ch) => {
                    return Ok(Left(ch));
                }
                // Double escape to escape
                (State::ReadingProsign { len: 0, .. }, ESCAPE) => {
                    self.state = State::Idle;
                    return Ok(Left(ESCAPE));
                }
                (State::ReadingProsign { len, read }, b']')
                    if let Some(prosign) = Prosign::from_code(unsafe {
                        // Safety: len cannot get over MAX_PROSIGN_LEN
                        // without triggering the last arm
                        read.get_unchecked(..*len as usize).assume_init_ref()
                    }) =>
                {
                    self.state = State::Idle;
                    return Ok(Right(prosign));
                }
                (State::ReadingProsign { len, read }, ch) if *len < MAX_PROSIGN_LEN as u8 => {
                    read[*len as usize].write(ch.to_ascii_uppercase());
                    *len += 1;
                }
                // Prosign too long or unknown
                (State::ReadingProsign { .. }, _) => self.state = State::Idle,
            }
        }
    }
}

const MAX_PROSIGN_LEN: usize = const {
    let mut max = 0;

    // Check all the u8
    let mut next = Some(u8::MIN);
    while let Some(ch) = {
        let ch = next;
        if let Some(n) = next {
            next = n.checked_add(1);
        }
        ch
    } {
        let Some(prosign) = Prosign::new(ch) else {
            continue;
        };

        let len = prosign.code().as_slice().len();

        if len > max {
            max = len
        };
    }

    max
};
