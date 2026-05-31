use std::{
    cmp::{Reverse, max, min},
    collections::{BTreeMap, BTreeSet},
    env, fs,
    hash::Hash,
    io,
    iter::{self, once},
    num::{NonZero, NonZeroU8, NonZeroU16, NonZeroUsize},
    path::PathBuf,
    range::Range,
    time::{Duration, Instant},
};

const MORSE_DEFINITIONS: &str = "src/morse.txt";

#[path = "src/encoding/symbol.rs"]
mod symbol;
use rand::{
    SeedableRng,
    distr::{Distribution, StandardUniform, Uniform, slice::Choose},
    prelude::Rng,
};
use serde::{Deserialize, Serialize};
use symbol::*;

#[path = "src/encoding/perfect_hash.rs"]
mod perfect_hash;
use perfect_hash::*;

use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
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
    let decode = decode(&definitions, &symbols_slice);
    let generated = quote! {
        #static_symbols_slice
        #is_morse_letter
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
    let symbols_end_static = generate_symbol_bounds_static(
        &states,
        PHTables::SymbolsEnd,
        "SYMBOLS_END_LOOKUP",
        "SYMBOLS_END_HASHER",
        |range| range.end as u8,
    );
    let not_finals = states
        .values()
        .filter_map(|StateData { name, value, .. }| value.is_none().then_some(name));

    quote! {
        #digest_static
        #symbols_start_static
        #symbols_end_static

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
                let end = SYMBOLS_END_LOOKUP[SYMBOLS_END_HASHER.hash(*self as u8) as usize] as usize;
                &SYMBOLS_SLICE[start..end]
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

type RNG = chacha8rand::ChaCha8Rand;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum PHTables {
    Digest,
    SymbolsStart,
    SymbolsEnd,
}

const fn default_seed() -> <RNG as SeedableRng>::Seed {
    // Chosen by fair dice roll
    *b"\x42\x0e\x9a\x74\x81\x86\x1d\xe6\xde\x5e\xb8\x30\xe8\x68\xe5\x48\x20\x9d\x11\x01\x3b\xb3\xe9\xbe\x60\xaf\x1c\x36\x6b\xd0\x13\x99"
}

fn default_max_load() -> BTreeMap<PHTables, f32> {
    // Chosen by compilation times
    BTreeMap::from_iter([
        (PHTables::Digest, 1.5),
        (PHTables::SymbolsStart, 2.5),
        (PHTables::SymbolsEnd, 2.),
    ])
}

fn seed_to_hex<S: serde::Serializer>(seed: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error> {
    let mut hex = String::with_capacity(64);
    for byte in seed {
        use std::fmt::Write;
        write!(&mut hex, "{:02x}", *byte).unwrap();
    }
    serializer.serialize_str(&hex)
}

fn seed_from_hex<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
    use serde::de::Error;
    let hex = String::deserialize(deserializer)?;
    if hex.len() != 64 {
        return Err(D::Error::custom(format!(
            "hex seed must be 64 characters, got {}",
            hex.len()
        )));
    }
    let mut seed = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let high = hex_char_to_nibble(chunk[0])
            .map_err(|c| D::Error::custom(format!("invalid hex char: {}", c)))?;
        let low = hex_char_to_nibble(chunk[1])
            .map_err(|c| D::Error::custom(format!("invalid hex char: {}", c)))?;
        seed[i] = high << 4 | low;
    }
    Ok(seed)
}

fn hex_char_to_nibble(c: u8) -> Result<u8, char> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(c as char),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TableCacheInfo {
    #[serde(
        default = "default_seed",
        serialize_with = "seed_to_hex",
        deserialize_with = "seed_from_hex"
    )]
    seed: <RNG as SeedableRng>::Seed,
    #[serde(default = "default_max_load")]
    max_load: BTreeMap<PHTables, f32>,
    #[serde(flatten)]
    cached: BTreeMap<PHTables, TableCacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TableCacheEntry {
    values: BTreeMap<String, toml::Value>,
    max_load: f32,
    multiplier: u16,
    modulus: NonZeroU16,
    size: NonZeroU16,
}

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
        fs::write(
            Self::PATH,
            format!("# Perfect Hash Cache\n# auto-generated by `build.rs`\n\n{contents}"),
        )
        .expect("Cannot save perfect hash cache")
    }
}

mod primes;

/// Calculate a perfect hash for a lookup table
///
/// Returns the hasher and the size of the array (the maximum mapped + 1)
fn calculate_perfect_hash<T: Clone + Eq + Serialize + Hash>(
    table: PHTables,
    values: BTreeMap<u8, T>,
) -> (Hasher, Box<[Option<T>]>) {
    cargo_emit::rerun_if_env_changed!("CALC_HASHER");

    let calc_hash = env::var_os("CALC_HASHER").is_some();

    let mut cache_info = TableCacheInfo::load();

    let max_load = *cache_info.max_load.get(&table).unwrap_or(&3.);

    if !calc_hash {
        if let Some(cached) = cache_info.cached.get(&table) {
            if cached.max_load <= max_load {
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
                cargo_emit::warning!(
                    "Recalculating perfect hasher - `.phcache` entry not satisfactory"
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
    // Max acceptable size - some multiplier of the minimum
    let max_size = (min_size as f32 * max_load).floor() as usize;

    println!("Limits are {} .. {}", min_size, max_size);

    struct HasherDistr(StandardUniform, Choose<'static, u16>, Uniform<u16>);

    impl HasherDistr {
        fn new(size_range: Range<NonZeroU16>) -> Self {
            Self(
                StandardUniform,
                Choose::new(&primes::PRIMES).unwrap(),
                Uniform::new(size_range.start.get(), size_range.end.get()).unwrap(),
            )
        }
    }

    impl Distribution<Hasher> for HasherDistr {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hasher {
            Hasher {
                multiplier: self.0.sample(rng),
                modulus: NonZero::new(*self.1.sample(rng)).unwrap(),
                size: NonZero::new(self.2.sample(rng)).unwrap(),
            }
        }
    }

    let mut hashers = RNG::new(&cache_info.seed).sample_iter::<Hasher, _>(HasherDistr::new(
        (NonZero::new(min_size as _).unwrap()..NonZero::new(max_size as _).unwrap()).into(),
    ));

    let search_start = Instant::now();
    let mut checked = 0usize;

    let best_hasher = 'hashers: loop {
        if search_start.elapsed() > Duration::from_secs(30) {
            panic!("Hasher search taking too long! (checked {checked} hashers)")
        }

        let hasher = hashers.next().unwrap();

        let mut mapped_values: BTreeMap<u16, &T> = BTreeMap::new();

        for (key, value) in &values {
            let mapped = hasher.hash(*key);

            if mapped_values
                .get(&mapped)
                .is_some_and(|mapped| mapped != &value)
            {
                checked += 1;

                // Collision with different values, this hasher is no good
                continue 'hashers;
            }
            mapped_values.insert(mapped, value);
        }

        // Found a good hasher
        break hasher;
    };

    println!(
        "Found best hasher: mul: {} mod: {} siz: {}",
        best_hasher.multiplier, best_hasher.modulus, best_hasher.size
    );

    let achieved_max_load = best_hasher.size.get() as f32 / min_size as f32;
    cache_info.cached.insert(
        table,
        TableCacheEntry {
            values: values
                .iter()
                .map(|(&k, v)| (k.to_string(), toml::Value::try_from(v.clone()).unwrap()))
                .collect(),
            max_load: achieved_max_load,
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
