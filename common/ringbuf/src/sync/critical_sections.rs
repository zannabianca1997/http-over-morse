//! Synchronization through critical sections

use core::cell::Cell;

use avr_device::interrupt::{CriticalSection, Mutex, free};

use crate::{
    ConstDefault, ConstZero,
    sync::{DefineCriticalSectionMethods, ProvidesIndices},
};

use super::{Synchronization, sealed::Sealed};

/// Synchronization through short critical sections around the atomic operations
#[derive(Default, Clone, Copy)]
pub struct ShortCriticalSections;
impl Sealed for ShortCriticalSections {}
impl ProvidesIndices for ShortCriticalSections {
    type Usize = Mutex<Cell<usize>>;

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut().get_mut()
    }
}
impl Synchronization for ShortCriticalSections {
    fn get(self, var: &Self::Usize) -> usize {
        free(|cs| var.borrow(cs).get())
    }

    fn wrapping_add(self, var: &Self::Usize, value: usize) {
        free(|cs| var.borrow(cs).update(|t| t.wrapping_add(value)))
    }

    type CriticalSection<'cs> = CriticalSection<'cs>;

    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        free(fun)
    }
}
impl ConstDefault for ShortCriticalSections {
    const DEFAULT: Self = ShortCriticalSections;
}
impl ConstZero for Mutex<Cell<usize>> {
    const ZERO: Self = Mutex::new(Cell::new(0));
}

impl DefineCriticalSectionMethods for ShortCriticalSections {}

/// Syncronization in a critical section
impl Sealed for CriticalSection<'_> {}
impl ProvidesIndices for CriticalSection<'_> {
    type Usize = Mutex<Cell<usize>>;
    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut().get_mut()
    }
}
impl Synchronization for CriticalSection<'_> {
    type CriticalSection<'cs> = CriticalSection<'cs>;

    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R {
        fun(self)
    }

    fn get(self, var: &Self::Usize) -> usize {
        var.borrow(self).get()
    }

    fn wrapping_add(self, var: &Self::Usize, value: usize) {
        var.borrow(self).update(|t| t.wrapping_add(value))
    }
}
