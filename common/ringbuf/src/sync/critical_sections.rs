//! Synchronization through critical sections

use core::cell::Cell;

use avr_device::interrupt::{CriticalSection, Mutex, free};

use crate::{Index, sealed::Sealed, sync::DefineCriticalSectionMethods, sync::SyncIndex};

use super::Synchronization;

/// Synchronization through critical sections around the atomic operations
#[derive(Default, Clone, Copy)]
pub struct CriticalSections;
impl Sealed for CriticalSections {}
impl<Idx: Index + Copy> Synchronization<Mutex<Cell<Idx>>> for CriticalSections {
    fn get(self, var: &Mutex<Cell<Idx>>) -> usize {
        free(|cs| Idx::get_mut(&mut var.borrow(cs).get()))
    }

    fn wrapping_add(self, var: &Mutex<Cell<Idx>>, value: usize) {
        free(|cs| {
            var.borrow(cs).update(|mut t| {
                Idx::wrapping_add_mut(&mut t, value);
                t
            })
        })
    }

    type CriticalSection<'cs> = CriticalSection<'cs>;

    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        free(fun)
    }
}

impl<Idx: Index + Copy> Sealed for Mutex<Cell<Idx>> {}
impl<Idx: Index + Copy> Index for Mutex<Cell<Idx>> {
    const ZERO: Self = Mutex::new(Cell::new(Idx::ZERO));
    const MAX: usize = usize::MAX;
    fn get_mut(&mut self) -> usize {
        Idx::get_mut(self.get_mut().get_mut())
    }
    fn wrapping_add_mut(&mut self, value: usize) {
        Idx::wrapping_add_mut(self.get_mut().get_mut(), value)
    }

    unsafe fn get_unsync(&self) -> usize {
        // Safety: the caller guarantees exclusive access to the buffer, so
        // nothing (main code or ISR) can be mutating this cell concurrently —
        // the plain read a conjured token permits is race-free, and no `cli`
        // needs to be emitted.
        Idx::get_mut(&mut self.borrow(unsafe { CriticalSection::new() }).get())
    }
}
impl<Idx: Index + Copy> SyncIndex for Mutex<Cell<Idx>> {
    type Sync = CriticalSections;
    const SYNC: Self::Sync = CriticalSections;
}

impl DefineCriticalSectionMethods for CriticalSections {}

/// Syncronization in a critical section
impl Sealed for CriticalSection<'_> {}
impl<Idx: Index + Copy> Synchronization<Mutex<Cell<Idx>>> for CriticalSection<'_> {
    type CriticalSection<'cs> = CriticalSection<'cs>;

    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        fun(self)
    }

    fn get(self, var: &Mutex<Cell<Idx>>) -> usize {
        Idx::get_mut(&mut var.borrow(self).get())
    }

    fn wrapping_add(self, var: &Mutex<Cell<Idx>>, value: usize) {
        var.borrow(self).update(|mut t| {
            Idx::wrapping_add_mut(&mut t, value);
            t
        })
    }
}
