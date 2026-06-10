//! Synchronize using atomics

use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

use crate::{Index, sealed::Sealed, sync::SyncIndex};

use super::Synchronization;

/// Synchronize using atomics
#[derive(Default, Clone, Copy)]
pub struct Atomics;

impl Sealed for Atomics {}

macro_rules! define_scalar {
    (
        $ ( $cfg:literal , $atomic:ty , $scalar:ty ; )*
    ) => {
        $(
        #[cfg(target_has_atomic = $cfg)]
        impl Sealed for $atomic {}
        #[cfg(target_has_atomic = $cfg)]
        impl Index for $atomic {
            const ZERO: Self = Self::new(0);
            const MAX: usize = <$scalar as Index>::MAX;
            fn get_mut(&mut self) -> usize {
                Index::get_mut(self.get_mut())
            }
            fn wrapping_add_mut(&mut self, value: usize) {
                Index::wrapping_add_mut(self.get_mut(), value);
            }

            unsafe fn get_unsync(&self) -> usize {
                // Relaxed is enough: the caller guarantees exclusive access, so there
                // is no concurrent writer to pair an `Acquire` with. Ordered reads go
                // through `Synchronization::get` instead.
                Index::get_mut(&mut self.load(Relaxed))
            }
        }
        #[cfg(target_has_atomic = $cfg)]
        impl SyncIndex for $atomic {
            type Sync = Atomics;
            const SYNC: Self::Sync = Atomics;
        }
        #[cfg(target_has_atomic = $cfg)]
        impl Synchronization<$atomic> for Atomics {
            fn get(self, var: &$atomic) -> usize {
                // Acquire pairs with the `Release` in `wrapping_add` on the other side:
                // a reader that observes the writer's new `tail` (or vice-versa for
                // `head`) is guaranteed to also see the slot writes that preceded it,
                // giving the happens-before this SPSC buffer relies on.
                Index::get_mut(&mut var.load(Acquire))
            }

            fn wrapping_add(self, var: &$atomic, value: usize) {
                // Release: publish the slot reads/writes performed by this region
                // before the cursor advance becomes visible to the other side.
                let value: $scalar = if const { usize::BITS >= <$scalar>::BITS } {
                    (value & <$scalar>::MAX as usize) as _
                } else {
                    value as _
                };
                var.fetch_add(value, Release);
            }

            // Atomics do not care about critical sections
            type CriticalSection<'cs> = Self;
            fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
                fun(Atomics)
            }
        }
        )*
    };
}

define_scalar! {
    "ptr", core::sync::atomic::AtomicUsize, usize;
    "8",  core::sync::atomic::AtomicU8, u8;
    "16",  core::sync::atomic::AtomicU16, u16;
    "32",  core::sync::atomic::AtomicU32, u32;
    "64",  core::sync::atomic::AtomicU64, u64;
    "128",  core::sync::atomic::AtomicU128, u128;
}
