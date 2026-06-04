use bytemuck::{NoUninit, must_cast};

use crate::address::{BroadcastAddress, ThisDeviceAddress};

use super::{broadcast::Zero, device::this::ThisAddr};

/// A network address relevant to this device
#[derive(Clone, Copy, PartialEq, Eq, NoUninit)]
#[repr(transparent)]
pub struct RelevantAddress(RelevantAddr);

#[derive(Clone, Copy, PartialEq, Eq, NoUninit)]
#[repr(u8)]
pub(super) enum RelevantAddr {
    Broadcast = Zero::Zero as u8,
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

    /// Get the broadcast address if any
    pub const fn as_broadcast(&self) -> Option<&BroadcastAddress> {
        if self.0 as u8 == RelevantAddr::Broadcast as u8 {
            let ptr = self as *const Self as *const BroadcastAddress;
            Some(unsafe {
                // Safety: representation is equal to the Broadcast
                // representation (see below const)
                &*ptr
            })
        } else {
            None
        }
    }

    /// Get this device address, if any
    pub const fn as_this_device(&self) -> Option<&ThisDeviceAddress> {
        if self.0 as u8 == RelevantAddr::This as u8 {
            let ptr = self as *const Self as *const ThisDeviceAddress;
            Some(unsafe {
                // Safety: representation is equal to the ThisDeviceAddress
                // representation (see below const)
                &*ptr
            })
        } else {
            None
        }
    }
}

const _: () = const {
    let [a]: [u8; 1] = must_cast(RelevantAddress::broadcast());
    let [b]: [u8; 1] = must_cast(BroadcastAddress::new());
    assert!(a == b)
};

const _: () = const {
    let [a]: [u8; 1] = must_cast(RelevantAddress::this_device());
    let [b]: [u8; 1] = must_cast(ThisDeviceAddress::new());
    assert!(a == b)
};
