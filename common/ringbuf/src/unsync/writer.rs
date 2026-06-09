use core::{cmp::min, marker::PhantomData, mem::MaybeUninit, slice};

use crate::{Buffer, ProvidesIndices, unsync::Unsync};

use super::WriteRegion;

pub struct Writer<'b, T, const CAP: usize, Sync: ProvidesIndices = Unsync> {
    buffer: &'b mut Buffer<T, CAP, Sync>,
    _phantom: PhantomData<fn(T)>,
}

impl<'b, T, const CAP: usize, Sync: ProvidesIndices> Writer<'b, T, CAP, Sync> {
    pub(in crate::unsync) const fn new(buffer: &'b mut Buffer<T, CAP, Sync>) -> Self {
        Self {
            buffer,
            _phantom: PhantomData,
        }
    }

    pub fn available(&mut self) -> usize {
        let (start, end) = self.fetch_start_and_end_mut();
        end.wrapping_sub(start)
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        self.write(|r| r.push(item).map(|_| ()))
    }

    pub fn write<R>(&mut self, fun: impl FnOnce(&mut WriteRegion<'_, T>) -> R) -> R {
        let (start, len) = self.fetch_start_and_len_contiguous_mut();

        let buffer = self.buffer.content.get().cast::<MaybeUninit<T>>();

        let start = unsafe { buffer.add(start & (CAP - 1)) };

        let region = unsafe { slice::from_raw_parts_mut(start, len) };

        let tail = Sync::get_mut(&mut self.buffer.tail);
        let mut region = WriteRegion::new(region, tail);

        let res = fun(&mut region);

        drop(region);

        res
    }

    fn fetch_start_and_end_mut(&mut self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail_mut();

        let start = tail;
        let end = head.wrapping_add(CAP);

        (start, end)
    }

    fn fetch_start_and_len_contiguous_mut(&mut self) -> (usize, usize) {
        let (start, end) = self.fetch_start_and_end_mut();

        let non_wrapping_len = CAP - (start & (CAP - 1));
        let uninit_len = end.wrapping_sub(start);
        let uninit_non_wrapping_len = min(non_wrapping_len, uninit_len);

        (start, uninit_non_wrapping_len)
    }
}
