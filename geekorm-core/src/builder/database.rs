//! # GeekORM Database
use serde::{Deserialize, Serialize};

use super::table::Table;

/// GeekORM Database
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    /// The tables in the database
    pub tables: Vec<Table>,
}

impl Database {
    /// Find a table by name
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|table| table.name == name)
    }
}
