use bytemuck::{NoUninit, Zeroable};

use crate::address::NetAddress;

/// Broadcast address
#[derive(Clone, Copy, PartialEq, Eq, Zeroable, NoUninit, Default)]
#[repr(transparent)]
pub struct BroadcastAddress(Zero);

#[derive(Clone, Copy, PartialEq, Eq, Zeroable, NoUninit, Default)]
#[repr(u8)]
pub(super) enum Zero {
    #[default]
    Zero = 0,
}

impl BroadcastAddress {
    /// Get the broadcast address
    pub const fn new() -> Self {
        Self(Zero::Zero)
    }
}

const _: () = const {
    let align = align_of::<BroadcastAddress>();
    assert!(align == 1)
};
impl crate::address::sealed::Sealed for BroadcastAddress {}
unsafe impl NetAddress for BroadcastAddress {}
