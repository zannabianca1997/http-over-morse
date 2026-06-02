use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{Context, Result};
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

use crate::build::{
    codegen::{is_morse_letter, static_symbols_slice},
    decode::decode,
    grammar::Grammar,
    supersequence::supersequence,
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

    let Grammar { letters, .. } = Grammar::load()?;

    let (symbols_slice, definitions) = supersequence(letters);

    let static_symbols_slice = static_symbols_slice(&symbols_slice);
    let is_morse_letter = is_morse_letter(&definitions);
    let decode = decode(&definitions, &symbols_slice)?;
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
