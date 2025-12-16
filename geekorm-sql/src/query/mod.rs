//! # Query Builder module

pub mod builder;
pub mod columns;
pub mod columntypes;
pub mod conditions;
pub mod joins;
pub mod ordering;
pub mod qtype;
pub mod table;

use std::collections::HashMap;

pub use builder::QueryBuilder;
pub use conditions::{QueryCondition, WhereClause, WhereCondition};
pub use joins::{TableJoin, TableJoinOptions, TableJoins};
pub use ordering::{OrderClause, QueryOrder};
pub use qtype::QueryType;
use table::Table;

use crate::{Error, QueryBackend, ToSql, Value, Values, sql::*};
use columns::Columns;

/// Query struct
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// Query Backend
    pub(crate) backend: QueryBackend,
    /// Query type
    pub(crate) query_type: QueryType,

    /// Tables to query
    pub(crate) database: Vec<Table>,

    /// These are the columns for INSERT and UPDATE queries
    pub(crate) columns: Vec<String>,

    /// Query where conditions
    pub(crate) where_clause: WhereClause,

    /// Joins
    pub(crate) joins: TableJoins,

    /// Order by conditions
    pub(crate) order_by: OrderClause,

    /// Limit the number of rows returned
    pub(crate) limit: Option<usize>,

    /// Offset the starting point of the rows returned
    pub(crate) offset: Option<usize>,

    /// Parameters
    pub(crate) values: Values,
}

impl Query {
    /// Count query builder
    pub fn count() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Count,
            ..Default::default()
        }
    }
    /// Select query builder
    pub fn select() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Select,
            ..Default::default()
        }
    }

    /// Build a create query
    pub fn create() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Create,
            ..Default::default()
        }
    }

    /// Build a "get all rows" query
    pub fn all() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Select,
            ..Default::default()
        }
    }

    /// Build an insert query
    pub fn insert() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Insert,
            ..Default::default()
        }
    }

    /// Build an update query
    pub fn update() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Update,
            ..Default::default()
        }
    }

    /// Build a delete query
    pub fn delete() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Delete,
            ..Default::default()
        }
    }

    /// Find the default table in the query
    pub(crate) fn find_table_default(&self) -> Option<Table> {
        if self.database.is_empty() {
            None
        } else {
            self.database.first().cloned()
        }
    }

    /// Convert the query to SQL
    pub fn to_sql(&self) -> Result<SqlQuery, Error> {
        self.query_type.to_sql(self)
    }
}

// impl<'a> ToSql for Query<'a> {
//     fn to_sql_stream(&self, stream: &mut SqlQuery, _query: &Query) -> Result<(), Error> {
//         let sql = self.to_sql()?;
//         stream.push_str(&sql);
//         Ok(())
//     }
// }
