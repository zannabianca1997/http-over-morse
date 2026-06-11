#![doc = include_str!("../README.md")]
#![no_std]

use core::fmt::Debug;

include! { env!("GENERATED_RS") }

impl Prime {
    /// Check a value for primality and return it
    pub const fn new(value: u16) -> Option<Self> {
        // Adapted from rust core lib

        let mut size = COUNT;
        let mut base = 0usize;

        // This loop intentionally doesn't have an early exit if the comparison
        // returns Equal. We want the number of loop iterations to depend *only*
        // on the size of the input slice so that the CPU can reliably predict
        // the loop count.
        while size > 1 {
            let half = size / 2;
            let mid = base + half;

            unsafe {
                // SAFETY: made safe by:
                // `mid < COUNT`: `mid = COUNT / 2 + COUNT / 4 + COUNT / 8 ...`
                core::hint::assert_unchecked(mid < COUNT);
            }
            let cmp = value > ALL[mid] as u16;

            // Binary search interacts poorly with branch prediction, so force
            // the compiler to use conditional moves if supported by the target
            // architecture.
            // TODO: make this select unpredictabily when that's const
            base = if cmp { base } else { mid };

            // This is imprecise in the case where `size` is odd and the
            // comparison returns Greater: the mid element still gets included
            // by `size` even though it's known to be larger than the element
            // being searched for.
            //
            // This is fine though: we gain more performance by keeping the
            // loop iteration count invariant (and thus predictable) than we
            // lose from considering one additional element.
            size -= half;
        }

        unsafe {
            // SAFETY: base is always in [0, size) because base <= mid.
            core::hint::assert_unchecked(base < COUNT);
        }
        let found = ALL[base];
        if found as u16 == value {
            Some(found)
        } else {
            None
        }
    }

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
