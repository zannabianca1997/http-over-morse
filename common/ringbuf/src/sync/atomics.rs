//! Synchronize using atomics

use core::sync::atomic::{
    AtomicUsize,
    Ordering::{Acquire, Release},
};

use crate::sync::NoCriticalSection;

use super::{Synchronization, sealed::Sealed};

/// Synchronize using atomics
pub struct Atomics;
impl Sealed for Atomics {}
impl Synchronization for Atomics {
    type Usize = AtomicUsize;
    type CS<'cs> = NoCriticalSection;

    const ZERO: Self::Usize = AtomicUsize::new(0);

    fn free<R>(fun: impl FnOnce(Self::CS<'_>) -> R) -> R {
        fun(NoCriticalSection)
    }

    fn get(NoCriticalSection: Self::CS<'_>, var: &Self::Usize) -> usize {
        // Acquire pairs with the `Release` in `wrapping_add` on the other side:
        // a reader that observes the writer's new `tail` (or vice-versa for
        // `head`) is guaranteed to also see the slot writes that preceded it,
        // giving the happens-before this SPSC buffer relies on.
        var.load(Acquire)
    }

    fn wrapping_add(NoCriticalSection: Self::CS<'_>, var: &Self::Usize, value: usize) {
        // Release: publish the slot reads/writes performed by this region
        // before the cursor advance becomes visible to the other side.
        var.fetch_add(value, Release);
    }

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut()
    }
}
