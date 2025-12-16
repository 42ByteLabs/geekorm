//! # SQL Modules
//!
//! This is where the builders for SQL queries are defined.

pub mod postgres;
pub mod sqlite;

use crate::{Error, ToSql, Values};

/// SQL Query struct
///
/// This is the final SQL query with its associated values.
#[derive(Debug, Clone, Default)]
pub struct SqlQuery {
    query: String,
    parameters: Values,
    values: Values,
}

impl SqlQuery {
    /// Create a new SQL Query
    pub fn new() -> Self {
        SqlQuery::default()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    pub(crate) fn ends_with(&self, c: &str) -> bool {
        self.query.ends_with(c)
    }

    pub(crate) fn starts_with(&self, c: &str) -> bool {
        self.query.starts_with(c)
    }

    pub(crate) fn trim(&mut self) {
        self.query = self.query.trim().to_string();
    }

    pub(crate) fn push(&mut self, sql: char) {
        self.query.push(sql);
    }

    pub(crate) fn push_str(&mut self, sql: impl AsRef<str>) {
        self.query.push_str(sql.as_ref());
    }
}

impl ToString for SqlQuery {
    fn to_string(&self) -> String {
        self.query.clone()
    }
}

impl From<String> for SqlQuery {
    fn from(query: String) -> Self {
        SqlQuery {
            query,
            parameters: Values::new(),
            values: Values::new(),
        }
    }
}
