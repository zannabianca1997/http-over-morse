use std::{
    collections::{BTreeMap, BTreeSet},
    iter::once,
    num::NonZero,
    range::Range,
};

use anyhow::{Context, Result, bail, ensure};
use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::build::{codegen::variant_name, perfect_hash_search::calculate_perfect_hash};
use crate::src::{
    perfect_hash::Hasher,
    symbol::{
        Symbol::{self, *},
        u7_and_symbol,
    },
};

struct StateData {
    name: Ident,
    value: Option<NonZero<u8>>,
    range: Range<usize>,
    variant: Option<u8>,
}

pub fn decode(
    definitions: &[(NonZero<u8>, Range<usize>)],
    symbols_slice: &[Symbol],
) -> Result<TokenStream> {
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
            variant: Some(value.get()),
        };
        states.insert(symbols.to_owned(), entry);
    }

    if states.len() > 256 {
        bail!("Too many states - #[repr(u8)] is going to fail")
    } else if states.len() > 128 {
        bail!("Too many states - packing for lookup is going to fail")
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
    ensure!(states.values().all(|v| v.variant.is_some_and(|v| v < 128)));

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
        lookup_name: &str,
        hasher_name: &str,
        bound: impl Fn(&Range<usize>) -> u8,
    ) -> Result<TokenStream> {
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
        let result = calculate_perfect_hash(lookup)?;
        Ok(generate_ph_static(
            lookup_name,
            hasher_name,
            quote! { u8 },
            |v| {
                let v = Literal::u8_unsuffixed(v.unwrap_or(0));
                quote! { #v }
            },
            result,
        ))
    }

    let variants = states.values().map(|v| {
        let name = &v.name;
        let variant = v.variant.unwrap();
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
            ensure!(current_state_variant < 128);

            for next in [DOT, LINE] {
                let next_symbols = symbols.iter().copied().chain(once(next)).collect_vec();
                if let Some(StateData {
                    name: next_name, ..
                }) = states.get(&next_symbols)
                {
                    let key = u7_and_symbol(current_state_variant, next);
                    let old = lookup.insert(key, next_name.to_string());
                    ensure!(old.is_none(), "Duplicate digest key for {:?}", key);
                }
            }
        }

        let result = calculate_perfect_hash(lookup)?;
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
        "SYMBOLS_START_LOOKUP",
        "SYMBOLS_START_HASHER",
        |range| range.start as u8,
    )
    .context("Calculating symbols start table")?;
    let symbols_len_static = generate_symbol_bounds_static(
        &states,
        "SYMBOLS_LEN_LOOKUP",
        "SYMBOLS_LEN_HASHER",
        |range| (range.end - range.start) as u8,
    )
    .context("Calculating symbols length table")?;
    let not_finals = states
        .values()
        .filter_map(|StateData { name, value, .. }| value.is_none().then_some(name));

    Ok(quote! {
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
    })
}
