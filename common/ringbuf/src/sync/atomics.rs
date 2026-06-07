//! Synchronize using atomics

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use super::{MaybeCriticalSection, Synchronization, sealed::Sealed};

/// Synchronize using atomics
pub struct Atomics;
impl Sealed for Atomics {}
impl Synchronization for Atomics {
    type Usize = AtomicUsize;
    type CS<'cs> = NoCriticalSectionNeeded;

    const ZERO: Self::Usize = AtomicUsize::new(0);

    fn free<R>(fun: impl FnOnce(Self::CS<'_>) -> R) -> R {
        fun(NoCriticalSectionNeeded)
    }

    fn get(NoCriticalSectionNeeded: Self::CS<'_>, var: &Self::Usize) -> usize {
        var.load(Relaxed)
    }

    fn set(NoCriticalSectionNeeded: Self::CS<'_>, var: &Self::Usize, value: usize) {
        var.store(value, Relaxed);
    }

    fn increment(NoCriticalSectionNeeded: Self::CS<'_>, var: &Self::Usize, value: usize) {
        var.fetch_add(value, Relaxed);
    }

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut()
    }
}

/// Atomics do not need critical sections
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCriticalSectionNeeded;

impl<'cs> MaybeCriticalSection<Atomics> for <Atomics as Synchronization>::CS<'cs> {
    fn free<R>(self, fun: impl FnOnce(<Atomics as Synchronization>::CS<'_>) -> R) -> R {
        fun(self)
    }
}
