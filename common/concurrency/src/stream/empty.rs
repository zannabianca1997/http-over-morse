//! Empty stream

use core::{convert::Infallible, marker::PhantomData};

use crate::{ready::Ready, stream::BufferStream};

/// Empty stream
pub struct Empty<I>(PhantomData<I>);

impl<I> Empty<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> BufferStream for Empty<I> {
    type Item = I;
    type Err = Infallible;

    type Ready<'s> = NoneReady<I>;

    fn with_ready<R, F>(&mut self, fun: F) -> Result<R, Self::Err>
    where
        F: FnOnce(&mut Self::Ready<'_>) -> R,
    {
        Ok(fun(&mut NoneReady { _item: PhantomData }))
    }
}

pub struct NoneReady<I> {
    _item: PhantomData<fn() -> I>,
}
impl<I> Ready for NoneReady<I> {
    type Item = I;

    fn peek(&self) -> Option<&Self::Item> {
        None
    }

    fn pop(&mut self) -> Option<Self::Item> {
        None
    }
}
