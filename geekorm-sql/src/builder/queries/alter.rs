//! # Alter Queries

use std::default;

use crate::{
    Column, ColumnOptions, ColumnType, Query, QueryBuilder, QueryType, Table, ToSql, query,
};

/// Alter mode
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum AlterMode {
    /// Add a table
    AddTable,
    /// Rename a table
    RenameTable,
    /// Drop a table
    DropTable,

    /// Add a column
    AddColumn,
    /// Rename a column
    RenameColumn,
    /// Drop a column
    DropColumn,

    /// Skip
    Skip,
    /// Unknown
    #[default]
    Unknown,
}

/// Alter query builder
#[derive(Debug, Clone, Default)]
pub struct AlterQuery {
    pub(crate) mode: AlterMode,

    /// Table name
    pub(crate) table: Table,
    /// Column name
    pub(crate) column: Column,
    /// Rename column (if applicable)
    pub(crate) rename: Option<String>,
}

impl AlterQuery {
    /// New Alter query
    pub fn new() -> Self {
        Self::default()
    }

    /// Set mode
    pub fn mode(&mut self, mode: AlterMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Set table
    pub fn table(&mut self, table: &Table) -> &mut Self {
        self.table = table.clone();
        self
    }

    /// Set column
    pub fn column(&mut self, column: &Column) -> &mut Self {
        self.column = column.clone();
        self
    }

    /// Rename the table
    pub fn rename(&mut self, name: impl Into<String>) -> &mut Self {
        self.rename = Some(name.into());
        self
    }

    /// Build the Alter query to return a Query
    pub fn build(&self) -> Result<Query, crate::Error> {
        Ok(Query {
            query: self.sql(),
            query_type: QueryType::Alter,
            values: crate::Values::new(),
            params: crate::Values::new(),
        })
    }

    // -- Helpers

    /// Add new Table
    pub fn add_table(table: &Table, column: &Column) -> Result<Query, crate::Error> {
        Self {
            mode: AlterMode::AddTable,
            table: table.clone(),
            column: column.clone(),
            rename: None,
        }
        .build()
    }
    /// Rename existing table
    pub fn rename_table(table: &Table, name: impl Into<String>) -> Result<Query, crate::Error> {
        Self {
            mode: AlterMode::RenameTable,
            table: table.clone(),
            rename: Some(name.into()),
            ..Default::default()
        }
        .build()
    }
    /// Drop table (if exists)
    pub fn drop_table(table: &Table) -> Result<Query, crate::Error> {
        Self {
            mode: AlterMode::DropTable,
            table: table.clone(),
            ..Default::default()
        }
        .build()
    }
    /// Rename Column
    pub fn rename_column(
        table: &Table,
        column: &Column,
        name: impl Into<String>,
    ) -> Result<Query, crate::Error> {
        Self {
            mode: AlterMode::RenameColumn,
            table: table.clone(),
            column: column.clone(),
            rename: Some(name.into()),
        }
        .build()
    }
    /// Drop column
    pub fn drop_column(table: &Table, column: &Column) -> Result<Query, crate::Error> {
        Self {
            mode: AlterMode::DropColumn,
            table: table.clone(),
            column: column.clone(),
            ..Default::default()
        }
        .build()
    }

    pub(crate) fn to_sql_alter(
        &self,
        column_type: &ColumnType,
        options: &ColumnOptions,
    ) -> Result<String, crate::Error> {
        match column_type {
            ColumnType::Text => {
                if options.not_null {
                    Ok("TEXT NOT NULL DEFAULT ''".to_string())
                } else {
                    Ok("TEXT".to_string())
                }
            }
            ColumnType::Integer | ColumnType::Boolean => {
                if options.not_null {
                    Ok("INTEGER NOT NULL DEFAULT 0".to_string())
                } else {
                    Ok("INTEGER".to_string())
                }
            }
            ColumnType::Blob => {
                if options.not_null {
                    Ok("BLOB NOT NULL DEFAULT ''".to_string())
                } else {
                    Ok("BLOB".to_string())
                }
            }
            _ => Ok("BEANS".to_string()),
        }
    }
}

impl QueryType {
    pub(crate) fn sql_alter(&self, query: &QueryBuilder) -> String {
        if let Some(alter) = &query.alter {
            alter.to_sql(query).unwrap()
        } else {
            String::new()
        }
    }
}

impl ToSql for AlterQuery {
    fn sql(&self) -> String {
        let mut stream = format!("ALTER TABLE {} ", self.table.name);

        let sql = match self.mode {
            AlterMode::AddTable => {
                format!("ADD COLUMN {}", self.column.name)
            }
            AlterMode::RenameTable => {
                // TODO(geekmasher): Is this the right choice?
                let name = self.rename.clone().unwrap_or_default();
                format!("RENAME TO {}", name)
            }
            AlterMode::DropTable => {
                // https://sqlite.org/lang_droptable.html
                return format!("DROP TABLE IF EXISTS {};", self.table.name);
            }
            AlterMode::AddColumn => {
                format!(
                    "ADD COLUMN {} {}",
                    self.column.name,
                    self.to_sql_alter(&self.column.column_type, &self.column.column_options)
                        .unwrap()
                )
            }
            AlterMode::RenameColumn => {
                format!(
                    "RENAME COLUMN {} TO {}",
                    self.column.name,
                    self.rename.as_ref().unwrap_or(&self.column.name)
                )
            }
            AlterMode::DropColumn => {
                format!("DROP COLUMN {}", self.column.name)
            }
            AlterMode::Skip | AlterMode::Unknown => String::new(),
        };

        stream.push_str(&sql);
        stream.push(';');

        stream
    }
}

#[cfg(test)]
mod tests {
    use crate::{Column, ColumnOptions, ColumnType, Table};

    use super::*;

    fn table() -> Table {
        Table {
            name: "Test",
            columns: crate::Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from((
                    "email".to_string(),
                    ColumnType::Text,
                    ColumnOptions::unique(),
                )),
                Column::from((
                    "image_id".to_string(),
                    ColumnType::ForeignKey,
                    "Images.id".to_string(),
                )),
            ])
            .into(),
        }
    }

    #[test]
    fn sqlite_alter() {
        let table = table();
        let column = table.find_column("email").unwrap().clone();

        let query = QueryBuilder::alter()
            .mode(AlterMode::AddTable)
            .table(&table)
            .column(&column)
            .build()
            .unwrap();

        assert_eq!(query.query, "ALTER TABLE Test ADD COLUMN email;")
    }

    #[test]
    fn sqlite_alter_add_table() {
        let table = table();
        let column = table.find_column("email").unwrap().clone();

        let query = AlterQuery::add_table(&table, &column).unwrap();

        assert_eq!(query.query, "ALTER TABLE Test ADD COLUMN email;");
    }

    #[test]
    fn sqlite_alter_rename_table() {
        let table = table();
        let query = AlterQuery::rename_table(&table, "Test2").unwrap();

        assert_eq!(query.query, "ALTER TABLE Test RENAME TO Test2;");
    }

    #[test]
    fn sqlite_alter_drop_table() {
        let table = table();
        let query = AlterQuery::drop_table(&table).unwrap();

        assert_eq!(query.query, "DROP TABLE IF EXISTS Test;");
    }

    #[test]
    fn sqlite_alter_rename_column() {
        let table = table();
        let column = table.find_column("email").unwrap().clone();

        let query = AlterQuery::rename_column(&table, &column, "bmail").unwrap();

        assert_eq!(
            query.query,
            "ALTER TABLE Test RENAME COLUMN email TO bmail;"
        );
    }

    #[test]
    fn sqlite_alter_drop_column() {
        let table = table();
        let column = table.find_column("email").unwrap().clone();

        let query = AlterQuery::drop_column(&table, &column).unwrap();

        assert_eq!(query.query, "ALTER TABLE Test DROP COLUMN email;");
    }
}
