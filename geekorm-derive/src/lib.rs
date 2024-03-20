#![allow(unused_imports)]
#![forbid(unsafe_code)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod derive;
mod parser;

use parser::derive_parser;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(GeekTable)]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    derive_parser(&ast).unwrap().into()
}
