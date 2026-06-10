//! Concurrent access to the buffer through separate smart pointers for writing
//! and reader

use crate::indices::Index;

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

/// Synchronization method
pub trait Synchronization<Idx: Index>: Copy {
    type CriticalSection<'cs>: Synchronization<Idx>;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R;

    fn get(self, var: &Idx) -> usize;

    /// Wrapping add to a value
    ///
    /// This need to wrap aroung a power of 2, and to be coherent with Idx::wrapping_add
    fn wrapping_add(self, var: &Idx, value: usize);
}
/// This synchronization has significant critical sections, so methods to
/// provide them should be offered
///
/// This avoids polluting the API of `crate::Buffer` when a synchronization
/// without critical sections is used
pub trait DefineCriticalSectionMethods {}

/// Index that can be used in sync operations
pub trait SyncIndex: Index {
    type Sync: Synchronization<Self>;
    const SYNC: Self::Sync;
}

#[cfg(any(
    target_has_atomic = "ptr",
    target_has_atomic = "8",
    target_has_atomic = "16",
    target_has_atomic = "32",
    target_has_atomic = "64",
    target_has_atomic = "128"
))]
mod atomics;
#[cfg(any(
    target_has_atomic = "ptr",
    target_has_atomic = "8",
    target_has_atomic = "16",
    target_has_atomic = "32",
    target_has_atomic = "64",
    target_has_atomic = "128"
))]
pub use atomics::Atomics;

#[cfg(feature = "avr-device")]
mod critical_sections;
#[cfg(feature = "avr-device")]
pub use critical_sections::CriticalSections;

pub type DefaultIdx = cfg_select! {
    target_has_atomic = "ptr" => { core::sync::atomic::AtomicUsize }
    target_has_atomic = "32"  => { core::sync::atomic::AtomicU32 }
    target_has_atomic = "64"  => { core::sync::atomic::AtomicU64 }
    target_has_atomic = "128" => { core::sync::atomic::AtomicU128 }
    target_has_atomic = "16"  => { core::sync::atomic::AtomicU16 }
    target_has_atomic = "8"   => { core::sync::atomic::AtomicU8 }
    feature = "avr-device"    => { avr_device::interrupt::Mutex<core::cell::Cell<usize>> }
};

pub type Buffer<T, const CAP: usize, Idx = DefaultIdx> = crate::Buffer<T, CAP, Idx>;

impl<T, const CAP: usize, Idx: Index + SyncIndex> crate::Buffer<T, CAP, Idx> {
    /// Get both handlers to the buffer.
    ///
    /// Ensure uniqueness by taking a mutable reference
    pub const fn split(
        &mut self,
    ) -> (
        Writer<'_, T, CAP, Idx, Idx::Sync>,
        Reader<'_, T, CAP, Idx, Idx::Sync>,
    ) {
        (Writer::new(self, Idx::SYNC), Reader::new(self, Idx::SYNC))
    }

    /// Get a writing handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer(&self) -> Writer<'_, T, CAP, Idx, Idx::Sync> {
        Writer::new(self, Idx::SYNC)
    }

    /// Get a writing handle to this buffer inside a critical section.
    ///
    /// This will avoid declaring unneeded critical sections.
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer_cs<'cs>(
        &self,
        cs: <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>,
    ) -> Writer<'_, T, CAP, Idx, <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>>
    where
        Idx::Sync: DefineCriticalSectionMethods,
    {
        Writer::new(self, cs)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader(&self) -> Reader<'_, T, CAP, Idx, Idx::Sync> {
        Reader::new(self, Idx::SYNC)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader_cs<'cs>(
        &self,
        cs: <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>,
    ) -> Reader<'_, T, CAP, Idx, <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>>
    where
        Idx::Sync: DefineCriticalSectionMethods,
    {
        Reader::new(self, cs)
    }

    fn get_head_and_tail(&self, sync: impl Synchronization<Idx>) -> (usize, usize) {
        sync.free(|cs| {
            let head = cs.get(&self.head);
            let tail = cs.get(&self.tail);

            (head, tail)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;
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
        let mut buf = Buffer::<i32, 4>::new();
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
        let mut buf = Buffer::<i32, 4>::new();
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
        let mut buf = Buffer::<i32, 4>::new();
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
            let mut buf = Buffer::<DropCounter, 4>::new();
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
            let mut buf = Buffer::<DropCounter, 4>::new();
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
        let mut buf = Buffer::<i32, 8>::new();
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
        let mut buf = Buffer::<(), 4>::new();
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
        let mut buf = Buffer::<i32, 16>::new();
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
