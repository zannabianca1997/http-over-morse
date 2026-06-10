//! Contiguous views into buffers

use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

struct RawRegion<'o> {
    region: NonNull<()>,
    len: usize,
    start_index_storage: &'o mut usize,
}

impl<'o> RawRegion<'o> {
    fn new<T>(region: &'o mut [MaybeUninit<T>], start_index_storage: &'o mut usize) -> Self {
        let len = region.len();
        let region = NonNull::from_mut(region).cast();
        Self {
            region,
            len,
            start_index_storage,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub struct ReadRegion<'o, T> {
    raw: RawRegion<'o>,
    _phantom: PhantomData<T>,
}

impl<'o, T> ReadRegion<'o, T> {
    pub(in crate::unsync) fn new(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o mut usize,
    ) -> Self {
        Self {
            raw: RawRegion::new(region, start_index_storage),
            _phantom: PhantomData,
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.raw.len() == 0 {
            return None;
        }

        let region: NonNull<T> = self.raw.region.cast();

        *self.raw.start_index_storage += 1;
        self.raw.region = unsafe { region.add(1) }.cast();

        Some(unsafe { region.read() })
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { NonNull::slice_from_raw_parts(self.raw.region.cast(), self.raw.len).as_ref() }
    }
}

pub struct WriteRegion<'o, T> {
    raw: RawRegion<'o>,
    _phantom: PhantomData<fn(T)>,
}

impl<'o, T> WriteRegion<'o, T> {
    pub(in crate::unsync) fn new(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o mut usize,
    ) -> Self {
        Self {
            raw: RawRegion::new(region, start_index_storage),
            _phantom: PhantomData,
        }
    }

    pub fn slot(&mut self) -> Option<WriteRegionSlot<'_, T>> {
        if self.raw.is_empty() {
            return None;
        } else {
            return Some(WriteRegionSlot(
                &mut self.raw.region,
                self.raw.start_index_storage,
                PhantomData,
            ));
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        match self.slot() {
            Some(slot) => Ok(slot.fill(item)),
            None => Err(item),
        }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

pub struct WriteRegionSlot<'o, T>(&'o mut NonNull<()>, &'o mut usize, PhantomData<fn(T)>);

impl<'o, T> WriteRegionSlot<'o, T> {
    pub fn fill(self, value: T) {
        let t: NonNull<MaybeUninit<T>> = self.0.cast();
        unsafe { t.write(MaybeUninit::new(value)) };
        *self.0 = unsafe { t.add(1) }.cast();
        *self.1 += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::unsync::Buffer;

    #[test]
    fn partial_read_advances_by_popped_count() {
        let mut buf = Buffer::<i32, 8>::new();
        let mut w = buf.writer_mut();
        for i in 0..5 {
            w.push(i).unwrap();
        }
        drop(w);

        let mut r = buf.reader_mut();
        r.read(|region| {
            assert_eq!(region.len(), 5);
            assert!(!region.is_empty());
            assert_eq!(region.pop(), Some(0));
            assert_eq!(region.pop(), Some(1));
        });

        assert_eq!(r.available(), 3);
        assert_eq!(r.pop(), Some(2));
    }

    #[test]
    fn partial_write_advances_by_pushed_count() {
        let mut buf = Buffer::<i32, 8>::new();

        let mut w = buf.writer_mut();
        w.write(|region| {
            assert_eq!(region.len(), 8);
            region.push(10).unwrap();
            region.push(11).unwrap();
        });
        drop(w);

        let mut r = buf.reader_mut();
        assert_eq!(r.available(), 2);
        assert_eq!(r.pop(), Some(10));
        assert_eq!(r.pop(), Some(11));
        assert_eq!(r.pop(), None);
    }
}
