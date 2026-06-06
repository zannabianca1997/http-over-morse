#![doc = include_str!("../README.md")]
#![no_std]
#![feature(offset_of_enum)]

mod events;
mod traits;

pub mod from_physical;
pub mod to_physical;
