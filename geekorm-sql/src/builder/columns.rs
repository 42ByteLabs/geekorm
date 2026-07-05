//! # Columns
//!
//! This module is for handling SQL column definitions and their conversions to SQL strings.

use super::QueryType;
use super::columntypes::ColumnType;
use crate::{Error, ToSql};

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
    pub(crate) table_name: Option<String>,
}

impl Column {
    /// Get the column name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Full column name
    pub fn fullname(&self) -> String {
        if let Some(table_name) = &self.table_name {
            format!("{}.{}", table_name, self.name)
        } else {
            self.name.clone()
        }
    }

    /// Get the column name
    pub fn name_alias(&self) -> String {
        if let Some(alias) = &self.alias {
            alias.clone()
        } else {
            self.name.clone()
        }
    }

    /// Create a new column as a foreign key.
    pub fn new_foreign_key(name: impl Into<String>, foreign_key: impl Into<String>) -> Self {
        Column {
            name: name.into(),
            column_type: ColumnType::ForeignKey,
            foreign_key: Some(foreign_key.into()),
            ..Default::default()
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
            .filter(|col| col.column_type == ColumnType::ForeignKey && col.foreign_key.is_some())
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

    /// Unique (not null)
    pub fn unique() -> Self {
        ColumnOptions {
            primary_key: false,
            unique: true,
            not_null: true,
            auto_increment: false,
        }
    }
}

impl ToSql for Column {
    fn sql(&self) -> String {
        // Simple name
        let mut stream = String::new();
        stream.push_str(&self.name());

        // Append the alias
        if let Some(alias) = &self.alias {
            if !alias.is_empty() {
                stream.push_str(" AS ");
                stream.push_str(alias);
            }
        }
        stream
    }

    fn to_sql_stream(&self, stream: &mut String, query: &super::QueryBuilder) -> Result<(), Error> {
        match query.query_type {
            QueryType::Create => {
                stream.push_str(&self.name);
                stream.push(' ');
                let col_type = self
                    .column_type
                    .to_sql_with_options(&self.column_options, query)?;
                stream.push_str(&col_type);
            }
            _ => {
                if query.joins.is_empty() {
                    stream.push_str(&self.sql());
                } else {
                    stream.push_str(&self.fullname());
                }
            }
        }
        Ok(())
    }
}

impl ToSql for Columns {
    fn sql(&self) -> String {
        // Simple column names (no tables)
        let mut sql = Vec::new();
        for col in &self.columns {
            sql.push(col.name.clone());
        }
        sql.join(", ")
    }

    fn to_sql_stream(&self, stream: &mut String, query: &super::QueryBuilder) -> Result<(), Error> {
        let mut sql = Vec::new();

        for col in &self.columns {
            sql.push(col.to_sql(query)?);
        }

        // TODO(geekmasher): Why is a clone required here?
        for join in query.joins.joins.iter() {
            match join {
                super::TableJoin::Join { right, .. }
                | super::TableJoin::InnerJoin { right, .. } => {
                    let (right_table_name, _) = right.split_once('.').unwrap();

                    if let Some(right_table) = query.find_table(right_table_name) {
                        for col in &right_table.columns.columns {
                            sql.push(col.to_sql(query)?);
                        }
                    }
                }
            }
        }

        stream.push_str(&sql.join(", "));
        Ok(())
    }
}

impl From<Vec<Column>> for Columns {
    fn from(value: Vec<Column>) -> Self {
        Columns { columns: value }
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

impl From<(&str, ColumnType)> for Column {
    fn from((name, ctype): (&str, ColumnType)) -> Self {
        Column {
            name: name.to_string(),
            column_type: ctype,
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

// ForeignKey
impl From<(String, ColumnType, String)> for Column {
    fn from((name, ctype, foreign_key): (String, ColumnType, String)) -> Self {
        Column {
            name,
            column_type: ctype,
            foreign_key: Some(foreign_key),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{
        QueryBuilder,
        tests::{table_images, table_roles, table_users},
    };
    use crate::{Query, Table};

    #[test]
    fn test_column_name() {
        let mut column = Column::from(("id".to_string(), ColumnType::Text));
        assert_eq!(&column.name(), "id");

        column.table_name = Some("Test".to_string());
        assert_eq!(&column.name(), "id");
        assert_eq!(&column.fullname(), "Test.id");
    }

    #[test]
    fn sqlite_single_column_to_sql() {
        let table = table_users();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let column = Column::from(("id".to_string(), ColumnType::Integer));

        let column_sql = column.to_sql(&query).unwrap();

        assert_eq!(column_sql.as_str(), "id");
    }

    #[test]
    fn sqlite_column_alias() {
        let table = table_users();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let column = Column {
            name: "id".to_string(),
            column_type: ColumnType::Integer,
            alias: Some("pk".to_string()),
            column_options: ColumnOptions::primary_key(),
            ..Default::default()
        };

        let column_sql = column.to_sql(&query).unwrap();

        assert_eq!(column_sql.as_str(), "id AS pk");
    }

    #[test]
    fn test_columns_to_sql() {
        let table = table_images();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let columns = Columns::to_sql(&table.columns, &query).unwrap();

        assert_eq!(columns.as_str(), "id, title, url");
    }

    #[test]
    fn sqlite_column_joins() {
        let table = table_users();
        let roles_table = table_roles();
        let image_table = table_images();

        let mut query = QueryBuilder::select();
        query.table(&table).join(&image_table).join(&roles_table);

        println!("ERRORS :: {:?}", query.errors);
        assert_eq!(query.joins.joins.len(), 2);

        // No concept of joins
        let columns = image_table.columns.sql();
        assert_eq!(columns.as_str(), "id, title, url");

        let mut full_sql = String::new();
        table.columns.to_sql_stream(&mut full_sql, &query).unwrap();

        assert_eq!(
            full_sql.as_str(),
            "Users.id, Users.username, Users.email, Users.roles, Users.profile, Images.id, Images.title, Images.url, Roles.id"
        );
    }
}
