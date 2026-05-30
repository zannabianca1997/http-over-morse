//! Convert bytes into morse letters

use crate::{
    encoding::LetterOrSpace,
    process::Process,
    strategies::{Discard, Strategy},
};

/// Read morse letters from a byte source
pub struct BytesToLetters<Inner, S: Strategy> {
    inner: Inner,
    data: S::ReadData,
}

impl<Inner, S: Strategy> BytesToLetters<Inner, S> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            data: S::new_read(),
        }
    }
}

impl<Inner: Process<Ok = u8>> Process for BytesToLetters<Inner, Discard> {
    type Ok = LetterOrSpace;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            let mut ch = self.inner.poll()?;

            if ch.is_ascii_whitespace() {
                ch = b' '
            } else {
                ch = ch.to_ascii_uppercase()
            }

            if let Some(ch) = LetterOrSpace::new(ch) {
                break Ok(ch);
            }
        }
    }
}
