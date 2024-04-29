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
///
/// # Generate New Rows
///
/// When the `new` feature is enabled, the following methods are generated for the struct:
///
/// - `PrimaryKey<T>` fields are not generated
/// - `Option<T>` fields are not generated
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
///     country: Option<String>,
/// }
///
/// let user = Users::new(
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
/// // Create a new table query
/// let create = Users::create().build()
///     .expect("Failed to build CREATE TABLE query");
///
/// // Select data from the table
/// let select = Users::select()
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
/// // Select by column helper function
/// let user = Users::select_by_name("geekmasher");
/// # assert_eq!(user.query, String::from("SELECT id, name, age, occupation FROM Users WHERE name = ?;"));
/// let user = Users::select_by_age(69);
/// # assert_eq!(user.query, String::from("SELECT id, name, age, occupation FROM Users WHERE age = ?;"));
/// let user = Users::select_by_occupation("Software Developer");
/// # assert_eq!(user.query, String::from("SELECT id, name, age, occupation FROM Users WHERE occupation = ?;"));
/// ```
///
/// # Generate Random Data for Column
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::{GeekTable, PrimaryKeyInteger};
///
/// #[derive(GeekTable, Debug)]
/// pub struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
///     #[geekorm(rand)]
///     token: String
/// }
///
/// let user = Users::new(String::from("geekmasher"));
///
/// ```
///
/// # Generate Hash for storing passwords
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::{GeekTable, PrimaryKeyInteger};
///
/// #[derive(GeekTable, Debug)]
/// pub struct Users {
///     id: PrimaryKeyInteger,
///     username: String,
///
///     #[geekorm(password)]
///     password: String,
/// }
///
/// let user = Users::new(String::from("geekmasher"), String::from("password"));
/// # assert_eq!(user.password.len(), 20);
/// ```
#[proc_macro_derive(GeekTable, attributes(geekorm))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    derive_parser(&ast).unwrap().into()
}
