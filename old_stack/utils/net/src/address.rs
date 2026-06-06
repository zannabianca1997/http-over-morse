//! Network addresses

use core::num::NonZero;

pub use broadcast::BroadcastAddress;
pub use device::{DeviceAddress, this::ThisDeviceAddress};
pub use relevant::RelevantAddress;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

mod broadcast;
mod device;
mod relevant;

/// A network address
#[derive(Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct Address(Option<NonZero<u8>>);

impl Address {
    /// Address of this device
    pub const fn this_device() -> Self {
        Self::device(DeviceAddress::this_device())
    }

    /// Address of the given device
    pub const fn device(address: DeviceAddress) -> Self {
        Self(Some(address.0))
    }

    /// Broadcast address
    pub const fn broadcast() -> Self {
        Self(None)
    }
}
