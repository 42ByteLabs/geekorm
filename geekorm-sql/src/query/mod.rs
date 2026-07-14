//! # Query Builder

use std::fmt::Display;
use std::path::PathBuf;

pub mod batch;

use crate::builder::queries::transaction::TransactionQuery;
use crate::{Error, QueryBuilder, QueryType, ToSql, Values};
pub use batch::BatchQueries;

/// Query Builder
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// The SQL query string
    pub(crate) query: String,

    /// The type of query
    pub(crate) query_type: QueryType,

    /// Values are data for inserting
    pub(crate) values: Values,

    /// Parameters are data for binding
    pub(crate) params: Values,
}

impl Query {
    /// Start building a select query
    pub fn select() -> QueryBuilder<'static> {
        QueryBuilder::select()
    }

    /// CREATE table Query
    pub fn create() -> QueryBuilder<'static> {
        QueryBuilder::create()
    }

    /// INSERT Query
    pub fn insert() -> QueryBuilder<'static> {
        QueryBuilder::insert()
    }

    /// UPDATE Query
    pub fn update() -> QueryBuilder<'static> {
        QueryBuilder::update()
    }

    /// DELETE Query
    pub fn delete() -> QueryBuilder<'static> {
        QueryBuilder::delete()
    }

    /// Transaction Query
    pub fn transaction() -> TransactionQuery {
        TransactionQuery::new()
    }

    /// Get the SQL query string
    pub fn sql(&self) -> String {
        self.query.clone()
    }

    /// Get Query Type
    pub fn query_type(&self) -> &QueryType {
        &self.query_type
    }

    /// Get the parameters for the query
    pub fn parameters(&self) -> &Values {
        &self.params
    }

    /// Push a string onto the query
    pub(crate) fn push(&mut self, value: String) {
        self.query.push_str(&value);
    }
}

impl From<String> for Query {
    fn from(value: String) -> Self {
        Query {
            query: value,
            ..Default::default()
        }
    }
}
