//! Network addresses

use bytemuck::{NoUninit, Pod, Zeroable, must_cast};

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
}

const _: () = const {
    let align = align_of::<Address>();
    assert!(align == 1)
};
impl crate::address::sealed::Sealed for Address {}
unsafe impl NetAddress for Address {}
unsafe impl NetAddressSuperSet<DeviceAddress> for Address {
    fn is(&self) -> bool {
        self.0.is_some()
    }
}
unsafe impl NetAddressSuperSet<BroadcastAddress> for Address {
    fn is(&self) -> bool {
        self.0.is_none()
    }
}
unsafe impl NetAddressSuperSet<ThisDeviceAddress> for Address {
    fn is(&self) -> bool {
        self.downcast_ref::<DeviceAddress>()
            .is_some_and(|d| d.is_::<ThisDeviceAddress>())
    }
}
unsafe impl NetAddressSuperSet<RelevantAddress> for Address {
    fn is(&self) -> bool {
        self.is_::<ThisDeviceAddress>() || self.is_::<BroadcastAddress>()
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

mod sealed {
    pub trait Sealed {}
}

/// Trait implemented by all addresses
///
/// Also guarantee that alignement is 1
pub unsafe trait NetAddress: Sized + Copy + NoUninit + sealed::Sealed + 'static {
    fn upcast<Super>(self) -> Super
    where
        Super: NetAddressSuperSet<Self>,
    {
        *self.upcast_ref()
    }

    fn upcast_ref<Super>(&self) -> &Super
    where
        Super: NetAddressSuperSet<Self>,
    {
        let ptr = self as *const Self as *const Super;
        unsafe {
            // Safety: the representations are the same
            &*ptr
        }
    }

    fn is_<Sub>(&self) -> bool
    where
        Self: NetAddressSuperSet<Sub>,
        Sub: NetAddress,
    {
        NetAddressSuperSet::<Sub>::is(self)
    }

    fn downcast<Sub>(self) -> Option<Sub>
    where
        Self: NetAddressSuperSet<Sub>,
        Sub: NetAddress,
    {
        self.downcast_ref().copied()
    }

    fn downcast_ref<Sub>(&self) -> Option<&Sub>
    where
        Self: NetAddressSuperSet<Sub>,
        Sub: NetAddress,
    {
        if self.is_::<Sub>() {
            let ptr = self as *const Self as *const Sub;
            Some(unsafe {
                // Safety: the representations are the same
                &*ptr
            })
        } else {
            None
        }
    }

    fn downcast_mut<Sub>(&mut self) -> Option<&mut Sub>
    where
        Self: NetAddressSuperSet<Sub>,
        Sub: NetAddress,
    {
        if self.is_::<Sub>() {
            let ptr = self as *mut Self as *mut Sub;
            Some(unsafe {
                // Safety: the representations are the same, and any
                // representation they set will be a valid representation of
                // this
                &mut *ptr
            })
        } else {
            None
        }
    }
}

/// Trait implemented by a superset of addresses
///
/// The bit representations must be a strict superset of sub, and they have the same size
pub unsafe trait NetAddressSuperSet<Sub>: NetAddress
where
    Sub: NetAddress,
{
    #[doc(hidden)]
    fn is(&self) -> bool;
}
