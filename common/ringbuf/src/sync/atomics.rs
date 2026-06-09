//! Synchronize using atomics

use core::sync::atomic::{
    AtomicUsize,
    Ordering::{Acquire, Release},
};

use crate::StaticDefault;

use super::{Synchronization, sealed::Sealed};

/// Synchronize using atomics
#[derive(Default, Clone, Copy)]
pub struct Atomics;
impl Sealed for Atomics {}
impl Synchronization for Atomics {
    type Usize = AtomicUsize;

    const ZERO: Self::Usize = AtomicUsize::new(0);

    fn get(self, var: &Self::Usize) -> usize {
        // Acquire pairs with the `Release` in `wrapping_add` on the other side:
        // a reader that observes the writer's new `tail` (or vice-versa for
        // `head`) is guaranteed to also see the slot writes that preceded it,
        // giving the happens-before this SPSC buffer relies on.
        var.load(Acquire)
    }

    fn wrapping_add(self, var: &Self::Usize, value: usize) {
        // Release: publish the slot reads/writes performed by this region
        // before the cursor advance becomes visible to the other side.
        var.fetch_add(value, Release);
    }

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut()
    }

    // Atomics do not care about critical sections
    type CriticalSection<'cs> = Self;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        fun(Atomics)
    }
}
impl StaticDefault for Atomics {
    const DEFAULT: Self = Atomics;
}
