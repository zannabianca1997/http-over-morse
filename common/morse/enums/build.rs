use std::{env, fs, path::Path};

use morse_build::Encoding;
use quote::quote;

fn main() {
    let ascii = morse_build::enums(Encoding::Ascii);
    let base32 = morse_build::enums(Encoding::Base32);

    let out = quote! { pub mod ascii { #ascii } pub mod base32 { #base32 } };

    let outfile = Path::new(&env::var_os("OUT_DIR").unwrap()).join("generated.rs");

    fs::write(&outfile, out.to_string()).unwrap();

    cargo_emit::rustc_env!("GENERATED_RS", "{}", outfile.display())
}
