#![doc = include_str!("../README.md")]
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    LINE,
    DOT,
}
