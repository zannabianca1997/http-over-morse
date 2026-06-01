//! Convert letters into signals

use process::Process;

use crate::encoding::{LETTER_SPACE, LetterOrSpace, Signal, Symbol, WORD_SPACE};

pub struct LettersToSignals<Inner> {
    inner: Inner,
    emitting: Option<&'static nunny::Slice<Symbol>>,
    last_emitted_was_symbol: bool,
}

impl<Inner> LettersToSignals<Inner> {
    pub const fn new(inner: Inner) -> Self {
        Self {
            inner,
            emitting: None,
            last_emitted_was_symbol: false,
        }
    }
}

impl<Inner> Process for LettersToSignals<Inner>
where
    Inner: Process<Ok = LetterOrSpace>,
{
    type Ok = Signal;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        // Empty the emitting buffer first
        if let Some(emitting) = self.emitting {
            let (next, rest) = emitting.split_first();
            self.emitting = nunny::Slice::new(rest);
            self.last_emitted_was_symbol = true;
            return Ok(Signal::Symbol(*next));
        }

        match self.inner.poll()?.letter() {
            Some(letter) => {
                if self.last_emitted_was_symbol {
                    self.emitting = Some(letter.symbols());
                    self.last_emitted_was_symbol = false;
                    Ok(Signal::Space(LETTER_SPACE))
                } else {
                    let (next, rest) = letter.symbols().split_first();
                    self.emitting = nunny::Slice::new(rest);
                    self.last_emitted_was_symbol = true;
                    return Ok(Signal::Symbol(*next));
                }
            }
            None => {
                self.last_emitted_was_symbol = false;
                Ok(Signal::Space(WORD_SPACE))
            }
        }
    }
}
