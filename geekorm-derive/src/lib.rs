#![allow(unused_imports)]
#![forbid(unsafe_code)]

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

use parsers::derive_parser;
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
/// - `PrimaryKey<T>` fields are not generated
/// - `Option<T>` fields are not generated
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
///     42,
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
/// # Generated Helper Methods
///
/// When the `helpers` feature is enabled, the following helper methods are generated for the struct:
///
/// Note: This is a very experimental feature and might change in the future.
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
/// // Select by column helper function
/// let user = User::select_by_name("geekmasher");
/// # assert_eq!(user.query, String::from("SELECT * FROM User WHERE name = ?;"));
/// let user = User::select_by_age(69);
/// # assert_eq!(user.query, String::from("SELECT * FROM User WHERE age = ?;"));
/// let user = User::select_by_occupation("Software Developer");
/// # assert_eq!(user.query, String::from("SELECT * FROM User WHERE occupation = ?;"));
/// ```
#[proc_macro_derive(GeekTable, attributes(geekorm))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    derive_parser(&ast).unwrap().into()
}
