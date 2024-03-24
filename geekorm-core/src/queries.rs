use std::fmt::Display;

use crate::builder::{
    joins::{TableJoin, TableJoins},
    models::{QueryCondition, QueryOrder, QueryType, WhereCondition},
};

use crate::{
    builder::values::{Value, Values},
    Error, Table, ToSqlite,
};

/// The built Query struct with the query and values to use
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// The resulting SQLite Query
    pub query: String,
    /// The values to use in the query (where / insert / update)
    pub values: Values,
}

impl Query {
    /// Create a new Query
    pub fn new(query: String, values: Values) -> Self {
        Query { query, values }
    }

    /// Initialize using the QueryBuilder struct
    pub fn init() -> QueryBuilder {
        QueryBuilder::default()
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

/// The QueryBuilder struct for building queries using the builder pattern
///
/// # Example
/// ```rust
/// use geekorm::{GeekTable, QueryOrder};
/// use geekorm::prelude::*;
///
/// #[derive(Debug, Default, GeekTable)]
/// pub struct User {
///     pub username: String,
///     pub age: i32,
///     pub postcode: Option<String>,
/// }
///
/// # fn main() {
/// // Build a query to create a new table
/// let create_query = User::create().build().expect("Failed to build create query");
/// println!("Create Query :: {}", create_query);
///
/// // Build a query to select rows from the table
/// let select_query = User::select()
///     .where_eq("username", "geekmaher")
///     .order_by("age", QueryOrder::Asc)
///     .build()
///     .expect("Failed to build select query");
/// println!("Select Query :: {}", select_query);
/// // Output:
/// // SELECT * FROM User WHERE username = ? ORDER BY age ASC;
/// # assert_eq!(select_query.query, "SELECT * FROM User WHERE username = ? ORDER BY age ASC;");
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    pub(crate) table: Table,
    pub(crate) query_type: QueryType,
    /// If a query should use aliases
    pub(crate) aliases: bool,

    pub(crate) columns: Vec<String>,

    /// Count the rows instead of returning them
    pub(crate) count: bool,
    /// The limit of the rows to return
    pub(crate) limit: Option<usize>,
    /// The offset of the rows to return
    pub(crate) offset: Option<usize>,

    /// The where clause
    pub(crate) where_clause: Vec<String>,
    /// This variable is used to determine if the last where condition was set
    pub(crate) where_condition_last: bool,
    /// The order by clause
    pub(crate) order_by: Vec<(String, QueryOrder)>,

    pub(crate) joins: TableJoins,

    /// The values to use (where / insert)
    pub(crate) values: Values,

    pub(crate) error: Option<Error>,
}

impl QueryBuilder {
    /// Create a new QueryBuilder
    pub fn new() -> Self {
        QueryBuilder::default()
    }
    /// Build a select query
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

    /// Set the table for the query builder
    pub fn table(mut self, table: Table) -> Self {
        self.table = table.clone();
        self
    }

    /// Set the columns for the query builder
    pub fn columns(mut self, columns: Vec<&str>) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Add a value to the list of values for parameterized queries
    pub fn add_value(mut self, value: Value) -> Self {
        self.values.push(value);
        self
    }

    /// Add an AND condition to the where clause
    pub fn and(mut self) -> Self {
        self.where_clause.push(WhereCondition::And.to_sqlite());
        self.where_condition_last = true;
        self
    }

    /// Add an OR condition to the where clause
    pub fn or(mut self) -> Self {
        self.where_clause.push(WhereCondition::Or.to_sqlite());
        self.where_condition_last = true;
        self
    }

    /// The underlying function to add a where clause
    fn add_where(&mut self, column: &str, condition: QueryCondition, value: Value) {
        if self.table.is_valid_column(column) {
            // Check if the last condition was set
            if !self.where_clause.is_empty() && !self.where_condition_last {
                // Use the default where condition
                self.where_clause
                    .push(WhereCondition::default().to_sqlite());
            }

            self.where_clause
                .push(format!("{} {} ?", column, condition.to_sqlite()));
            self.values.push(value);
            self.where_condition_last = false;
        } else {
            self.error = Some(Error::QueryBuilderError(
                format!(
                    "Column `{}` does not exist in table `{}`",
                    column, self.table.name
                ),
                String::from("where_eq"),
            ));
        }
    }

    /// Where clause for equals
    pub fn where_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Eq, value.into());
        self
    }

    /// Where clause for not equals
    pub fn where_ne(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Ne, value.into());
        self
    }

    /// Where clause for like
    pub fn where_like(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Like, value.into());
        self
    }

    /// Where clause for greater than
    pub fn where_gt(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Gt, value.into());
        self
    }

    /// Where clause for less than
    pub fn where_lt(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Lt, value.into());
        self
    }

    /// Where clause for greater than or equal to
    pub fn where_gte(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Gte, value.into());
        self
    }

    /// Where clause for less than or equal to
    pub fn where_lte(mut self, column: &str, value: impl Into<Value>) -> Self {
        QueryBuilder::add_where(&mut self, column, QueryCondition::Lte, value.into());
        self
    }

    /// Order the query by a particular column
    pub fn order_by(mut self, column: &str, order: QueryOrder) -> Self {
        if self.table.is_valid_column(column) {
            self.order_by.push((column.to_string(), order));
        } else {
            self.error = Some(Error::QueryBuilderError(
                format!(
                    "Column `{}` does not exist in table `{}`",
                    column, self.table.name
                ),
                String::from("order_by"),
            ));
        }
        self
    }

    /// Adds a table to join with the current table
    ///
    /// Note: GeekOrm only joins tables with the `INNER JOIN` clause and primary keys
    pub fn join(mut self, table: Table) -> Self {
        if let Some(key) = self.table.get_primary_key() {
            if table.is_valid_column(key.as_str()) || self.table.is_valid_column(key.as_str()) {
                // TODO(geekmasher): The tables should be references to avoid cloning
                self.joins
                    .push(TableJoin::new(self.table.clone(), table.clone()));
            } else {
                self.error = Some(Error::QueryBuilderError(
                    format!("Column `{}` does not exist in table `{}`", key, table.name),
                    String::from("join"),
                ));
            }
        }
        self
    }

    /// Count the number of rows in the query
    pub fn count(mut self) -> Self {
        self.count = true;
        self
    }

    /// Add a limit to the query
    pub fn limit(mut self, limit: usize) -> Self {
        if limit != 0 {
            self.limit = Some(limit);
        } else {
            self.error = Some(Error::QueryBuilderError(
                String::from("Limit cannot be 0"),
                String::from("limit"),
            ));
        }
        self
    }

    /// Add an offset to the query
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build a Query from the QueryBuilder and perform some checks
    pub fn build(&self) -> Result<Query, crate::Error> {
        if let Some(ref error) = self.error {
            return Err(error.clone());
        }
        match self.query_type {
            QueryType::Create => Ok(Query::new(self.table.on_create(), Values::new())),
            QueryType::Select => {
                let query = self.table.on_select(self)?;
                Ok(Query::new(query.clone(), self.values.clone()))
            }
            _ => todo!("Implement other query types"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        builder::values::Value, Column, ColumnType, ColumnTypeOptions, QueryBuilder, Table,
    };

    fn simple_table() -> Table {
        Table {
            name: "users".to_string(),
            columns: crate::Columns::from(vec![
                Column::new(
                    "id".to_string(),
                    ColumnType::Integer(ColumnTypeOptions::primary_key()),
                ),
                Column::new(
                    "username".to_string(),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
                Column::new(
                    "email".to_string(),
                    ColumnType::Text(ColumnTypeOptions::null()),
                ),
            ]),
        }
    }

    #[test]
    fn test_simple_select() {
        let table = simple_table();

        let query = QueryBuilder::select()
            .table(table)
            .build()
            .expect("Failed to build query");

        assert_eq!(query.query, "SELECT * FROM users;");
    }

    #[test]
    fn test_where() {
        let table = simple_table();
        let query = QueryBuilder::select()
            .table(table)
            .where_eq("username", "geekmasher")
            .or()
            .where_like("email", "%geekmasher%")
            .build()
            .expect("Failed to build query");

        assert_eq!(
            query.query,
            "SELECT * FROM users WHERE username = ? OR email LIKE ?;"
        );
        let first = query.values.get(0).unwrap();
        assert_eq!(first, &Value::Text(String::from("geekmasher")));
        let second = query.values.get(1).unwrap();
        assert_eq!(second, &Value::Text(String::from("%geekmasher%")));
    }
}
