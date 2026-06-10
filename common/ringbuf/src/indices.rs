//! Methods on indices

use crate::sealed::Sealed;

/// Type that can be used as a index in an buffer
pub trait Index: 'static + Sealed + Sized {
    const ZERO: Self;
    const MAX: usize;
    fn get_mut(&mut self) -> usize;

    /// Wrapping add to a value
    ///
    /// This need to wrap aroung a power of 2
    fn wrapping_add_mut(&mut self, value: usize);

    /// Read the index through a shared reference, without synchronization.
    ///
    /// Compiles to a plain load: no atomics ordering, no critical section.
    ///
    /// # Safety
    /// The caller must guarantee exclusive access to the buffer this index
    /// belongs to (e.g. by holding it through `&mut Buffer`), so that no
    /// other party can be concurrently mutating the index.
    unsafe fn get_unsync(&self) -> usize;
}
