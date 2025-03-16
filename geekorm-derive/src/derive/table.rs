use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{GenericArgument, Ident, Type, TypePath};

use crate::attr::{GeekAttribute, GeekAttributeKeys, GeekAttributeValue};
use crate::derive::column::{ColumnDerive, ColumnsDerive};

use geekorm_core::{PrimaryKey, Table};

#[derive(Debug, Clone)]
pub(crate) struct TableDerive {
    pub name: String,
    // pub alias: Option<String>,
    pub columns: ColumnsDerive,
    /// Database name
    pub database: Option<String>,
}

impl TableDerive {
    pub(crate) fn apply_attributes(&mut self, attributes: &Vec<GeekAttribute>) {
        for attr in attributes {
            if let Some(GeekAttributeKeys::Key) = &attr.key {
                if let Some(GeekAttributeValue::String(name)) = &attr.value {
                    self.name = name.to_string();
                }
            } else if let Some(GeekAttributeKeys::Rename) = &attr.key {
                if let Some(GeekAttributeValue::String(name)) = &attr.value {
                    self.name = name.to_string();
                }
            } else if Some(GeekAttributeKeys::Database {}) == attr.key {
                if let Some(GeekAttributeValue::String(name)) = &attr.value {
                    self.database = Some(name.to_string());
                }
            }
        }
    }
}

impl ToTokens for TableDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let columns = &self.columns;
        let database = self.database.clone().unwrap_or("Database".to_string());

        tokens.extend(quote! {
            geekorm::Table {
                name: String::from(#name),
                columns: #columns,
                database: Some(String::from(#database)),
            }
        });
    }
}

impl From<TableDerive> for Table {
    fn from(value: TableDerive) -> Self {
        Table {
            name: value.name,
            columns: value.columns.into(),
            database: value.database,
        }
    }
}
