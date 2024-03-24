use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

use geekorm_core::{Columns, Table};

use crate::{
    derive::{ColumnDerive, ColumnTypeDerive, ColumnTypeOptionsDerive, ColumnsDerive, TableDerive},
    internal::TableState,
};

pub(crate) fn derive_parser(ast: &DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &ast.ident;

    match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let columns: Vec<ColumnDerive> = fields
                .named
                .iter()
                .map(|f| {
                    let name = f.ident.as_ref().unwrap().to_string();
                    // TODO(geekmasher): handle unwrap here better
                    ColumnDerive::new(name, ColumnTypeDerive::try_from(&f.ty).unwrap())
                })
                .collect();
            let table = TableDerive {
                name: name.to_string(),
                columns: ColumnsDerive::from(columns),
            };

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

    Ok(stream)
}

fn generate_table_builder(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics geekorm::prelude::TableBuilder for #ident #ty_generics #where_clause {
            fn table() -> geekorm::Table {
                #table
            }

            fn get_table(&self) -> geekorm::Table {
                #ident::table()
            }

            fn table_name() -> String {
                stringify!(#ident).to_string()
            }

            fn get_primary_key(&self) -> Option<String> {
                #table.get_primary_key()
            }

            fn create() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::create()
                    .table(#ident::table())
            }
            fn select() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::select()
                    .table(#ident::table())
            }

            fn count() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::select()
                    .table(#ident::table())
                    .count()
            }
        }
    })
}

#[allow(dead_code)]
fn generate_serde(
    ident: &syn::Ident,
    generics: &syn::Generics,
    _table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics serde::Serialize for #ident #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let table = #ident::table();
                table.serialize(serializer)
            }
        }

        impl #impl_generics serde::Deserialize<'de> for #ident #ty_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let table = geekorm::Table::deserialize(deserializer)?;
                Ok(Self::from_table(table))
            }
        }
    })
}
