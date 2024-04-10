use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields,
};

use geekorm_core::{Columns, Table};

mod helpers;
mod tablebuilder;

use crate::{
    attr::GeekAttribute,
    derive::{ColumnDerive, ColumnTypeDerive, ColumnTypeOptionsDerive, ColumnsDerive, TableDerive},
    internal::TableState,
    parsers::tablebuilder::generate_query_builder,
};
use helpers::{generate_helpers, generate_new};
use tablebuilder::{generate_table_builder, generate_table_primary_key};

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

            let mut tokens = generate_struct(name, &ast.generics, table)?;
            if !errors.is_empty() {
                for error in errors {
                    tokens.extend(error.to_compile_error());
                }
            }
            Ok(tokens)
        }
        _ => Ok(syn::Error::new(
            ast.span(),
            "GeekTable only supported derived structs with named fields",
        )
        .to_compile_error()),
    }
}

#[allow(unused_variables)]
fn generate_struct(
    ident: &syn::Ident,
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

    // Backends
    // #[cfg(feature = "libsql")]
    // stream.extend(generate_backend_libsql(ident, generics, &table)?);

    #[cfg(feature = "new")]
    stream.extend(generate_new(ident, generics, &table));

    #[cfg(feature = "helpers")]
    stream.extend(generate_helpers(ident, generics, &table)?);

    Ok(stream)
}
