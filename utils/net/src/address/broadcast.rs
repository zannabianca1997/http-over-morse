use bytemuck::{NoUninit, Zeroable};

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
