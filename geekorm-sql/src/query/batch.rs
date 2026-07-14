//! # Batches of Queries
//!
//! This is primary used for transactions
use std::path::PathBuf;

use crate::{Error, QueryBuilder, QueryType, ToSql, Values};

use super::Query;

/// A collection of queries to be executed in a batch
#[derive(Debug, Clone, Default)]
pub struct BatchQueries {
    /// Queries
    queries: Vec<Query>,
}

impl BatchQueries {
    /// New
    pub fn new() -> Self {
        Self::default()
    }

    /// Append query
    pub fn push(&mut self, query: Query) {
        self.queries.push(query)
    }

    /// Load a SQL file
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        if !path.exists() && !path.is_file() {
            return Err(Error::SqlFileNotFound {
                path: path.display().to_string(),
            });
        }
        let query = std::fs::read_to_string(path)?;

        // Split by semicolon to get individual queries
        let queries: Vec<String> = query
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut batch_queries = BatchQueries::new();
        for query in queries {
            // TODO: Determine query type based on the content of the query
            let query_type = QueryType::Unknown;
            let values = Values::new(); // Placeholder for actual values
            let params = Values::new(); // Placeholder for actual parameters

            batch_queries.queries.push(Query {
                query,
                query_type,
                values,
                params,
            });
        }
        Ok(batch_queries)
    }

    /// Gets all the internal queries
    pub fn queries(&self) -> &Vec<Query> {
        &self.queries
    }
}

impl From<Vec<Query>> for BatchQueries {
    fn from(value: Vec<Query>) -> Self {
        BatchQueries { queries: value }
    }
}

impl From<&Vec<Query>> for BatchQueries {
    fn from(value: &Vec<Query>) -> Self {
        BatchQueries::from(value.clone())
    }
}
