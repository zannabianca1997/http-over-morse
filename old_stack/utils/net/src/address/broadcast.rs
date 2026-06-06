use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, Unaligned};

/// Broadcast address
#[derive(
    Clone, Copy, PartialEq, Eq, FromZeros, Immutable, IntoBytes, KnownLayout, Default, Unaligned,
)]
#[repr(transparent)]
pub struct BroadcastAddress(Zero);

#[derive(
    Clone, Copy, PartialEq, Eq, FromZeros, Immutable, IntoBytes, KnownLayout, Default, Unaligned,
)]
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
