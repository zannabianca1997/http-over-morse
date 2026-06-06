#![doc = include_str!("../README.md")]
#![no_std]
use zerocopy::{Immutable, IntoBytes};

#[cfg(feature = "configure")]
pub mod configure;

pub use configurable_macros::ConfigStruct;

pub trait ConfigStruct: IntoBytes + Immutable {
    /// Section of the linker where the config is situated
    const SECTION: &str;
    /// Size of the section
    const SIZE: u64;
}
