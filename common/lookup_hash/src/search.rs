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

/// Slot occupancy sentinel: no class mapped to it yet
///
/// Class indices are always < 255, as the search only runs when the values
/// fit a `u8`, so `u8::MAX` is free for this
const EMPTY: u8 = u8::MAX;

/// Length of the fast-abort probe prefix
///
/// Each additional probe of the biggest class survives only with probability
/// ~1/size, so a dozen is plenty to weed out almost every multiplier before
/// the full sweep
const FAST_PROBES: usize = 12;

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

    let (order, fast) = probe_plan(&cached_idx);

    // Multipliers must be scanned over the whole `u16` range: the runtime
    // hash wraps the product to 16 bits *before* reducing it mod `modulus`,
    // so a multiplier cannot be canonicalized to its residue mod `modulus`
    let all_moduli = Prime::all().as_slice();
    let mut moduli_start = 0;

    for size in usize::from(min_size)..max_size {
        let size = size as u8; // always fits, as max_size is at most 256

        // A modulus below the size cannot fill the upper slots, and any
        // solution found with it also solves the smaller `modulus` size,
        // already searched. Skip those
        moduli_start += all_moduli[moduli_start..]
            .iter()
            .take_while(|prime| usize::from(prime.get()) < usize::from(size))
            .count();

        let Some((multiplier, modulus, size, mapped)) =
            scan_moduli(&all_moduli[moduli_start..], size, &cached_idx, &values, &order, fast)
        else {
            continue;
        };

        // Found a better-than-identity mapper
        return hashing(typ_name, multiplier, modulus, size, &mapped, &cached_fun);
    }

    // Failed, returning identity
    identity(typ_name, fun)
}

/// Probe inputs biggest-class-first, so inconsistent multipliers abort after
/// a handful of hashes without ever touching most inputs
///
/// Returns the probe order and the length of the fast-abort prefix: the
/// members of the largest class
fn probe_plan(cached_idx: &[Option<u8>; 256]) -> (Vec<u8>, usize) {
    let mut popularity = [0usize; 255];
    for idx in cached_idx.iter().flatten() {
        popularity[*idx as usize] += 1;
    }

    let mut order: Vec<u8> = (0..=u8::MAX)
        .filter(|&input| cached_idx[input as usize].is_some())
        .collect();
    order.sort_unstable_by_key(|&input| {
        let idx = cached_idx[input as usize].unwrap();
        (core::cmp::Reverse(popularity[usize::from(idx)]), idx, input)
    });

    let fast = popularity[usize::from(cached_idx[order[0] as usize].unwrap())].min(FAST_PROBES);

    (order, fast)
}

/// Clear the slots a failed multiplier touched
fn reset(occupied: &mut Vec<u8>, mapped: &mut [u8; 256]) {
    for slot in occupied.drain(..) {
        mapped[usize::from(slot)] = EMPTY;
    }
}

/// Scan the multipliers for *some* working hash of the given size
///
/// All arithmetic stays on 16 bits, mirroring [`crate::hash`]
fn scan_moduli<T: Clone + Send + Sync>(
    moduli: &[Prime],
    size: u8,
    cached_idx: &[Option<u8>; 256],
    values: &[T],
    order: &[u8],
    fast: usize,
) -> Option<(NonZeroU16, Prime, NonZeroU8, Box<[Option<T>; 256]>)> {
    moduli.par_iter().find_map_any(|&modulus| {
        let modulus_value = modulus.get();

        // Slot occupancies, reset cheaply through `occupied` between
        // multipliers
        let mut mapped = [EMPTY; 256];
        let mut occupied: Vec<u8> = Vec::with_capacity(usize::from(size));

        'multipliers: for multiplier in 1..=u16::MAX {
            // Fast abort: members of the biggest class must all land on the
            // same slot, each one weeds out all but a `1/size` of multipliers
            for &input in &order[..fast] {
                let idx = cached_idx[input as usize].unwrap();
                let slot = usize::from(hash(multiplier, modulus_value, size, input));
                match mapped[slot] {
                    EMPTY => {
                        mapped[slot] = idx;
                        occupied.push(slot as u8);
                    }
                    previous if previous != idx => {
                        reset(&mut occupied, &mut mapped);
                        continue 'multipliers;
                    }
                    _ => {}
                }
            }

            // Slow sweep: a negligible amount of multipliers survive the
            // fast prefix. Walk all inputs ascending, tracking the wrapped
            // product incrementally in plain 16-bit adds
            let mut product = 0u16; // 0 * multiplier
            for input in 0..=u8::MAX {
                if input > 0 {
                    product = product.wrapping_add(multiplier);
                }

                let Some(idx) = cached_idx[input as usize] else {
                    // function has no requirements on this value
                    continue;
                };

                let slot = usize::from(
                    product
                        .wrapping_rem_euclid(modulus_value)
                        .wrapping_rem(u16::from(size)),
                );
                match mapped[slot] {
                    EMPTY => {
                        mapped[slot] = idx;
                        occupied.push(slot as u8);
                    }
                    previous if previous != idx => {
                        reset(&mut occupied, &mut mapped);
                        continue 'multipliers;
                    }
                    _ => {}
                }
            }

            // Found a match: build the value mapping, cloning values only now
            let slots: [Option<u8>; 256] =
                array::from_fn(|slot| (mapped[slot] != EMPTY).then_some(mapped[slot]));
            let mapped: [Option<T>; 256] =
                array::from_fn(|slot| slots[slot].map(|idx| values[usize::from(idx)].clone()));

            return Some((
                NonZeroU16::new(multiplier).unwrap(),
                modulus,
                NonZeroU8::new(size).unwrap(),
                Box::new(mapped),
            ));
        }

        None
    })
}

/// Build an hashing table
fn hashing<T: ToTokens + Serialize>(
    typ_name: impl ToTokens,
    multiplier: NonZeroU16,
    modulus: Prime,
    size: NonZeroU8,
    mapped: &[Option<T>; 256],
    inputs: &[Option<T>; 256],
) -> Found {
    table(
        typ_name,
        multiplier.get(),
        modulus.get(),
        size.get() as usize,
        mapped,
        inputs,
    )
}

/// Build the identity table
fn identity<T: ToTokens + Serialize>(
    typ_name: impl ToTokens,
    fun: impl Fn(u8) -> Option<T> + Send + Sync,
) -> Found {
    let mapped: [Option<T>; 256] = array::from_fn(|i| fun(i as u8));
    table(typ_name, 0, 0, 256, &mapped, &mapped)
}

fn table<T: ToTokens + Serialize>(
    typ_name: impl ToTokens,
    multiplier: u16,
    modulus: u16,
    size: usize,
    mapped: &[Option<T>; 256],
    inputs: &[Option<T>; 256],
) -> Found {
    // The mapping is keyed by the function input, so it is comparable with
    // the mapping rebuilt from the function when querying the cache
    let mapping = inputs
        .iter()
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
        parse_quote! { unsafe { * #name . lookup ( #arg ) } }
    }

    /// Size of the generated table
    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::array;
    use primes::Prime;

    /// Whatever table the scan finds must be consistent with the function
    /// under the exact runtime hash
    #[test]
    fn scan_matches_function() {
        // Three interleaved classes, unconstrained bytes in between: a
        // working table needs both collisions and separations
        let fun = |input: u8| match input % 4 {
            0 => Some(0u8),
            1 => Some(37),
            2 => Some(91),
            _ => None,
        };

        let values: Vec<u8> = {
            let set: BTreeSet<u8> = (u8::MIN..=u8::MAX).filter_map(&fun).collect();
            set.into_iter().collect()
        };
        let cached_idx: [Option<u8>; 256] = array::from_fn(|i| {
            fun(i as u8).map(|v| values.iter().position(|&x| x == v).unwrap() as u8)
        });

        let (order, fast) = probe_plan(&cached_idx);
        // The fast prefix probes only the biggest class, contiguously
        let biggest = cached_idx[usize::from(order[0])].unwrap();
        assert!(
            order[..fast]
                .iter()
                .all(|&input| cached_idx[usize::from(input)] == Some(biggest))
        );

        for size in u16::try_from(values.len()).unwrap()..=32 {
            let Some((multiplier, modulus, _, mapped)) =
                scan_moduli(Prime::all(), size as u8, &cached_idx, &values, &order, fast)
            else {
                continue;
            };

            for input in u8::MIN..=u8::MAX {
                let Some(expected) = fun(input) else {
                    continue;
                };
                let slot = crate::hash(multiplier.get(), modulus.get(), size as u8, input);
                assert_eq!(
                    mapped[usize::from(slot)].as_ref(),
                    Some(&expected),
                    "wrong value for input {input}"
                );
            }
            return;
        }

        panic!("no table found up to size 32");
    }
}
