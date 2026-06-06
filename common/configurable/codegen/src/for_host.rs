use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate(name: &Ident, feature: &str, section: &str) -> TokenStream {
    quote! {
        #[cfg(feature = #feature)]
        impl ConfigStruct for #name {
            const SECTION: &str = #section;
            const SIZE: u64 = ::core::mem::size_of::<#name>() as u64;
        }
    }
}
