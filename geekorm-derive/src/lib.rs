#![allow(unused_imports)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/42ByteLabs/geekorm/main/assets/geekorm.png"
)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

#[macro_use]
mod macros;

mod attr;
mod derive;
mod errors;
mod internal;
mod parsers;

use geekorm_core::Table;
use parsers::derive_parser;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

use crate::derive::TableDerive;

/// Derive macro for `GeekTable` trait.
///
/// This macro will generate the implementation of `GeekTable` trait for the given struct.
/// The struct must have named fields.
///
/// # Example
///
/// This macro generates a number of methods for the struct, including `table` and `table_name` methods.
///
/// ```rust
/// use geekorm::{GeekTable, PrimaryKeyInteger};
/// use geekorm::prelude::*;
///
/// #[derive(GeekTable)]
/// struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
///     age: i32,
///     occupation: String,
/// }
///
/// // This will get you the underlying table information.
/// let table = Users::table();
/// assert_eq!(Users::table_name(), "Users");
/// ```
#[proc_macro_derive(GeekTable, attributes(geekorm))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    derive_parser(&ast).unwrap().into()
}

#[proc_macro]
pub fn tables(_input: TokenStream) -> TokenStream {
    let state = internal::TableState::load_state_file();

    let mut table_names: Vec<String> = Vec::new();
    let mut tables: Vec<Table> = Vec::new();

    // TODO: Maybe this could be better?
    let mut index = 0;
    while tables.len() != state.tables.len() {
        let table = state.tables.get(index).unwrap();

        if !table_names.contains(&table.name) {
            let fkeys = table.get_foreign_keys();

            if fkeys.is_empty() {
                tables.push(table.clone());
                table_names.push(table.name.clone());
            } else if fkeys
                .iter()
                .all(|fkey| table_names.contains(&fkey.foreign_key_table().unwrap()))
            {
                tables.push(table.clone());
                table_names.push(table.name.clone());
            }
        }

        if index == state.tables.len() - 1 {
            index = 0;
        } else {
            index += 1;
        }
    }

    let mut tables_ast = proc_macro2::TokenStream::new();

    tables.iter().for_each(|table| {
        let derive_table: TableDerive = TableDerive::from(table);
        tables_ast.extend(quote! {
            #derive_table ,
        });
    });

    quote! {
        vec![ #tables_ast ]
    }
    .into()
}
