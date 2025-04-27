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

use geekorm_core::Error;

pub mod backends;
pub mod builder;
pub mod query;

pub use builder::{QueryBuilder, QueryCondition, QueryOrder, QueryType, WhereCondition};
pub use query::Query;

use self::backends::QueryBackend;

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
