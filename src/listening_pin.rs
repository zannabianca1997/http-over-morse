use core::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit};

use arduino_hal::{
    pac::EXINT,
    port::{
        D2,
        mode::{Floating, Input},
    },
};
use avr_device::interrupt::{CriticalSection, Mutex, free};
use embedded_hal::digital::PinState;

mod exint;

pub type Pin = arduino_hal::port::Pin<Input<Floating>, D2>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Rising,
    Falling,
}

/// Callback triggered on pin state change
pub trait ListeningPinCallback {
    /// An edge has been observed
    fn triggered(&mut self, cs: CriticalSection, edge: Edge);
}

/// Pin that has been set up to listen for interrupts
pub struct ListeningPin<CB> {
    _exint: exint::InitForD2Toggle,
    _cb: PhantomData<CB>,
}

impl<CB> ListeningPin<CB> {
    pub fn new(external_interrupts: EXINT, pin: Pin, callback: CB) -> (Self, Interrupts<CB>) {
        let (_exint,) = exint::init(external_interrupts);

        free(|cs| {
            let storage = PIN.borrow(cs);

            let storage = unsafe {
                // Safety: as we are in a critical section, we are the only one
                // with access. The reference will be dropped before the section
                // ends.
                &mut *storage.get()
            };

            storage.write(pin);
        });

        (
            Self {
                _exint,
                _cb: PhantomData,
            },
            Interrupts { callback },
        )
    }

    /// Read the state of the pin
    pub fn state(&self) -> PinState {
        pin_state()
    }
}

static PIN: Mutex<UnsafeCell<MaybeUninit<Pin>>> =
    Mutex::new(UnsafeCell::new(MaybeUninit::uninit()));

fn pin_state() -> PinState
where
    Pin: Sync,
{
    let cs = unsafe {
        // Safety: we need only a shared reference. Given that [`Pin`] is Sync,
        // the only reason we cannot access PIN is the need to set it once. Once
        // set, it could be a simple `static PIN: Pin`, and we are using it as
        // such.
        CriticalSection::new()
    };

    let storage = PIN.borrow(cs);

    let storage = unsafe {
        // Safety: as we are in a critical section, we are the only one
        // with access. The reference will be dropped before the section
        // ends.
        &*storage.get()
    };

    let storage = unsafe {
        // Safety: pin_state is only called from types that can be created only
        // initializing PIN
        storage.assume_init_ref()
    };

    storage.is_high().into()
}

pub struct Interrupts<CB> {
    callback: CB,
}

impl<CB> interrupts::Interrupts for Interrupts<CB>
where
    CB: ListeningPinCallback,
{
    fn int0(&mut self, cs: CriticalSection) {
        // TODO: probably can avoid reading the pin again with a toggle

        let edge = match pin_state() {
            PinState::High => Edge::Rising,
            PinState::Low => Edge::Falling,
        };
        self.callback.triggered(cs, edge);
    }
}
