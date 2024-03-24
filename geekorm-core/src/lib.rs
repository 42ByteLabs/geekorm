//! GeekORM is a simple ORM for SQLite databases.
#![forbid(unsafe_code)]
#![allow(dead_code)]
#![warn(missing_docs)]

/// Builder module
pub mod builder;
/// Query module
pub mod queries;

#[cfg(feature = "backends")]
pub(crate) mod backends;

pub use crate::builder::columns::{Column, Columns};
pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use crate::builder::keys::{ForeignKey, PrimaryKey};
pub use crate::builder::table::Table;
pub use crate::queries::{Query, QueryBuilder};

use thiserror::Error;

/// Error type for the crate
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Query Builder Error
    #[error("QueryBuilderError: {0} ({1})")]
    QueryBuilderError(String, String),

    /// Unknown / Generic Error
    #[error("Unknown Error / Generic Error occurred")]
    Unknown,
}

/// Trait for creating tables
pub trait TableBuilder {
    /// Get the table struct
    fn table() -> Table
    where
        Self: Sized;

    /// Get the table struct for the current instance
    fn get_table(&self) -> Table
    where
        Self: Sized;

    /// Get the name of the table
    fn table_name() -> String
    where
        Self: Sized;

    /// Get the primary key name
    fn get_primary_key(&self) -> Option<String>;

    /// Create a new table
    fn create() -> QueryBuilder
    where
        Self: Sized;
    /// Select rows in the table
    fn select() -> QueryBuilder
    where
        Self: Sized,
    {
        QueryBuilder::select()
    }

    /// Count the rows in the table
    fn count() -> QueryBuilder
    where
        Self: Sized;
}

/// Trait for converting a struct to SQLite
pub trait ToSqlite {
    /// Convert to generic SQLite (only use for some generic types)
    fn to_sqlite(&self) -> String {
        String::new()
    }

    /// Convert to SQLite for creating a table
    fn on_create(&self) -> String {
        String::new()
    }

    /// Convert to SQLite for selecting a row
    fn on_select(&self, query: &QueryBuilder) -> Result<String, Error> {
        Err(Error::QueryBuilderError(
            format!("on_select not implemented for table: {}", query.table),
            String::from("on_select"),
        ))
    }
}
