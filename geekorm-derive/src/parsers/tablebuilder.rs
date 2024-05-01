use std::any::Any;

#[allow(unused_imports)]
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields, FieldsNamed,
    GenericArgument, Type, TypePath,
};

use crate::{derive::TableDerive, internal::TableState};

/// Generate implementation of `TableBuilder` trait for the struct.
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::{GeekTable, PrimaryKeyInteger};
///
/// #[derive(GeekTable, Default, Clone)]
/// struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
///     age: i32,
///     occupation: String,
/// }
///
/// let user_table = Users::table();
/// let user_table_name = Users::table_name();
///
/// let user = Users::default();
/// # let user_table2 = user.get_table();
/// ```
pub fn generate_table_builder(
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
        }
    })
}

/// Generate implementation of `QueryBuilderTrait` for the struct.
/// This provides a number of methods for building queries.
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::{GeekTable, PrimaryKeyInteger};
///
/// #[derive(GeekTable, Default, Clone)]
/// pub struct Users {
///     pub id: PrimaryKeyInteger,
///     pub name: String,
/// }
///
///
/// # fn main() {
/// let create = Users::create().build()
///     .expect("Failed to build CREATE TABLE query");
/// # assert_eq!(create.to_str(), "CREATE TABLE IF NOT EXISTS Users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);");
///
/// let select = Users::select().build()
///     .expect("Failed to build SELECT query");
/// # assert_eq!(select.to_str(), "SELECT id, name FROM Users;");
///
/// let user = Users::default();
/// let insert = Users::insert(&user);
/// # assert_eq!(insert.to_str(), "INSERT INTO Users (name) VALUES (?);");
///
/// let update = Users::update(&user);
/// # assert_eq!(update.to_str(), "UPDATE Users SET name = ? WHERE id = 0;");
///
/// let count = Users::count().build()
///     .expect("Failed to build COUNT query");
/// # assert_eq!(count.to_str(), "SELECT COUNT(1) FROM Users;");
/// }
/// ```
pub fn generate_query_builder(
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
        impl #impl_generics geekorm::prelude::QueryBuilderTrait for #ident #ty_generics #where_clause {
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

            fn update(item: &Self) -> geekorm::Query {
                geekorm::QueryBuilder::update()
                    .table(#ident::table())
                    #insert_values
                    .build()
                    .expect("Failed to build update query")
            }

            fn count() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::select()
                    .table(#ident::table())
                    .count()
            }
        }
    })
}

/// Generate implementation of `TablePrimaryKey` for the struct.
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::PrimaryKeyInteger;
/// # use geekorm::Value;
///
/// #[derive(GeekTable, Default, Clone)]
/// pub struct Userss {
///    pub id: PrimaryKeyInteger,
///    pub name: String,
///    pub age: i32,
/// }
///
/// let user = Userss::new(String::from("John Doe"), 30);
///
/// # assert_eq!(Userss::primary_key(), "id");
/// # assert_eq!(Userss::primary_key_value(&user), Value::from(0));
///
/// ```
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
                    geekorm::Value::from(&self.#identifier)
                }
            }
        })
    } else {
        Err(syn::Error::new(
            ident.span(),
            "Table must have a primary key",
        ))
    }
}

/// Generate `execute` helper functions for the struct.
///
/// - `execute_update()`
#[allow(dead_code)]
pub fn generate_table_execute(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut insert_values = TokenStream::new();
    for column in table.columns.columns.iter() {
        let name = &column.name;
        let ident = syn::Ident::new(name.as_str(), name.span());

        // TODO(geekmasher): This clone isn't ideal, but it's the only way to get this to work.
        insert_values.extend(quote! {
            self.#ident = item.#ident.clone();
        });
    }

    // TODO(geekmasher): The execute_insert method might have an issue as we don't have a lock and
    // the last inserted item might not be the one we inserted.
    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            /// Execute an update query for the struct.
            pub async fn execute_update(&self, connection: &libsql::Connection) -> Result<(), geekorm::Error> {
                #ident::execute(&connection, #ident::update(self)).await
            }

            /// Execute an update query for the struct.
            pub async fn execute_insert(&mut self, connection: &libsql::Connection) -> Result<(), geekorm::Error> {
                #ident::execute(&connection, #ident::insert(self)).await?;
                let select_query = #ident::select()
                    .order_by(#ident::primary_key().as_str(), geekorm::QueryOrder::Desc)
                    .limit(1)
                    .build()?;

                log::debug!("Insert query: {}", select_query.to_str());
                let item: #ident = #ident::query_first(connection, select_query).await?;

                #insert_values
                Ok(())
            }
        }
    })
}

/// Generate fetch methods for the struct.
pub fn generate_table_fetch(
    ident: &syn::Ident,
    fields: &FieldsNamed,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();
    let mut fetch_functions = TokenStream::new();

    // Generate the selectors for the columns
    for column in table.columns.get_foreign_keys() {
        let field = fields
            .named
            .iter()
            .find(|f| f.ident.as_ref().unwrap() == &column.name)
            .unwrap();

        // Inner type of the field
        // ForeignKey<i32, Users>,
        let field_type = match &field.ty {
            syn::Type::Path(path) => path.path.segments.first().unwrap(),
            _ => {
                return Err(syn::Error::new(
                    field.ty.span(),
                    "Only path types are supported for foreign keys",
                ))
            }
        };

        let inner_type = match &field_type.arguments {
            syn::PathArguments::AngleBracketed(args) => args.args.last().unwrap(),
            _ => {
                return Err(syn::Error::new(
                    field.ty.span(),
                    "Only angle bracketed arguments are supported for foreign keys",
                ))
            }
        };

        match inner_type {
            syn::GenericArgument::Type(Type::Path(path)) => {
                let fident = path.path.segments.first().unwrap().ident.clone();

                stream.extend(column.get_fetcher(ident, &fident));

                // Add fetch function to the list of fetch functions
                let func_name = format!("fetch_{}", column.identifier);
                let func = Ident::new(&func_name, Span::call_site());

                fetch_functions.extend(quote! {
                    Self::#func(self, connection).await?;
                });
            }
            _ => {
                return Err(syn::Error::new(
                    field.ty.span(),
                    "Only type arguments are supported for foreign keys",
                ))
            }
        }
    }

    // Generate a fetch all method for the struct
    match cfg!(feature = "libsql") {
        true => {
            stream.extend(quote! {
                pub async fn fetch_all(&mut self, connection: &libsql::Connection) -> Result<(), geekorm::Error> {
                    #fetch_functions
                    Ok(())
                }
            });
        }
        false => {
            stream.extend(quote! {});
        }
    }

    // Generate the fetch method for PrimaryKey
    if let Some(key) = table.columns.get_primary_key() {
        stream.extend(key.get_fetcher_pk(ident));
    }

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #stream
        }
    })
}
