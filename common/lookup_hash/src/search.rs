extern crate std;

use core::{
    array,
    num::{NonZeroU8, NonZeroU16},
};
use primes::Prime;
use proc_macro2::Ident;
use quote::{ToTokens, quote};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use std::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::ToString,
    vec::Vec,
};
use syn::{Expr, ItemStatic, Type, Visibility, parse_quote};

use crate::hash;

pub mod cache;

/// Build a lookup table for the given function
pub fn search<T: Eq + Ord + ToTokens + Send + Serialize + Clone + Sync>(
    typ_name: impl ToTokens,
    fun: impl Fn(u8) -> Option<T> + Send + Sync,
) -> Found {
    // Count the number of different values fun return to limit the search space
    let values: BTreeSet<T> = (u8::MIN..=u8::MAX).filter_map(&fun).collect();

    let Ok(min_size) = u8::try_from(values.len()) else {
        // function maps all bytes to different values
        return identity(typ_name, fun);
    };

    let values: Vec<T> = values.into_iter().collect();

    let cached_fun: [Option<T>; 256] = array::from_fn(|i| fun(i as u8));

    // Refer to values by their index in `values`, so the search loop can
    // compare cheap `u8`s instead of cloning and comparing whole values
    let cached_idx: [Option<u8>; 256] = array::from_fn(|i| {
        cached_fun[i]
            .as_ref()
            .map(|v| values.iter().position(|x| x == v).unwrap() as u8)
    });

    // max size of an efficient hash
    //
    // over this, just skip hashing and do a plain lookup
    let max_size = cached_fun
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.is_some().then_some(i))
        .last()
        .unwrap()
        + 1;

    let mut moduli = Prime::all().as_slice();
    while let Some((first, rest)) = moduli.split_first()
        && first.get() < min_size as u16
    {
        moduli = rest
    }

    for size in usize::from(min_size)..max_size {
        let size = size as u8; // always fits, as max_size is at most 256

        let found = moduli.par_iter().find_map_any(|&modulus| {
            'multipliers: for multiplier in 1u16..modulus.get() {
                let mut mapped: [Option<u8>; 256] = array::from_fn(|_| None);

                for input in u8::MIN..=u8::MAX {
                    let hash = hash(multiplier, modulus.get(), size, input);
                    let Some(idx) = cached_idx[input as usize] else {
                        // function has no requirements on this value
                        continue;
                    };

                    if mapped[hash as usize].is_some_and(|previous| previous != idx) {
                        // Hash is not a match
                        continue 'multipliers;
                    }
                    mapped[hash as usize] = Some(idx)
                }

                // Found a match: build the value mapping, cloning values only now
                let mapped: [Option<T>; 256] =
                    array::from_fn(|slot| mapped[slot].map(|idx| values[idx as usize].clone()));

                return Some((
                    NonZeroU16::new(multiplier).unwrap(),
                    modulus,
                    NonZeroU8::new(size).unwrap(),
                    Box::new(mapped),
                ));
            }
            None
        });

        if let Some((multiplier, modulus, size, mapped)) = found {
            // Found a better-than-identity mapper
            return hashing(typ_name, multiplier, modulus, size, &*mapped);
        }
    }

    // Failed, returning identity
    identity(typ_name, fun)
}

/// Build an hashing table
fn hashing<T: ToTokens + Serialize>(
    typ_name: impl ToTokens,
    multiplier: NonZeroU16,
    modulus: Prime,
    size: NonZeroU8,
    mapped: &[Option<T>; 256],
) -> Found {
    table(
        typ_name,
        multiplier.get(),
        modulus.get(),
        size.get() as usize,
        mapped,
    )
}

/// Build the identity table
fn identity<T: ToTokens + Serialize>(
    typ_name: impl ToTokens,
    fun: impl Fn(u8) -> Option<T> + Send + Sync,
) -> Found {
    let mapped: [Option<T>; 256] = array::from_fn(|i| fun(i as u8));
    table(typ_name, 0, 0, 256, &mapped)
}

fn table<T: ToTokens + Serialize>(
    typ_name: impl ToTokens,
    multiplier: u16,
    modulus: u16,
    size: usize,
    mapped: &[Option<T>; 256],
) -> Found {
    let mapping = mapped
        .into_iter()
        .enumerate()
        .filter_map(|(k, v)| {
            v.as_ref()
                .map(|v| (k as u8, serde_json::to_value(v).unwrap()))
        })
        .collect();

    let mapped = mapped[..size].into_iter().map(|value| {
        if let Some(value) = value {
            quote! { :: core :: mem :: MaybeUninit :: new ( #value ) }
        } else {
            quote! { :: core :: mem :: MaybeUninit :: uninit ( ) }
        }
    });

    let def = parse_quote! {
        const {
            unsafe {
                :: lookup_hash :: LookupTable
                :: < _, #multiplier, #modulus, #size >
                :: new (
                    [ #( #mapped ),* ]
                )
            }
        }
    };

    let typ = parse_quote! {
        :: lookup_hash :: LookupTable
        :: < #typ_name, #multiplier, #modulus, #size >
    };

    Found {
        typ,
        def,
        size,
        mapping,
    }
}

/// Found lookup table
#[derive(Clone)]
pub struct Found {
    /// Actual mapping the function represent
    mapping: BTreeMap<u8, serde_json::Value>,
    /// Type of the lookup table
    typ: Type,
    /// Const definition of the table
    def: Expr,
    /// Size of the table
    size: usize,
}

impl Found {
    /// Definition of the static lookup table
    pub fn static_def(&self, name: Ident, vis: Visibility, doc: impl ToString) -> ItemStatic {
        let doc = doc.to_string();
        let typ = &self.typ;
        let def = &self.def;
        parse_quote! {
            #[doc = #doc]
            #vis static #name : #typ = #def ;
        }
    }

    /// Call to lookup in the table
    pub fn lookup_call(&self, name: Ident, arg: impl ToTokens) -> Expr {
        parse_quote! { #name . lookup ( #arg ) }
    }

    /// Size of the generated table
    pub fn size(&self) -> usize {
        self.size
    }
}
