#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream;
use quote::quote;

use input::{Features, Input};

mod input;

mod for_bin;

mod for_host;

/// Set up this type as the configuration of a binary
pub fn config_struct(input: TokenStream) -> TokenStream {
    let Input {
        ident: name,
        features: Features {
            host: host_feature,
            bin: bin_feature,
        },
        section,
    } = match Input::parse(input) {
        Ok(input) => input,
        Err(err_stream) => return err_stream,
    };

    let section = section.unwrap_or_else(|| format!(".configurable.{name}"));

    let for_bin = for_bin::generate(&name, &bin_feature, &section);
    let for_host = for_host::generate(&name, &host_feature, &section);

    quote! {
        #for_bin
        #for_host
    }
}

#[cfg(test)]
mod tests {
    use quote::{format_ident, quote};

    use super::input::Input;

    #[test]
    fn it_parses() {
        let example = quote! {
            #[config_struct(
                features(
                    bin = "bin",
                    host = "host"
                ),
                section = ".configurable.Config"
            )]
            #[section = "hehehe"]
            pub struct Config {
                pub answer: u8
            }
        };

        let input = Input::parse(example).unwrap();

        assert_eq!(input.ident, format_ident!("Config"));
        assert_eq!(input.section, Some(String::from(".configurable.Config")));
        assert_eq!(input.features.bin, "bin");
        assert_eq!(input.features.host, "host");
    }
}
