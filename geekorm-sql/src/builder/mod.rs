//! # Query Builder module

pub mod columns;
pub mod columntypes;
pub mod conditions;
pub mod joins;
pub mod ordering;
pub mod queries;
pub mod table;

pub use conditions::{QueryCondition, WhereCondition};
use geekorm_core::backends::connect::{Backend, Connection};
use geekorm_core::builder::joins::{TableJoin, TableJoinOptions, TableJoins};
use geekorm_core::{Error, PrimaryKey, Table, Value, Values};
pub use ordering::QueryOrder;

use crate::backends::QueryBackend;
use crate::{Query, ToSql};

use self::conditions::WhereClause;
use self::ordering::OrderClause;

/// Query Type enum
#[derive(Debug, Clone, Default)]
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

/// Query struct
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder<'a> {
    /// Query Backend
    pub(crate) backend: QueryBackend,
    /// Query type
    pub(crate) query_type: QueryType,

    pub(crate) table: Option<&'a Table>,

    /// These are the columns for INSERT and UPDATE queries
    pub(crate) columns: Vec<String>,

    /// Query where conditions
    pub(crate) where_clause: WhereClause,
    pub(crate) where_condition_last: bool,

    pub(crate) joins: TableJoins,

    /// Order by conditions
    pub(crate) order_by: OrderClause,

    /// Limit the number of rows returned
    pub(crate) limit: Option<usize>,
    /// Offset the starting point of the rows returned
    pub(crate) offset: Option<usize>,

    pub(crate) values: Values,

    pub(crate) errors: Vec<String>,
}

impl ToSql for QueryType {
    fn to_sql(&self, query: &crate::QueryBuilder) -> Result<String, Error> {
        match self {
            QueryType::Create => Ok(self.sql_create(query)),
            QueryType::Select => Ok(self.sql_select(query)),
            QueryType::Count => Ok(self.sql_count(query)),
            QueryType::Insert => Ok(self.sql_insert(query)),
            QueryType::Update => Ok(self.sql_update(query)),
            QueryType::Delete => Ok(self.sql_delete(query)),
            QueryType::Unknown => Err(Error::NotImplemented),
        }
    }
}

impl<'a> QueryBuilder<'a> {
    /// Count query builder
    pub fn count() -> Self {
        Self {
            query_type: QueryType::Count,
            ..Default::default()
        }
    }
    /// Select query builder
    pub fn select() -> Self {
        Self {
            query_type: QueryType::Select,
            ..Default::default()
        }
    }

    /// Build a create query
    pub fn create() -> Self {
        Self {
            query_type: QueryType::Create,
            ..Default::default()
        }
    }

    /// Build a "get all rows" query
    pub fn all() -> Self {
        Self {
            query_type: QueryType::Select,
            ..Default::default()
        }
    }

    /// Build an insert query
    pub fn insert() -> Self {
        Self {
            query_type: QueryType::Insert,
            ..Default::default()
        }
    }

    /// Build an update query
    pub fn update() -> Self {
        Self {
            query_type: QueryType::Update,
            ..Default::default()
        }
    }

    /// Build a delete query
    pub fn delete() -> Self {
        Self {
            query_type: QueryType::Delete,
            ..Default::default()
        }
    }

    /// Sets the Backend for the query
    pub fn backend(&mut self, backend: QueryBackend) -> &mut Self {
        self.backend = backend;
        self
    }

    /// Sets the backend based on a connection passed in
    pub fn connection(&mut self, connection: &Connection<'_>) -> &mut Self {
        self.backend = QueryBackend::from(connection);
        self
    }

    /// The table to query
    pub fn table(&mut self, table: &'a Table) -> &mut Self {
        self.table = Some(table);
        self
    }

    /// Add columns to the query
    pub fn columns(&mut self, columns: Vec<&str>) -> &mut Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a value to the list of values for parameterized queries
    pub fn add_value(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.values.push(column.to_string(), value.into());
        self
    }

    /// Add an AND condition to the where clause
    pub fn and(&mut self) -> &mut Self {
        match self.where_clause.push_condition(WhereCondition::And) {
            Ok(_) => {
                self.where_condition_last = true;
            }
            Err(e) => {
                self.set_error(e);
            }
        }
        self
    }

    /// Add an OR condition to the where clause
    pub fn or(&mut self) -> &mut Self {
        match self.where_clause.push_condition(WhereCondition::Or) {
            Ok(_) => {
                self.where_condition_last = true;
            }
            Err(e) => {
                self.set_error(e);
            }
        }
        self
    }

    /// The underlying function to add a where clause
    fn add_where(&mut self, column: &str, condition: QueryCondition, value: Value) {
        let mut column_name: &str = column;

        // Check if there is a `.` in the column name
        let table = if let Some((ftable, fcolumn)) = column.split_once('.') {
            match self.joins.get(ftable) {
                Some(TableJoin::InnerJoin(TableJoinOptions { child, .. })) => {
                    column_name = fcolumn;
                    child
                }
                _ => {
                    self.set_error(Error::QueryBuilderError(
                        format!("Table `{}` does not exist", ftable),
                        String::from("where_eq"),
                    ));

                    self.table.unwrap()
                }
            }
        } else if let Some(table) = self.table {
            table
        } else {
            self.set_error(Error::QueryBuilderError(
                String::from("No table specified"),
                String::from("where_eq"),
            ));
            return;
        };

        if table.is_valid_column(column_name) {
            // Check if the last condition was set
            if !self.where_clause.is_empty() && !self.where_condition_last {
                // Use the default where condition
                if let Err(err) = self.where_clause.push_condition(WhereCondition::default()) {
                    self.set_error(err);
                }
            }

            // self.where_clause
            //     .push(format!("{} {} ?", column, condition.sql()));
            self.where_clause.push(column.to_string(), condition);
            self.values.push(column.to_string(), value);
            self.where_condition_last = false;
        } else {
            self.set_error(Error::QueryBuilderError(
                format!(
                    "Column `{}` does not exist in table `{}`",
                    column_name, table.name
                ),
                String::from("where_eq"),
            ));
        }
    }

    /// Where clause for equals
    pub fn where_eq(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Eq, value.into());
        self
    }

    /// Where clause for not equals
    pub fn where_ne(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Ne, value.into());
        self
    }

    /// Where clause for like
    pub fn where_like(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Like, value.into());
        self
    }

    /// Where clause for greater than
    pub fn where_gt(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Gt, value.into());
        self
    }

    /// Where clause for less than
    pub fn where_lt(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Lt, value.into());
        self
    }

    /// Where clause for greater than or equal to
    pub fn where_gte(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Gte, value.into());
        self
    }

    /// Where clause for less than or equal to
    pub fn where_lte(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        QueryBuilder::add_where(self, column, QueryCondition::Lte, value.into());
        self
    }

    /// Where Primary Key
    pub fn where_primary_key(&mut self, value: impl Into<Value>) -> &mut Self {
        if let Some(table) = self.table {
            self.where_eq(table.get_primary_key().as_str(), value.into());
        } else {
            self.set_error(Error::QueryBuilderError(
                String::from("No table specified"),
                String::from("where_primary_key"),
            ));
        }
        self
    }

    /// Filter the query by multiple fields
    pub fn filter(&mut self, fields: Vec<(&str, impl Into<Value>)>) -> &mut Self {
        for (field, value) in fields {
            if field.starts_with("=") {
                let field = &field[1..];
                self.where_eq(field, value.into());
            } else if field.starts_with("~") {
                let field = &field[1..];
                self.where_like(field, value.into());
            } else if field.starts_with("!") {
                let field = &field[1..];
                self.where_ne(field, value.into());
            } else {
                // Default to WHERE field = value with an OR operator
                self.where_eq(field, value.into());
                self.or();
            }
        }
        self
    }

    /// Order the query by a particular column
    pub fn order_by(&mut self, column: &str, order: QueryOrder) -> &mut Self {
        if let Some(table) = self.table {
            if table.is_valid_column(column) {
                self.order_by.push(column.to_string(), order);
            } else {
                self.set_error(Error::QueryBuilderError(
                    format!(
                        "Column `{}` does not exist in table `{}`",
                        column, table.name
                    ),
                    String::from("order_by"),
                ));
            }
        }
        // TODO(geekmasher): What if there is no table?
        self
    }

    /// Add a limit to the query
    pub fn limit(&mut self, limit: usize) -> &mut Self {
        if limit != 0 {
            self.limit = Some(limit);
        } else {
            self.set_error(Error::QueryBuilderError(
                String::from("Limit cannot be 0"),
                String::from("limit"),
            ));
        }
        self
    }

    /// Add an offset to the query
    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    /// Set internal error
    fn set_error(&mut self, error: Error) {
        self.errors.push(error.to_string());
    }

    /// Build a Query from the QueryBuilder
    pub fn build(&self) -> Result<Query, geekorm_core::Error> {
        let query = Query {
            query: self.query_type.to_sql(self)?,
            values: self.values.clone(),
            params: self.values.clone(),
        };

        Ok(query)
    }
}
