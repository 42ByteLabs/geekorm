#![allow(unused_imports)]
#![forbid(unsafe_code)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

#[macro_use]
mod macros;

mod derive;
mod errors;
mod internal;
mod parser;

use parser::derive_parser;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

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
/// use geekorm::GeekTable;
/// use geekorm::prelude::*;
///
/// #[derive(GeekTable)]
/// struct User {
///     name: String,
///     age: i32,
///     occupation: String,
/// }
///
/// // This will get you the underlying table information.
/// let table = User::table();
/// assert_eq!(User::table_name(), "User");
/// ```
///
/// # Generate New Rows
///
/// When the `new` feature is enabled, the following methods are generated for the struct:
///
/// ```rust
/// use geekorm::GeekTable;
/// use geekorm::prelude::*;
///
/// #[derive(GeekTable)]
/// struct User {
///     name: String,
///     age: i32,
///     occupation: String,
///     country: Option<String>,
/// }
///
/// let user = User::new(
///     String::from("geekmasher"),
///     69,
///     String::from("Software Developer")
/// );
///
/// ```
///
/// # Generated Query Methods
///
/// The following methods are generated for the struct:
///
/// ```rust
/// use geekorm::GeekTable;
/// use geekorm::prelude::*;
///
/// #[derive(GeekTable)]
/// struct User {
///    name: String,
///    age: i32,
///    occupation: String,
/// }
///
/// // Create a new table query
/// let create = User::create().build()
///     .expect("Failed to build CREATE TABLE query");
///
/// // Select data from the table
/// let select = User::select()
///     .where_eq("name", "geekmasher")
///     .build()
///     .expect("Failed to build SELECT query");
/// ```
///
#[proc_macro_derive(GeekTable)]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    derive_parser(&ast).unwrap().into()
}
