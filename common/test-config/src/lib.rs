#![no_std]

use ufmt::derive::uDebug;

use configurable::ConfigStruct;
use zerocopy::{Immutable, IntoBytes};

#[derive(ConfigStruct, uDebug, Immutable, IntoBytes)]
#[cfg_attr(feature = "host", derive(serde::Deserialize))]
pub struct Config {
    pub value: u8,
}
