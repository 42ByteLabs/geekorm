//! # Column types

use super::QueryType;
use super::columns::ColumnOptions;
use crate::{Error, QueryBackend, ToSql};

/// Column types
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Text column type
    #[default]
    Text,
    /// Integer / Numeric column type (32-bit)
    Integer,
    /// Big Integer / Numeric column type (64-bit)
    BigInt,
    /// Boolean column type
    Boolean,
    /// Blob / Byte Array column type
    Blob,
    /// Foreign key column type
    ForeignKey,
}

impl ColumnType {
    /// Convert the column type to SQL string with options
    pub fn to_sql_with_options(
        &self,
        options: &ColumnOptions,
        query: &super::QueryBuilder,
    ) -> Result<String, Error> {
        let mut stream = String::new();

        match query.backend {
            QueryBackend::Postgres => {
                match self {
                    ColumnType::ForeignKey => {
                        // For PostgreSQL, use INTEGER or BIGINT
                        if options.primary_key && options.auto_increment {
                            stream.push_str("SERIAL");
                        } else {
                            stream.push_str("INTEGER");
                            let opts = options.to_sql(query)?;
                            if !opts.is_empty() {
                                stream.push(' ');
                                stream.push_str(&opts);
                            }
                        }
                    }
                    ColumnType::Text => {
                        stream.push_str("TEXT");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                    ColumnType::Integer => {
                        // PostgreSQL uses SERIAL for auto-increment integers
                        if options.primary_key && options.auto_increment {
                            stream.push_str("SERIAL");
                            // SERIAL already implies NOT NULL, so we only add PRIMARY KEY
                            if options.primary_key {
                                stream.push_str(" PRIMARY KEY");
                            }
                        } else {
                            stream.push_str("INTEGER");
                            let opts = options.to_sql(query)?;
                            if !opts.is_empty() {
                                stream.push(' ');
                                stream.push_str(&opts);
                            }
                        }
                    }
                    ColumnType::BigInt => {
                        // PostgreSQL uses BIGSERIAL for auto-increment big integers
                        if options.primary_key && options.auto_increment {
                            stream.push_str("BIGSERIAL");
                            // BIGSERIAL already implies NOT NULL, so we only add PRIMARY KEY
                            if options.primary_key {
                                stream.push_str(" PRIMARY KEY");
                            }
                        } else {
                            stream.push_str("BIGINT");
                            let opts = options.to_sql(query)?;
                            if !opts.is_empty() {
                                stream.push(' ');
                                stream.push_str(&opts);
                            }
                        }
                    }
                    ColumnType::Boolean => {
                        stream.push_str("BOOLEAN");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                    ColumnType::Blob => {
                        stream.push_str("BYTEA");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                }
            }
            QueryBackend::Sqlite | QueryBackend::Unknown => {
                // SQLite implementation (existing code)
                match self {
                    ColumnType::ForeignKey => {
                        stream.push_str("INTEGER");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                    ColumnType::Text => {
                        stream.push_str("TEXT");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                    ColumnType::Integer | ColumnType::BigInt => {
                        // SQLite treats both INTEGER and BIGINT as INTEGER
                        stream.push_str("INTEGER");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                    ColumnType::Boolean => {
                        stream.push_str("INTEGER");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                    ColumnType::Blob => {
                        stream.push_str("BLOB");
                        let opts = options.to_sql(query)?;
                        if !opts.is_empty() {
                            stream.push(' ');
                            stream.push_str(&opts);
                        }
                    }
                }
            }
        }

        Ok(stream)
    }
}

impl ToSql for ColumnOptions {
    fn to_sql_stream(&self, stream: &mut String, query: &super::QueryBuilder) -> Result<(), Error> {
        let mut sql = Vec::new();

        match query.backend {
            QueryBackend::Postgres => {
                // PostgreSQL syntax
                if self.primary_key {
                    // For SERIAL types, PRIMARY KEY is added separately
                    // AUTOINCREMENT is implicit in SERIAL
                    if !self.auto_increment {
                        sql.push("PRIMARY KEY");
                    }
                } else {
                    if self.not_null {
                        sql.push("NOT NULL");
                    }
                    if self.unique {
                        sql.push("UNIQUE");
                    }
                }
            }
            QueryBackend::Sqlite | QueryBackend::Unknown => {
                // SQLite syntax (existing logic)
                if self.primary_key {
                    sql.push("PRIMARY KEY");
                    if self.auto_increment {
                        sql.push("AUTOINCREMENT");
                    }
                } else {
                    if self.not_null {
                        sql.push("NOT NULL");
                    }
                    if self.unique {
                        sql.push("UNIQUE");
                    }
                }
            }
        }

        stream.push_str(&sql.join(" "));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QueryType, Table,
        builder::{
            QueryBuilder,
            columns::{Column, Columns},
        },
    };

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

    fn query() -> crate::QueryBuilder<'static> {
        crate::QueryBuilder::default()
    }

    #[test]
    fn test_column_type_boolean() {
        let column_type = ColumnType::Boolean;
        let options = ColumnOptions::default();
        let query = query();
        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "INTEGER"
        );
    }

    #[test]
    fn test_column_type_to_sql() {
        let query = query();
        let options = ColumnOptions::default();
        let column_type = ColumnType::Text;
        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "TEXT"
        );

        let column_type = ColumnType::Integer;
        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "INTEGER"
        );
    }

    #[test]
    fn test_column_type_options_to_sql() {
        let query = query();
        let column_type_options = ColumnOptions::default();
        assert_eq!(column_type_options.to_sql(&query).unwrap(), "");

        let column_type_options = ColumnOptions {
            primary_key: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.to_sql(&query).unwrap(), "PRIMARY KEY");

        let column_type_options = ColumnOptions {
            primary_key: true,
            auto_increment: true,
            ..Default::default()
        };
        assert_eq!(
            column_type_options.to_sql(&query).unwrap(),
            "PRIMARY KEY AUTOINCREMENT"
        );
    }

    // PostgreSQL specific tests
    #[test]
    fn test_postgres_column_type_boolean() {
        let column_type = ColumnType::Boolean;
        let options = ColumnOptions::default();
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "BOOLEAN"
        );
    }

    #[test]
    fn test_postgres_column_type_blob() {
        let column_type = ColumnType::Blob;
        let options = ColumnOptions::default();
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "BYTEA"
        );
    }

    #[test]
    fn test_postgres_column_type_text() {
        let column_type = ColumnType::Text;
        let options = ColumnOptions::default();
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "TEXT"
        );
    }

    #[test]
    fn test_postgres_column_type_integer_with_autoincrement() {
        let column_type = ColumnType::Integer;
        let options = ColumnOptions {
            primary_key: true,
            auto_increment: true,
            not_null: true,
            unique: true,
        };
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "SERIAL PRIMARY KEY"
        );
    }

    #[test]
    fn test_postgres_column_type_integer_without_autoincrement() {
        let column_type = ColumnType::Integer;
        let options = ColumnOptions {
            primary_key: false,
            auto_increment: false,
            not_null: true,
            unique: false,
        };
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "INTEGER NOT NULL"
        );
    }

    #[test]
    fn test_postgres_column_options_primary_key_no_autoincrement() {
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        let column_type_options = ColumnOptions {
            primary_key: true,
            auto_increment: false,
            ..Default::default()
        };
        assert_eq!(column_type_options.to_sql(&query).unwrap(), "PRIMARY KEY");
    }

    #[test]
    fn test_postgres_column_options_with_autoincrement() {
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        let column_type_options = ColumnOptions {
            primary_key: true,
            auto_increment: true,
            ..Default::default()
        };
        // When auto_increment is true, PRIMARY KEY is not added to options
        // because SERIAL already implies it
        assert_eq!(column_type_options.to_sql(&query).unwrap(), "");
    }

    #[test]
    fn test_postgres_column_options_not_null_unique() {
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        let column_type_options = ColumnOptions {
            primary_key: false,
            auto_increment: false,
            not_null: true,
            unique: true,
        };
        assert_eq!(
            column_type_options.to_sql(&query).unwrap(),
            "NOT NULL UNIQUE"
        );
    }

    #[test]
    fn test_postgres_column_type_bigint_with_autoincrement() {
        let column_type = ColumnType::BigInt;
        let options = ColumnOptions {
            primary_key: true,
            auto_increment: true,
            not_null: true,
            unique: true,
        };
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "BIGSERIAL PRIMARY KEY"
        );
    }

    #[test]
    fn test_postgres_column_type_bigint_without_autoincrement() {
        let column_type = ColumnType::BigInt;
        let options = ColumnOptions {
            primary_key: false,
            auto_increment: false,
            not_null: true,
            unique: false,
        };
        let mut query = crate::QueryBuilder::default();
        query.backend(crate::QueryBackend::Postgres);

        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "BIGINT NOT NULL"
        );
    }

    #[test]
    fn test_sqlite_bigint_as_integer() {
        let column_type = ColumnType::BigInt;
        let options = ColumnOptions::default();
        let query = crate::QueryBuilder::default();

        // SQLite should treat BigInt as INTEGER
        assert_eq!(
            column_type.to_sql_with_options(&options, &query).unwrap(),
            "INTEGER"
        );
    }
}
