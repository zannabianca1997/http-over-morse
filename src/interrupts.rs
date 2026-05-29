use core::{cell::UnsafeCell, mem::MaybeUninit};

use avr_device::interrupt::{CriticalSection, Mutex, free};

pub trait Interrupts {
    fn timer0compa(&mut self, _cs: CriticalSection) {}
}

impl Interrupts for () {}
impl<A> Interrupts for (A,)
where
    A: Interrupts,
{
    fn timer0compa(&mut self, cs: CriticalSection) {
        self.0.timer0compa(cs);
    }
}
impl<A, B> Interrupts for (A, B)
where
    A: Interrupts,
    B: Interrupts,
{
    fn timer0compa(&mut self, cs: CriticalSection) {
        self.0.timer0compa(cs);
        self.1.timer0compa(cs);
    }
}

pub struct InterruptStorage<I> {
    interrupts: Mutex<UnsafeCell<MaybeUninit<I>>>,
}

impl<I> InterruptStorage<I> {
    pub const fn new() -> Self {
        Self {
            interrupts: Mutex::new(UnsafeCell::new(MaybeUninit::uninit())),
        }
    }

    /// Init the interrupt storage
    ///
    /// Safety: Must be called before interrupts are turned on.
    pub unsafe fn init(&self, interrupts: I) {
        free(|cs| {
            let storage = self.interrupts.borrow(cs);

            let storage = unsafe {
                // Safety: as we are in a critical section, we are the only one
                // with access. The reference will be dropped before the section
                // ends.
                &mut *storage.get()
            };

            storage.write(interrupts);
        })
    }

    /// Call an interrupt
    ///
    /// Safety: must be called _only_ from an interrupt.
    /// [`InterruptStorage::init`] must have been called before. If the used
    /// define their own interrupts, they must not call this function again.
    pub unsafe fn trigger<C: Caller>(&self)
    where
        I: Interrupts,
    {
        free(|cs| {
            let storage = self.interrupts.borrow(cs);

            let storage = unsafe {
                // Safety: as we are in a critical section, we are the only one
                // with access. The reference will be dropped before the section
                // ends.
                &mut *storage.get()
            };

            let storage = unsafe {
                // Safety: per function contract,[`InterruptStorage::init`] was called.
                storage.assume_init_mut()
            };

            C::call(storage, cs);
        })
    }
}

mod sealed {
    pub trait Sealed {}
}
pub trait Caller: sealed::Sealed {
    fn call(interrupts: &mut impl Interrupts, cs: CriticalSection);
}

#[allow(non_camel_case_types)]
pub enum TIMER0_COMPA {}
impl sealed::Sealed for TIMER0_COMPA {}
impl Caller for TIMER0_COMPA {
    fn call(interrupts: &mut impl Interrupts, cs: CriticalSection) {
        interrupts.timer0compa(cs);
    }
}
