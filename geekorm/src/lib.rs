#![forbid(unsafe_code)]
#![allow(dead_code)]
pub mod builder;
pub mod queries;

pub use crate::builder::columns::{Column, Columns};
pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use crate::builder::table::Table;
pub use crate::queries::{Query, QueryBuilder};

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("QueryBuilderError: {0} ({1})")]
    QueryBuilderError(String, String),

    #[error("Unknown Error / Generic Error occurred")]
    Unknown,
}

pub trait TableBuilder {
    /// Get the table struct
    fn table() -> Table;
    /// Get the name of the table
    fn table_name() -> String;

    /// Create a new table
    fn create() -> QueryBuilder;
    /// Select rows in the table
    fn select() -> QueryBuilder {
        QueryBuilder::select()
    }

    /// Count the rows in the table
    fn count() -> QueryBuilder;
}

pub trait ToSqlite {
    fn to_sqlite(&self) -> String {
        String::new()
    }

    fn on_create(&self) -> String {
        String::new()
    }

    fn on_select(&self, query: &QueryBuilder) -> Result<String, Error> {
        Err(Error::QueryBuilderError(
            format!("on_select not implemented for table: {}", query.table),
            String::from("on_select"),
        ))
    }
}

#[doc(hidden)]
pub mod prelude {
    pub use crate::builder::columns::{Column, Columns};
    pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
    pub use crate::builder::table::Table;

    pub use crate::builder::models::{QueryCondition, QueryOrder, QueryType};
    pub use crate::queries::QueryBuilder;

    pub use crate::TableBuilder;
    pub use crate::ToSqlite;
}
