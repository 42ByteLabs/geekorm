//! # Query Builder

use crate::backends::QueryBackend;
use crate::queries::Queries;
use crate::query::conditions::{QueryCondition, WhereClause, WhereCondition};
use crate::query::joins::{TableJoin, TableJoinOptions, TableJoins};
use crate::query::ordering::{OrderClause, QueryOrder};
use crate::query::qtype::QueryType;
use crate::{Error, Table, ToSql, Value, Values, sql::*};

/// Query struct
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
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

impl QueryBuilder {
    /// Sets the Backend for the query
    pub fn backend(&mut self, backend: QueryBackend) -> &mut Self {
        self.backend = backend;
        self
    }

    /// Set the database to query
    pub fn database(&mut self, database: Vec<Table>) -> &mut Self {
        self.database = database;
        self
    }

    /// Set the table to query
    pub fn table(&mut self, table: impl Into<Table>) -> &mut Self {
        let table = table.into();
        self.database.push(table);
        self
    }

    /// Add columns to the query
    pub fn columns(&mut self, columns: Vec<String>) -> &mut Self {
        self.columns = columns;
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
        let table: Table = if let Some((ftable, fcolumn)) = column.split_once('.') {
            match self.joins.get(ftable) {
                Some(TableJoin::InnerJoin(TableJoinOptions { child, .. })) => {
                    column_name = fcolumn;
                    child.clone()
                }
                _ => {
                    self.set_error(Error::QueryBuilderError {
                        error: format!("Table `{}` does not exist", ftable),
                        location: String::from("where_eq"),
                    });

                    self.database
                        .iter()
                        .find(|t| t.name == ftable)
                        .clone()
                        .unwrap()
                        .clone()
                }
            }
        } else if let Some(table) = self.find_column(column) {
            table
        } else if let Some(table) = self.find_table("self") {
            table
        } else {
            self.set_error(Error::QueryBuilderError {
                error: String::from("No table specified"),
                location: String::from("where_eq"),
            });
            return;
        };

        if table.find_column(column_name).is_none() {
            self.set_error(Error::QueryBuilderError {
                error: format!(
                    "Column `{column_name}` does not exist in table `{}`",
                    table.name
                ),
                location: String::from("where_eq"),
            });
        }

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
        if let Some(table) = self.find_table_default() {
            let pk = table.get_primary_key().unwrap();
            self.where_eq(&pk.name(), value.into());
        } else if let Some(table) = self.find_table("self") {
            let pk = table.get_primary_key().unwrap();
            self.where_eq(&pk.name(), value.into());
        } else {
            self.set_error(Error::QueryBuilderError {
                error: String::from("No table specified"),
                location: String::from("where_primary_key"),
            });
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
        match self.validate_table_column(column) {
            Ok(true) => self.order_by.push(column.to_string(), order),
            Ok(false) => {
                self.set_error(Error::QueryBuilderError {
                    error: format!(
                        "Column `{column}` does not exist in table `{}`",
                        self.find_table("self").unwrap().name
                    ),
                    location: String::from("order_by"),
                });
            }
            Err(e) => {
                self.set_error(e);
            }
        }
        // TODO(geekmasher): What if there is no table?
        self
    }

    /// Find a table in the database
    fn find_table(&self, table: &str) -> Option<Table> {
        self.database.iter().find_map(|t| {
            if t.name == table {
                Some(t.clone())
            } else {
                None
            }
        })
    }

    /// Find the columns for the default table
    fn find_column(&self, column: &str) -> Option<Table> {
        if let Some(table) = self.find_table_default() {
            if table.columns.contains(&column.to_string()) {
                return Some(table);
            }
        }
        None
    }

    pub(crate) fn find_table_default(&self) -> Option<Table> {
        if self.database.is_empty() {
            None
        } else {
            self.database.first().cloned()
        }
    }

    fn validate_table_column(&self, column: &str) -> Result<bool, Error> {
        if let Some(table) = self.find_column(column) {
            Ok(table.columns.contains(&column))
        } else if let Some(table) = self.find_table("self") {
            Ok(table.columns.contains(&column))
        } else {
            return Err(Error::QueryBuilderError {
                error: String::from("No table specified"),
                location: String::from("validate_table_column"),
            });
        }
    }

    /// Add a limit to the query
    pub fn limit(&mut self, limit: usize) -> &mut Self {
        if limit != 0 {
            self.limit = Some(limit);
        } else {
            self.set_error(Error::QueryBuilderError {
                error: String::from("Limit cannot be 0"),
                location: String::from("limit"),
            });
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

    /// Build the query
    pub fn build(&self) -> Result<crate::Query, Error> {
        if !self.errors.is_empty() {
            return Err(Error::QueryBuilderError {
                error: self.errors.join(", "),
                location: String::from("build"),
            });
        }

        Ok(crate::Query {
            query_type: self.query_type.clone(),
            backend: self.backend.clone(),
            database: self.database.clone(),
            columns: self.columns.clone(),
            where_clause: self.where_clause.clone(),
            joins: self.joins.clone(),
            order_by: self.order_by.clone(),
            limit: self.limit,
            offset: self.offset,
            values: self.values.clone(),
        })
    }
}
