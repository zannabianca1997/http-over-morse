use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    num::{NonZero, NonZeroU8},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use quote::quote;

mod src {
    //! Code shared with the built program

    // not using all from the imported stuff
    #![allow(unused)]

    pub mod perfect_hash;
    pub mod symbol;
}

mod build {
    pub mod cache;
    pub mod codegen;
    pub mod decode;
    pub mod grammar;
    pub mod perfect_hash_search;
    pub mod primes;
    pub mod shims;
    pub mod supersequence;
}

use crate::{
    build::{
        codegen::{enums, static_symbols_slice},
        decode::decode,
        grammar::{Grammar, Prosign},
        supersequence::supersequence,
    },
    src::symbol::Symbol,
};

fn main() -> Result<()> {
    // == Schema update ==

    let schema = schemars::schema_for!(Grammar);
    serde_json::to_writer_pretty(
        File::create("build/schemas/grammar.json").context("Cannot open grammar schema file")?,
        &schema,
    )
    .context("Cannot write grammar schema")?;

    // == Code generation ==

    let Grammar { letters, prosigns } = Grammar::load()?;

    let mut definitions: BTreeMap<NonZeroU8, Vec<Symbol>> = letters.clone().into_iter().collect();

    for (name, Prosign { code, maps_to }) in &prosigns {
        if definitions.contains_key(&maps_to) {
            bail!("Prosign {name} maps to occupied value")
        }

        let mut symbols: Vec<Symbol> = vec![];
        for ch in code.bytes() {
            let ch_symbols = definitions
                .get(&NonZero::new(ch).context("\\0 is not valid in prosigns")?)
                .context("Prosign contains unknown letter")?;
            symbols.extend(ch_symbols);
        }

        definitions.insert(*maps_to, symbols);
    }

    let (symbols_slice, definitions) = supersequence(definitions);

    let static_symbols_slice = static_symbols_slice(&symbols_slice);
    let decode = decode(&definitions, &symbols_slice)?;
    let enums = enums(&letters, &prosigns);

    let generated = quote! {
        #static_symbols_slice
        #decode
        #enums
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
