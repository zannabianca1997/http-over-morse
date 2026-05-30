//! Read letters from a signal source

use core::hint::cold_path;

use crate::{
    encoding::{Decoder, InvalidContinuation, InvalidLetter, LetterOrSpace, Signal, WORD_SPACE},
    process::Process,
};

pub struct SignalsToLetters<Inner> {
    inner: Inner,
    decoder: Decoder,
    space_pushback: bool,
}

impl<Inner> SignalsToLetters<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            decoder: Decoder::new(),
            space_pushback: false,
        }
    }
}

impl<Inner> Process for SignalsToLetters<Inner>
where
    Inner: Process<Ok = Signal>,
{
    type Ok = LetterOrSpace;
    type Err = Error<Inner::Err>;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            if self.space_pushback {
                self.space_pushback = false;
                return Ok(LetterOrSpace::SPACE);
            }

            let signal = self.inner.poll().map_err(|err| err.map(Error::Inner))?;

            match signal {
                Signal::Symbol(symbol) => match self.decoder.digest(symbol) {
                    // try to process another symbol
                    Ok(()) => (),
                    Err(err) => {
                        // Invalid letter
                        // TODO: error correction?

                        cold_path();
                        return Err(nb::Error::Other(Error::InvalidContinuation(err)));
                    }
                },
                Signal::Space(_) if self.decoder.is_empty() => {
                    // Received a space on an empty decoder -> the wire
                    // is stopped Continue reading to empty the buffer,
                    // the inner is responsible for sending WouldBlock
                    // signals
                    ()
                }
                Signal::Space(space) => {
                    match self.decoder.finish() {
                        Ok(ch) => {
                            self.space_pushback = space == WORD_SPACE;
                            return Ok(ch.into());
                        }
                        Err(err) => {
                            // Invalid letter
                            // TODO: error correction?

                            cold_path();
                            return Err(nb::Error::Other(Error::InvalidLetter(err)));
                        }
                    }
                }
            }
        }
    }
}

pub enum Error<I> {
    Inner(I),
    InvalidLetter(InvalidLetter),
    InvalidContinuation(InvalidContinuation),
}
