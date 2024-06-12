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

use parsers::{derive_parser, enum_parser};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields};

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

/// GeekValue is the derive macro for serializing and deserializing custom column types.
///
/// ```rust
/// use geekorm::prelude::*;
///
/// # #[derive(Eq, PartialEq, Debug)]
/// #[derive(GeekValue, Default)]
/// enum Role {
///     Admin,
///     Moderator,
///     User,
///     #[default]
///     Guest,
/// }
///
/// #[derive(GeekTable)]
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
#[proc_macro_derive(GeekValue)]
pub fn value_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    enum_parser(&ast).unwrap().into()
}
