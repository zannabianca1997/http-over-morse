//! Synchronize using atomics

use core::sync::atomic::{
    AtomicUsize,
    Ordering::{Acquire, Relaxed, Release},
};

use crate::{ConstDefault, ConstZero, sealed::Sealed, sync::ProvidesIndices};

use super::{Synchronization};

/// Synchronize using atomics
#[derive(Default, Clone, Copy)]
pub struct Atomics;
impl Sealed for Atomics {}
impl ProvidesIndices for Atomics {
    type Usize = AtomicUsize;
    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut()
    }

    unsafe fn get(var: &Self::Usize) -> usize {
        // Relaxed is enough: the caller guarantees exclusive access, so there
        // is no concurrent writer to pair an `Acquire` with. Ordered reads go
        // through `Synchronization::get` instead.
        var.load(Relaxed)
    }
}
impl Synchronization for Atomics {
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

    // Atomics do not care about critical sections
    type CriticalSection<'cs> = Self;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        fun(Atomics)
    }
}
impl ConstDefault for Atomics {
    const DEFAULT: Self = Atomics;
}
impl ConstZero for AtomicUsize {
    const ZERO: Self = AtomicUsize::new(0);
}
