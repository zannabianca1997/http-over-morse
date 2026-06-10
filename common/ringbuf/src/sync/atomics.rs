//! Synchronize using atomics

use core::sync::atomic::{
    AtomicUsize,
    Ordering::{Acquire, Relaxed, Release},
};

use crate::{Index, indices::SyncIndex, sealed::Sealed};

use super::Synchronization;

/// Synchronize using atomics
#[derive(Default, Clone, Copy)]
pub struct Atomics;
impl Sealed for Atomics {}
impl Synchronization<AtomicUsize> for Atomics {
    fn get(self, var: &AtomicUsize) -> usize {
        // Acquire pairs with the `Release` in `wrapping_add` on the other side:
        // a reader that observes the writer's new `tail` (or vice-versa for
        // `head`) is guaranteed to also see the slot writes that preceded it,
        // giving the happens-before this SPSC buffer relies on.
        var.load(Acquire)
    }

    fn wrapping_add(self, var: &AtomicUsize, value: usize) {
        // Release: publish the slot reads/writes performed by this region
        // before the cursor advance becomes visible to the other side.
        var.fetch_add(value, Release);
    }

    // Atomics do not care about critical sections
    type CriticalSection<'cs> = Self;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        fun(Atomics)
    }
}

impl Sealed for AtomicUsize {}
impl Index for AtomicUsize {
    const ZERO: Self = AtomicUsize::new(0);
    fn get_mut(&mut self) -> usize {
        *self.get_mut()
    }
    fn wrapping_add_mut(&mut self, value: usize) {
        *self.get_mut() = self.get_mut().wrapping_add(value);
    }

    unsafe fn get_unsync(&self) -> usize {
        // Relaxed is enough: the caller guarantees exclusive access, so there
        // is no concurrent writer to pair an `Acquire` with. Ordered reads go
        // through `Synchronization::get` instead.
        self.load(Relaxed)
    }
}
impl SyncIndex for AtomicUsize {
    type Sync = Atomics;
    const SYNC: Self::Sync = Atomics;
}
