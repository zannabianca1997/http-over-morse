use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, Unaligned};

use super::device::this::ThisAddr;

/// A network address relevant to this device
#[derive(Clone, Copy, PartialEq, Eq, Immutable, FromZeros, IntoBytes, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct RelevantAddress(RelevantAddr);

#[derive(Clone, Copy, PartialEq, Eq, Immutable, IntoBytes, KnownLayout, FromZeros, Unaligned)]
#[repr(u8)]
pub(super) enum RelevantAddr {
    Broadcast = 0,
    This = ThisAddr::Value as u8,
}

impl RelevantAddress {
    /// Broadcast address
    pub const fn broadcast() -> Self {
        Self(RelevantAddr::Broadcast)
    }

    /// This device address
    pub const fn this_device() -> Self {
        Self(RelevantAddr::This)
    }
}
