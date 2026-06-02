use either::{Either, Left, Right};
use morse::Prosign;
use process::Process;

pub struct PrintProsigns<Inner> {
    inner: Inner,
    printing: Option<&'static [u8]>,
}

impl<Inner> PrintProsigns<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            printing: None,
        }
    }
}

impl<Inner> Process for PrintProsigns<Inner>
where
    Inner: Process<Ok = Either<u8, Prosign>>,
{
    type Ok = u8;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        Ok(match self.printing {
            None => match self.inner.poll()? {
                Left(ch) => ch,
                Right(prosign) => {
                    self.printing = Some(prosign.code());
                    b'['
                }
            },
            Some([next, rest @ ..]) => {
                self.printing = Some(rest);
                *next
            }
            Some([]) => {
                self.printing = None;
                b']'
            }
        })
    }
}
