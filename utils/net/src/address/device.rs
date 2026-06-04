//! Address of a device

use core::num::NonZero;

use bytemuck::{NoUninit, PodInOption, ZeroableInOption, must_cast};

use this::ThisDeviceAddress;

use crate::address::{NetAddress, NetAddressSuperSet};

pub mod this;

/// Address of a device
#[derive(Clone, Copy, PartialEq, Eq, NoUninit)]
#[repr(transparent)]
pub struct DeviceAddress(NonZero<u8>);

/// Safety: `#[repr(transparent)]`
unsafe impl ZeroableInOption for DeviceAddress where NonZero<u8>: ZeroableInOption {}
/// Safety: `#[repr(transparent)]`
unsafe impl PodInOption for DeviceAddress where NonZero<u8>: PodInOption {}

impl DeviceAddress {
    /// Get the address of this device
    pub const fn this_device() -> Self {
        Self(NonZero::new(this::ThisAddr::Value as u8).unwrap())
    }
}

const _: () = const {
    let [a]: [u8; 1] = must_cast(DeviceAddress::this_device());
    let [b]: [u8; 1] = must_cast(this::ThisDeviceAddress::new());
    assert!(a == b)
};

const _: () = const {
    let align = align_of::<DeviceAddress>();
    assert!(align == 1)
};
impl crate::address::sealed::Sealed for DeviceAddress {}
unsafe impl NetAddress for DeviceAddress {}
unsafe impl NetAddressSuperSet<ThisDeviceAddress> for DeviceAddress {
    fn is(&self) -> bool {
        self.0.get() == this::ThisAddr::Value as u8
    }
}
