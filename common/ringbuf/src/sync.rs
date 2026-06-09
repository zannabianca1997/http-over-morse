//! Concurrent access to the buffer through separate smart pointers for writing
//! and reader

use crate::{ConstDefault, ProvidesIndices};

mod sealed {
    pub trait Sealed {}
}

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

/// Synchronization method
pub trait Synchronization: sealed::Sealed + Copy + ProvidesIndices {
    type CriticalSection<'cs>: Synchronization<Usize = Self::Usize>;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R;

    fn get(self, var: &Self::Usize) -> usize;
    fn wrapping_add(self, var: &Self::Usize, value: usize);
}
/// This synchronization has significant critical sections, so methods to
/// provide them should be offered
///
/// This avoids polluting the API of `crate::Buffer` when a synchronization
/// without critical sections is used
pub trait DefineCriticalSectionMethods: Synchronization {}

#[cfg(target_has_atomic = "ptr")]
mod atomics;
#[cfg(target_has_atomic = "ptr")]
pub use atomics::Atomics;

#[cfg(feature = "avr-device")]
mod critical_sections;
#[cfg(feature = "avr-device")]
pub use critical_sections::ShortCriticalSections;

pub type DefaultSynchronization = cfg_select! {
    target_has_atomic = "ptr" => {Atomics}
    feature = "avr-device" => {ShortCriticalSections}
    _ => { compile_error!("No synchronization method available") }
};

pub type Buffer<T, const CAP: usize, Sync = DefaultSynchronization> = crate::Buffer<T, CAP, Sync>;

impl<T, const CAP: usize, Sync: ProvidesIndices> crate::Buffer<T, CAP, Sync> {
    /// Get both handlers to the buffer.
    ///
    /// Ensure uniqueness by taking a mutable reference
    pub const fn split(&mut self) -> (Writer<'_, T, CAP, Sync>, Reader<'_, T, CAP, Sync>)
    where
        Sync: ConstDefault + Synchronization,
    {
        (
            Writer::new(self, Sync::DEFAULT),
            Reader::new(self, Sync::DEFAULT),
        )
    }

    /// Get a writing handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer(&self) -> Writer<'_, T, CAP, Sync>
    where
        Sync: ConstDefault + Synchronization,
    {
        Writer::new(self, Sync::DEFAULT)
    }

    /// Get a writing handle to this buffer inside a critical section.
    ///
    /// This will avoid declaring unneeded critical sections.
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer_cs<'cs>(
        &self,
        cs: Sync::CriticalSection<'cs>,
    ) -> Writer<'_, T, CAP, Sync, Sync::CriticalSection<'cs>>
    where
        Sync: DefineCriticalSectionMethods,
    {
        Writer::new(self, cs)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader(&self) -> Reader<'_, T, CAP, Sync>
    where
        Sync: ConstDefault + Synchronization,
    {
        Reader::new(self, Sync::DEFAULT)
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader_cs<'cs>(
        &self,
        cs: Sync::CriticalSection<'cs>,
    ) -> Reader<'_, T, CAP, Sync, Sync::CriticalSection<'cs>>
    where
        Sync: DefineCriticalSectionMethods,
    {
        Reader::new(self, cs)
    }

    fn get_head_and_tail(&self, sync: impl Synchronization<Usize = Sync::Usize>) -> (usize, usize) {
        sync.free(|cs| {
            let head = cs.get(&self.head);
            let tail = cs.get(&self.tail);

            (head, tail)
        })
    }
}
