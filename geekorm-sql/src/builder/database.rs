//! # Database

use serde::{Deserialize, Serialize};

use crate::{Column, Table};

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
