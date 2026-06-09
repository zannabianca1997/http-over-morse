//! Reader in a buffer

use core::{cmp::min, marker::PhantomData, mem::MaybeUninit, slice};

use crate::{Buffer, ProvidesIndices, unsync::Unsync};

use super::ReadRegion;

pub struct Reader<'b, T, const CAP: usize, Sync: ProvidesIndices = Unsync> {
    buffer: &'b mut Buffer<T, CAP, Sync>,
    _phantom: PhantomData<fn() -> T>,
}

impl<'b, T, const CAP: usize, Sync: ProvidesIndices> Reader<'b, T, CAP, Sync> {
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

    pub fn pop(&mut self) -> Option<T> {
        self.read(|r| r.pop())
    }

    pub fn read<R>(&mut self, fun: impl FnOnce(&mut ReadRegion<'_, T>) -> R) -> R {
        let (start, len) = self.fetch_start_and_len_contiguous_mut();

        let buffer = self.buffer.content.get().cast::<MaybeUninit<T>>();

        let start = unsafe {
            buffer.add(start & (CAP - 1))
        };

        let region = unsafe {
            slice::from_raw_parts_mut(start, len)
        };

        let head = Sync::get_mut(&mut self.buffer.head);
        let mut region = ReadRegion::new(region, head);

        let res = fun(&mut region);

        drop(region);

        res
    }

    fn fetch_start_and_end_mut(&mut self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail_mut();

        let start = head;
        let end = tail;

        (start, end)
    }

    fn fetch_start_and_len_contiguous_mut(&mut self) -> (usize, usize) {
        let (start, end) = self.fetch_start_and_end_mut();

        let non_wrapping_len = CAP - (start & (CAP - 1));
        let init_len = end.wrapping_sub(start);
        let init_non_wrapping_len = min(non_wrapping_len, init_len);

        (start, init_non_wrapping_len)
    }
}

#[cfg(test)]
mod tests {
    use crate::unsync::Buffer;

    #[test]
    fn read_region_empty_only_when_empty() {
        let mut buf = Buffer::<i32, 4>::new();
        let mut r = buf.reader_mut();

        assert!(r.read(|reg| reg.is_empty()));
        drop(r);

        let mut w = buf.writer_mut();
        w.push(7).unwrap();
        drop(w);

        let mut r = buf.reader_mut();
        assert!(!r.read(|reg| reg.is_empty()));
    }

    #[test]
    fn read_region_truncates_at_wrap() {
        let mut buf = Buffer::<i32, 4>::new();
        let mut w = buf.writer_mut();
        for i in 0..4 {
            w.push(i).unwrap();
        }
        drop(w);

        let mut r = buf.reader_mut();
        assert_eq!(r.pop(), Some(0));
        assert_eq!(r.pop(), Some(1));
        drop(r);

        let mut w = buf.writer_mut();
        w.push(4).unwrap();
        w.push(5).unwrap();
        drop(w);

        let mut r = buf.reader_mut();
        assert_eq!(r.read(|reg| reg.len()), 2);

        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), Some(4));
        assert_eq!(r.pop(), Some(5));
        assert_eq!(r.pop(), None);
    }
}
