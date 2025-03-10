/// The Query struct to hold the query and values to use
use std::fmt::Display;

use crate::builder::models::QueryType;
use crate::queries::QueryBuilder;
use crate::{Table, builder::values::Values};

/// The built Query struct with the query and values to use
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// The type of query (select, insert, update, delete)
    pub query_type: QueryType,
    /// The resulting SQLite Query
    pub query: String,
    /// The values to use in the query (where / insert / update)
    pub values: Values,
    /// List of parameters for the query (update / insert)
    pub parameters: Values,

    /// The output columns for the query (used in raw queries)
    pub columns: Vec<String>,

    pub(crate) table: Table,
}

impl Query {
    /// Create a new Query
    pub fn new(
        query_type: QueryType,
        query: String,
        values: Values,
        parameters: Values,
        columns: Vec<String>,
        table: Table,
    ) -> Self {
        Query {
            query_type,
            query,
            values,
            parameters,
            columns,
            table,
        }
    }

    /// Initialize using the QueryBuilder struct
    pub fn init() -> QueryBuilder {
        QueryBuilder::default()
    }

    /// Batch query
    pub fn batch(statement: impl Into<String>) -> Self {
        Query::new(
            QueryType::Batch,
            statement.into(),
            Values::default(),
            Values::default(),
            vec![],
            Table::default(),
        )
    }

    /// Get the query as a &str
    pub fn to_str(&self) -> &str {
        &self.query
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query)
    }
}
