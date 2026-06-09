#![doc = include_str!("../README.md")]
#![no_std]

pub mod process;
pub mod sink;
pub mod stream;

pub mod ready;
pub mod slots;

pub mod handle_errors;

pub mod ringbuf;
