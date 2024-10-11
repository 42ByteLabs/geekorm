use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DataEnum,
    DataStruct, DeriveInput, Fields,
};

#[cfg(feature = "rand")]
use geekorm_core::utils::generate_random_string;
use geekorm_core::{Columns, Table};
use values::{generate_from_value, generate_serde};

mod helpers;
mod tablebuilder;
mod values;

use crate::{
    attr::GeekAttribute,
    derive::{ColumnDerive, ColumnTypeDerive, ColumnTypeOptionsDerive, ColumnsDerive, TableDerive},
    internal::TableState,
    parsers::tablebuilder::generate_query_builder,
};
use helpers::{generate_helpers, generate_new, generate_random_helpers};
use tablebuilder::{generate_backend, generate_table_builder, generate_table_primary_key};

use self::helpers::generate_hash_helpers;

pub(crate) fn derive_parser(ast: &DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &ast.ident;

    let attributes = GeekAttribute::parse_all(&ast.attrs)?;

    match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let mut errors: Vec<syn::Error> = Vec::new();
            let mut columns: Vec<ColumnDerive> = Vec::new();

            for field in fields.named.iter() {
                match ColumnDerive::try_from(field) {
                    Ok(column) => columns.push(column),
                    Err(err) => errors.push(err),
                }
            }

            let mut table = TableDerive {
                name: name.to_string(),
                columns: ColumnsDerive::from(columns),
            };
            table.apply_attributes(&attributes);

            TableState::add(table.clone().into());

            // Generate for the whole table
            let mut tokens = generate_struct(name, &fields, &ast.generics, table)?;

            if !errors.is_empty() {
                for error in errors {
                    tokens.extend(error.to_compile_error());
                }
            }
            Ok(tokens)
        }
        _ => Ok(syn::Error::new(
            ast.span(),
            "Table only supported derived structs with named fields",
        )
        .to_compile_error()),
    }
}

pub(crate) fn enum_parser(ast: &DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &ast.ident;

    match &ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            let mut tokens = TokenStream::new();

            tokens.extend(generate_from_value(name, variants, &ast.generics)?);
            tokens.extend(generate_serde(name, variants, &ast.generics)?);

            Ok(tokens)
        }
        _ => Ok(syn::Error::new(
            ast.span(),
            "Table only supported derived enums with named fields",
        )
        .to_compile_error()),
    }
}

#[allow(unused_variables)]
fn generate_struct(
    ident: &syn::Ident,
    fields: &syn::FieldsNamed,
    generics: &syn::Generics,
    table: TableDerive,
) -> Result<TokenStream, syn::Error> {
    let mut stream = TokenStream::new();

    // Table
    stream.extend(generate_table_builder(ident, generics, &table)?);
    // Query Builder
    stream.extend(generate_query_builder(ident, generics, &table)?);
    // Primary Key
    stream.extend(generate_table_primary_key(ident, generics, &table)?);

    #[cfg(feature = "backends")]
    {
        // Backend implementations and fetch methods
        stream.extend(generate_backend(ident, fields, generics, &table)?);
    }

    #[cfg(feature = "new")]
    stream.extend(generate_new(ident, generics, &table));

    #[cfg(feature = "helpers")]
    stream.extend(generate_helpers(ident, generics, &table)?);

    #[cfg(feature = "hash")]
    stream.extend(generate_hash_helpers(ident, generics, &table)?);

    #[cfg(feature = "rand")]
    stream.extend(generate_random_helpers(ident, generics, &table)?);

    Ok(stream)
}
