//! Structure of the `config_struct` attribute

use darling::{FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(config_struct))]
pub struct Input {
    /// Name of the type
    pub ident: syn::Ident,

    /// Features to activate from the bin or the host
    #[darling(default)]
    pub features: Features,

    /// Linker section where to put the configuration
    #[darling(default)]
    pub section: Option<String>,
}

impl Input {
    pub fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        let input: DeriveInput = match syn::parse2(input) {
            Ok(data) => data,
            Err(err) => return Err(err.to_compile_error()),
        };
        let input = match Input::from_derive_input(&input) {
            Ok(data) => data,
            Err(err) => return Err(err.write_errors()),
        };

        Ok(input)
    }
}

#[derive(Debug, FromMeta)]
pub struct Features {
    #[darling(default = || Features::default().host)]
    pub host: String,
    #[darling(default = || Features::default().bin)]
    pub bin: String,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            host: String::from("host"),
            bin: String::from("bin"),
        }
    }
}
