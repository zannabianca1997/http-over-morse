#![doc = include_str!("../README.md")]
#![no_std]

use proc_macro::TokenStream;

/// Set up this type as the configuration of a binary
///
/// Create a library `bin-config` (the configuration values shown are the
/// default ones):
///
/// ```ignore
/// use zerocopy::{Immutable, FromBytes, IntoBytes}
/// #[derive(ConfigStruct, Immutable, FromBytes, IntoBytes)]
/// #[config_struct(
///     features(
///       bin = "bin",
///       host = "host"
///     ),
///     // default subsection is the structure name
///     section = ".configurable.Config"
/// )]
/// pub struct Config {
///    pub answer: u8
/// }
/// ```
///
/// Then make the binary depend on `bin-config:
///
/// ```ignore
/// use bin-config::Config;
///
/// let config: &'static Config = Config::get();
/// ```
///
/// And from the host:
///
/// ```ignore
/// use configurable::configure;
/// use bin-config::Config;
///
/// configure("/path/to/bin", Config { answer: 42 }).unwrap();
/// ```
#[proc_macro_derive(ConfigStruct, attributes(config_struct))]
pub fn config_struct(input: TokenStream) -> TokenStream {
    codegen::config_struct(input.into()).into()
}
