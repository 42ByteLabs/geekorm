//! GeekORM is a simple ORM for SQLite databases.
#![forbid(unsafe_code)]
#![allow(dead_code)]
#![warn(missing_docs)]

/// Builder module
pub mod builder;
/// Query module
pub mod queries;

/// Backend module
pub mod backends;

#[cfg(feature = "libsql")]
pub use backends::libsql;

pub use crate::builder::columns::{Column, Columns};
pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use crate::builder::keys::{ForeignKey, PrimaryKey};
pub use crate::builder::table::Table;
pub use crate::builder::values::{Value, Values};
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

    /// Insert a row into the table
    fn insert(item: &Self) -> Query
    where
        Self: Sized;

    /// Count the rows in the table
    fn count() -> QueryBuilder
    where
        Self: Sized;
}

/// Trait for Tables with a primary key
///
pub trait TablePrimaryKey
where
    Self: TableBuilder,
{
    /// Get the name of the primary key column
    fn primary_key() -> String;

    /// Get the primary key column name
    fn primary_key_value(&self) -> Value;
}

/// Trait for converting a struct to SQLite
pub trait ToSqlite {
    /// Convert to generic SQLite (only use for some generic types)
    fn to_sqlite(&self) -> String {
        String::new()
    }

    /// Convert to SQLite for creating a table
    #[allow(unused_variables)]
    fn on_create(&self, query: &QueryBuilder) -> Result<String, Error> {
        Ok(String::new())
    }

    /// Convert to SQLite for selecting a row
    fn on_select(&self, query: &QueryBuilder) -> Result<String, Error> {
        Err(Error::QueryBuilderError(
            format!("on_select not implemented for table: {}", query.table),
            String::from("on_select"),
        ))
    }

    /// Convert to SQLite for inserting a row
    fn on_insert(&self, query: &QueryBuilder) -> Result<String, Error> {
        Err(Error::QueryBuilderError(
            format!("on_insert not implemented for table: {}", query.table),
            String::from("on_insert"),
        ))
    }
}
