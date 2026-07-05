//! # Joins

use crate::{Table, ToSql};

/// Struct for joining tables
#[derive(Debug, Clone, Default)]
pub struct TableJoins {
    pub(crate) joins: Vec<TableJoin>,
}

impl Iterator for TableJoins {
    type Item = TableJoin;

    fn next(&mut self) -> Option<Self::Item> {
        self.joins.pop()
    }
}

impl TableJoins {
    /// Add a new join to the table
    pub fn push(&mut self, join: TableJoin) {
        self.joins.push(join);
    }

    /// Get the join by name
    pub fn get(&self, name: &str) -> Option<&TableJoin> {
        self.joins.iter().find(|join| match join {
            TableJoin::Join { left, .. } => left == name,
            TableJoin::InnerJoin { left, .. } => left == name,
        })
    }

    /// Check if the joins are empty
    pub fn is_empty(&self) -> bool {
        self.joins.is_empty()
    }
}

impl ToSql for TableJoins {
    fn to_sql(&self, query: &super::QueryBuilder) -> Result<String, crate::Error> {
        let mut full_query = String::new();
        for join in self.joins.iter() {
            let sql = join.sql();
            full_query.push_str(sql.as_str());
        }
        Ok(full_query)
    }

    fn to_sql_stream(
        &self,
        stream: &mut String,
        query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        if !self.is_empty() {
            stream.push(' ');
            stream.push_str(&self.to_sql(query)?);
        }
        Ok(())
    }
}

/// Enum for joining tables
#[derive(Debug, Clone)]
pub enum TableJoin {
    /// Join
    Join {
        /// Left side (parent)
        left: String,
        /// Right side (child)
        right: String,
    },
    /// Inner Join
    InnerJoin {
        /// Left side (parent)
        left: String,
        /// Right side (child)
        right: String,
    },
}

impl TableJoin {
    /// Create a new join between 2 tables
    pub fn new(left: impl Into<String>, right: impl Into<String>) -> Self {
        TableJoin::Join {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl ToSql for TableJoin {
    fn sql(&self) -> String {
        match self {
            TableJoin::Join { left, right } => {
                let (right_table, _) = right.split_once('.').unwrap();
                format!("JOIN {} ON {} = {}", right_table, left, right)
            }
            TableJoin::InnerJoin { left, right } => {
                let (right_table, _) = right.split_once('.').unwrap();
                format!("INNER JOIN {} ON {} = {}", right_table, left, right)
            }
        }
    }
}

/// Struct for Options regarding joining tables together
///
/// Parent Table is the left, Child Table is the right
#[derive(Debug, Clone)]
pub struct TableJoinOptions {
    /// Parent Table
    pub parent: Table,
    /// Child Table
    pub child: Table,
}

impl TableJoinOptions {
    /// Check if a Table.Column is valid
    pub fn is_valid_column(&self, column: &str) -> bool {
        self.parent.is_valid_column(column) || self.child.is_valid_column(column)
    }
}

impl ToSql for TableJoinOptions {
    fn to_sql_stream(
        &self,
        stream: &mut String,
        _query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        // Child table
        stream.push_str(self.child.name);
        stream.push('.');
        // Get the column to join on or use the primary key of the table
        // TODO: Add support for dynamic column lookup
        let ccolumn = self
            .child
            .get_primary_key()
            .expect("Failed to get child primary key");
        stream.push_str(&ccolumn.name);

        stream.push_str(" = ");

        // Parent table

        // Get the parent column to join on
        let pcolumn = self
            .parent
            .get_foreign_key(self.child.name.to_string())
            .expect("Failed to get Foreign Key");
        stream.push_str(self.parent.name);
        stream.push('.');

        // Get the column name or alias
        let pcolumn_name = pcolumn.name();
        stream.push_str(&pcolumn_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QueryBuilder,
        builder::{
            columns::{Column, ColumnOptions, Columns},
            columntypes::ColumnType,
            tests::table_users,
        },
    };

    #[test]
    fn test_get_fk() {
        let users = table_users();

        let fk = users.get_foreign_key("Roles.id".to_string());
        assert!(fk.is_some());
        let fk = users.get_foreign_key("Roles".to_string());
        assert!(fk.is_some());
    }

    #[test]
    fn sqlite_table_join_on_select() {
        let join = TableJoin::Join {
            left: "Users.roles".to_string(),
            right: "Roles.id".to_string(),
        };

        let select_query = join.sql();

        assert_eq!(select_query, "JOIN Roles ON Users.roles = Roles.id")
    }
}
