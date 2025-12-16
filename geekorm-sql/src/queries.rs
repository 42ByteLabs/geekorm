//! # GeekORM Queries

use std::path::PathBuf;

use super::{Query, QueryType};
use crate::{Error, SqlQuery, Values};

/// A collection of queries to be executed in a batch
pub struct Queries {
    pub(crate) queries: Vec<SqlQuery>,
}

impl Queries {
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

        let mut batch_queries = Queries {
            queries: Vec::new(),
        };
        for query in queries {
            let values = Values::new(); // Placeholder for actual values
            let params = Values::new(); // Placeholder for actual parameters

            // batch_queries.queries.push(SqlQuery::new(
            //     query,
            //     params,
            //     values,
            // ));
        }
        Ok(batch_queries)
    }
}
