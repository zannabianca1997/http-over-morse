//! Convert letters into signals

use process::Process;

use morse::{LETTER_SPACE, Letter, LetterOrProsignOrSpace, Signal, Symbol, WORD_SPACE, WordSpace};

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
    Inner: Process<Ok = LetterOrProsignOrSpace>,
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

        match WordSpace::try_from(self.inner.poll()?) {
            Err(letter_or_prosign) => {
                if self.last_emitted_was_symbol {
                    self.emitting = Some(letter_or_prosign.symbols());
                    self.last_emitted_was_symbol = false;
                    Ok(Signal::Space(LETTER_SPACE))
                } else {
                    let (next, rest) = letter_or_prosign.symbols().split_first();
                    self.emitting = nunny::Slice::new(rest);
                    self.last_emitted_was_symbol = true;
                    return Ok(Signal::Symbol(*next));
                }
            }
            Ok(WordSpace::Space) => {
                self.last_emitted_was_symbol = false;
                Ok(Signal::Space(WORD_SPACE))
            }
        }
    }
}
