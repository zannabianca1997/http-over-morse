//! Synchronize using atomics

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

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
        var.load(Relaxed)
    }

    fn wrapping_add(NoCriticalSection: Self::CS<'_>, var: &Self::Usize, value: usize) {
        var.fetch_add(value, Relaxed);
    }

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut()
    }
}
