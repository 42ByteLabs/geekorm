use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
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
    pub columns: ColumnsDerive,
}

impl TableDerive {
    pub(crate) fn get_primary_key(&self) -> Option<ColumnDerive> {
        for column in &self.columns.columns {
            if column.is_primary_key() {
                return Some(column.clone());
            }
        }
        None
    }

    #[allow(irrefutable_let_patterns)]
    pub(crate) fn apply_attributes(&mut self, attributes: &Vec<GeekAttribute>) {
        for attr in attributes {
            if let Some(key) = &attr.key {
                match key {
                    GeekAttributeKeys::Rename => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                self.name = name.to_string();
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                // TODO(geekmasher): Handle this better
            }
        }
    }
}

impl ToTokens for TableDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let columns = &self.columns;
        tokens.extend(quote! {
            geekorm::Table {
                name: String::from(#name),
                columns: #columns
            }
        });
    }
}

impl From<TableDerive> for Table {
    fn from(value: TableDerive) -> Self {
        Table {
            name: value.name,
            columns: value.columns.into(),
        }
    }
}

impl From<&Table> for TableDerive {
    fn from(value: &Table) -> Self {
        TableDerive {
            name: value.name.clone(),
            columns: (&value.columns).into(),
        }
    }
}
