//! # GeekORM Database
use serde::{Deserialize, Serialize};

use super::table::Table;
use crate::Column;

/// GeekORM Database
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
    /// The tables in the database
    pub tables: Vec<Table>,
}

impl Database {
    /// Find a table by name
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|table| table.name == name)
    }

    /// Get the column by table and column name
    pub fn get_table_column(&self, table: &str, column: &str) -> Option<&Column> {
        self.get_table(table)
            .unwrap()
            .columns
            .columns
            .iter()
            .find(|col| col.name == column)
    }

    /// Get the list of table names
    pub fn get_table_names(&self) -> Vec<&str> {
        self.tables.iter().map(|t| t.name.as_str()).collect()
    }

    /// Get the list of columns for a table name
    pub fn get_table_columns(&self, table: &str) -> Vec<&str> {
        self.get_table(table)
            .unwrap()
            .columns
            .columns
            .iter()
            .map(|col| col.name.as_str())
            .collect()
    }
}

#[cfg(feature = "migrations")]
impl quote::ToTokens for Database {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // TODO: Support for multiple databases

        // let name = self.name.clone();
        // let name_ident = format_ident!("{}", name);
        let tables = &self.tables;

        tokens.extend(quote::quote! {
            pub static ref Database: Box<geekorm::Database> = Box::new(
                geekorm::Database {
                    tables: Vec::from([
                        #(#tables),*
                    ])
                }
            );
        });
    }
}
