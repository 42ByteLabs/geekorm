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

impl From<geekorm_sql::Query> for Query {
    fn from(value: geekorm_sql::Query) -> Self {
        // TODO: This is a temp solution
        Query {
            query: value.sql(),
            query_type: value.query_type().clone().into(),
            parameters: value.parameters().clone(),
            values: value.values().clone(),
            ..Default::default()
        }
    }
}

impl From<Query> for geekorm_sql::Query {
    fn from(value: Query) -> Self {
        geekorm_sql::Query::from((
            value.query,
            value.query_type.into(),
            value.parameters,
            value.values,
        ))
    }
}

impl From<QueryType> for geekorm_sql::QueryType {
    fn from(value: QueryType) -> Self {
        match value {
            QueryType::Create => geekorm_sql::QueryType::Create,
            QueryType::Select => geekorm_sql::QueryType::Select,
            QueryType::Insert => geekorm_sql::QueryType::Insert,
            QueryType::Update => geekorm_sql::QueryType::Update,
            QueryType::Delete => geekorm_sql::QueryType::Delete,
            _ => geekorm_sql::QueryType::Unknown,
        }
    }
}
