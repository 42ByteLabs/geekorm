//! # Column types

use geekorm_core::{ColumnType, ColumnTypeOptions, Error};

use super::QueryType;
use crate::ToSql;

impl ToSql for ColumnType {
    fn to_sql_stream(&self, stream: &mut String, query: &super::QueryBuilder) -> Result<(), Error> {
        match self {
            ColumnType::Identifier(opts) => {
                stream.push_str("INTEGER ");
                opts.to_sql_stream(stream, query)?;
            }
            ColumnType::ForeignKey(options) => {
                let opts = options.to_sql(query)?;
                if opts.is_empty() {
                    stream.push_str("INTEGER");
                } else {
                    stream.push_str("INTEGER ");
                    stream.push_str(&opts);
                }
            }
            ColumnType::Text(options) => {
                let opts = options.to_sql(query)?;
                if opts.is_empty() {
                    stream.push_str("TEXT");
                } else {
                    stream.push_str("TEXT ");
                    stream.push_str(&opts);
                }
            }
            ColumnType::Integer(options) => {
                let opts = options.to_sql(query)?;
                if opts.is_empty() {
                    stream.push_str("INTEGER");
                } else {
                    stream.push_str("INTEGER ");
                    stream.push_str(&opts);
                }
            }
            ColumnType::Boolean(options) => {
                let opts = options.to_sql(query)?;
                if opts.is_empty() {
                    stream.push_str("INTEGER");
                } else {
                    stream.push_str("INTEGER ");
                    stream.push_str(&opts);
                }
            }
            ColumnType::Blob(options) => {
                let opts = options.to_sql(query)?;
                if opts.is_empty() {
                    stream.push_str("BLOB");
                } else {
                    stream.push_str("BLOB ");
                    stream.push_str(&opts);
                }
            }
        }

        Ok(())
    }
}

impl ToSql for ColumnTypeOptions {
    fn to_sql_stream(
        &self,
        stream: &mut String,
        _query: &super::QueryBuilder,
    ) -> Result<(), Error> {
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryType;
    use crate::builder::QueryBuilder;
    use geekorm_core::{Column, Table};

    fn table() -> Table {
        Table {
            name: "Test".to_string(),
            database: None,
            columns: vec![
                Column::new(
                    "id".to_string(),
                    ColumnType::Identifier(ColumnTypeOptions {
                        primary_key: true,
                        foreign_key: String::new(),
                        unique: true,
                        not_null: true,
                        auto_increment: true,
                    }),
                ),
                Column::new(
                    "name".to_string(),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
                Column::new(
                    "image_id".to_string(),
                    ColumnType::ForeignKey(ColumnTypeOptions {
                        foreign_key: "images.id".to_string(),
                        ..Default::default()
                    }),
                ),
            ]
            .into(),
        }
    }

    fn query() -> crate::QueryBuilder<'static> {
        crate::QueryBuilder::default()
    }

    #[test]
    fn test_column_type_boolean() {
        let column_type = ColumnType::Boolean(ColumnTypeOptions::default());
        let query = query();
        assert_eq!(column_type.to_sql(&query).unwrap(), "INTEGER");
    }

    #[test]
    fn test_column_type_to_sql() {
        let query = query();
        let column_type = ColumnType::Text(ColumnTypeOptions::default());
        assert_eq!(column_type.to_sql(&query).unwrap(), "TEXT");

        let column_type = ColumnType::Integer(ColumnTypeOptions::default());
        assert_eq!(column_type.to_sql(&query).unwrap(), "INTEGER");
    }

    #[test]
    fn test_column_type_options_to_sql() {
        let query = query();
        let column_type_options = ColumnTypeOptions::default();
        assert_eq!(column_type_options.to_sql(&query).unwrap(), "");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.to_sql(&query).unwrap(), "PRIMARY KEY");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            auto_increment: true,
            ..Default::default()
        };
        assert_eq!(
            column_type_options.to_sql(&query).unwrap(),
            "PRIMARY KEY AUTOINCREMENT"
        );
    }
}
