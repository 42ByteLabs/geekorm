use geekorm_core::ColumnType;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{GenericArgument, Ident, Type, TypePath};

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
                if let ColumnTypeDerive::Identifier(_) = &col.coltype {
                    Some(col.clone())
                } else {
                    None
                }
            })
            .next()
    }

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
    pub(crate) alias: Option<String>,
    pub(crate) coltype: ColumnTypeDerive,
    pub(crate) skip: bool,
}

impl ColumnDerive {
    /// Create a new instance of Column
    pub(crate) fn new(identifier: Ident, itype: Type, attributes: Vec<GeekAttribute>) -> Self {
        let name = identifier.to_string();
        let coltype = ColumnTypeDerive::try_from(&itype).unwrap();
        let mut col = ColumnDerive {
            identifier,
            itype,
            attributes,
            name,
            coltype,
            alias: None,
            skip: false,
        };
        col.apply_attributes();
        col
    }

    #[allow(irrefutable_let_patterns)]
    pub(crate) fn apply_attributes(&mut self) {
        let attributes = &self.attributes;

        for attr in attributes {
            if let Some(key) = &attr.key {
                match key {
                    GeekAttributeKeys::Skip => {
                        self.skip = true;
                    }
                    GeekAttributeKeys::ForeignKey => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                // TODO(geekmasher): Handle this better
                                let (table, column) = name.split_once('.').unwrap_or_else(|| {
                                    panic!("Invalid foreign key format (table.column): {}", name)
                                });

                                let tables = TableState::load_state_file();
                                let table = tables.find_table(table).unwrap_or_else(|| {
                                    panic!("ForeignKey Table '{}' not found", table)
                                });

                                if !table.is_valid_column(column) {
                                    panic!(
                                        "ForeignKey Column '{}' not found in Table '{}'",
                                        column, table.name
                                    )
                                }
                                self.alias = Some(name.to_string());

                                self.coltype =
                                    ColumnTypeDerive::ForeignKey(ColumnTypeOptionsDerive {
                                        foreign_key: name.to_string(),
                                        ..Default::default()
                                    });
                            }
                        }
                    }
                    GeekAttributeKeys::Skip | GeekAttributeKeys::Rename => {}
                }
            } else {
                // TODO(geekmasher): Handle this better
            }
        }
    }

    /// Convert the column into a list of parameters for a function
    pub(crate) fn to_params(&self) -> Option<TokenStream> {
        let identifier = &self.identifier;
        let itype = &self.itype;
        // Ignore PrimaryKey / ForeignKey / Option<T>
        if let Type::Path(TypePath { path, .. }) = itype {
            if let Some(segment) = path.segments.first() {
                match segment.ident.to_string().as_str() {
                    "Option" | "PrimaryKey" | "PrimaryKeyInteger" => return None,
                    "ForeignKey" => {
                        // We want a user to pass in the actual type in the ForeignKey
                        // so we need to extract the inner type
                        let inner_type = match segment.arguments {
                            syn::PathArguments::AngleBracketed(ref args) => {
                                args.args.first().unwrap()
                            }
                            _ => panic!("Unsupported ForeignKey type (to_params)"),
                        };
                        // Return the inner type
                        return Some(quote! {
                            #identifier: impl Into< #inner_type >
                        });
                    }
                    _ => {}
                }
            }
        }
        Some(quote! {
            #identifier: #itype
        })
    }

    /// Create a new instance of the struct and pass in the column
    pub(crate) fn to_self(&self) -> TokenStream {
        let identifier = &self.identifier;
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
                    "PrimaryKeyInteger" => {
                        // Generate a new primary key
                        return quote! {
                            #identifier: geekorm::PrimaryKeyInteger::new(0)
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
            fn #func(value: impl Into<geekorm::Value>) -> geekorm::Query {
                geekorm::QueryBuilder::select()
                    .table(#table_ident::table())
                    .where_eq(#name, value.into())
                    .build()
                    .expect("Failed to build query")
            }
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
        tokens.extend(quote! {
            geekorm::Column::new(
                String::from(#name),
                #coltype
            )
        });
    }
}

impl From<ColumnDerive> for geekorm_core::Column {
    fn from(value: ColumnDerive) -> Self {
        geekorm_core::Column {
            name: value.name,
            column_type: ColumnType::from(value.coltype),
        }
    }
}