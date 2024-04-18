use geekorm_core::ColumnType;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{
    parse::Parse, spanned::Spanned, Attribute, Field, GenericArgument, Ident, Type, TypePath,
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
    pub(crate) skip: bool,
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
                    GeekAttributeKeys::Rename => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                self.alias = name.to_string();
                            }
                        }
                    }
                    GeekAttributeKeys::ForeignKey => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                let (table, column) = match name.split_once('.') {
                                    Some((table, column)) => (table, column),
                                    None => {
                                        return Err(syn::Error::new(
                                            attr.span.span(),
                                            "Invalid foreign key format (table.column)",
                                        ))
                                    }
                                };

                                let tables = TableState::load_state_file();
                                let table = match tables.find_table(table) {
                                    Some(table) => table,
                                    None => {
                                        return Err(syn::Error::new(
                                            attr.span.span(),
                                            "ForeignKey Table not found",
                                        ))
                                    }
                                };

                                if !table.is_valid_column(column) {
                                    return Err(syn::Error::new(
                                        attr.span.span(),
                                        "ForeignKey Column not found in Table",
                                    ));
                                }
                                self.coltype =
                                    ColumnTypeDerive::ForeignKey(ColumnTypeOptionsDerive {
                                        foreign_key: name.to_string(),
                                        ..Default::default()
                                    });
                            }
                        }
                    }
                    _ => {}
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

    /// Convert the column into a list of parameters for a function
    pub(crate) fn to_params(&self) -> Option<TokenStream> {
        // Skip the column if it's marked as such
        if self.skip {
            return None;
        }

        let identifier = &self.identifier;
        let itype = &self.itype;
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
            #identifier: #itype
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
            #identifier
        }
    }

    pub(crate) fn get_selector(&self, table_ident: &Ident) -> TokenStream {
        let identifier = &self.identifier;
        let name = &self.name;

        let func_name = format!("select_by_{}", identifier);
        let func = Ident::new(&func_name, Span::call_site());

        quote! {
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
    pub(crate) fn get_fetcher(&self, table_ident: &Ident, foreign_ident: &Ident) -> TokenStream {
        let identifier = &self.identifier; // `user`

        let func_name = format!("fetch_{}", identifier);
        let func = Ident::new(&func_name, Span::call_site());

        quote! {
            pub async fn #func(&mut self, connection: &impl geekorm::prelude::GeekConnection) -> Result<#foreign_ident, geekorm::Error> {
                Err(geekorm::Error::NotImplemented)
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
            attributes: Vec::new(),
            identifier: Ident::new("column", Span::call_site()),
            itype: syn::parse_quote! { String },
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
                ))
            }
        };

        let itype = value.ty.clone();
        let attributes = match GeekAttribute::parse_all(&value.attrs) {
            Ok(attributes) => attributes,
            Err(e) => return Err(e),
        };
        let coltype = match ColumnTypeDerive::try_from(&itype) {
            Ok(coltype) => coltype,
            Err(e) => return Err(e),
        };

        let mut col = ColumnDerive {
            name: name.to_string(),
            identifier: name,
            itype,
            attributes,
            coltype,
            alias: String::from(""),
            skip: false,
        };
        col.apply_attributes()?;
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
        };
        assert!(column.is_primary_key());
    }
}
