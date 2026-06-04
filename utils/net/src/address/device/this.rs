//! Address of this particular device
//!
//! Address is set at compilation time via the "ADDRESS" env var

use bytemuck::NoUninit;

/// This device address
///
/// Value is configured via the "ADDRESS" env var at compilation time
#[derive(Clone, Copy, PartialEq, Eq, NoUninit, Default)]
#[repr(transparent)]
pub struct ThisDeviceAddress(ThisAddr);

#[derive(Clone, Copy, PartialEq, Eq, NoUninit, Default)]
#[repr(u8)]
pub(in crate::address) enum ThisAddr {
    #[default]
    Value = const {
        if let Some(env) = option_env!("ADDRESS") {
            match u8::from_str_radix(env, 10) {
                Ok(0) => panic!("0 is reserved as the broadcast address"),
                Ok(v) => v,
                Err(_) => panic!("Invalid address"),
            }
        } else {
            // Mainly needed for typechecking
            1
        }
    },
}

impl ThisDeviceAddress {
    /// This device address
    pub const fn new() -> Self {
        Self(ThisAddr::Value)
    }
}

const _: () = const { assert!(ThisAddr::Value as u8 != 0) };
