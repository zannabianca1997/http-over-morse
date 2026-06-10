//! Synchronization through critical sections

use core::cell::Cell;

use avr_device::interrupt::{CriticalSection, Mutex, free};

use crate::{
    ConstDefault, Index, indices::SyncIndex, sealed::Sealed, sync::DefineCriticalSectionMethods,
};

use super::Synchronization;

/// Synchronization through critical sections around the atomic operations
#[derive(Default, Clone, Copy)]
pub struct CriticalSections;
impl Sealed for CriticalSections {}
impl Synchronization<Mutex<Cell<usize>>> for CriticalSections {
    fn get(self, var: &Mutex<Cell<usize>>) -> usize {
        free(|cs| var.borrow(cs).get())
    }

    fn wrapping_add(self, var: &Mutex<Cell<usize>>, value: usize) {
        free(|cs| var.borrow(cs).update(|t| t.wrapping_add(value)))
    }

    type CriticalSection<'cs> = CriticalSection<'cs>;

    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        free(fun)
    }
}
impl ConstDefault for CriticalSections {
    const DEFAULT: Self = CriticalSections;
}

impl Sealed for Mutex<Cell<usize>> {}
impl Index for Mutex<Cell<usize>> {
    const ZERO: Self = Mutex::new(Cell::new(0));
    fn get_mut(&mut self) -> &mut usize {
        self.get_mut().get_mut()
    }

    unsafe fn get_unsync(&self) -> usize {
        // Safety: the caller guarantees exclusive access to the buffer, so
        // nothing (main code or ISR) can be mutating this cell concurrently —
        // the plain read a conjured token permits is race-free, and no `cli`
        // needs to be emitted.
        self.borrow(unsafe { CriticalSection::new() }).get()
    }
}
impl SyncIndex for Mutex<Cell<usize>> {
    type Sync = CriticalSections;
}

impl DefineCriticalSectionMethods for CriticalSections {}

/// Syncronization in a critical section
impl Sealed for CriticalSection<'_> {}
impl Synchronization<Mutex<Cell<usize>>> for CriticalSection<'_> {
    type CriticalSection<'cs> = CriticalSection<'cs>;

    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        fun(self)
    }

    fn get(self, var: &Mutex<Cell<usize>>) -> usize {
        var.borrow(self).get()
    }

    fn wrapping_add(self, var: &Mutex<Cell<usize>>, value: usize) {
        var.borrow(self).update(|t| t.wrapping_add(value))
    }
}
