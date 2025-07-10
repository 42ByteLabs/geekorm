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
pub mod builder;
pub mod error;
pub mod query;
pub mod values;

pub use backends::QueryBackend;
pub use builder::{
    QueryBuilder, QueryCondition, QueryOrder, QueryType, WhereCondition,
    columns::{Column, ColumnOptions, Columns},
    columntypes::ColumnType,
    table::{Table, TableExpr},
};
pub use error::Error;
pub use query::Query;
pub use values::{value::Value, values::Values};

/// To SQL trait
pub trait ToSql {
    /// Convert to SQL string
    fn sql(&self) -> String {
        let mut sql = String::new();
        self.to_sql_stream(&mut sql, &QueryBuilder::default())
            .unwrap();
        sql
    }

    /// Convert to SQL string with query
    fn to_sql(&self, query: &QueryBuilder) -> Result<String, Error> {
        let mut sql = String::new();
        self.to_sql_stream(&mut sql, query)?;
        Ok(sql)
    }

    /// Construct SQL string using a stream
    fn to_sql_stream(&self, stream: &mut String, query: &QueryBuilder) -> Result<(), Error> {
        let sql = self.to_sql(query)?;
        stream.push_str(&sql);
        Ok(())
    }
}

/// Trait for converting to a Value
pub trait ToValue {
    /// Convert to Value
    fn to_value(&self) -> Value;
}
