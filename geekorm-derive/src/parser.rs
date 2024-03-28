use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields};

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
                    // TODO(geekmasher): handle unwrap here better
                    ColumnDerive::new(f.ident.as_ref().unwrap().clone(), f.ty.clone())
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
    stream.extend(generate_table_primary_key(ident, generics, &table)?);

    // TODO(geekmasher): Generate the Foreign Keys for the struct
    // stream.extend(generate_foreign_key(ident, generics, &table)?);

    #[cfg(feature = "new")]
    stream.extend(generate_new(ident, generics, &table));

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

fn generate_table_primary_key(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let Some(key) = table.columns.get_primary_key() {
        let name = key.name.clone();

        let identifier = syn::Ident::new(name.as_str(), name.span());

        Ok(quote! {
            impl #impl_generics geekorm_core::TablePrimaryKey for #ident #ty_generics #where_clause {
                fn primary_key() -> String {
                    String::from(#name)
                }

                fn primary_key_value(&self) -> geekorm::Value {
                    geekorm::Value::from(self.#identifier)
                }
            }
        })
    } else {
        Ok(quote! {})
    }
}

/// Generate the a `new()` function for the struct that will be used to create a new instance of the struct
#[allow(dead_code, unused_variables)]
fn generate_new(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let params = table.columns.to_params();
    let self_block = table.columns.to_self();

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn new(#params) -> Self {
                #self_block
            }
        }
    })
}

#[allow(dead_code, unused_variables)]
fn generate_foreign_key(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let keys = table.columns.get_foreign_keys();

    let mut stream = TokenStream::new();
    for key in keys {
        stream.extend(quote! {
            impl #impl_generics geekorm::ForeignKey for #ident #ty_generics #where_clause {
                fn foreign_key() -> geekorm::ForeignKey {
                    #table.foreign_key()
                }
            }
        })
    }
    Ok(stream)
}

#[allow(dead_code, unused_variables)]
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
