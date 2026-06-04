//! Network addresses

use bytemuck::{Pod, Zeroable, must_cast};

pub use broadcast::BroadcastAddress;
pub use device::{DeviceAddress, this::ThisDeviceAddress};
pub use relevant::RelevantAddress;

mod broadcast;
mod device;
mod relevant;

/// A network address
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
pub struct Address(Option<DeviceAddress>);

impl Address {
    /// Address of this device
    pub const fn this_device() -> Self {
        Self::device(DeviceAddress::this_device())
    }

    /// Address of the given device
    pub const fn device(address: DeviceAddress) -> Self {
        Self(Some(address))
    }

    /// Broadcast address
    pub const fn broadcast() -> Self {
        Self(None)
    }

    /// Get the device address, if any
    pub const fn as_device(&self) -> Option<&DeviceAddress> {
        self.0.as_ref()
    }

    /// Get the broadcast address, if any
    pub const fn as_broadcast(&self) -> Option<&BroadcastAddress> {
        if self.0.is_none() {
            let ptr = self as *const Self as *const BroadcastAddress;
            Some(unsafe {
                // Safety: the None representation is equal to the Broadcast
                // representation (see below const)
                &*ptr
            })
        } else {
            None
        }
    }

    /// If a relevant address, get it
    pub const fn as_relevant(&self) -> Option<&RelevantAddress> {
        if self.0.is_none() || self.0.as_ref().unwrap().as_this_device().is_some() {
            let ptr = self as *const Self as *const RelevantAddress;
            Some(unsafe {
                // Safety: both representations are equal (See under)
                &*ptr
            })
        } else {
            None
        }
    }
}

const _: () = const {
    let [a]: [u8; 1] = must_cast(Address::broadcast());
    let [b]: [u8; 1] = must_cast(BroadcastAddress::new());
    assert!(a == b)
};

const _: () = const {
    let [a]: [u8; 1] = must_cast(Address::broadcast());
    let [b]: [u8; 1] = must_cast(RelevantAddress::broadcast());
    assert!(a == b)
};

const _: () = const {
    let [a]: [u8; 1] = must_cast(Address::this_device());
    let [b]: [u8; 1] = must_cast(RelevantAddress::this_device());
    assert!(a == b)
};
