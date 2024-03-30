#[allow(unused_imports)]
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields};

use crate::derive::TableDerive;

/// Generate implementation of `TableBuilder` trait for the struct.
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm_derive::TableBuilder;
///
/// #[derive(GeekTable)]
/// struct User {
///     name: String,
///     age: i32,
///     occupation: String,
/// }
///
/// let user_table = User::table();
/// let user_table_name = User::table_name();
///
/// ```
pub fn generate_table_builder(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut insert_values = TokenStream::new();
    for column in table.columns.columns.iter() {
        let name = &column.name;
        let ident = syn::Ident::new(name.as_str(), name.span());
        insert_values.extend(quote! {
            .add_value(#name, &item.#ident)
        });
    }

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

            fn insert(item: &Self) -> geekorm::Query {
                geekorm::QueryBuilder::insert()
                    .table(#ident::table())
                    #insert_values
                    .build()
                    .expect("Failed to build insert query")
            }

            fn count() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::select()
                    .table(#ident::table())
                    .count()
            }
        }
    })
}

pub fn generate_table_primary_key(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let Some(key) = table.columns.get_primary_key() {
        let name = key.name.clone();

        let identifier = syn::Ident::new(name.as_str(), name.span());

        Ok(quote! {
            impl #impl_generics geekorm::prelude::TablePrimaryKey for #ident #ty_generics #where_clause {
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

#[allow(dead_code, unused_variables)]
pub fn generate_foreign_key(
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
