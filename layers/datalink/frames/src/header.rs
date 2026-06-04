use bytemuck::{AnyBitPattern, NoUninit, Zeroable, bytes_of};
use net::ip_address::{Address, BroadcastAddress, DeviceAddress, NetAddress, NetAddressSuperSet};
use pack1::U16LE as U16NE_AVR;

/// Frame header
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Header<ToAddr: NetAddress>(HeaderData<ToAddr>);

impl<ToAddr: NetAddress> Header<ToAddr> {
    pub fn new(from: Address, to: ToAddr, ttl: u8, payload_size: u16) -> Self {
        let mut header = HeaderData {
            from,
            to,
            ttl,
            payload_size: U16NE_AVR::new(payload_size),
            parity: 0,
        };

        // This will flip all the right bits: parity was zero before
        header.parity = parity(&header);

        Self(header)
    }

    pub fn from(&self) -> Address {
        self.0.from
    }

    pub fn to(&self) -> ToAddr {
        self.0.to
    }

    pub fn ttl(&self) -> u8 {
        self.0.ttl
    }

    pub fn payload_size(&self) -> u16 {
        self.0.payload_size.get()
    }

    pub fn upcast<NewToAddr>(self) -> Header<NewToAddr>
    where
        NewToAddr: NetAddressSuperSet<ToAddr>,
    {
        *self.upcast_ref()
    }

    pub fn upcast_ref<NewToAddr>(&self) -> &Header<NewToAddr>
    where
        NewToAddr: NetAddressSuperSet<ToAddr>,
    {
        let ptr = self as *const Self as *const Header<NewToAddr>;
        unsafe {
            // Safety: guarantee by [`NetAddressSuperSet`] guarantees
            &*ptr
        }
    }

    pub fn downcast<NewToAddr>(self) -> Option<Header<NewToAddr>>
    where
        ToAddr: NetAddressSuperSet<NewToAddr>,
        NewToAddr: NetAddress,
    {
        self.downcast_ref().copied()
    }

    pub fn downcast_ref<NewToAddr>(&self) -> Option<&Header<NewToAddr>>
    where
        ToAddr: NetAddressSuperSet<NewToAddr>,
        NewToAddr: NetAddress,
    {
        if self.to().is_::<NewToAddr>() {
            let ptr = self as *const Self as *const Header<NewToAddr>;
            Some(unsafe {
                // Safety: guarantee by [`NetAddressSuperSet`] guarantees
                &*ptr
            })
        } else {
            None
        }
    }

    pub fn downcast_mut<NewToAddr>(&mut self) -> Option<&mut Header<NewToAddr>>
    where
        ToAddr: NetAddressSuperSet<NewToAddr>,
        NewToAddr: NetAddress,
    {
        if self.to().is_::<NewToAddr>() {
            let ptr = self as *mut Self as *mut Header<NewToAddr>;
            Some(unsafe {
                // Safety: guarantee by [`NetAddressSuperSet`] guarantees
                &mut *ptr
            })
        } else {
            None
        }
    }
}

/// Safety: `#[repr(transparent)]`
unsafe impl<ToAddr: NetAddress> NoUninit for Header<ToAddr> where HeaderData<ToAddr>: NoUninit {}
/// Safety: `#[repr(transparent)]`, plus all zero have the correct parity
unsafe impl<ToAddr: NetAddress> Zeroable for Header<ToAddr> where HeaderData<ToAddr>: Zeroable {}
// AnyBitPattern is obviously missing as the parity cannot be checked

/// Frame header with unchecked parity
#[derive(Clone, Copy, AnyBitPattern)]
#[repr(transparent)]
pub struct UnverifiedHeader(HeaderData<Address>);

impl UnverifiedHeader {
    /// Verify parity, and if it is correct return a reference to the verified header
    pub fn verify(&self) -> Option<&Header<Address>> {
        if parity(&self.0) == 0 {
            let ptr = self as *const Self as *const Header<Address>;
            Some(unsafe {
                // Safety: `#[repr(transparent)]` + we checked the parity
                &*ptr
            })
        } else {
            None
        }
    }
}

/// Header with unverified parity
#[derive(Clone, Copy, AnyBitPattern)]
#[repr(C)]
struct HeaderData<ToAddr: NetAddress> {
    /// Address this frame is coming from
    from: Address,
    /// Address this frame is going to
    to: ToAddr,
    /// Time to live
    ttl: u8,
    /// Lenght of the payload
    payload_size: U16NE_AVR,
    /// Header parity
    parity: u8,
}

/// Safety: `#[repr(C)]` mean specified order, and all other fields are checked and have alignement 1
unsafe impl<ToAddr> NoUninit for HeaderData<ToAddr>
where
    ToAddr: NetAddress,
    Address: NetAddress,
    u8: NoUninit,
    U16NE_AVR: NoUninit,
{
}

#[inline(always)]
fn parity<T: NoUninit>(t: &T) -> u8 {
    bytes_of(t).iter().copied().reduce(|a, b| a ^ b).unwrap()
}

/// Check trait needed for usage
const _: () = const {
    const fn ensure_no_uninit<T: NoUninit>() {}
    const fn ensure_any_bit_pattern<T: AnyBitPattern>() {}

    // Deserialization of all packets
    ensure_any_bit_pattern::<UnverifiedHeader>();

    // Serialization of packets to other device and broacasted
    ensure_no_uninit::<Header<BroadcastAddress>>();
    ensure_no_uninit::<Header<DeviceAddress>>();
};
