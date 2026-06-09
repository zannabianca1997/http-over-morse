pub trait Slots {
    type Item;
    type Slot<'s>: Slot<Item = Self::Item>
    where
        Self: 's;

    fn is_empty(&self) -> bool;
    fn slot(&mut self) -> Option<Self::Slot<'_>>;
}

impl<T> Slots for &mut T
where
    T: Slots,
{
    type Item = T::Item;

    type Slot<'s>
        = T::Slot<'s>
    where
        Self: 's;

    fn slot(&mut self) -> Option<Self::Slot<'_>> {
        T::slot(self)
    }

    fn is_empty(&self) -> bool {
        T::is_empty(self)
    }
}

pub trait Slot {
    type Item;

    fn fill(self, item: Self::Item);
}
