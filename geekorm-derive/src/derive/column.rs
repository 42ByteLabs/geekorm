use geekorm_core::{ColumnType, utils::crypto::HashingAlgorithm};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{
    Attribute, Field, GenericArgument, Ident, Type, TypePath, Visibility, parse::Parse,
    spanned::Spanned, token::Pub,
};

use crate::{
    attr::{GeekAttribute, GeekAttributeKeys, GeekAttributeValue},
    derive::{ColumnTypeDerive, ColumnTypeOptionsDerive},
    internal::TableState,
};

#[derive(Debug, Clone)]
pub(crate) struct ColumnsDerive {
    pub(crate) columns: Vec<ColumnDerive>,
}

impl ColumnsDerive {
    pub(crate) fn get_primary_key(&self) -> Option<ColumnDerive> {
        self.columns
            .iter()
            .filter_map(|col| {
                if col.is_primary_key() {
                    Some(col.clone())
                } else {
                    None
                }
            })
            .next()
    }

    #[allow(dead_code)]
    pub(crate) fn get_foreign_keys(&self) -> Vec<ColumnDerive> {
        self.columns
            .iter()
            .filter_map(|c| {
                if let ColumnTypeDerive::ForeignKey(_) = &c.coltype {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the columns that are marked as a hash
    #[allow(dead_code)]
    pub(crate) fn get_hash_columns(&self) -> Vec<ColumnDerive> {
        self.columns
            .iter()
            .filter_map(|c| match c.mode {
                Some(ColumnMode::Hash { .. }) => Some(c.clone()),
                _ => None,
            })
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn get_random_columns(&self) -> Vec<ColumnDerive> {
        self.columns
            .iter()
            .filter_map(|c| match c.mode {
                Some(ColumnMode::Rand { .. }) => Some(c.clone()),
                _ => None,
            })
            .collect()
    }

    /// Convert the columns into a list of parameters for a function
    pub(crate) fn to_params(&self) -> TokenStream {
        let columns = self.columns.iter().map(|c| c.to_params()).filter_map(|c| c);
        quote! {
            #(#columns),*
        }
    }

    /// Creates a new instance of the struct and passes in the columns
    pub(crate) fn to_self(&self) -> TokenStream {
        let columns = self.columns.iter().map(|c| c.to_self());
        quote! {
            Self {
                #(#columns),*
            }
        }
    }
}

impl Iterator for ColumnsDerive {
    type Item = ColumnDerive;

    fn next(&mut self) -> Option<Self::Item> {
        self.columns.pop()
    }
}

impl ToTokens for ColumnsDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let columns = &self.columns;
        tokens.extend(quote! {
            geekorm::Columns {
                columns: vec![
                    #(#columns ),*
                ]
            }
        })
    }
}

impl From<ColumnsDerive> for geekorm_core::Columns {
    fn from(value: ColumnsDerive) -> Self {
        geekorm_core::Columns {
            columns: value.columns.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<Vec<ColumnDerive>> for ColumnsDerive {
    fn from(columns: Vec<ColumnDerive>) -> Self {
        ColumnsDerive { columns }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ColumnMode {
    New {
        enabled: bool,
        data: Option<String>,
    },
    Rand {
        len: usize,
        prefix: Option<String>,
        env: Option<String>,
    },
    Hash(HashingAlgorithm),
    Searchable {
        enabled: bool,
    },
}

#[derive(Clone)]
pub(crate) struct ColumnDerive {
    pub(crate) identifier: Ident,
    pub(crate) itype: Type,
    pub(crate) attributes: Vec<GeekAttribute>,

    /// Name to be used in the database
    pub(crate) name: String,
    /// Alias to the original struct name
    pub(crate) alias: String,
    pub(crate) coltype: ColumnTypeDerive,
    /// Skip the column
    pub(crate) skip: bool,
    /// Update the column
    pub(crate) update: Option<String>,
    pub(crate) save: Option<String>,

    pub(crate) mode: Option<ColumnMode>,
}

impl ColumnDerive {
    #[allow(irrefutable_let_patterns, clippy::collapsible_match)]
    pub(crate) fn apply_attributes(&mut self) -> Result<(), syn::Error> {
        let attributes = &self.attributes;

        for attr in attributes {
            if let Some(key) = &attr.key {
                match key {
                    GeekAttributeKeys::Skip => {
                        self.skip = true;
                    }
                    GeekAttributeKeys::Unique => {
                        self.coltype.set_unique(true);
                        // If the column is unique, then it should be searchable by default
                        self.mode = Some(ColumnMode::Searchable { enabled: true });
                    }
                    GeekAttributeKeys::OnValidate => {
                        if let Some(GeekAttributeValue::Bool(validate)) = &attr.value {
                            self.save = Some(validate.to_string());
                            self.update = Some(validate.to_string());
                        }
                    }
                    GeekAttributeKeys::OnUpdate => {
                        if let Some(GeekAttributeValue::String(value)) = &attr.value {
                            self.update = Some(value.to_string());
                        }
                    }
                    GeekAttributeKeys::OnSave => {
                        if let Some(GeekAttributeValue::String(value)) = &attr.value {
                            self.save = Some(value.to_string());
                        }
                    }
                    GeekAttributeKeys::Searchable => {
                        // Make the column searchable
                        self.mode = Some(ColumnMode::Searchable { enabled: true });
                    }
                    GeekAttributeKeys::New => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(content) = value {
                                self.mode = Some(ColumnMode::New {
                                    enabled: true,
                                    data: Some(content.to_string()),
                                });
                            } else if let GeekAttributeValue::Bool(new) = value {
                                self.mode = Some(ColumnMode::New {
                                    enabled: *new,
                                    data: None,
                                });
                            }
                        }
                    }
                    GeekAttributeKeys::Rename => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                self.alias = name.to_string();
                            }
                        }
                    }
                    GeekAttributeKeys::PrimaryKey => {
                        if let ColumnTypeDerive::Identifier(_) = self.coltype {
                            // Skip as the column type is already set
                        } else {
                            self.coltype = ColumnTypeDerive::Identifier(ColumnTypeOptionsDerive {
                                primary_key: true,
                                auto_increment: true,
                                ..Default::default()
                            });
                        }
                    }
                    GeekAttributeKeys::AutoIncrement => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::Bool(auto_increment) = value {
                                self.coltype.set_auto_increment(*auto_increment);
                            }
                        } else {
                            // If no value is set, then set it to true
                            self.coltype.set_auto_increment(true);
                        }
                    }
                    GeekAttributeKeys::NotNull => self.coltype.set_notnull(true),
                    GeekAttributeKeys::ForeignKey => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                let (table, column) = match name.split_once('.') {
                                    Some((table, column)) => (table, column),
                                    None => {
                                        return Err(syn::Error::new(
                                            attr.value_span.unwrap_or(attr.span.span()),
                                            "Invalid foreign key format (table.column)",
                                        ));
                                    }
                                };

                                // TODO: (geekmasher) These validation checks currently don't work

                                // let tables = TableState::load_state_file();
                                //
                                // let table = match tables.find_table(table) {
                                //     Some(table) => table,
                                //     None => {
                                //         return Err(syn::Error::new(
                                //             attr.value_span.unwrap_or(attr.span.span()),
                                //             "ForeignKey Table not found",
                                //         ));
                                //     }
                                // };
                                //
                                // if !table.is_valid_column(column) {
                                //     return Err(syn::Error::new(
                                //         attr.span.span(),
                                //         "ForeignKey Column not found in Table",
                                //     ));
                                // }
                                self.coltype =
                                    ColumnTypeDerive::ForeignKey(ColumnTypeOptionsDerive {
                                        foreign_key: format!("{}.{}", table, column),
                                        ..Default::default()
                                    });
                            }
                        }
                    }
                    GeekAttributeKeys::Rand => {
                        let len: usize = attributes
                            .iter()
                            .find(|a| a.key == Some(GeekAttributeKeys::RandLength))
                            .map(|a| {
                                if let Some(GeekAttributeValue::Int(len)) = &a.value {
                                    *len as usize
                                } else {
                                    32
                                }
                            })
                            .unwrap_or(32);

                        let prefix: Option<String> = attributes
                            .iter()
                            .find(|a| a.key == Some(GeekAttributeKeys::RandPrefix))
                            .and_then(|a| {
                                if let Some(GeekAttributeValue::String(prefix)) = &a.value {
                                    Some(prefix.clone())
                                } else {
                                    None
                                }
                            });

                        let env = attributes
                            .iter()
                            .find(|a| a.key == Some(GeekAttributeKeys::RandEnv))
                            .and_then(|a| {
                                if let Some(GeekAttributeValue::String(env)) = &a.value {
                                    Some(env.clone())
                                } else {
                                    None
                                }
                            });

                        self.mode = Some(ColumnMode::Rand { len, prefix, env });
                    }
                    GeekAttributeKeys::Hash => {
                        self.mode = Some(ColumnMode::Hash(HashingAlgorithm::Pbkdf2));
                    }
                    _ => {
                        // Skip
                    }
                }
            } else {
                // TODO(geekmasher): Handle this better
            }
        }
        Ok(())
    }

    pub(crate) fn is_primary_key(&self) -> bool {
        // Check the options for a primary key
        match &self.coltype {
            ColumnTypeDerive::Identifier(_) => true,
            ColumnTypeDerive::Text(opts) => {
                if opts.primary_key {
                    return true;
                }
                false
            }
            ColumnTypeDerive::Integer(opts) => {
                if opts.primary_key {
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    pub(crate) fn is_foreign_key(&self) -> bool {
        matches!(&self.coltype, ColumnTypeDerive::ForeignKey(_))
    }

    /// Check if the column is unique
    pub(crate) fn is_unique(&self) -> bool {
        match &self.coltype {
            ColumnTypeDerive::Identifier(opts) => opts.unique,
            ColumnTypeDerive::Text(opts) => opts.unique,
            ColumnTypeDerive::Integer(opts) => opts.unique,
            ColumnTypeDerive::ForeignKey(opts) => opts.unique,
            ColumnTypeDerive::Blob(opts) => opts.unique,
            _ => false,
        }
    }

    pub(crate) fn is_searchable(&self) -> bool {
        matches!(&self.mode, Some(ColumnMode::Searchable { enabled: true }))
    }

    /// Convert the column into a list of parameters for a function
    pub(crate) fn to_params(&self) -> Option<TokenStream> {
        // Skip the column if it's marked as such
        if self.skip {
            return None;
        }

        let identifier = &self.identifier;
        let itype = &self.itype;

        // Modes
        if let Some(ColumnMode::Rand { .. }) = &self.mode {
            return None;
        } else if let Some(ColumnMode::New { enabled, data }) = &self.mode {
            if !*enabled || data.is_some() {
                return None;
            }
            // Let it continue to the default
        }

        // Ignore PrimaryKey / ForeignKey / Option<T>
        if let Type::Path(TypePath { path, .. }) = itype {
            if let Some(segment) = path.segments.first() {
                match segment.ident.to_string().as_str() {
                    "Option" | "PrimaryKey" | "PrimaryKeyInteger" => return None,
                    "ForeignKey" => {
                        // ForeignKey<T, D>

                        let inner_key_type = match segment.arguments {
                            syn::PathArguments::AngleBracketed(ref args) => args
                                .args
                                .first()
                                .unwrap_or_else(|| panic!("No inner type found in ForeignKey")),
                            _ => panic!("Unsupported ForeignKey type (to_params)"),
                        };
                        // TODO(geekmasher): Do we care about the inner table type?
                        return Some(self.to_params_foreign_key_int(identifier, inner_key_type));
                    }
                    "ForeignKeyInteger" => {
                        // GenericArgument of i32
                        let inner_key_type = GenericArgument::Type(syn::parse_quote! { i32 });
                        return Some(self.to_params_foreign_key_int(identifier, &inner_key_type));
                    }
                    _ => {}
                }
            }
        }
        Some(quote! {
            #identifier: impl Into< #itype >
        })
    }

    pub(crate) fn to_params_foreign_key_int(
        &self,
        identifier: &Ident,
        inner_type: &GenericArgument,
    ) -> TokenStream {
        quote! {
            #identifier: impl Into< #inner_type >
        }
    }

    /// Create a new instance of the struct and pass in the column
    pub(crate) fn to_self(&self) -> TokenStream {
        let identifier = &self.identifier;

        // For Skipped columns, return the identifier
        if self.skip {
            return quote! { #identifier: Default::default() };
        }

        // Modes
        if let Some(ColumnMode::New { enabled, data }) = &self.mode {
            if !*enabled {
                return quote! { #identifier: Default::default() };
            } else if let Some(data) = data {
                // TODO: We might want to handle this better as users can pass in any data
                // to this field
                let data = syn::parse_str::<TokenStream>(data)
                    .map_err(|err| {
                        syn::Error::new(
                            self.span(),
                            format!("Failed to parse data for New mode: {}", err),
                        )
                    })
                    .unwrap();
                return quote! { #identifier: #data };
            }
        } else if let Some(ColumnMode::Rand { len, prefix, env }) = &self.mode {
            let mut pre = String::new();
            if let Some(prefix) = prefix {
                pre.push_str(prefix.as_str());
                pre.push('_');
            }
            if let Some(env) = env {
                pre.push_str(env.as_str());
                pre.push('_');
            }

            return quote! {
                #identifier: geekorm::utils::generate_random_string(#len, #pre)
            };
        } else if let Some(ColumnMode::Hash(alg)) = &self.mode {
            let hash = alg.to_str();
            return quote! {
                #identifier: geekorm::utils::generate_hash(
                    #identifier.into(),
                    geekorm::utils::crypto::HashingAlgorithm::try_from(#hash).unwrap_or_default()
                ).unwrap()
            };
        }

        if let Type::Path(TypePath { path, .. }) = &self.itype {
            if let Some(segment) = path.segments.first() {
                match segment.ident.to_string().as_str() {
                    "Option" => {
                        // Option is always None in new()
                        return quote! {
                            #identifier: None
                        };
                    }
                    // TODO(geekmasher): Add PrimaryKey<T> support
                    "PrimaryKey" | "PrimaryKeyInteger" | "PrimaryKeyString" | "PrimaryKeyUuid" => {
                        // Generate a new primary key
                        return quote! {
                            #identifier: geekorm::PrimaryKey::default()
                        };
                    }
                    "ForeignKey" => {
                        // Generate a new foreign key
                        return quote! {
                            #identifier: geekorm::ForeignKey::from(#identifier.into())
                        };
                    }
                    _ => {}
                }
            }
        }
        quote! {
            #identifier: #identifier.into()
        }
    }

    pub(crate) fn get_selector(&self, table_ident: &Ident) -> TokenStream {
        let identifier = &self.identifier;
        let name = &self.name;

        if self.skip {
            return quote! {};
        }

        let func_name = format!("query_select_by_{}", identifier);
        let func = Ident::new(&func_name, Span::call_site());

        quote! {
            /// Select by the column value
            pub fn #func(value: impl Into<geekorm::Value>) -> geekorm::Query {
                geekorm::QueryBuilder::select()
                    .table(#table_ident::table())
                    .where_eq(#name, value.into())
                    .build()
                    .expect("Failed to build query")
            }
        }
    }

    /// Generate a fetcher function for the column
    #[allow(unused_variables)]
    pub(crate) fn get_fetcher_pk(&self, ident: &Ident) -> TokenStream {
        let identifier = &self.identifier; // `user`

        quote! {
            /// Fetch a row by the primary key value
            pub async fn fetch_by_primary_key<'a, C>(
                connection: &'a C,
                pk: impl Into<geekorm::Value>
            ) -> Result<Self, geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
                Self: geekorm::QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned,
            {
                let mut r: #ident = C::query_first::<Self>(
                    connection,
                    #ident::query_select_by_primary_key(pk.into())
                ).await?;

                r.fetch(connection).await?;

                Ok(r)
            }
        }
    }

    /// Generate a fetcher function for the column
    #[allow(unused_variables)]
    pub(crate) fn get_fetcher(&self, table_ident: &Ident, foreign_ident: &Ident) -> TokenStream {
        let identifier = &self.identifier;

        if self.skip {
            return quote! {};
        }

        let func_name = format!("fetch_{}", identifier);
        let func = Ident::new(&func_name, Span::call_site());

        quote! {
            /// Fetch the foreign key data for the column
            pub async fn #func<'a, C>(
                &mut self,
                connection: &'a C
            ) -> Result<#foreign_ident, geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
                Self: geekorm::QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned
            {
                let q = #foreign_ident::query_select_by_primary_key(self.#identifier.key);
                let r = C::query_first::<#foreign_ident>(connection, q).await?;
                self.#identifier.data = r.clone();
                Ok(r)
            }
        }
    }

    /// Generate a hash helper functions for the column
    pub(crate) fn get_hash_helpers(&self) -> TokenStream {
        let identifier = &self.identifier;

        let hash_func_name = format!("hash_{}", identifier);
        let hash_func = Ident::new(&hash_func_name, Span::call_site());

        let check_func_name = format!("check_{}", identifier);
        let check_func = Ident::new(&check_func_name, Span::call_site());

        quote! {
            /// Hash the data for the column
            pub fn #hash_func(&mut self, data: impl Into<String>) -> Result<(), geekorm::Error> {
                self.#identifier = geekorm::utils::generate_hash(
                    data.into(),
                    geekorm::utils::crypto::HashingAlgorithm::Pbkdf2
                )?;

                Ok(())
            }

            /// Check / Verify the hash for the column
            pub fn #check_func(&self, data: impl Into<String>) -> Result<bool, geekorm::Error> {
                geekorm::utils::verify_hash(
                    data.into(),
                    self.#identifier.clone(),
                    geekorm::utils::crypto::HashingAlgorithm::Pbkdf2
                )
            }
        }
    }

    pub(crate) fn get_random_helpers(&self) -> TokenStream {
        let identifier = &self.identifier;

        let random_func_name = format!("regenerate_{}", identifier);
        let random_func = Ident::new(&random_func_name, Span::call_site());

        let len: usize = if let Some(ColumnMode::Rand { len, .. }) = self.mode {
            len
        } else {
            10
        };
        let prefix: String = if let Some(mode) = &self.mode {
            match mode {
                ColumnMode::Rand { prefix, .. } => prefix.clone().unwrap_or_default(),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        quote! {
            /// Generate a random value for the column
            pub fn #random_func(&mut self) {
                self.#identifier = geekorm::utils::generate_random_string(#len, String::from(#prefix));
            }
        }
    }
}

impl Default for ColumnDerive {
    fn default() -> Self {
        ColumnDerive {
            name: String::new(),
            coltype: ColumnTypeDerive::Text(ColumnTypeOptionsDerive::default()),
            alias: String::new(),
            skip: false,
            update: None,
            save: None,
            attributes: Vec::new(),
            identifier: Ident::new("column", Span::call_site()),
            itype: syn::parse_quote! { String },
            mode: None,
        }
    }
}

impl Debug for ColumnDerive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnDerive")
            .field("name", &self.name)
            .field("coltype", &self.coltype)
            .finish()
    }
}

impl ToTokens for ColumnDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let coltype = &self.coltype;
        let alias = &self.alias;
        let skip = &self.skip;

        tokens.extend(quote! {
            geekorm::Column {
                name: String::from(#name),
                column_type: #coltype,
                alias: String::from(#alias),
                skip: #skip,
            }
        });
    }
}

impl From<ColumnDerive> for geekorm_core::Column {
    fn from(value: ColumnDerive) -> Self {
        geekorm_core::Column {
            name: value.name,
            column_type: ColumnType::from(value.coltype),
            alias: value.alias,
            skip: value.skip,
        }
    }
}

impl TryFrom<&Field> for ColumnDerive {
    type Error = syn::Error;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let name: Ident = match &value.ident {
            Some(ident) => ident.clone(),
            None => {
                return Err(syn::Error::new(
                    value.span(),
                    "Column must have an identifier",
                ));
            }
        };

        let itype = value.ty.clone();
        let attributes = GeekAttribute::parse_all(&value.attrs)?;
        let coltype = ColumnTypeDerive::try_from(&itype)?;

        let mut col = ColumnDerive {
            name: name.to_string(),
            identifier: name,
            itype,
            attributes,
            coltype,
            alias: String::from(""),
            skip: false,
            update: None,
            save: None,
            mode: None,
        };
        col.apply_attributes()?;

        // TODO(geekmasher): Check if the column is public
        // if let Some(ref mode) = col.mode {
        //     if let ColumnMode::Hash(_) = mode {
        //         if let Visibility::Public(Pub { .. }) = value.vis {
        //             todo!("")
        //         }
        //     }
        // }

        Ok(col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_key() {
        let column = ColumnDerive {
            name: "id".to_string(),
            identifier: Ident::new("id", Span::call_site()),
            itype: syn::parse_quote! { i32 },
            attributes: vec![],
            coltype: ColumnTypeDerive::Identifier(Default::default()),
            alias: String::from(""),
            skip: false,
            mode: None,
            ..Default::default()
        };
        assert!(column.is_primary_key());

        let column = ColumnDerive {
            name: "id".to_string(),
            identifier: Ident::new("id", Span::call_site()),
            itype: syn::parse_quote! { i32 },
            attributes: vec![],
            coltype: ColumnTypeDerive::Text(ColumnTypeOptionsDerive {
                primary_key: true,
                ..Default::default()
            }),
            alias: String::from(""),
            skip: false,
            mode: None,
            ..Default::default()
        };
        assert!(column.is_primary_key());

        let column = ColumnDerive {
            name: "id".to_string(),
            identifier: Ident::new("id", Span::call_site()),
            itype: syn::parse_quote! { i32 },
            attributes: vec![],
            coltype: ColumnTypeDerive::Integer(ColumnTypeOptionsDerive {
                primary_key: true,
                ..Default::default()
            }),
            alias: String::from(""),
            skip: false,
            mode: None,
            ..Default::default()
        };
        assert!(column.is_primary_key());
    }
}
