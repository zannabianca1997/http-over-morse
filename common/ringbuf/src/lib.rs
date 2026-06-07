#![doc = include_str!("../README.md")]
#![no_std]

use core::{cell::UnsafeCell, mem::MaybeUninit};

use sync::{MaybeCriticalSection, NoCriticalSection, Synchronization};

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion};
pub use writer::Writer;

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
    pub fn split(&mut self) -> (Writer<'_, T, CAP, Sync>, Reader<'_, T, CAP, Sync>) {
        (
            Writer::new(self, NoCriticalSection),
            Reader::new(self, NoCriticalSection),
        )
    }

    /// Get a writing handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer(&self) -> Writer<'_, T, CAP, Sync> {
        Writer::new(self, NoCriticalSection)
    }

    /// Get a writing handle to this buffer while a critical section
    ///
    /// This will enable the handle to avoid declaring unnecessary critical
    /// sections.
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists, even while in a critical section (the other
    /// might be halfway writing)
    pub const unsafe fn writer_cs<'b, 's>(
        &'b self,
        cs: Sync::CS<'s>,
    ) -> Writer<'b, T, CAP, Sync, Sync::CS<'s>> {
        Writer::new(self, cs)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader(&self) -> Reader<'_, T, CAP, Sync> {
        Reader::new(self, NoCriticalSection)
    }

    /// Get a reading handle to this buffer while in a critical section
    ///
    /// This will enable the handle to avoid declaring unnecessary critical
    /// sections.
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader_cs<'b, 's>(
        &'b self,
        cs: Sync::CS<'s>,
    ) -> Reader<'b, T, CAP, Sync, Sync::CS<'s>> {
        Reader::new(self, cs)
    }

    fn get_head_and_tail<CS: MaybeCriticalSection<Sync>>(&self, cs: &CS) -> (usize, usize) {
        cs.free(|cs| {
            let head = Sync::get(cs, &self.head);
            let tail = Sync::get(cs, &self.tail);

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
