//! # Query Types
//!
//! This is the Query Type definitions for GeekORM SQL Builder.

use crate::{Error, SqlQuery, ToSql};

/// Query Type enum
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum QueryType {
    /// Create Query
    Create,
    /// Count Query
    Count,
    /// Select Query
    Select,
    /// Insert Query
    Insert,
    /// Update Query
    Update,
    /// Delete Query
    Delete,

    /// Unknown Query
    #[default]
    Unknown,
}

impl ToSql for QueryType {
    fn to_sql(&self, query: &crate::Query) -> Result<SqlQuery, Error> {
        match self {
            QueryType::Create => Ok(self.sql_create(query)),
            QueryType::Select => Ok(self.sql_select(query)),
            QueryType::Count => Ok(self.sql_count(query)),
            QueryType::Insert => Ok(self.sql_insert(query)),
            QueryType::Update => Ok(self.sql_update(query)),
            QueryType::Delete => Ok(self.sql_delete(query)),
            QueryType::Unknown => Err(Error::QueryBuilderError {
                error: String::from("Unknown query type"),
                location: String::from("to_sql"),
            }),
        }
    }
}
