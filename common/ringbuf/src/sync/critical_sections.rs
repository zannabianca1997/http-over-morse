//! Synchronization through critical sections

use core::cell::Cell;

use avr_device::interrupt::{CriticalSection, Mutex, free};

use crate::sync::CostlyCriticalSection;

use super::{MaybeCriticalSection, Synchronization, sealed::Sealed};

/// Synchronization through critical sections
pub struct CriticalSections;
impl Sealed for CriticalSections {}
impl Synchronization for CriticalSections {
    type Usize = Mutex<Cell<usize>>;
    type CS<'cs> = CriticalSection<'cs>;

    const ZERO: Self::Usize = Mutex::new(Cell::new(0));

    fn free<R>(fun: impl FnOnce(Self::CS<'_>) -> R) -> R {
        free(fun)
    }

    fn get(cs: Self::CS<'_>, var: &Self::Usize) -> usize {
        var.borrow(cs).get()
    }

    fn wrapping_add(cs: Self::CS<'_>, var: &Self::Usize, value: usize) {
        var.borrow(cs).update(|t| t.wrapping_add(value));
    }

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var.get_mut().get_mut()
    }
}

impl<'cs> MaybeCriticalSection<CriticalSections>
    for <CriticalSections as Synchronization>::CS<'cs>
{
    fn free<R>(self, fun: impl FnOnce(<CriticalSections as Synchronization>::CS<'_>) -> R) -> R {
        fun(self)
    }
}

impl Sealed for CriticalSection<'_> {}
impl CostlyCriticalSection for CriticalSection<'_> {}
