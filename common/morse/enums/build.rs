use std::{env, fs, path::Path};

use quote::quote;

fn main() {
    let out = quote! {};

    let outfile = Path::new(&env::var_os("OUT_DIR").unwrap()).join("generated.rs");

    fs::write(&outfile, out.to_string()).unwrap();

    cargo_emit::rustc_env!("GENERATED_RS", "{}", outfile.display())
}
