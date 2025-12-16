//! # Column types

use super::QueryType;
use super::columns::ColumnOptions;
use crate::{Error, ToSql};

/// Column types
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Text column type
    #[default]
    Text,
    /// Integer / Numeric column type
    Integer,
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
        _query: &super::Query,
    ) -> Result<String, Error> {
        let mut stream = String::new();

        match self {
            ColumnType::ForeignKey => {
                stream.push_str("INTEGER");
                let opts = options.sql();
                if !opts.is_empty() {
                    stream.push(' ');
                    stream.push_str(&opts);
                }
            }
            ColumnType::Text => {
                stream.push_str("TEXT");
                let opts = options.sql();
                if !opts.is_empty() {
                    stream.push(' ');
                    stream.push_str(&opts);
                }
            }
            ColumnType::Integer => {
                stream.push_str("INTEGER");
                let opts = options.sql();
                if !opts.is_empty() {
                    stream.push(' ');
                    stream.push_str(&opts);
                }
            }
            ColumnType::Boolean => {
                stream.push_str("INTEGER");
                let opts = options.sql();
                if !opts.is_empty() {
                    stream.push(' ');
                    stream.push_str(&opts);
                }
            }
            ColumnType::Blob => {
                stream.push_str("BLOB");
                let opts = options.sql();

                if !opts.is_empty() {
                    stream.push(' ');
                    stream.push_str(&opts);
                }
            }
        }

        Ok(stream)
    }
}

impl ToSql for ColumnOptions {
    fn sql(&self) -> String {
        let mut stream = String::new();
        let mut sql = Vec::new();

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

        stream.push_str(&sql.join(" "));
        stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QueryType, Table,
        query::{
            Query,
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

    fn query() -> crate::Query {
        crate::Query::default()
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
        let column_type_options = ColumnOptions::default();
        assert_eq!(column_type_options.sql(), "");

        let column_type_options = ColumnOptions {
            primary_key: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.sql(), "PRIMARY KEY");

        let column_type_options = ColumnOptions {
            primary_key: true,
            auto_increment: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.sql(), "PRIMARY KEY AUTOINCREMENT");

        // Unique
        let column_type_options = ColumnOptions {
            unique: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.sql(), "UNIQUE");
    }
}
