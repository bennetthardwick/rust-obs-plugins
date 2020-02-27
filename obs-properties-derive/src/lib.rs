extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

// TODO: allow people to use a derive macro to load properties

#[proc_macro_derive(Properties)]
pub fn properties_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(s) = input.data {
    } else {
        panic!("Expected struct!");
    }

    let tokens = quote! {
        struct Okay {
        }
    };

    tokens.into()
}
