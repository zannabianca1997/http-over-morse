use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate(name: &Ident, feature: &str, section: &str) -> TokenStream {
    quote! {
        #[cfg(feature = #feature)]
        impl #name {
            /// Obtain the config
            pub fn get() -> &'static Self {
                #[used]
                #[unsafe(link_section = #section)]
                static INSTANCE: ::core::mem::MaybeUninit<#name> = ::core::mem::MaybeUninit::uninit();

                unsafe {
                    ::core::mem::MaybeUninit::assume_init_ref(&INSTANCE)
                }
            }
        }
    }
}
