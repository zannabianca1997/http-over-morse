//! Different types of synchronization

mod sealed {
    pub trait Sealed {}
}

pub trait Synchronization: sealed::Sealed {
    type Usize;
    type CS<'cs>: MaybeCriticalSection<Self>;

    const ZERO: Self::Usize;

    fn free<R>(fun: impl FnOnce(Self::CS<'_>) -> R) -> R;

    fn get(cs: Self::CS<'_>, var: &Self::Usize) -> usize;
    fn wrapping_add(cs: Self::CS<'_>, var: &Self::Usize, value: usize);

    fn get_mut(var: &mut Self::Usize) -> &mut usize;
}

#[cfg(target_has_atomic = "ptr")]
mod atomics;
#[cfg(target_has_atomic = "ptr")]
pub use atomics::Atomics;

#[cfg(feature = "avr-device")]
mod critical_sections;
#[cfg(feature = "avr-device")]
pub use critical_sections::CriticalSections;

pub type DefaultSynchronization = cfg_select! {
    target_has_atomic = "ptr" => {Atomics}
    feature = "avr-device" => {CriticalSections}
    _ => { compile_error!("No synchronization method available") }
};

pub trait MaybeCriticalSection<Sync: Synchronization + ?Sized>: Copy {
    fn free<R>(self, fun: impl FnOnce(Sync::CS<'_>) -> R) -> R;
}

/// This writer does not have a critical section
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCriticalSection;

impl<Sync: Synchronization> MaybeCriticalSection<Sync> for NoCriticalSection {
    fn free<R>(self, fun: impl FnOnce(Sync::CS<'_>) -> R) -> R {
        Sync::free(fun)
    }
}
