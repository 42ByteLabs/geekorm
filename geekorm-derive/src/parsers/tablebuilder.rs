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
///
/// #[derive(Table, Default, Clone, serde::Serialize, serde::Deserialize)]
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
            /// Get the table instance.
            fn table() -> geekorm::Table {
                #table
            }
            /// Get the table name.
            fn get_table(&self) -> geekorm::Table {
                #ident::table()
            }
            /// Get the table name.
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
///
/// #[derive(Table, Default, Clone, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///     pub id: PrimaryKeyInteger,
///     pub name: String,
/// }
///
///
/// # fn main() {
/// let create = Users::query_create().build()
///     .expect("Failed to build CREATE TABLE query");
/// # assert_eq!(create.to_str(), "CREATE TABLE IF NOT EXISTS Users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);");
///
/// let select = Users::query_select().build()
///     .expect("Failed to build SELECT query");
/// # assert_eq!(select.to_str(), "SELECT id, name FROM Users;");
///
/// let user = Users::default();
/// let insert = Users::query_insert(&user);
/// # assert_eq!(insert.to_str(), "INSERT INTO Users (name) VALUES (?);");
///
/// let update = Users::query_update(&user);
/// # assert_eq!(update.to_str(), "UPDATE Users SET name = ? WHERE id = 0;");
///
/// let count = Users::query_count().build()
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
        if column.skip {
            continue;
        }
        let name = &column.name;
        let ident = syn::Ident::new(name.as_str(), name.span());
        insert_values.extend(quote! {
            .add_value(#name, &item.#ident)
        });
    }

    Ok(quote! {
        impl #impl_generics geekorm::prelude::QueryBuilderTrait for #ident #ty_generics #where_clause {
            /// Create table query.
            fn query_create() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::create()
                    .table(#ident::table())
            }
            /// Select query.
            fn query_select() -> geekorm::QueryBuilder {
                geekorm::QueryBuilder::select()
                    .table(#ident::table())
            }
            /// Insert query.
            fn query_insert(item: &Self) -> geekorm::Query {
                geekorm::QueryBuilder::insert()
                    .table(#ident::table())
                    #insert_values
                    .build()
                    .expect("Failed to build insert query")
            }
            /// Update query.
            fn query_update(item: &Self) -> geekorm::Query {
                geekorm::QueryBuilder::update()
                    .table(#ident::table())
                    #insert_values
                    .build()
                    .expect("Failed to build update query")
            }
            /// Delete query.
            fn query_delete(item: &Self) -> geekorm::Query {
                geekorm::QueryBuilder::delete()
                    .table(#ident::table())
                    .where_eq(#ident::primary_key().as_str(), item.primary_key_value())
                    .build()
                    .expect("Failed to build delete query")
            }
            /// Count query.
            fn query_count() -> geekorm::QueryBuilder {
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
///
/// #[derive(Table, Default, Clone, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///    pub id: PrimaryKeyInteger,
///    pub name: String,
///    pub age: i32,
/// }
///
/// let user = Users::new(String::from("John Doe"), 30);
///
/// # assert_eq!(Users::primary_key(), "id");
/// # assert_eq!(Users::primary_key_value(&user), Value::from(0));
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
                /// Get the primary key of the table.
                fn primary_key() -> String {
                    String::from(#name)
                }

                /// Get the primary key value of the table.
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

/// Generate the backend implementation for the struct and fetch methods.
///
/// - `fetch_by_primary_key()` - Gets an item by the primary key.
/// - `fetch_by_{field}()` - Gets an item by the field.
/// - `fetch_{field}()` - Fetch foreign key items.
#[allow(dead_code)]
pub fn generate_backend(
    ident: &syn::Ident,
    fields: &FieldsNamed,
    _generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    // Finished Stream
    let mut stream = TokenStream::new();
    // Insert values
    let mut insert_values = TokenStream::new();
    // Fetch implementation
    let mut fetch_impl = TokenStream::new();
    // Fetch functions
    let mut fetch_functions = TokenStream::new();
    // Auto Update fields
    let mut auto_update = TokenStream::new();
    // Stream of where clauses
    let mut where_previous = false;
    let mut where_clauses = TokenStream::new();
    // Unique where clause
    let mut unique_where = TokenStream::new();

    // Generate the selectors for the columns
    for column in table.columns.columns.iter() {
        // If the column is skipped, then we don't need to fetch it.
        if column.skip {
            continue;
        }

        let name = &column.name;
        let ident = syn::Ident::new(name.as_str(), name.span());

        // TODO(geekmasher): This clone isn't ideal, but it's the only way to get this to work.
        insert_values.extend(quote! {
            self.#ident = item.#ident.clone();
        });

        if let Some(update) = &column.update {
            // self.updated = chrono::Utc::now();

            let auto = syn::parse_str::<TokenStream>(update).map_err(|err| {
                syn::Error::new(
                    column.span(),
                    format!("Failed to parse data for New mode: {}", err),
                )
            })?;

            auto_update.extend(quote! {
                self.#ident = #auto;
            });
        }

        if column.is_searchable() {
            if where_previous {
                where_clauses.extend(quote! {
                    .or()
                });
            }

            where_clauses.extend(quote! {
                .where_like(stringify!(#ident), format!("%{}%", search))
            });
            where_previous = true;
        }

        if column.is_unique() {
            unique_where.extend(quote! {
                .where_eq(stringify!(#ident), &self.#ident)
            });
        }

        if column.is_foreign_key() == true {
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
                    fetch_impl.extend(column.get_fetcher(&ident, &fident));

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

        let func = syn::Ident::new(format!("fetch_by_{}", name).as_str(), name.span());

        let select_func =
            syn::Ident::new(format!("query_select_by_{}", name).as_str(), name.span());

        if column.is_unique() {
            fetch_impl.extend(quote! {
                /// Fetch the data from the table by the field (unique).
                pub async fn #func<'a, C>(
                    connection: &'a C,
                    value: impl Into<geekorm::Value>
                ) -> Result<Self, geekorm::Error>
                where
                    C: geekorm::GeekConnection<Connection = C> + 'a,
                    Self: geekorm::QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned
                {
                    C::query_first::<Self>(
                        connection,
                        Self:: #select_func(value.into())
                    ).await
                }
            });
        } else {
            fetch_impl.extend(quote! {
                /// Fetch the data from the table by the field (non-unique).
                pub async fn #func<'a, C>(
                    connection: &'a C,
                    value: impl Into<geekorm::Value>
                ) -> Result<Vec<Self>, geekorm::Error>
                where
                    C: geekorm::GeekConnection<Connection = C> + 'a,
                    Self: geekorm::QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned
                {
                    C::query::<Self>(
                        connection,
                        Self:: #select_func(value.into())
                    ).await
                }
            });
        }
    }

    // GeekConnector implementation
    stream.extend(quote! {
        #[automatically_derived]
        impl<'a, T> geekorm::GeekConnector<'a, T> for #ident where
            T: geekorm::GeekConnection<Connection = T> + 'a,
            Self: geekorm::QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned
        {
            /// Save a new item to the database and return the last inserted item from the database.
            #[allow(async_fn_in_trait, unused_variables)]
            async fn save(&mut self, connection: &'a T) -> Result<(), geekorm::Error>
            {
                T::execute(connection, Self::query_insert(self)).await?;
                let select_query = #ident::query_select()
                    .order_by(#ident::primary_key().as_str(), geekorm::QueryOrder::Desc)
                    .limit(1)
                    .build()?;

                let item: #ident = T::query_first::<Self>(connection, select_query).await?;

                #insert_values
                Ok(())
            }

            /// Update the item in the database.
            #[allow(async_fn_in_trait, unused_variables)]
            async fn update(&mut self, connection: &'a T) -> Result<(), geekorm::Error> {
                #auto_update
                T::execute(connection, Self::query_update(self)).await
            }

            /// Fetch all the data from foreign tables and store them in the struct.
            #[allow(async_fn_in_trait, unused_variables)]
            async fn fetch(&mut self, connection: &'a T) -> Result<(), geekorm::Error>
            {
                #fetch_functions
                Ok(())
            }

            /// Fetch or create a row in the database
            #[allow(async_fn_in_trait, unused_variables)]
            async fn fetch_or_create(
                &mut self,
                connection: &'a T,
            ) -> Result<(), geekorm::Error>
            {
                let query = Self::query_select()
                    #unique_where
                    .build()?;

                match T::query_first::<Self>(connection, query).await {
                    Ok(item) => {
                        *self = item;
                    },
                    Err(_) => {
                        self.save(connection).await?;
                    }
                }
                Ok(())
            }

            #[allow(async_fn_in_trait, unused_variables)]
            async fn search(
                connection: &'a T,
                search: impl Into<String>,
            ) -> Result<Vec<Self>, geekorm::Error>
            {
                let search = search.into();
                Ok(T::query::<Self>(
                    connection,
                    geekorm::QueryBuilder::select()
                        .table(Self::table())
                        #where_clauses
                        .build()?
                ).await?)
            }
        }
    });

    // Generate the fetch method for PrimaryKey
    if let Some(key) = table.columns.get_primary_key() {
        fetch_impl.extend(key.get_fetcher_pk(ident));
    }

    // Fetch functions
    stream.extend(quote! {
        /// Fetch methods for the model.
        #[automatically_derived]
        impl #ident
        {
            #fetch_impl
        }
    });

    Ok(stream)
}
