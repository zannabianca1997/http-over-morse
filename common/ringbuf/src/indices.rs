//! Methods on indices

use crate::{sealed::Sealed, sync::Synchronization};

/// Const trait returning the zero of a type
pub trait Index: 'static + Sealed + Sized {
    const ZERO: Self;
    fn get_mut(&mut self) -> usize;
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
pub trait SyncIndex: Index {
    type Sync: Synchronization<Self>;
    const SYNC: Self::Sync;
}
