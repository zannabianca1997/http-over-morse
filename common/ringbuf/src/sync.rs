//! Different types of synchronization

mod sealed {
    pub trait Sealed {}
}

/// Syncronization method
pub trait Synchronization: sealed::Sealed + Copy {
    type Usize: 'static;
    const ZERO: Self::Usize;

    type CriticalSection<'cs>: Synchronization<Usize = Self::Usize>;
    fn free<R>(self, fun: impl FnOnce(Self::CriticalSection<'_>) -> R) -> R;

    fn get(self, var: &Self::Usize) -> usize;
    fn wrapping_add(self, var: &Self::Usize, value: usize);
    fn get_mut(var: &mut Self::Usize) -> &mut usize;
}
/// This sinchronization has significant critical sections, so methods to
/// provide them should be offered
///
/// This avoids polluting the API of `crate::Buffer` when a syncronization
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
