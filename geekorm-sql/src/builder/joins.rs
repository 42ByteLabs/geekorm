//! # Joins

use crate::{Table, ToSql};

/// Struct for joining tables
#[derive(Debug, Clone, Default)]
pub struct TableJoins {
    joins: Vec<TableJoin>,
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
            TableJoin::InnerJoin(opts) => opts.child.name == name,
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
            let sql = join.to_sql(query)?;
            full_query.push_str(sql.as_str());
        }
        Ok(full_query)
    }
}

/// Enum for joining tables
#[derive(Debug, Clone)]
pub enum TableJoin {
    /// Inner Join
    InnerJoin(TableJoinOptions),
}

impl TableJoin {
    /// Create a new inner join between two tables
    pub fn new(parent: Table, child: Table) -> Self {
        TableJoin::InnerJoin(TableJoinOptions { parent, child })
    }

    /// Check if a Table.Column is valid
    pub fn is_valid_column(&self, column: &str) -> bool {
        match self {
            TableJoin::InnerJoin(opts) => opts.parent.is_valid_column(column),
        }
    }
}

impl ToSql for TableJoin {
    fn to_sql(&self, query: &super::QueryBuilder) -> Result<String, crate::Error> {
        match self {
            TableJoin::InnerJoin(opts) => {
                let mut inner_join = String::new();
                inner_join.push_str("INNER JOIN ");
                inner_join.push_str(&opts.child.name);
                inner_join.push_str(" ON ");
                inner_join.push_str(&opts.to_sql(query)?);
                Ok(inner_join)
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
        let ccolumn = self.child.get_primary_key().unwrap();
        stream.push_str(&ccolumn.name);

        stream.push_str(" = ");

        // Parent table

        // Get the parent column to join on
        let pcolumn = self
            .parent
            .get_foreign_key(self.child.name.to_string())
            .unwrap();
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

    use crate::{
        QueryBuilder,
        builder::{
            columns::{Column, ColumnOptions, Columns},
            columntypes::ColumnType,
        },
    };

    use super::*;

    fn table_parent() -> Table {
        Table {
            name: "Test",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::new_foreign_key("image_id", "Child.id"),
            ])
            .into(),
        }
    }

    fn table_child() -> Table {
        Table {
            name: "Child",
            columns: Columns::new(vec![Column::from((
                "id".to_string(),
                ColumnType::Integer,
                ColumnOptions::primary_key(),
            ))]),
        }
    }

    // #[test]
    // fn test_table_join_on_select() {
    //     let parent = table_parent();
    //     let child = table_child();
    //
    //     let join = TableJoin::InnerJoin(TableJoinOptions {
    //         parent: parent.clone(),
    //         child: child.clone(),
    //     });
    //
    //     // TODO: Add test
    // }
    //
    // #[test]
    // fn test_join_options() {
    //     let join = TableJoinOptions {
    //         parent: table_parent(),
    //         child: table_child(),
    //     };
    //
    //     let select_query = join.on_select(&crate::QueryBuilder::select()).unwrap();
    //     assert_eq!(select_query, "Child.id = Parent.image_id");
    // }
}
