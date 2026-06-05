use net::ip_address::{Address, BroadcastAddress, RelevantAddress};

use parity::{Checked, Unchecked};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, little_endian::U16};

pub type Read = Unchecked<FrameHeader<Address>>;
pub type Written = Checked<FrameHeader<Address>>;

/// Frame header
///
/// This is the wire serialization of the header of a frame on the wire
#[derive(Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
pub struct FrameHeader<To> {
    /// Address this frame is coming from
    pub from: Address,
    /// Address this frame is going to
    pub to: To,
    /// Time to live
    pub ttl: u8,
    /// Length of the payload
    payload_size: U16,
}

impl<To> FrameHeader<To> {
    pub fn new(from: Address, to: To, ttl: u8, payload_size: u16) -> Self {
        Self {
            from,
            to,
            ttl,
            payload_size: U16::new(payload_size),
        }
    }

    /// Length of the payload
    pub fn payload_size(&self) -> u16 {
        self.payload_size.get()
    }

    /// Set the length of the payload
    pub fn set_payload_size(&mut self, payload_size: u16) {
        self.payload_size = U16::new(payload_size);
    }
}

impl FrameHeader<Address> {
    /// Detect if this frame is relevant to this device
    pub fn as_relevant(&self) -> Option<&FrameHeader<RelevantAddress>> {
        zerocopy::try_transmute_ref!(self).ok()
    }
}
impl FrameHeader<RelevantAddress> {
    /// Detect if this frame is to be be broadcasted
    pub fn as_broadcast(&self) -> Option<&FrameHeader<BroadcastAddress>> {
        zerocopy::try_transmute_ref!(self).ok()
    }
}
impl FrameHeader<BroadcastAddress> {
    /// Cast this to a general header
    pub fn upcast(self) -> FrameHeader<Address> {
        zerocopy::transmute!(self)
    }
}

/// Check trait needed for usage
const _: () = const {
    const fn ensure_into_bytes<T: IntoBytes + Immutable + Unaligned>() {}
    const fn ensure_from_bytes<T: FromBytes + Immutable + Unaligned>() {}

    // Deserialization of packets
    ensure_from_bytes::<Unchecked<FrameHeader<Address>>>();

    // Serialization of packets
    ensure_into_bytes::<Checked<FrameHeader<Address>>>();
};

/// Check we did choose the right endianness
///
/// As we want to compile this package for multiple devices, we want to specify
/// the endianness. But also make so that on the less powerful (the Nano) the
/// endianness matches the native.
#[cfg(target_arch = "avr")]
const _: U16 = zerocopy::native_endian::U16::new(0);
