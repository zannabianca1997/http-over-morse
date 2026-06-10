//! Direct access to the buffer through mutable borrowing reader and writer
//!
//! Avoid paying the cost of synchronization

use crate::{Index, sealed::Sealed};

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

impl Sealed for usize {}
impl Index for usize {
    const ZERO: Self = 0;
    const MAX: usize = usize::MAX;

    fn get_mut(&mut self) -> usize {
        *self
    }

    unsafe fn get_unsync(&self) -> usize {
        *self
    }

    fn wrapping_add_mut(&mut self, value: usize) {
        *self = self.wrapping_add(value);
    }
}

macro_rules! define_scalar {
    ( $( $t:ty )* ) => {
        $(
            impl Sealed for $t {}
            impl Index for $t {
                const ZERO: Self = 0;
                const MAX: usize = Self::MAX as usize;

                fn get_mut(&mut self) -> usize {
                    if const { usize::BITS >= Self::BITS } {
                        *self as _
                    } else {
                        (*self & usize::MAX as Self) as usize
                    }
                }

                unsafe fn get_unsync(&self) -> usize {
                    if const { usize::BITS >= Self::BITS } {
                        *self as _
                    } else {
                        (*self & usize::MAX as Self) as usize
                    }
                }

                fn wrapping_add_mut(&mut self, value: usize) {
                    if const { usize::BITS >= Self::BITS } {
                        *self = self.wrapping_add((value & Self::MAX as usize) as Self);
                    } else {
                        *self = self.wrapping_add(value as Self);
                    }
                }
            }
        )*
    };
}
define_scalar! {u8 u16 u32 u64 u128}

pub type Buffer<T, const CAP: usize, Idx = usize> = crate::Buffer<T, CAP, Idx>;

impl<T, const CAP: usize, Idx: Index> crate::Buffer<T, CAP, Idx> {
    /// Get a writing handle to this buffer through a mutable reference
    pub const fn writer_mut(&mut self) -> Writer<'_, T, CAP, Idx> {
        Writer::new(self)
    }

    /// Get a reading handle to this buffer through a mutable reference
    pub const fn reader_mut(&mut self) -> Reader<'_, T, CAP, Idx> {
        Reader::new(self)
    }

    /// Safety: call only with a mutable borrow to the buffer
    unsafe fn get_head_and_tail_unsync(&self) -> (usize, usize) {
        // Safety: callers (the unsync Reader/Writer) hold this buffer through
        // a mutable borrow, guaranteeing exclusive access.
        unsafe { (Index::get_unsync(&self.head), Index::get_unsync(&self.tail)) }
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::SeqCst},
    };

    #[derive(Debug)]
    struct DropCounter(Arc<AtomicUsize>);
    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, SeqCst);
        }
    }

    #[test]
    fn fifo_roundtrip() {
        let mut buf = Buffer::<i32, 4>::new();
        let mut w = buf.writer_mut();

        assert_eq!(w.push(1), Ok(()));
        assert_eq!(w.push(2), Ok(()));
        assert_eq!(w.push(3), Ok(()));
        drop(w);

        let mut r = buf.reader_mut();
        assert_eq!(r.pop(), Some(1));
        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), None);
    }

    #[test]
    fn fills_and_reports_full() {
        let mut buf = Buffer::<i32, 4>::new();
        let mut w = buf.writer_mut();

        assert_eq!(w.available(), 4);
        for i in 0..4 {
            assert_eq!(w.push(i), Ok(()));
        }
        assert_eq!(w.available(), 0);
        assert_eq!(w.push(99), Err(99));
        drop(w);

        let mut r = buf.reader_mut();
        assert_eq!(r.available(), 4);
        assert_eq!(r.pop(), Some(0));
        drop(r);

        let w = buf.writer_mut();
        assert_eq!(w.available(), 1);
    }

    #[test]
    fn wraps_around() {
        let mut buf = Buffer::<i32, 4>::new();

        let mut next_push = 0;
        let mut next_pop = 0;
        for _ in 0..1000 {
            let mut w = buf.writer_mut();
            while w.available() > 2 {
                w.push(next_push).unwrap();
                next_push += 1;
            }
            drop(w);

            let mut r = buf.reader_mut();
            for _ in 0..2 {
                if let Some(v) = r.pop() {
                    assert_eq!(v, next_pop);
                    next_pop += 1;
                }
            }
            drop(r);
        }

        let mut r = buf.reader_mut();
        while let Some(v) = r.pop() {
            assert_eq!(v, next_pop);
            next_pop += 1;
        }
        assert_eq!(next_pop, next_push);
    }

    #[test]
    fn buffer_drop_is_exact() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let mut buf = Buffer::<DropCounter, 4>::new();
            let mut w = buf.writer_mut();
            for _ in 0..3 {
                w.push(DropCounter(counter.clone())).unwrap();
            }
            drop(w);
            assert_eq!(counter.load(SeqCst), 0);
        }
        assert_eq!(counter.load(SeqCst), 3);
    }

    #[test]
    fn pop_then_buffer_drop_is_exact() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let mut buf = Buffer::<DropCounter, 4>::new();
            let mut w = buf.writer_mut();
            for _ in 0..4 {
                w.push(DropCounter(counter.clone())).unwrap();
            }
            drop(w);

            let mut r = buf.reader_mut();
            drop(r.pop());
            drop(r.pop());
            drop(r);
            assert_eq!(counter.load(SeqCst), 2);
        }
        assert_eq!(counter.load(SeqCst), 4);
    }

    #[test]
    fn zst_makes_progress() {
        let mut buf = Buffer::<(), 4>::new();
        let mut w = buf.writer_mut();

        assert_eq!(w.push(()), Ok(()));
        drop(w);

        let mut r = buf.reader_mut();
        assert_eq!(r.available(), 1);
        assert_eq!(r.pop(), Some(()));
        assert_eq!(r.pop(), None);
    }
}
