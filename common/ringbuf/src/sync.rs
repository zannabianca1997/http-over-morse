//! Concurrent access to the buffer through separate smart pointers for writing
//! and reader

use crate::{
    ConstDefault,
    indices::{Index, SyncIndex},
};

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

/// Synchronization method
pub trait Synchronization<Idx: Index>: Copy {
    type CriticalSection<'cs>: Synchronization<Idx>;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R;

    fn get(self, var: &Idx) -> usize;
    fn wrapping_add(self, var: &Idx, value: usize);
}
/// This synchronization has significant critical sections, so methods to
/// provide them should be offered
///
/// This avoids polluting the API of `crate::Buffer` when a synchronization
/// without critical sections is used
pub trait DefineCriticalSectionMethods {}

#[cfg(target_has_atomic = "ptr")]
mod atomics;
#[cfg(target_has_atomic = "ptr")]
pub use atomics::Atomics;

#[cfg(feature = "avr-device")]
mod critical_sections;
#[cfg(feature = "avr-device")]
pub use critical_sections::CriticalSections;

pub type DefaultIdx = cfg_select! {
    target_has_atomic = "ptr" => { core::sync::atomic::AtomicUsize }
    feature = "avr-device" => { avr_device::interrupt::Mutex<core::cell::Cell<usize>> }
};

pub type Buffer<T, const CAP: usize, Idx = DefaultIdx> = crate::Buffer<T, CAP, Idx>;

impl<T, const CAP: usize, Idx: Index + SyncIndex> crate::Buffer<T, CAP, Idx> {
    /// Get both handlers to the buffer.
    ///
    /// Ensure uniqueness by taking a mutable reference
    pub const fn split(
        &mut self,
    ) -> (
        Writer<'_, T, CAP, Idx, Idx::Sync>,
        Reader<'_, T, CAP, Idx, Idx::Sync>,
    ) {
        (
            Writer::new(self, ConstDefault::DEFAULT),
            Reader::new(self, ConstDefault::DEFAULT),
        )
    }

    /// Get a writing handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer(&self) -> Writer<'_, T, CAP, Idx, Idx::Sync> {
        Writer::new(self, ConstDefault::DEFAULT)
    }

    /// Get a writing handle to this buffer inside a critical section.
    ///
    /// This will avoid declaring unneeded critical sections.
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer_cs<'cs>(
        &self,
        cs: <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>,
    ) -> Writer<'_, T, CAP, Idx, <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>> {
        Writer::new(self, cs)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader(&self) -> Reader<'_, T, CAP, Idx, Idx::Sync> {
        Reader::new(self, ConstDefault::DEFAULT)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader_cs<'cs>(
        &self,
        cs: <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>,
    ) -> Reader<'_, T, CAP, Idx, <Idx::Sync as Synchronization<Idx>>::CriticalSection<'cs>> {
        Reader::new(self, cs)
    }

    fn get_head_and_tail(&self, sync: impl Synchronization<Idx>) -> (usize, usize) {
        sync.free(|cs| {
            let head = cs.get(&self.head);
            let tail = cs.get(&self.tail);

            (head, tail)
        })
    }
}
