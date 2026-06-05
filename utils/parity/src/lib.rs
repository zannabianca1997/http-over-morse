#![no_std]

use core::ops::Deref;

use bytemuck::TransparentWrapper;
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, Unaligned};

/// Type with checked parity byte
#[derive(
    Clone, Copy, Immutable, IntoBytes, KnownLayout, Unaligned, FromZeros, TransparentWrapper,
)]
#[repr(transparent)]
pub struct Checked<T>(WithParity<T>);

impl<T> Checked<T> {
    /// Add a parity byte to an existing instance
    pub fn new(payload: T) -> Self
    where
        T: Immutable + IntoBytes + Unaligned,
    {
        // Set the parity to zero
        let mut with_parity = WithParity { payload, parity: 0 };
        // Calculate parity and set it so the new parity will be zero
        with_parity.parity = parity(zerocopy::transmute_ref!(&with_parity));
        
        Self(with_parity)
    }

    /// Remove the parity bit
    pub fn unwrap(this: Self) -> T {
        this.0.payload
    }

    /// Get a reference to the payload
    pub fn payload(this: &Self) -> &T
    where
        T: Immutable,
    {
        &this.0.payload
    }
}

/// We can offer access to T as long as it is not mutable
impl<T> Deref for Checked<T>
where
    T: Immutable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::payload(self)
    }
}

/// Type with unchecked parity byte
#[derive(
    Clone, Copy, Immutable, IntoBytes, KnownLayout, Unaligned, FromBytes, TransparentWrapper,
)]
#[repr(transparent)]
pub struct Unchecked<T>(WithParity<T>);

impl<T> Unchecked<T>
where
    T: Immutable + IntoBytes + Unaligned,
{
    /// Check the parity byte
    ///
    /// Check if the parity byte is correct and then return a checked version of
    /// this type
    pub fn check(self) -> Result<Checked<T>, Self> {
        if parity(zerocopy::transmute_ref!(&self.0)) == 0 {
            Ok(Checked(self.0))
        } else {
            Err(self)
        }
    }

    /// Check the parity byte without copying
    ///
    /// Check if the parity byte is correct and then return a reference to a
    /// checked version of this type
    pub fn check_ref(&self) -> Option<&Checked<T>> {
        if parity(zerocopy::transmute_ref!(&self.0)) == 0 {
            Some(Checked::wrap_ref(&self.0))
        } else {
            None
        }
    }

    /// Check the parity byte without copying
    ///
    /// Check if the parity byte is correct and then return a mutable reference
    /// to a checked version of this type. This is sound as the checked version
    /// cannot modify the payload
    pub fn check_mut(&mut self) -> Option<&mut Checked<T>> {
        if parity(zerocopy::transmute_ref!(&self.0)) == 0 {
            Some(Checked::wrap_mut(&mut self.0))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Immutable, IntoBytes, KnownLayout, Unaligned, FromBytes)]
#[repr(C)]
struct WithParity<T> {
    payload: T,
    parity: u8,
}

fn parity(bytes: &[u8]) -> u8 {
    bytes.iter().copied().reduce(|a, b| a ^ b).unwrap_or(0)
}
