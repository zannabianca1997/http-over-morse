#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(test)]
extern crate std;

use core::{cell::UnsafeCell, mem::MaybeUninit};

use sync::Synchronization;

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

use crate::sync::DefineCriticalSectionMethods;

mod reader;
mod region;
pub mod sync;
mod writer;

/// A ring buffer for type T and with capacity CAP
///
/// The only way of interacting with this is through [`Writer`] and [`Reader`]
pub struct Buffer<T, const CAP: usize, Sync: Synchronization> {
    /// Content of the buffer
    ///
    /// All the elements from `head % CAP` included onward, possibly wrapping
    /// around up to `tail % CAP` excluded are initialized.
    content: UnsafeCell<[MaybeUninit<T>; CAP]>,
    /// Head of the buffer
    ///
    /// Points to the oldest filled slot.
    ///
    /// Can surpass CAP, as it is wrapped around on access.
    head: Sync::Usize,
    /// Tail of the buffer
    ///
    /// Points to the oldest unfilled slot.
    ///
    /// Can surpass CAP, as it is wrapped around on access.
    tail: Sync::Usize,
}

impl<T, const CAP: usize, Sync: Synchronization> Buffer<T, CAP, Sync> {
    pub const fn new() -> Self {
        // Force a compile-time failure if the buffer length is not a power of
        // 2.
        //
        // A length equal to a power of two enable us to easy wrap around tail
        // and head via precalculated bitmasks.
        const {
            assert!(
                (CAP != 0) && ((CAP - 1) & CAP == 0),
                "buffer size must be a power of 2"
            )
        };

        Self {
            content: UnsafeCell::new(unsafe {
                // Safety: An array of MaybeUninit has no requirements
                MaybeUninit::uninit().assume_init()
            }),
            head: Sync::ZERO,
            tail: Sync::ZERO,
        }
    }

    /// Get both handlers to the buffer.
    ///
    /// Ensure uniqueness by taking a mutable reference
    pub const fn split(&mut self) -> (Writer<'_, T, CAP, Sync>, Reader<'_, T, CAP, Sync>)
    where
        Sync: StaticDefault,
    {
        (
            Writer::new(self, Sync::DEFAULT),
            Reader::new(self, Sync::DEFAULT),
        )
    }

    /// Get a writing handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer(&self) -> Writer<'_, T, CAP, Sync>
    where
        Sync: StaticDefault,
    {
        Writer::new(self, Sync::DEFAULT)
    }

    /// Get a writing handle to this buffer inside a critical section.
    ///
    /// This will avoid declaring unneeded critical sections.
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer_cs<'cs>(
        &self,
        cs: Sync::CriticalSection<'cs>,
    ) -> Writer<'_, T, CAP, Sync, Sync::CriticalSection<'cs>>
    where
        Sync: DefineCriticalSectionMethods,
    {
        Writer::new(self, cs)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader(&self) -> Reader<'_, T, CAP, Sync>
    where
        Sync: StaticDefault,
    {
        Reader::new(self, Sync::DEFAULT)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader_cs<'cs>(
        &self,
        cs: Sync::CriticalSection<'cs>,
    ) -> Reader<'_, T, CAP, Sync, Sync::CriticalSection<'cs>>
    where
        Sync: DefineCriticalSectionMethods,
    {
        Reader::new(self, cs)
    }

    fn get_head_and_tail(&self, sync: impl Synchronization<Usize = Sync::Usize>) -> (usize, usize) {
        sync.free(|cs| {
            let head = cs.get(&self.head);
            let tail = cs.get(&self.tail);

            (head, tail)
        })
    }
}

/// As we handle concurrency inside, [`Buffer`] can implement [`Sync`].
///
/// Safety: The tail and heads are checked with the [`sync`] primitives, making
/// them already [`Sync`]. The content is not sync, but both the writer and the
/// reader check they have exclusive access to it before interacting. Finally, T
/// must be [`Send`] as we can read it out from another thread.
unsafe impl<T, const CAP: usize, S: Synchronization> Sync for Buffer<T, CAP, S> where T: Send {}

/// Need to drop all elements still in the buffer
impl<T, const CAP: usize, Sync: Synchronization> Drop for Buffer<T, CAP, Sync> {
    fn drop(&mut self) {
        let head = Sync::get_mut(&mut self.head);
        let tail = Sync::get_mut(&mut self.tail);
        while head != tail {
            let init = &mut self.content.get_mut()[*head & (CAP - 1)];

            unsafe {
                // Safety: we know all elements between head and tail have been
                // initialized
                init.assume_init_drop();
            }

            *head = head.wrapping_add(1);
        }
    }
}

pub trait StaticDefault {
    const DEFAULT: Self;
}

#[cfg(test)]
#[cfg(target_has_atomic = "ptr")]
mod tests {
    use super::*;
    use crate::sync::Atomics;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::SeqCst},
    };

    /// A `T` whose destructor counts how many times it runs, to catch
    /// double-frees and missed drops.
    #[derive(Debug)]
    struct DropCounter(Arc<AtomicUsize>);
    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, SeqCst);
        }
    }

    /// Basic FIFO ordering.
    #[test]
    fn fifo_roundtrip() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        assert_eq!(w.push(1), Ok(()));
        assert_eq!(w.push(2), Ok(()));
        assert_eq!(w.push(3), Ok(()));

        assert_eq!(r.pop(), Some(1));
        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), None);
    }

    /// Filling to capacity, `Err` on overflow, and `available()` tracking on
    /// both sides.
    #[test]
    fn fills_and_reports_full() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        assert_eq!(w.available(), 4);
        for i in 0..4 {
            assert_eq!(w.push(i), Ok(()));
        }
        assert_eq!(w.available(), 0);
        assert_eq!(w.push(99), Err(99));

        assert_eq!(r.available(), 4);
        assert_eq!(r.pop(), Some(0));
        assert_eq!(w.available(), 1);
    }

    /// Push/pop many more times than CAP so the cursors wrap the backing array
    /// repeatedly; ordering and values must stay correct.
    #[test]
    fn wraps_around() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        let mut next_push = 0;
        let mut next_pop = 0;
        for _ in 0..1000 {
            while w.available() > 2 {
                w.push(next_push).unwrap();
                next_push += 1;
            }
            if let Some(v) = r.pop() {
                assert_eq!(v, next_pop);
                next_pop += 1;
            }
        }
        while let Some(v) = r.pop() {
            assert_eq!(v, next_pop);
            next_pop += 1;
        }
        assert_eq!(next_pop, next_push);
    }

    /// Dropping the buffer with elements still inside must run each destructor
    /// exactly once.
    #[test]
    fn buffer_drop_is_exact() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let mut buf = Buffer::<DropCounter, 4, Atomics>::new();
            let (mut w, _r) = buf.split();
            for _ in 0..3 {
                w.push(DropCounter(counter.clone())).unwrap();
            }
            assert_eq!(counter.load(SeqCst), 0);
            // w, _r, then buf drop here; buf::drop owns the 3 remaining
        }
        assert_eq!(counter.load(SeqCst), 3);
    }

    /// Popped elements are dropped exactly once and the remainder are dropped
    /// by the buffer — no double free, no leak.
    #[test]
    fn pop_then_buffer_drop_is_exact() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let mut buf = Buffer::<DropCounter, 4, Atomics>::new();
            let (mut w, mut r) = buf.split();
            for _ in 0..4 {
                w.push(DropCounter(counter.clone())).unwrap();
            }
            drop(r.pop());
            drop(r.pop());
            assert_eq!(counter.load(SeqCst), 2);
            // 2 remain, dropped by buf::drop
        }
        assert_eq!(counter.load(SeqCst), 4);
    }

    /// A read region and a write region alive at the same time (nested
    /// closures) carve two `&mut` slices out of the same backing array over
    /// disjoint ranges. This must be accepted by Miri's aliasing model.
    #[test]
    fn regions_coexist() {
        let mut buf = Buffer::<i32, 8, Atomics>::new();
        let (mut w, mut r) = buf.split();
        for i in 0..3 {
            w.push(i).unwrap();
        }

        // `rd` covers the filled range [0, 3); `wr` covers the empty range
        // [3, 8). Both are live at once inside the nested closures.
        r.read(|rd| {
            w.write(|wr| {
                assert_eq!(rd.pop(), Some(0));
                wr.push(100).unwrap();
            }); // wr drops here: tail += 1
        }); // rd drops here: head += 1

        assert_eq!(r.available(), 3); // {1, 2, 100}
        assert_eq!(r.pop(), Some(1));
        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(100));
    }

    /// A buffer of zero-sized elements must still advance its cursors
    #[test]
    fn zst_makes_progress() {
        let mut buf = Buffer::<(), 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        assert_eq!(w.push(()), Ok(()));
        assert_eq!(r.available(), 1);
        assert_eq!(r.pop(), Some(()));
        assert_eq!(r.pop(), None);
    }

    /// Concurrent producer/consumer. With the `Acquire`/`Release` ordering this
    /// is race-free; run it under Miri (ideally `MIRIFLAGS=-Zmiri-many-seeds`)
    /// to exercise many interleavings and confirm no data race remains.
    #[test]
    fn spsc_concurrent() {
        const N: i32 = 1000;
        let mut buf = Buffer::<i32, 16, Atomics>::new();
        let (mut w, mut r) = buf.split();

        std::thread::scope(|s| {
            s.spawn(move || {
                let mut i = 0;
                while i < N {
                    if w.push(i).is_ok() {
                        i += 1;
                    } else {
                        std::thread::yield_now();
                    }
                }
            });
            s.spawn(move || {
                let mut expected = 0;
                while expected < N {
                    if let Some(v) = r.pop() {
                        assert_eq!(v, expected, "Data race: corrupted / reordered value");
                        expected += 1;
                    } else {
                        std::thread::yield_now();
                    }
                }
            });
        });
    }
}
