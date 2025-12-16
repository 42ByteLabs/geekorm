//! # Columns
//!
//! This module is for handling SQL column definitions and their conversions to SQL strings.

use super::QueryType;
use super::columntypes::ColumnType;
use crate::{Error, SqlQuery, ToSql};

/// Columns is a collection of `Column` definitions for a table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Columns {
    pub(crate) columns: Vec<Column>,
}

/// Options for a column, such as primary key, unique, not null, and auto increment.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ColumnOptions {
    /// If the column is a primary key
    pub primary_key: bool,
    /// If the column is unique
    pub unique: bool,
    /// If the column is not null
    pub not_null: bool,
    /// If the column is auto incrementing
    pub auto_increment: bool,
}

/// Column structure representing a single column in a table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) column_type: ColumnType,
    pub(crate) column_options: ColumnOptions,
    pub(crate) alias: Option<String>,
    pub(crate) foreign_key: Option<String>,
}

impl Column {
    /// Get the column name, using the alias if it exists.
    pub fn name(&self) -> String {
        if let Some(alias) = &self.alias {
            alias.clone()
        } else {
            self.name.clone()
        }
    }

    /// Create a new primary key column.
    pub fn primary_key(name: impl Into<String>) -> Self {
        Column {
            name: name.into(),
            column_type: ColumnType::Integer,
            column_options: ColumnOptions::primary_key(),
            alias: None,
            foreign_key: None,
        }
    }

    /// Create a new column as a foreign key.
    pub fn new_foreign_key(name: impl Into<String>, foreign_key: impl Into<String>) -> Self {
        Column {
            name: name.into(),
            column_type: ColumnType::ForeignKey,
            column_options: ColumnOptions::default(),
            alias: None,
            foreign_key: Some(foreign_key.into()),
        }
    }

    /// Get the foreign key as a tuple of (table, column).
    pub fn get_foreign_key(&self) -> Option<(&str, &str)> {
        if let Some(fk) = &self.foreign_key {
            return fk.split_once('.');
        }
        None
    }
}

impl Columns {
    /// Create a new collection of columns.
    pub fn new(columns: Vec<Column>) -> Self {
        Columns { columns }
    }

    /// Add a new column to the collection.
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Check if the collection contains a column by name or alias.
    pub fn contains(&self, name: &str) -> bool {
        self.columns
            .iter()
            .any(|col| col.name == name || col.alias.as_deref() == Some(name))
    }

    /// Get all of the columns in the table that are a foreign key.
    pub fn get_foreign_keys(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|col| col.column_type == ColumnType::ForeignKey)
            .collect()
    }
}

impl ColumnOptions {
    /// Primary key options
    pub fn primary_key() -> Self {
        ColumnOptions {
            primary_key: true,
            unique: true,
            not_null: true,
            auto_increment: true,
        }
    }

    /// Unique option
    pub fn unique() -> Self {
        ColumnOptions {
            unique: true,
            ..Default::default()
        }
    }
}

impl ToSql for Column {
    fn sql(&self) -> String {
        let mut stream = String::new();
        let name = self.name();

        if let Some(alias) = &self.alias {
            stream.push_str(&self.name);
            if !alias.is_empty() {
                stream.push_str(" AS ");
                stream.push_str(alias);
            }
        } else {
            stream.push_str(&name);
        }
        stream
    }

    fn to_sql_stream(&self, stream: &mut SqlQuery, query: &super::Query) -> Result<(), Error> {
        match query.query_type {
            QueryType::Create => {
                stream.push_str(&self.name);
                stream.push(' ');
                stream.push_str(
                    self.column_type
                        .to_sql_with_options(&self.column_options, query)?,
                );
            }
            _ => {
                let name = self.name();

                if let Some(alias) = &self.alias {
                    if !alias.is_empty() {
                        stream.push_str(&self.name);
                        stream.push_str(" AS ");
                        stream.push_str(alias);
                    }
                } else {
                    stream.push_str(&name);
                }

                if query.joins.is_empty() {
                    stream.push_str(&name);
                } else {
                    let table = query.find_table_default().unwrap();
                    let fullname = table.get_fullname(name.as_str());
                    stream.push_str(&fullname);
                }
            }
        }
        Ok(())
    }
}

impl ToSql for Columns {
    fn to_sql_stream(&self, stream: &mut SqlQuery, query: &super::Query) -> Result<(), Error> {
        let last_column = self.columns.last();
        for col in &self.columns {
            // sql.push(col.sql());
            col.to_sql_stream(stream, query)?;
            if Some(col) != last_column {
                stream.push_str(", ");
            }
        }

        Ok(())
    }
}

impl From<String> for Column {
    fn from(name: String) -> Self {
        Column {
            name,
            ..Default::default()
        }
    }
}

impl From<(String, String)> for Column {
    fn from((name, alias): (String, String)) -> Self {
        Column {
            name,
            alias: Some(alias),
            ..Default::default()
        }
    }
}

impl From<(String, ColumnType)> for Column {
    fn from((name, ctype): (String, ColumnType)) -> Self {
        Column {
            name,
            column_type: ctype,
            ..Default::default()
        }
    }
}

impl From<(String, ColumnType, ColumnOptions)> for Column {
    fn from((name, ctype, options): (String, ColumnType, ColumnOptions)) -> Self {
        Column {
            name,
            column_type: ctype,
            column_options: options,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Query, Table};

    fn table() -> Table {
        Table {
            name: "Test",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from(("email".to_string(), ColumnType::Text)),
                Column::from(("image_id".to_string(), ColumnType::ForeignKey)),
            ])
            .into(),
        }
    }

    #[test]
    fn test_column() {
        let column = Column::from(("id".to_string(), ColumnType::Integer));

        assert_eq!(column.name, "id");
        assert_eq!(column.column_type, ColumnType::Integer);
        assert_eq!(column.alias, None);
        assert_eq!(column.sql(), "id");
    }

    #[test]
    fn test_column_with_alias() {
        let column_with_alias = Column::from(("name".to_string(), "username".to_string()));
        assert_eq!(column_with_alias.name, "name");
        assert_eq!(column_with_alias.alias.clone().unwrap(), "username");
        assert_eq!(column_with_alias.sql(), "name AS username");
    }

    #[test]
    fn test_column_foreign_key() {
        let column = Column::new_foreign_key("image_id", "Images.id");
        let (foreign_key_table, foreign_key_col) = column.get_foreign_key().unwrap();

        assert_eq!(foreign_key_table, "Images");
        assert_eq!(foreign_key_col, "id");
    }

    #[test]
    fn test_columns_create() {
        let table = table();
        let query = Query::create().table(table.clone()).build().unwrap();

        let mut sql = SqlQuery::new();
        Columns::to_sql_stream(&table.columns, &mut sql, &query).unwrap();

        assert_eq!(
            sql.to_string(),
            "id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT, image_id INTEGER"
        );
    }

    #[test]
    fn test_columns_select_to_sql() {
        let table = table();
        let query = Query::select().table(table.clone()).build().unwrap();

        let columns = Columns::to_sql(&table.columns, &query).unwrap();

        assert_eq!(columns.to_string(), "id, name, email, image_id");
    }
}
