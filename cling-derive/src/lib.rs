#![forbid(unsafe_code)]
//! Do not depend on this library directly. Instead, use `cling`

mod attributes;
#[cfg(debug_assertions)]
mod cling_handler;
mod derives;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Run, attributes(cling, command, clap))]
pub fn derive_run(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::derive_run(&input).into()
}

#[proc_macro_derive(Collect, attributes())]
pub fn derive_collect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::derive_collect(&input).into()
}

#[proc_macro_attribute]
pub fn cling_handler(_attr: TokenStream, function: TokenStream) -> TokenStream {
    #[cfg(not(debug_assertions))]
    return function;

    #[cfg(debug_assertions)]
    {
        use syn::ItemFn;
        let input = parse_macro_input!(function as ItemFn);
        cling_handler::expand_handler(&input).into()
    }
}
