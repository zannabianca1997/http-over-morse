use std::{
    cmp::{Reverse, min},
    collections::{BTreeMap, BTreeSet},
    env, fs,
    hash::Hash,
    io,
    iter::{self, once},
    num::NonZeroU16,
    path::PathBuf,
    range::Range,
};

use anyhow::{Context, Result};
use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use build::primes::PRIMES;
use src::{perfect_hash::*, symbol::*};

use crate::build::grammar::Grammar;

mod src {
    //! Things imported from the source, plus some implementations that would
    //! not make sense in the avr world

    pub mod perfect_hash;
    pub mod symbol;

    impl PartialOrd for symbol::Symbol {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(Self::cmp(self, other))
        }
    }

    impl Ord for symbol::Symbol {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            u8::cmp(&(*self as u8), &(*other as u8))
        }
    }

    impl std::fmt::Debug for symbol::Symbol {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::DOT => write!(f, "DOT"),
                Self::LINE => write!(f, "LINE"),
            }
        }
    }

    impl std::fmt::Debug for perfect_hash::Hasher {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Hasher")
                .field("multiplier", &self.multiplier)
                .field("modulus", &self.modulus)
                .field("size", &self.size)
                .finish()
        }
    }
}

mod build {
    //! Bits and pieces of the build script, organized in a separate dir

    pub mod grammar;
    pub mod primes;
}

fn main() -> Result<()> {
    let Grammar { letters } = Grammar::load()?;

    let (symbols_slice, definitions) = build_symbols_slice(letters);

    let static_symbols_slice = static_symbols_slice(&symbols_slice);
    let is_morse_letter = is_morse_letter(&definitions);
    let decode = decode(&definitions, &symbols_slice);
    let generated = quote! {
        #static_symbols_slice
        #is_morse_letter
        #decode
    };

    let out = PathBuf::from(env::var("OUT_DIR").context("Cargo should set OUT_DIR")?)
        .join("morse.rs")
        .to_str()
        .context("Non unicode paths are unsupported")?
        .to_owned();

    fs::write(&out, generated.to_string()).context("Cannot write generated code")?;
    cargo_emit::rustc_env!("MORSE_GENERATED_RS", "{}", out);

    Ok(())
}

/// Build a supersequence to use less memory
fn build_symbols_slice(
    definitions: impl IntoIterator<Item = (u8, Vec<Symbol>)>,
) -> (Vec<Symbol>, Vec<(u8, Range<usize>)>) {
    let mut supersequence = vec![];
    let mut ranges = vec![];

    for (ch, mut symbols) in definitions
        .into_iter()
        // Taking longer sequences first, so subsequences are guaranteed to
        // arrive later
        .sorted_by_key(|(_, s)| Reverse(s.len()))
    {
        if let Some((start, _)) = supersequence
            .windows(symbols.len())
            .enumerate()
            .find(|(_, window)| window == &symbols)
        {
            let end = start + symbols.len();

            ranges.push((ch, (start..end).into()));
        } else {
            let overlap = (0..=min(symbols.len(), supersequence.len()))
                .rev()
                .find(|overlap| {
                    &supersequence[supersequence.len() - overlap..] == &symbols[..*overlap]
                })
                .unwrap_or(0);

            let start = supersequence.len() - overlap;
            supersequence.extend_from_slice(&mut symbols[overlap..]);
            let end = supersequence.len();

            ranges.push((ch, (start..end).into()));
        }
    }

    (supersequence, ranges)
}

fn static_symbols_slice(symbols_slice: &[Symbol]) -> TokenStream {
    let len = symbols_slice.len();
    let symbols_slice = symbols_slice.into_iter().map(|s| match s {
        DOT => quote! { Symbol::DOT },
        LINE => quote! { Symbol::LINE },
    });

    quote! {
        /// Supersequence of symbols, guaranteed to contains all morse sequences as subsequences
        static SYMBOLS_SLICE: [ Symbol ; #len ] = [
            #( #symbols_slice ),*
        ];
    }
}

fn is_morse_letter(definitions: &[(u8, Range<usize>)]) -> TokenStream {
    let defined = definitions.iter().map(|(c, _)| Literal::byte_character(*c));
    quote! {
        pub(super) const fn is_morse_letter(ch: u8) -> bool {
            match ch {
                #( #defined  ) | * => true,
                _ => false
            }
        }
    }
}

fn variant_name(symbols: &[Symbol]) -> Ident {
    format_ident!(
        "{}",
        symbols
            .iter()
            .map(|s| match s {
                LINE => "Line",
                DOT => "Dot",
            })
            .join("")
    )
}

fn decode(definitions: &[(u8, Range<usize>)], symbols_slice: &[Symbol]) -> TokenStream {
    struct StateData {
        name: Ident,
        value: Option<u8>,
        range: Range<usize>,
        variant: Option<u8>,
    }

    let mut states = BTreeMap::from([(
        vec![],
        StateData {
            name: format_ident!("Empty"),
            value: None,
            range: (0..0).into(),
            variant: Some(0),
        },
    )]);
    for (value, range) in definitions {
        let symbols = &symbols_slice[*range];

        for i in 1..symbols.len() {
            states
                .entry(symbols[..i].to_vec())
                .or_insert_with_key(|key| StateData {
                    name: variant_name(key),
                    value: None,
                    range: (0..i).into(),
                    variant: None,
                });
        }
        let entry = StateData {
            name: variant_name(symbols),
            value: Some(*value),
            range: *range,
            variant: Some(*value),
        };
        states.insert(symbols.to_owned(), entry);
    }

    if states.len() > 256 {
        panic!("Too many states - #[repr(u8)] is going to fail")
    } else if states.len() > 128 {
        panic!("Too many states - packing for lookup is going to fail")
    }

    {
        let assigned = BTreeSet::from_iter(
            states
                .values()
                .filter_map(|StateData { variant, .. }| *variant),
        );
        let mut highest_unassigned = 1u8;

        for unassigned in states
            .values_mut()
            .filter_map(|StateData { variant, .. }| variant.is_none().then_some(variant))
        {
            while assigned.contains(&highest_unassigned) {
                highest_unassigned += 1
            }
            *unassigned = Some(highest_unassigned);
            highest_unassigned += 1
        }
    }

    let states = states;
    assert!(states.values().all(|v| v.variant.is_some_and(|v| v < 128)));

    fn generate_ph_static<T>(
        lookup_name: &str,
        hasher_name: &str,
        type_expr: TokenStream,
        to_token: impl Fn(Option<T>) -> TokenStream,
        (hash_result, lookup): (Hasher, Box<[Option<T>]>),
    ) -> TokenStream {
        let lookup_ident = format_ident!("{lookup_name}");
        let hasher_ident = format_ident!("{hasher_name}");
        let len = lookup.len();
        let lookup_items = lookup.into_vec().into_iter().map(to_token);
        let Hasher {
            multiplier,
            modulus,
            size,
        } = hash_result;
        let modulus = {
            let m = modulus.get();
            quote! { core::num::NonZeroU16::new(#m).unwrap() }
        };
        let size = {
            let s = size.get();
            quote! { core::num::NonZeroU16::new(#s).unwrap() }
        };
        quote! {
            static #lookup_ident: [#type_expr; #len] = [ #( #lookup_items ),* ];
            const #hasher_ident: Hasher = Hasher { multiplier: #multiplier, modulus: #modulus, size: #size };
        }
    }

    fn generate_symbol_bounds_static(
        states: &BTreeMap<Vec<Symbol>, StateData>,
        table_variant: PHTables,
        lookup_name: &str,
        hasher_name: &str,
        bound: impl Fn(&Range<usize>) -> u8,
    ) -> TokenStream {
        let mut lookup = BTreeMap::new();
        for StateData {
            variant: current_state_variant,
            range,
            ..
        } in states.values()
        {
            let key = current_state_variant.unwrap();
            lookup.insert(key, bound(range));
        }
        let result = calculate_perfect_hash(table_variant, lookup);
        generate_ph_static(
            lookup_name,
            hasher_name,
            quote! { u8 },
            |v| {
                let v = Literal::u8_unsuffixed(v.unwrap_or(0));
                quote! { #v }
            },
            result,
        )
    }

    let variants = states.values().map(|v| {
        let name = &v.name;
        let variant = v.variant.unwrap();
        // Making the printout a little nicer
        let variant = if v.value.is_some() {
            Literal::byte_character(variant)
        } else {
            Literal::u8_unsuffixed(variant)
        };
        quote! { #name = #variant }
    });

    let digest_static = {
        let mut lookup = BTreeMap::new();

        for (
            symbols,
            StateData {
                variant: current_state_variant,
                ..
            },
        ) in states.iter()
        {
            let current_state_variant = current_state_variant.unwrap();
            assert!(current_state_variant < 128);

            for next in [Symbol::DOT, Symbol::LINE] {
                let next_symbols = symbols.iter().copied().chain(once(next)).collect_vec();
                if let Some(StateData {
                    name: next_name, ..
                }) = states.get(&next_symbols)
                {
                    let key = u7_and_symbol(current_state_variant, next);
                    let old = lookup.insert(key, next_name.to_string());
                    assert!(old.is_none())
                }
            }
        }

        let result = calculate_perfect_hash(PHTables::Digest, lookup);
        generate_ph_static(
            "DIGEST_LOOKUP",
            "DIGEST_HASHER",
            quote! { Option<DecodeState> },
            |v| {
                if let Some(variant) = v {
                    let variant = format_ident!("{variant}");
                    quote! { Some( DecodeState :: #variant ) }
                } else {
                    quote! { None }
                }
            },
            result,
        )
    };
    let symbols_start_static = generate_symbol_bounds_static(
        &states,
        PHTables::SymbolsStart,
        "SYMBOLS_START_LOOKUP",
        "SYMBOLS_START_HASHER",
        |range| range.start as u8,
    );
    // let symbols_end_static = generate_symbol_bounds_static(
    //     &states,
    //     PHTables::SymbolsEnd,
    //     "SYMBOLS_END_LOOKUP",
    //     "SYMBOLS_END_HASHER",
    //     |range| range.end as u8,
    // );
    let symbols_len_static = generate_symbol_bounds_static(
        &states,
        PHTables::SymbolsLength,
        "SYMBOLS_LEN_LOOKUP",
        "SYMBOLS_LEN_HASHER",
        |range| (range.end - range.start) as u8,
    );
    let not_finals = states
        .values()
        .filter_map(|StateData { name, value, .. }| value.is_none().then_some(name));

    quote! {
        #digest_static
        #symbols_start_static
        // #symbols_end_static
        #symbols_len_static

        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub(super) enum DecodeState {
            #(#variants),*
        }

        impl DecodeState {
            pub const fn new() -> Self {
                Self::Empty
            }

            pub const fn is_empty(&self) -> bool {
                matches!(self, Self::Empty)
            }

            pub const fn symbols(&self) -> &'static [Symbol] {
                let start = SYMBOLS_START_LOOKUP[SYMBOLS_START_HASHER.hash(*self as u8) as usize] as usize;
                let end = start + SYMBOLS_LEN_LOOKUP[SYMBOLS_LEN_HASHER.hash(*self as u8) as usize] as usize;
                unsafe {
                    // Safety: we generated those limits and are inside the string
                    SYMBOLS_SLICE.get_unchecked(start .. end)
                }
            }

            pub const fn digest(self, s: Symbol) -> Option<Self> {
                DIGEST_LOOKUP[ DIGEST_HASHER.hash( u7_and_symbol(self as u8, s) ) as usize ]
            }

            pub const fn finish(self) -> Option<u8> {
                match self {
                    #( Self::#not_finals  )|* => None,
                    final_state => Some(final_state as u8)
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum PHTables {
    Digest,
    SymbolsStart,
    SymbolsEnd,
    SymbolsLength,
}

#[derive(Debug, Serialize, Deserialize)]
struct TableCacheInfo {
    #[serde(flatten)]
    cached: BTreeMap<PHTables, TableCacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TableCacheEntry {
    values: BTreeMap<String, toml::Value>,

    multiplier: u16,
    modulus: NonZeroU16,
    size: NonZeroU16,
}

const PHCACHE_HEADER: &str = "# Perfect Hash Cache
# auto-generated by `build.rs`
#
# Warning: generating this values can be a lengthy process.
# Beware of editing files in the `src` directory aside `lib.rs` if you are not in for a long wait";

impl TableCacheInfo {
    const PATH: &str = ".phcache";

    pub fn load() -> Self {
        cargo_emit::rerun_if_changed!(Self::PATH);
        match fs::read(Self::PATH) {
            Ok(str) => toml::from_slice(&str).expect("Cannot read perfect hash cache"),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                toml::map::Map::new().try_into().unwrap()
            }
            Err(err) => panic!("Cannot read table cache: {}", err),
        }
    }

    pub fn save(self) {
        let contents = toml::to_string(&self).unwrap();
        fs::write(Self::PATH, format!("{PHCACHE_HEADER}\n\n{contents}"))
            .expect("Cannot save perfect hash cache")
    }
}

/// Calculate a perfect hash for a lookup table
///
/// Returns the hasher and the lookup table content
fn calculate_perfect_hash<T: Clone + Eq + Serialize + Hash + Sync>(
    table: PHTables,
    values: BTreeMap<u8, T>,
) -> (Hasher, Box<[Option<T>]>) {
    cargo_emit::rerun_if_env_changed!("CALC_HASHER");

    let calc_hash = env::var_os("CALC_HASHER").is_some();

    let mut cache_info = TableCacheInfo::load();

    if !calc_hash {
        if let Some(cached) = cache_info.cached.get(&table) {
            let current = values
                .iter()
                .map(|(&k, v)| (k.to_string(), toml::Value::try_from(v.clone()).unwrap()))
                .collect::<BTreeMap<_, _>>();

            if current == cached.values {
                let hasher = Hasher {
                    multiplier: cached.multiplier,
                    modulus: cached.modulus,
                    size: cached.size,
                };
                let mut table_data: Box<[Option<T>]> = iter::from_fn(|| Some(None))
                    .take(cached.size.get() as usize)
                    .collect();
                for (key, value) in values {
                    table_data[hasher.hash(key) as usize] = Some(value);
                }
                return (hasher, table_data);
            } else {
                cargo_emit::warning!(
                    "Recalculating perfect hasher - `.phcache` entry refers to different values"
                )
            }
        } else {
            cargo_emit::warning!("Recalculating perfect hasher - `.phcache` entry missing")
        }
    } else {
        cargo_emit::warning!("Recalculating perfect hasher - CALC_HASHER set")
    }

    // Minimum possible size - all distinct values
    let min_size = values.values().unique().count();
    assert!(
        min_size < 256,
        "Refusing to create tabled of more than 256 elements as the identity would be more efficient"
    );

    // Moving the values to an array for faster iteration
    let values: Vec<(u8, T)> = values.into_iter().collect();

    let best_hasher = (min_size..=256).find_map(|size| {
        (1..=u16::MAX)
            .into_par_iter()
            .flat_map(|multiplier| {
                PRIMES.par_iter().rev().copied().map(move |modulus| Hasher {
                    multiplier,
                    modulus: NonZeroU16::new(modulus).unwrap(),
                    size: NonZeroU16::new(size as u16).unwrap(),
                })
            })
            .find_any(|hasher| {
                let mut mapped_values: [Option<&T>; 256] = [None; 256];

                for (key, value) in &values {
                    let mapped = hasher.hash(*key);

                    if mapped_values[mapped as usize].is_some_and(|mapped| mapped != value) {
                        // Collision with different values, this hasher is no good
                        return false;
                    }
                    mapped_values[mapped as usize] = Some(value);
                }

                // Found a good hasher
                true
            })
    });

    let Some(best_hasher) = best_hasher else {
        panic!("No hasher found in search space for {table:?}");
    };

    cache_info.cached.insert(
        table,
        TableCacheEntry {
            values: values
                .iter()
                .map(|(k, v)| (k.to_string(), toml::Value::try_from(v.clone()).unwrap()))
                .collect(),
            size: best_hasher.size,
            multiplier: best_hasher.multiplier,
            modulus: best_hasher.modulus,
        },
    );
    cache_info.save();

    let mut table_data: Box<[Option<T>]> = iter::from_fn(|| Some(None))
        .take(best_hasher.size.get() as usize)
        .collect();

    for (key, value) in values {
        table_data[best_hasher.hash(key) as usize] = Some(value);
    }

    (best_hasher, table_data)
}
