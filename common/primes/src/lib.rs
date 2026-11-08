#![doc = include_str!("../README.md")]
#![no_std]

use core::fmt::Debug;

include! { env!("GENERATED_RS") }

impl Prime {
    /// Obtain the u16 value
    pub const fn get(self) -> u16 {
        self as _
    }

    /// Get a slice of all primes, in sorted order
    pub const fn all() -> &'static [Self; COUNT] {
        &ALL
    }
}

impl PartialEq for Prime {
    fn eq(&self, other: &Self) -> bool {
        u16::eq(&self.get(), &other.get())
    }
}
impl Eq for Prime {}

impl PartialOrd for Prime {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        u16::partial_cmp(&self.get(), &other.get())
    }
}
impl Ord for Prime {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        u16::cmp(&self.get(), &other.get())
    }
}

impl Debug for Prime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.get().fmt(f)
    }
}
