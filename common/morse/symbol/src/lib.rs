#![doc = include_str!("../README.md")]
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Symbol {
    LINE = 0b1000_0000,
    DOT = 0b0000_0000,
}

/// Bits used for the symbol
pub const MASK: u8 = 0b1000_0000;

impl Symbol {
    pub const fn char(self) -> u8 {
        match self {
            Symbol::LINE => b'-',
            Symbol::DOT => b'.',
        }
    }
    pub const fn from_char(ch: u8) -> Option<Self> {
        Some(match ch {
            b'-' => Self::LINE,
            b'.' => Self::DOT,
            _ => return None,
        })
    }
}
