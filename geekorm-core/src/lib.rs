//! GeekORM is a simple ORM for SQLite databases.
#![forbid(unsafe_code)]
#![allow(dead_code)]
#![deny(missing_docs)]

/// Backend module
pub mod backends;
/// Builder module
pub mod builder;
/// Error module
pub mod error;
/// Query module
pub mod queries;

#[cfg(feature = "libsql")]
pub use backends::libsql;

pub use crate::backends::{GeekConnection, GeekConnector};
pub use crate::error::Error;

pub use crate::builder::columns::{Column, Columns};
pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use crate::builder::keys::{ForeignKey, PrimaryKey};
pub use crate::builder::table::Table;
pub use crate::builder::values::{Value, Values};
pub use crate::queries::{Query, QueryBuilder};

/// Trait for creating tables
pub trait TableBuilder
where
    Self: Sized,
{
    /// Get the table struct
    fn table() -> Table;

    /// Get the table struct for the current instance
    fn get_table(&self) -> Table;

    /// Get the name of the table
    fn table_name() -> String;
}

/// Trait for Building Queries
pub trait QueryBuilderTrait
where
    Self: TableBuilder + Sized,
{
    /// Create a new table
    fn create() -> QueryBuilder;

    /// Select rows in the table
    fn select() -> QueryBuilder {
        QueryBuilder::select()
    }

    /// Select all rows in the table
    fn all() -> Query {
        Self::select()
            .table(Self::table())
            .build()
            .expect("Failed to build SELECT ALL query")
    }

    /// Insert a row into the table
    fn insert(item: &Self) -> Query;

    /// Update a row in the table
    fn update(item: &Self) -> Query;

    /// Count the rows in the table
    fn count() -> QueryBuilder;
}

/// Trait for Tables with a primary key
///
pub trait TablePrimaryKey
where
    Self: TableBuilder + QueryBuilderTrait + Sized,
{
    /// Get the name of the primary key column
    fn primary_key() -> String;

    /// Get the primary key column name
    fn primary_key_value(&self) -> Value;

    /// Select a row by the primary key
    fn select_by_primary_key(&self) -> Query {
        Self::select()
            .table(Self::table())
            .where_eq(&Self::primary_key(), self.primary_key_value())
            .build()
            .expect("Failed to build SELECT BY PRIMARY KEY query")
    }
}

/// Trait for converting a struct to SQLite
///
/// This does not need to be implemented by the user and is used internally
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
    fn on_insert(&self, query: &QueryBuilder) -> Result<(String, Values), Error> {
        Err(Error::QueryBuilderError(
            format!("on_insert not implemented for table: {}", query.table),
            String::from("on_insert"),
        ))
    }

    /// Convert to SQLite for updating a row
    fn on_update(&self, query: &QueryBuilder) -> Result<(String, Values), Error> {
        Err(Error::QueryBuilderError(
            format!("on_update not implemented for table: {}", query.table),
            String::from("on_update"),
        ))
    }
}
