#![forbid(unsafe_code)]

mod attributes;
mod derives;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CliRunnable, attributes(cling, command))]
pub fn derive_cli_runnable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::derive_cli_runnable(&input).into()
}

#[proc_macro_derive(CliParam, attributes())]
pub fn derive_param(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::derive_cli_param(&input).into()
}
