//! # GeekORM SQL
//!
//! This crate provides a SQL Builder for GeekORM but can be used as a standalone
//! library.
//!
//! It is a simple SQL builder that can be used to build SQL queries in a
//! programmatic way using the builder pattern.
#![allow(dead_code, unused_imports)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod backends;
pub mod error;
#[cfg(feature = "parser")]
pub mod parser;
pub mod queries;
pub mod query;
pub mod sql;
pub mod values;

pub use backends::QueryBackend;
pub use error::Error;
pub use query::{
    Query, QueryBuilder, QueryCondition, QueryOrder, QueryType, WhereCondition,
    columns::{Column, ColumnOptions, Columns},
    columntypes::ColumnType,
    table::{Table, TableExpr},
};
pub use sql::SqlQuery;
pub use values::{value::Value, values::Values};

/// To SQL trait
pub trait ToSql {
    /// Convert to SQL string, no other input
    fn sql(&self) -> String {
        String::new()
    }

    /// Convert to SqlQuery string
    fn to_sql(&self, query: &Query) -> Result<SqlQuery, Error> {
        let mut sql = SqlQuery::new();
        self.to_sql_stream(&mut sql, query)?;
        Ok(sql)
    }

    /// Construct SQL string using a stream
    fn to_sql_stream(&self, stream: &mut SqlQuery, query: &Query) -> Result<(), Error> {
        let sql = self.to_sql(query)?;
        stream.push_str(&sql.to_string());
        Ok(())
    }
}

/// Trait for converting to a Value
pub trait ToValue {
    /// Convert to Value
    fn to_value(&self) -> Value;
}
