use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields};

use geekorm_core::{Columns, Table};

mod helpers;
mod taberbuilder;

use crate::{
    attr::GeekAttribute,
    derive::{ColumnDerive, ColumnTypeDerive, ColumnTypeOptionsDerive, ColumnsDerive, TableDerive},
    internal::TableState,
};
use helpers::{generate_backend, generate_helpers, generate_new};
use taberbuilder::{generate_table_builder, generate_table_primary_key};

pub(crate) fn derive_parser(ast: &DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &ast.ident;

    let attributes = GeekAttribute::parse_all(&ast.attrs)?;

    match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let mut columns: Vec<ColumnDerive> = Vec::new();

            for field in fields.named.iter() {
                // TODO(geekmasher): handle unwrap here better
                let field_attrs = GeekAttribute::parse_all(&field.attrs).unwrap();
                let col = ColumnDerive::new(
                    field.ident.as_ref().unwrap().clone(),
                    field.ty.clone(),
                    field_attrs,
                );
                columns.push(col);
            }

            let mut table = TableDerive {
                name: name.to_string(),
                columns: ColumnsDerive::from(columns),
            };
            table.apply_attributes(&attributes);

            TableState::add(table.clone().into());

            generate_struct(name, &ast.generics, table)
        }
        _ => abort!(
            ast,
            "GeekTable can only be derived for structs with named fields"
        ),
    }
}

#[allow(unused_variables)]
fn generate_struct(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();

    stream.extend(generate_table_builder(ident, generics, &table)?);
    stream.extend(generate_table_primary_key(ident, generics, &table)?);

    // TODO(geekmasher): Generate the Foreign Keys for the struct
    // stream.extend(generate_foreign_key(ident, generics, &table)?);

    #[cfg(feature = "new")]
    stream.extend(generate_new(ident, generics, &table));

    #[cfg(feature = "backends")]
    stream.extend(generate_backend(ident, generics, &table)?);

    #[cfg(feature = "helpers")]
    stream.extend(generate_helpers(ident, generics, &table)?);

    Ok(stream)
}
