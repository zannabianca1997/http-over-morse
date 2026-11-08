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
    let min_size = (u8::MIN..=u8::MAX)
        .filter_map(&fun)
        .collect::<BTreeSet<T>>()
        .len();

    let Ok(min_size) = u8::try_from(min_size) else {
        // function maps all bytes to different values
        return identity(typ_name, fun);
    };

    let cached_fun: [Option<T>; 256] = array::from_fn(|i| fun(i as u8));

    // max size of an efficient hash
    //
    // over this, just skip hashing and do a plain lookup
    let max_size = cached_fun
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.is_some().then_some(i))
        .last()
        + 1;

    for size in min_size..max_size {
        let found = Prime::all().par_iter().find_map_any(|&modulus| {
            'multipliers: for multiplier in 1u16..u16::MAX {
                let mut mapped: [Option<T>; 256] = array::from_fn(|_| None);

                for input in u8::MIN..=u8::MAX {
                    let hash = hash(multiplier, modulus.get(), size, input);
                    let Some(output) = &cached_fun[input as usize] else {
                        // function has no requirements on this value
                        continue;
                    };

                    if mapped[hash as usize].as_ref().is_some_and(|v| v != output) {
                        // Hash is not a match
                        continue 'multipliers;
                    }
                    mapped[hash as usize] = Some(output.clone())
                }

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
