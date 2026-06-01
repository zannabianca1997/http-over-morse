#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
// Needed for slicing the static storage in a const way
#![feature(const_trait_impl)]
#![feature(const_index)]

mod encoding;
pub mod interrupts;
pub mod listening_pin;
mod segments;
pub mod strategies;
pub mod time;

pub mod from_wire;
pub mod to_wire;
