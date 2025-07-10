//! # Query Builder

use std::fmt::Display;
use std::path::PathBuf;

use crate::{Error, QueryType, Values};

/// A collection of queries to be executed in a batch
pub struct BatchQueries {
    pub(crate) queries: Vec<Query>,
}

/// Query Builder
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
    /// Get the SQL query string
    pub fn query(&self) -> String {
        self.query.clone()
    }
}

impl BatchQueries {
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

        let mut batch_queries = BatchQueries {
            queries: Vec::new(),
        };
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
}
