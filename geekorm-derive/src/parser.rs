use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

use geekorm_core::{Columns, Table};

use crate::derive::{
    ColumnDerive, ColumnTypeDerive, ColumnTypeOptionsDerive, ColumnsDerive, TableDerive,
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
                    ColumnDerive::new(name, ColumnTypeDerive::from(&f.ty))
                })
                .collect();
            let table = TableDerive {
                name: name.to_string(),
                columns: ColumnsDerive::from(columns),
            };
            generate_struct(name, &ast.generics, table)
        }
        _ => panic!("GeekTable can only be derived for structs with named fields"),
    }
}

#[allow(unused_variables)]
fn generate_struct(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl geekorm::TableBuilder for #ident {
            fn table() -> geekorm::Table {
                #table
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
