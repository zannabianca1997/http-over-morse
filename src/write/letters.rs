//! Convert bytes into morse letters

use crate::{
    encoding::LetterOrSpace,
    process::Process,
    strategies::{Discard, Strategy},
};

/// Read morse letters from a byte source
pub struct ReadLetters<Inner, S: Strategy> {
    inner: Inner,
    data: S::ReadData,
}

impl<Inner, S: Strategy> ReadLetters<Inner, S> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            data: S::init_read(),
        }
    }
}

impl<Inner: Process<Ok = u8>> Process for ReadLetters<Inner, Discard> {
    type Ok = LetterOrSpace;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            let ch = self.inner.poll()?.to_ascii_uppercase();

            if let Some(ch) = LetterOrSpace::new(ch) {
                break Ok(ch);
            }
        }
    }
}
