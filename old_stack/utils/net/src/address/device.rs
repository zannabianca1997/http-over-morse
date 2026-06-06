//! Address of a device

use core::num::NonZero;

use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub mod this;

/// Address of a device
#[derive(
    Clone, Copy, PartialEq, Eq, TryFromBytes, KnownLayout, IntoBytes, Immutable, Unaligned,
)]
#[repr(transparent)]
pub struct DeviceAddress(pub(super) NonZero<u8>);

impl DeviceAddress {
    /// Get the address of this device
    pub const fn this_device() -> Self {
        Self(NonZero::new(this::ThisAddr::Value as u8).unwrap())
    }
}
