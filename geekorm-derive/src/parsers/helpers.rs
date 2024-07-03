use proc_macro2::TokenStream;
use quote::quote;

use crate::derive::TableDerive;

/// Generate the a `new()` function for the struct that will be used to create a new instance of the struct
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::PrimaryKeyInteger;
///
/// #[derive(GeekTable, Clone)]
/// struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
///     age: i32,
///     occupation: Option<String>,
///     country: Option<String>,
/// }
///
/// let user = Users::new(
///     String::from("geekmasher"),
///     42,
/// );
/// ```
#[allow(dead_code, unused_variables)]
pub(crate) fn generate_new(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let params = table.columns.to_params();
    let self_block = table.columns.to_self();

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            /// Create a new instance of the struct
            #[allow(dead_code)]
            pub fn new(#params) -> Self {
                #self_block
            }
        }
    })
}

/// Generate helper functions for the struct that will be used to access the columns
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::PrimaryKeyInteger;
///
/// #[derive(GeekTable, Default, Clone)]
/// struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
///     age: i32,
/// }
///
/// let user_id = Users::query_select_by_id(1);
/// let user_name = Users::query_select_by_name("geekmasher");
/// let user_age = Users::query_select_by_age(42);
/// ```
#[allow(dead_code, unused_variables)]
pub(crate) fn generate_helpers(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();
    // Generate the selectors for the columns
    for column in table.columns.columns.iter() {
        stream.extend(column.get_selector(ident));
    }

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #stream
        }
    })
}

#[allow(dead_code, unused_variables)]
pub(crate) fn generate_hash_helpers(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();
    // Generate the selectors for the columns
    for column in table.columns.get_hash_columns() {
        stream.extend(column.get_hash_helpers());
    }

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #stream
        }
    })
}

#[allow(dead_code, unused_variables)]
pub(crate) fn generate_random_helpers(
    ident: &syn::Ident,
    generics: &syn::Generics,
    table: &TableDerive,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();
    // Generate the selectors for the columns
    for column in table.columns.get_random_columns() {
        stream.extend(column.get_random_helpers());
    }

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #stream
        }
    })
}

#[allow(dead_code, unused_variables)]
pub(crate) fn generate_serde(
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
