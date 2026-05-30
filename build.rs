use std::{
    cmp::{Reverse, min},
    collections::{BTreeMap, BTreeSet},
    env, fs,
    iter::{self, once},
    num::NonZeroU16,
    path::PathBuf,
    range::Range,
};

const MORSE_DEFINITIONS: &str = "src/morse.txt";

#[path = "src/encoding/symbol.rs"]
mod symbol;
use rand::{distr::StandardUniform, prelude::Rng};
use symbol::*;

#[path = "src/encoding/perfect_hash.rs"]
mod perfect_hash;
use perfect_hash::*;

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

fn main() {
    cargo_emit::rerun_if_changed!(MORSE_DEFINITIONS);

    let definitions = fs::read_to_string(MORSE_DEFINITIONS).expect("Cannot read definition file");
    let definitions = definitions
        .lines()
        .flat_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let Some((ch, def)) = line.split_once(' ') else {
                panic!("All non-empty or comment lines should contain a space")
            };
            assert_eq!(1, ch.len(), "A single char should be before the space");
            let ch = ch.as_bytes()[0];
            if ch != ch.to_ascii_uppercase() {
                cargo_emit::warning!(
                    "Letters are automatically made uppercase, `{}` won't be matched",
                    ch as char
                )
            }

            let symbols = def
                .trim_start()
                .chars()
                .map(|ch| match ch {
                    '.' => DOT,
                    '-' => LINE,
                    _ => panic!("`{ch}` is not a valid morse symbol"),
                })
                .collect::<Vec<_>>();

            Some((ch, symbols))
        })
        .unique_by(|(k, _)| *k);

    let (symbols_slice, definitions) = build_symbols_slice(definitions);

    let static_symbols_slice = static_symbols_slice(&symbols_slice);
    let is_morse_letter = is_morse_letter(&definitions);
    let encode = encode(&definitions);
    let decode = decode(&definitions, &symbols_slice);
    let generated = quote! {
        #static_symbols_slice
        #is_morse_letter
        #encode
        #decode
    };

    let out = PathBuf::from(env::var("OUT_DIR").expect("Cargo should set OUT_DIR"))
        .join("morse.rs")
        .to_str()
        .unwrap()
        .to_owned();

    fs::write(&out, generated.to_string()).expect("Cannot write generated code");
    cargo_emit::rustc_env!("MORSE_GENERATED_RS", "{}", out)
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
    let defined = definitions.iter().map(|(c, _)| c);
    quote! {
        pub(super) const fn is_morse_letter(ch: u8) -> bool {
            match ch {
                #( #defined  ) | * => true,
                _ => false
            }
        }
    }
}

fn encode(definitions: &[(u8, Range<usize>)]) -> TokenStream {
    let definitions = definitions.iter().map(|(c, Range { start, end })| {
        quote! {
            #c => nunny::Slice::new(&SYMBOLS_SLICE[ #start .. #end  ]).unwrap()
        }
    });
    quote! {
        pub(super) const fn encode(ch: u8) -> &'static nunny::Slice<Symbol> {
            match ch {
                #( #definitions  , )*
                _ => unreachable!()
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
            variant: None,
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
        let mut highest_unassigned = 0u8;

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

    let variants = states.values().map(|v| {
        let name = &v.name;
        let variant = v.variant.unwrap();
        quote! { #name = #variant }
    });
    let (digest_lookup, digest_hasher) = {
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

                    let old = lookup.insert(key, next_name);

                    assert!(old.is_none())
                }
            }
        }

        let (
            Hasher {
                multiplier,
                modulus,
            },
            lookup,
        ) = calculate_perfect_hash(lookup);

        let modulus = modulus.get();
        let modulus = quote! { core::num::NonZeroU16::new(#modulus).unwrap() };

        let lookup_items = lookup.into_vec().into_iter().map(|v| {
            if let Some(variant) = v {
                quote! { Some( DecodeState :: #variant ) }
            } else {
                quote! { None }
            }
        });

        (
            quote! { [ #( #lookup_items ),* ] },
            quote! { Hasher { multiplier: #multiplier, modulus: #modulus } },
        )
    };
    let symbols_arms = states.iter().map(
        |(
            _,
            StateData {
                name,
                range: Range { start, end },
                ..
            },
        )| {
            quote! { Self::#name => &SYMBOLS_SLICE[ #start .. #end  ]}
        },
    );
    let not_finals = states
        .values()
        .filter_map(|StateData { name, value, .. }| value.is_none().then_some(name));

    quote! {
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
                match self {
                    #( #symbols_arms ),*
                }
            }

            pub const fn digest(self, s: Symbol) -> Option<Self> {
                ( #digest_lookup )[ ( #digest_hasher ) .hash( u7_and_symbol(self as u8, s) ) as usize ]
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

type RNG = chacha8rand::ChaCha8Rand;
const DEFAULT_PHSEED: [u8;32] = *b"\x42\x0e\x9a\x74\x81\x86\x1d\xe6\xde\x5e\xb8\x30\xe8\x68\xe5\x48\x20\x9d\x11\x01\x3b\xb3\xe9\xbe\x60\xaf\x1c\x36\x6b\xd0\x13\x99";
const DEFAULT_MULTIPLIER: [usize; 2] = [3, 2];

const PRECALCULATED_HASHER: (Hasher, usize) = (
    Hasher {
        multiplier: 0x25bd,
        modulus: NonZeroU16::new(0x39).unwrap(),
    },
    0x38,
);

/// Calculate a perfect hash for a lookup table
///
/// Returns the hasher and the size of the array (the maximum mapped + 1)
fn calculate_perfect_hash<T: Eq>(values: BTreeMap<u8, T>) -> (Hasher, Box<[Option<T>]>) {
    cargo_emit::rerun_if_env_changed!("CALC_HASHER");

    // Starting point -> null hasher
    let (mut best_hasher, mut best_size) = if env::var_os("CALC_HASHER").is_some() {
        (
            Hasher {
                multiplier: 1,
                modulus: NonZeroU16::new(1).unwrap(),
            },
            *values.keys().max().unwrap_or(&0) as usize + 1,
        )
    } else {
        PRECALCULATED_HASHER
    };

    let target = values.len() * DEFAULT_MULTIPLIER[0] / DEFAULT_MULTIPLIER[1];

    let mut hashers = RNG::new(&DEFAULT_PHSEED)
        .sample_iter::<(u16, NonZeroU16), _>(StandardUniform)
        .map(|(multiplier, modulus)| Hasher {
            multiplier,
            modulus,
        });

    'hashers: while best_size > target {
        let hasher = hashers.next().unwrap();
        let mut size = 0;

        let mut mapped_values: [Option<&T>; 256] = [None; _];

        for (key, value) in &values {
            let mapped = hasher.hash(*key);

            let req_size = mapped as usize + 1;
            if req_size >= best_size {
                // Even if perfect, we know of one better, or equivalent!
                continue 'hashers;
            }
            size = size.max(req_size);

            if mapped_values[mapped as usize].is_some_and(|mapped| mapped != value) {
                // Not a perfect hasher ...
                continue 'hashers;
            }
            mapped_values[mapped as usize] = Some(value);
        }

        // Did not exit, so it is a perfect hasher. And it has a better size, or it would have exited before
        best_hasher = hasher;
        best_size = size;
    }

    if env::var_os("CALC_HASHER").is_some() {
        let multiplier = best_hasher.multiplier;
        let modulus = best_hasher.modulus.get();
        let best_size = best_size;
        cargo_emit::warning!(
            "Hasher calculated. Remove `CALC_HASHER` env var and set the constant `PRECALCULATED_HASHER` to `{}`",
            quote! {
                (
                    Hasher {
                        multiplier: #multiplier,
                        modulus: NonZeroU16::new(#modulus).unwrap(),
                    },
                    #best_size,
                )
            }
        );
    }

    let mut table: Box<[Option<T>]> = iter::from_fn(|| Some(None)).take(best_size).collect();

    for (key, value) in values {
        table[best_hasher.hash(key) as usize] = Some(value);
    }

    (best_hasher, table)
}
