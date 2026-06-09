pub trait Ready {
    type Item;

    fn peek(&self) -> Option<&Self::Item>;
    fn pop(&mut self) -> Option<Self::Item>;
}

impl<T> Ready for &mut T
where
    T: Ready,
{
    type Item = T::Item;

    fn peek(&self) -> Option<&Self::Item> {
        T::peek(self)
    }
    fn pop(&mut self) -> Option<Self::Item> {
        T::pop(self)
    }
}
