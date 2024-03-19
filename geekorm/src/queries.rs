use std::fmt::Display;

use crate::builder::models::{QueryCondition, QueryOrder, QueryType, WhereCondition};

use crate::{
    builder::values::{Value, Values},
    Error, Table, ToSqlite,
};

#[derive(Debug, Clone, Default)]
pub struct Query {
    pub query: String,
    pub values: Values,
}

impl Query {
    pub fn new(query: String, values: Values) -> Self {
        Query { query, values }
    }

    pub fn init() -> QueryBuilder {
        QueryBuilder::default()
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query)
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    pub(crate) table: Table,
    pub(crate) query_type: QueryType,
    pub(crate) count: bool,
    /// The limit of the rows to return
    pub(crate) limit: Option<usize>,
    /// The offset of the rows to return
    pub(crate) offset: Option<usize>,

    /// The where clause
    pub(crate) where_clause: Vec<String>,
    pub(crate) where_condition_last: bool,
    /// The order by clause
    pub(crate) order_by: Vec<(String, QueryOrder)>,
    /// The values to use (where / insert)
    pub(crate) values: Values,

    pub(crate) error: Option<Error>,
}

impl QueryBuilder {
    pub fn select() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Select,
            ..Default::default()
        }
    }

    pub fn create() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Create,
            ..Default::default()
        }
    }

    pub fn insert() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Insert,
            ..Default::default()
        }
    }

    pub fn table(mut self, table: Table) -> Self {
        self.table = table.clone();
        self
    }

    pub fn add_value(mut self, value: Value) -> Self {
        self.values.push(value);
        self
    }

    pub fn and(mut self) -> Self {
        self.where_clause.push(WhereCondition::And.to_sqlite());
        self.where_condition_last = true;
        self
    }

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

    pub fn order_by(mut self, column: &str, order: QueryOrder) -> Self {
        // TODO(geekmasher): Check if column exists in table
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
