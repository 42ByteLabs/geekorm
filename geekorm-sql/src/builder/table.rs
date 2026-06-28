//! # Table Expression

use super::columns::{Column, Columns};
use crate::ToSql;

/// Table structure representing a database table.
#[derive(Debug, Clone, Default)]
pub struct Table {
    /// Name of the table
    pub name: &'static str,
    /// Columns in the table
    pub columns: Columns,
}

impl Table {
    /// Create a new table with the given name and columns.
    pub fn new(name: &'static str, columns: Columns) -> Self {
        Table { name, columns }
    }

    /// Add a column to the table.
    pub fn add_column(&mut self, column: Column) {
        self.columns.columns.push(column);
    }

    /// Is the table column valid?
    pub fn is_valid_column(&self, name: &str) -> bool {
        self.columns.contains(name)
    }

    /// Get the primary key column, if it exists.
    pub fn get_primary_key(&self) -> Option<&Column> {
        self.columns
            .columns
            .iter()
            .find(|col| col.column_options.primary_key)
    }

    /// Get a foreign key column by its name
    pub fn get_foreign_key(&self, name: String) -> Option<&Column> {
        self.columns
            .columns
            .iter()
            .find(|col| col.foreign_key.as_deref() == Some(&name))
    }

    /// Get all foreign key columns in the table.
    pub fn get_foreign_keys(&self) -> Vec<&Column> {
        self.columns
            .columns
            .iter()
            .filter(|col| col.foreign_key.is_some())
            .collect()
    }

    /// Get a column by its name or alias
    pub fn find_column(&self, name: &str) -> Option<&Column> {
        self.columns
            .columns
            .iter()
            .find(|col| col.name == name || col.alias.as_deref() == Some(name))
    }

    /// Get the full name of a column in the format "table.column"
    pub fn get_fullname(&self, column: &str) -> String {
        self.name.to_owned() + "." + column
    }
}

/// Table expression for SQL queries.
#[derive(Debug, Clone, Default)]
pub struct TableExpr {
    /// Table name
    pub name: String,
    /// Alias for the table
    pub alias: Option<String>,
}

impl ToSql for TableExpr {
    /// Generate the SQL for the table expression
    fn sql(&self) -> String {
        if let Some(ref alias) = self.alias {
            format!("{} AS {}", self.name.to_string(), alias.to_string())
        } else {
            self.name.clone()
        }
    }

    fn to_sql_stream(
        &self,
        stream: &mut String,
        _query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        if !stream.is_empty() && !stream.ends_with(' ') {
            stream.push(' ');
        }

        stream.push_str("FROM ");
        stream.push_str(&self.sql());
        Ok(())
    }
}

impl TableExpr {
    /// Create a new table expression
    pub fn new(name: &str) -> Self {
        TableExpr {
            name: name.to_string(),
            alias: None,
        }
    }

    /// Set the alias for the table expression
    pub fn alias(&mut self, alias: String) {
        self.alias = Some(alias);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryBuilder;

    #[test]
    fn test_table() {
        let table = TableExpr::new("users");
        let mut sql = String::new();
        table
            .to_sql_stream(&mut sql, &QueryBuilder::default())
            .unwrap();
        assert_eq!(sql, "FROM users");
    }

    #[test]
    fn test_table_expr_alias() {
        let mut table = TableExpr::new("users");
        table.alias("u".to_string());
        let mut sql = String::new();
        table
            .to_sql_stream(&mut sql, &QueryBuilder::default())
            .unwrap();
        assert_eq!(sql, "FROM users AS u");
    }
}
