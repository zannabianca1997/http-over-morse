#![doc = include_str!("../README.md")]
#![no_std]

mod address;

pub mod ip_address {
    pub use crate::address::*;
}
pub mod mac_address {
    pub use crate::address::*;
}
