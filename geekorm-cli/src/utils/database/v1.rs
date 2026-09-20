//! # v1
//!
//! This is the database spec files from geekorm v0.1 to v0.12

use anyhow::Result;
use geekorm::{Column, ColumnOptions, ColumnType, Columns, Table};

use crate::utils::database::v2::DatabaseV2;

/// This struct represents a database and is based on the `internal`
/// module of the `geekorm_derive` crate.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct DatabaseV1 {
    /// The tables in the database
    pub tables: Vec<TableV1>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TableV1 {
    pub name: String,
    pub columns: ColumnsV1,
}

impl From<TableV1> for Table {
    fn from(value: TableV1) -> Self {
        Self {
            name: value.name,
            columns: value.columns.into(),
            database: None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ColumnsV1 {
    pub columns: Vec<ColumnV1>,
}

impl From<ColumnsV1> for Columns {
    fn from(value: ColumnsV1) -> Self {
        Self::new(value.columns.iter().map(|c| c.clone().into()).collect())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ColumnV1 {
    name: String,
    column_type: ColumnTypeV1,
    alias: String,
    skip: bool,
}

impl From<ColumnV1> for Column {
    fn from(value: ColumnV1) -> Self {
        let (column_type, column_options) = match value.column_type {
            ColumnTypeV1::Text(opts) => (ColumnType::Text, opts),
            ColumnTypeV1::Integer(opts) => (ColumnType::Integer, opts),
            ColumnTypeV1::Blob(opts) => (ColumnType::Blob, opts),
            ColumnTypeV1::Identifier(opts) => (ColumnType::Text, opts),
            ColumnTypeV1::ForeignKey(opts) => (ColumnType::ForeignKey, opts),
        };

        let (column_options, foreign_key) = {
            let opts = ColumnOptions {
                primary_key: column_options.primary_key,
                unique: column_options.unique,
                not_null: column_options.not_null,
                auto_increment: column_options.auto_increment,
            };
            (opts, column_options.foreign_key)
        };

        Self {
            name: value.name,
            column_type,
            column_options,
            alias: Some(value.alias),
            foreign_key: Some(foreign_key),
            table_name: None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) enum ColumnTypeV1 {
    Identifier(ColumnOptionsV1),
    ForeignKey(ColumnOptionsV1),
    Text(ColumnOptionsV1),
    Integer(ColumnOptionsV1),
    Blob(ColumnOptionsV1),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ColumnOptionsV1 {
    primary_key: bool,
    foreign_key: String,
    unique: bool,
    not_null: bool,
    auto_increment: bool,
}

impl DatabaseV1 {
    /// Migrate helper
    pub fn migrate(self) -> Result<super::DatabaseV2> {
        let tables = self.tables.iter().map(|t| t.clone().into()).collect();
        let now = chrono::Utc::now();
        Ok(DatabaseV2 {
            name: String::from("Database"),
            tables,
            created_at: now,
            updated_at: now,
        })
    }
}
