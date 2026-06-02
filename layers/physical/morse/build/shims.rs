use std::{fmt::Debug, num::NonZeroU16};

use serde::{Deserialize, Serialize};

use crate::src::{perfect_hash::Hasher, symbol::Symbol};

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u8::cmp(&(*self as u8), &(*other as u8))
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DOT => write!(f, "DOT"),
            Self::LINE => write!(f, "LINE"),
        }
    }
}

impl Debug for Hasher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hasher")
            .field("multiplier", &self.multiplier)
            .field("modulus", &self.modulus)
            .field("size", &self.size)
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Hasher")]
pub struct SerdeHasher {
    pub multiplier: u16,
    pub modulus: NonZeroU16,
    pub size: NonZeroU16,
}
