use serde::{Deserialize, Serialize};

use crate::ToSqlite;

/// A column type and its options / properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnType {
    /// Identifier column type (Primary Key) which is a UUID (TEXT)
    Identifier(ColumnTypeOptions),
    /// Foreign Key column type with the table name
    ForeignKey(ColumnTypeOptions),
    /// Text column type with options
    Text(ColumnTypeOptions),
    /// Integer column type with options
    Integer(ColumnTypeOptions),
    /// Boolean column type with options
    Boolean(ColumnTypeOptions),
}

impl ToSqlite for ColumnType {
    fn on_create(&self) -> String {
        match self {
            ColumnType::Identifier(opts) => {
                format!("INTEGER {}", opts.on_create())
            }
            ColumnType::ForeignKey(options) => {
                let opts = options.on_create();
                if opts.is_empty() {
                    return "TEXT".to_string();
                }
                format!("TEXT {}", opts)
            }
            ColumnType::Text(options) => {
                let opts = options.on_create();
                if opts.is_empty() {
                    return "TEXT".to_string();
                }
                format!("TEXT {}", options.on_create())
            }
            ColumnType::Integer(options) => {
                let opts = options.on_create();
                if opts.is_empty() {
                    return "INTEGER".to_string();
                }
                format!("INTEGER {}", options.on_create())
            }
            ColumnType::Boolean(options) => {
                let opts = options.on_create();
                if opts.is_empty() {
                    return "INTEGER".to_string();
                }
                format!("INTEGER {}", options.on_create())
            }
        }
    }
}

impl ColumnType {
    /// Check if the column type is a primary key
    pub fn is_primary_key(&self) -> bool {
        matches!(self, ColumnType::Identifier(_))
    }

    /// Check if the column type is a foreign key
    pub fn is_foreign_key(&self) -> bool {
        matches!(self, ColumnType::ForeignKey(_))
    }
}

/// Column type options / properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnTypeOptions {
    /// Is the column a primary key for the table
    pub primary_key: bool,
    /// Is the column a foreign key
    /// TableName::ColumnName
    pub foreign_key: String,
    /// Is the column unique
    pub unique: bool,
    /// Is the column nullable
    pub not_null: bool,
}

impl ColumnTypeOptions {
    pub(crate) fn primary_key() -> Self {
        ColumnTypeOptions {
            primary_key: true,
            unique: false,
            not_null: true,
            foreign_key: String::new(),
        }
    }

    pub(crate) fn foreign_key(key: String) -> Self {
        ColumnTypeOptions {
            primary_key: false,
            foreign_key: key,
            unique: false,
            not_null: true,
        }
    }

    pub(crate) fn unique() -> Self {
        ColumnTypeOptions {
            unique: true,
            ..Default::default()
        }
    }

    pub(crate) fn null() -> Self {
        ColumnTypeOptions {
            not_null: false,
            ..Default::default()
        }
    }
}

impl ToSqlite for ColumnTypeOptions {
    fn on_create(&self) -> String {
        let mut sql = Vec::new();
        if self.not_null {
            sql.push("NOT NULL");
        }
        if self.primary_key {
            sql.push("PRIMARY KEY");
        }
        if self.unique {
            sql.push("UNIQUE");
        }
        sql.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_type_boolean() {
        let column_type = ColumnType::Boolean(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(), "INTEGER");
    }

    #[test]
    fn test_column_type_to_sql() {
        let column_type = ColumnType::Text(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(), "TEXT");

        let column_type = ColumnType::Integer(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(), "INTEGER");
    }

    #[test]
    fn test_column_type_options_to_sql() {
        let column_type_options = ColumnTypeOptions::default();
        assert_eq!(column_type_options.on_create(), "");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.on_create(), "PRIMARY KEY");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            not_null: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.on_create(), "NOT NULL PRIMARY KEY");
    }
}
