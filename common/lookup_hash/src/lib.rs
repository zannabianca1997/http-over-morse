#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "search")]
extern crate std;

use core::{
    array,
    num::{NonZeroU8, NonZeroU16},
};

use primes::Prime;

/// A u8 to u8 hasher
#[derive(Debug, Clone, Copy, Default)]
pub struct Hasher(Inner);

#[derive(Debug, Clone, Copy, Default)]
enum Inner {
    #[default]
    Identity,
    Hash {
        multiplier: NonZeroU16,
        modulus: Prime,
        size: NonZeroU8,
    },
}

impl Hasher {
    #[doc(hidden)]
    pub const fn identity() -> Self {
        Self(Inner::Identity)
    }
    #[doc(hidden)]
    pub const fn new(multiplier: NonZeroU16, modulus: Prime, size: NonZeroU8) -> Self {
        Self(Inner::Hash {
            multiplier,
            modulus,
            size,
        })
    }

    /// Hash a value
    pub const fn hash(&self, value: u8) -> u8 {
        match self.0 {
            Inner::Identity => value,
            Inner::Hash {
                multiplier,
                modulus,
                size,
            } => {
                let value = (value as u16)
                    .wrapping_mul(multiplier.get())
                    .wrapping_rem_euclid(modulus.get())
                    .wrapping_rem(size.get() as u16);
                unsafe {
                    core::hint::assert_unchecked(value < size.get() as u16);
                }
                value as u8
            }
        }
    }

    /// Size of an array that can be indexed by the mapped values
    pub const fn size(&self) -> usize {
        match self.0 {
            Inner::Identity => u8::MAX as usize + 1,
            Inner::Hash { size, .. } => size.get() as usize,
        }
    }

    /// Check if this hash has no collision on the given function
    ///
    /// That means that all inputs with different output are mapped on different hashes
    #[cfg(feature = "search")]
    pub fn is_perfect<T: Eq>(&self, fun: impl Fn(u8) -> T) -> bool {
        let mut mapped: [Option<T>; 256] = array::from_fn(|_| None);
        for input in u8::MIN..=u8::MAX {
            let hash = self.hash(input);
            let output = fun(hash);

            if mapped[hash as usize].as_ref().is_some_and(|v| v != &output) {
                return false;
            }
            mapped[hash as usize] = Some(output)
        }
        true
    }

    /// Find a perfect hasher for the given function
    ///
    /// This will run a comprehensive search on the whole space of hashers and
    /// return an hasher with the smallest possible `size`
    #[cfg(feature = "search")]
    pub fn search<T: Eq + Ord>(&self, fun: impl Fn(u8) -> T + Send + Sync) -> Self {
        use std::collections::BTreeSet;

        // Count the number of different values fun return to limit the search space
        let min_size = (u8::MIN..=u8::MAX).map(&fun).collect::<BTreeSet<T>>().len();

        let Ok(min_size) = u8::try_from(min_size) else {
            // function maps all bytes to different values
            return Self(Inner::Identity);
        };

        for size in min_size..=u8::MAX {
            let size = NonZeroU8::new(size).unwrap();

            use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

            let hasher = Prime::all().par_iter().find_map_any(|&modulus| {
                for multiplier in 1u16..u16::MAX {
                    let multiplier = NonZeroU16::new(multiplier).unwrap();

                    let hasher = Hasher(Inner::Hash {
                        multiplier,
                        modulus,
                        size,
                    });

                    if hasher.is_perfect(&fun) {
                        return Some(hasher);
                    }
                }
                None
            });

            if let Some(hasher) = hasher {
                return hasher;
            }
        }

        // Failed, returning identitity
        Self(Inner::Identity)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Hasher {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        match self.0 {
            Inner::Identity => quote! {
                const { :: lookup_hash :: Hasher :: identity () }
            },
            Inner::Hash {
                multiplier,
                modulus,
                size,
            } => {
                let multiplier = {
                    let value = multiplier.get();
                    quote! { :: core :: num :: nonzero :: NonZeroU16 :: new ( #value ) . unwrap ( ) }
                };
                let modulus = {
                    let value = modulus.get();
                    quote! { :: primes :: Prime :: new ( #value ) . unwrap ( ) }
                };
                let size = {
                    let value = size.get();
                    quote! { :: core :: num :: nonzero :: NonZeroU8 :: new ( #value ) . unwrap ( ) }
                };

                quote! { const { :: lookup_hash :: Hasher :: new ( #multiplier, #modulus, #size ) } }
            },
        }
        .to_tokens(tokens)
    }
}
