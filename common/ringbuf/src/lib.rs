#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(test)]
extern crate std;

use core::{cell::UnsafeCell, mem::MaybeUninit};

use indices::Index;

pub mod indices;

#[cfg(any(
    target_has_atomic = "ptr",
    target_has_atomic = "8",
    target_has_atomic = "16",
    target_has_atomic = "32",
    target_has_atomic = "64",
    target_has_atomic = "128",
    feature = "avr-device"
))]
pub mod sync;
pub mod unsync;

/// A ring buffer for type T and with capacity CAP
///
/// The only way of interacting with this is through [`Writer`] and [`Reader`]
pub struct Buffer<T, const CAP: usize, Idx: Index> {
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
    head: Idx,
    /// Tail of the buffer
    ///
    /// Points to the oldest unfilled slot.
    ///
    /// Can surpass CAP, as it is wrapped around on access.
    tail: Idx,
}

impl<T, const CAP: usize, Idx: Index> Buffer<T, CAP, Idx> {
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
            );
            assert!(
                CAP - 1 <= Idx::MAX,
                "Idx should be able to represent all indices in the array"
            )
        };

        Self {
            content: UnsafeCell::new(unsafe {
                // Safety: An array of MaybeUninit has no requirements
                MaybeUninit::uninit().assume_init()
            }),
            head: Idx::ZERO,
            tail: Idx::ZERO,
        }
    }
}

/// As we handle concurrency inside, [`Buffer`] can implement [`Sync`].
///
/// Safety: The tail and heads are checked with the [`sync`] primitives, making
/// them already [`Sync`]. The content is not sync, but both the writer and the
/// reader check they have exclusive access to it before interacting. Finally, T
/// must be [`Send`] as we can read it out from another thread.
unsafe impl<T, const CAP: usize, Idx: Index> Sync for Buffer<T, CAP, Idx> where T: Send {}

/// Need to drop all elements still in the buffer
impl<T, const CAP: usize, Idx: Index> Drop for Buffer<T, CAP, Idx> {
    fn drop(&mut self) {
        let mut head = Index::get_mut(&mut self.head);
        let tail = Index::get_mut(&mut self.tail);
        while head != tail {
            let init = &mut self.content.get_mut()[head & (CAP - 1)];

            unsafe {
                // Safety: we know all elements between head and tail have been
                // initialized
                init.assume_init_drop();
            }

            head = head.wrapping_add(1);
        }
        self.head = Idx::ZERO;
        self.tail = Idx::ZERO;
    }
}

mod sealed {
    pub trait Sealed {}
}
