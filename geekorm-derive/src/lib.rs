#![allow(unused_imports, dead_code)]
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

use parsers::{derive_parser, enum_parser};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields};

/// Derive macro for `Table` trait.
///
/// This macro will generate the implementation of `Table` trait for the given struct.
/// The struct must have named fields.
///
/// # Example
///
/// This macro generates a number of methods for the struct, including `table` and `table_name` methods.
///
/// ```rust
/// use geekorm::prelude::*;
///
/// #[derive(Table)]
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
#[proc_macro_derive(Table, attributes(geekorm, gorm))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    derive_parser(&ast).unwrap().into()
}

#[deprecated(
    since = "0.4.2",
    note = "This macro is depricated, please use `Table` instead."
)]
#[proc_macro_derive(GeekTable, attributes(geekorm))]
pub fn depricated_table_derive(input: TokenStream) -> TokenStream {
    table_derive(input)
}

/// Data is the derive macro for serializing and deserializing custom column types.
///
/// ```rust
/// use geekorm::prelude::*;
///
/// # #[derive(Eq, PartialEq, Debug)]
/// #[derive(Data, Default)]
/// enum Role {
///     Admin,
///     Moderator,
///     User,
///     #[default]
///     Guest,
/// }
///
/// #[derive(Table)]
/// struct Users {
///     #[geekorm(primary_key, auto_increment)]
///     id: PrimaryKeyInteger,
///     username: String,
///     role: Role,
/// }
///
/// let geekmasher = Users::new("geekmasher", Role::Admin);
///
/// # assert_eq!(geekmasher.role, Role::Admin);
/// # assert_eq!(Value::from(geekmasher.role), Value::Text("Admin".to_string()));
/// # assert_eq!(Role::from(Value::Text("Admin".to_string())), Role::Admin);
/// ```
#[proc_macro_derive(Data)]
pub fn data_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    enum_parser(&ast).unwrap().into()
}

/// Value is the derive macro for serializing and deserializing custom column types.
#[deprecated(
    since = "0.4.2",
    note = "This macro is depricated, please use `Data` instead."
)]
#[proc_macro_derive(GeekValue)]
pub fn depricated_value_derive(input: TokenStream) -> TokenStream {
    data_derive(input)
}
