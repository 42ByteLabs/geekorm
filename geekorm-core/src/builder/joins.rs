use crate::{Table, ToSqlite};

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

    /// Check if the joins are empty
    pub fn is_empty(&self) -> bool {
        self.joins.is_empty()
    }
}

impl ToSqlite for TableJoins {
    fn on_select(&self, query: &crate::QueryBuilder) -> Result<String, crate::Error> {
        let mut full_query = String::new();
        for join in self.joins.iter() {
            let sql = join.on_select(query)?;
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

impl ToSqlite for TableJoin {
    fn on_select(&self, qb: &crate::QueryBuilder) -> Result<String, crate::Error> {
        match self {
            TableJoin::InnerJoin(opts) => Ok(format!(
                "INNER JOIN {} ON {}",
                opts.child.name,
                opts.on_select(qb)?
            )),
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

impl ToSqlite for TableJoinOptions {
    /// Generate the SQL for the join statement
    fn on_select(&self, _: &crate::QueryBuilder) -> Result<String, crate::Error> {
        // Get the parent column to join on
        let pcolumn = self.parent.get_foreign_key(self.child.name.clone());
        // Get the column name or alias
        let pcolumn_name = if !pcolumn.alias.is_empty() {
            pcolumn.alias.clone()
        } else {
            pcolumn.name.clone()
        };
        // Get the column to join on or use the primary key of the table
        // TODO(geekmasher): Add support for dynamic column lookup
        let ccolumn = self.child.get_primary_key();

        Ok(format!(
            "{ctable}.{ccolumn} = {ptable}.{pcolumn}",
            ctable = self.child.name,
            ccolumn = ccolumn,
            ptable = self.parent.name,
            pcolumn = pcolumn_name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Column, Columns};

    use super::*;

    fn table_parent(name: String) -> Table {
        Table {
            name,
            columns: Columns {
                columns: vec![
                    Column {
                        name: String::from("id"),
                        column_type: crate::ColumnType::Identifier(
                            crate::ColumnTypeOptions::primary_key(),
                        ),
                        ..Default::default()
                    },
                    Column {
                        name: String::from("image_id"),
                        column_type: crate::ColumnType::ForeignKey(
                            crate::ColumnTypeOptions::foreign_key(String::from("Child.id")),
                        ),
                        ..Default::default()
                    },
                ],
            },
        }
    }

    fn table_child(name: String) -> Table {
        Table {
            name,
            columns: Columns {
                columns: vec![Column {
                    name: String::from("id"),
                    column_type: crate::ColumnType::Identifier(
                        crate::ColumnTypeOptions::primary_key(),
                    ),
                    ..Default::default()
                }],
            },
        }
    }

    #[test]
    fn test_table_join_on_select() {
        let join = TableJoin::InnerJoin(TableJoinOptions {
            parent: table_parent(String::from("Parent")),
            child: table_child(String::from("Child")),
        });

        let select_query = join
            .on_select(&crate::QueryBuilder::select())
            .expect("Failed to generate select query");
        assert_eq!(
            select_query,
            "INNER JOIN Child ON Child.id = Parent.image_id"
        )
    }

    #[test]
    fn test_join_options() {
        let join = TableJoinOptions {
            parent: table_parent(String::from("Parent")),
            child: table_child(String::from("Child")),
        };

        let select_query = join.on_select(&crate::QueryBuilder::select()).unwrap();
        assert_eq!(select_query, "Child.id = Parent.image_id");
    }
}
