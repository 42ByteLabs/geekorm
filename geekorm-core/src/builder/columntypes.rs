use std::fmt::Display;

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
    /// Blob / Vec / List column type with options
    Blob(ColumnTypeOptions),
}

impl Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Identifier(_) => write!(f, "PrimaryKey"),
            ColumnType::ForeignKey(fk) => write!(f, "ForeignKey<{}>", fk),
            ColumnType::Text(_) => write!(f, "Text"),
            ColumnType::Integer(_) => write!(f, "Integer"),
            ColumnType::Boolean(_) => write!(f, "Boolean"),
            ColumnType::Blob(_) => write!(f, "Blob"),
        }
    }
}

impl ToSqlite for ColumnType {
    fn on_create(&self, query: &crate::QueryBuilder) -> Result<String, crate::Error> {
        Ok(match self {
            ColumnType::Identifier(opts) => {
                format!("INTEGER {}", opts.on_create(query)?)
            }
            ColumnType::ForeignKey(options) => {
                // TODO(geekmasher): What type is the foreign key?
                let opts = options.on_create(query)?;
                if opts.is_empty() {
                    return Ok("INTEGER".to_string());
                }
                format!("INTEGER {}", opts)
            }
            ColumnType::Text(options) => {
                let opts = options.on_create(query)?;
                if opts.is_empty() {
                    return Ok("TEXT".to_string());
                }
                format!("TEXT {}", options.on_create(query)?)
            }
            ColumnType::Integer(options) => {
                let opts = options.on_create(query)?;
                if opts.is_empty() {
                    return Ok("INTEGER".to_string());
                }
                format!("INTEGER {}", options.on_create(query)?)
            }
            ColumnType::Boolean(options) => {
                let opts = options.on_create(query)?;
                if opts.is_empty() {
                    return Ok("INTEGER".to_string());
                }
                format!("INTEGER {}", options.on_create(query)?)
            }
            ColumnType::Blob(options) => {
                let opts = options.on_create(query)?;
                if opts.is_empty() {
                    return Ok("BLOB".to_string());
                }
                format!("BLOB {}", options.on_create(query)?)
            }
        })
    }
}

impl ColumnType {
    /// Check if the column type is a primary key
    pub fn is_primary_key(&self) -> bool {
        matches!(self, ColumnType::Identifier(_))
    }

    /// Check if the column type is an auto increment
    pub fn is_auto_increment(&self) -> bool {
        match self {
            ColumnType::Identifier(opts) => opts.auto_increment,
            ColumnType::Integer(opts) => opts.auto_increment,
            _ => false,
        }
    }

    /// Check if the column type is a foreign key
    pub fn is_foreign_key(&self) -> bool {
        matches!(self, ColumnType::ForeignKey(_))
    }
    /// Get the foreign key table & column name
    pub fn is_foreign_key_table(&self, table: &String) -> bool {
        match self {
            ColumnType::ForeignKey(opts) => {
                let (t, _) = opts.foreign_key.split_once('.').unwrap();
                t == table
            }
            _ => false,
        }
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
    /// Auto increment the column
    pub auto_increment: bool,
}

impl ColumnTypeOptions {
    pub(crate) fn primary_key() -> Self {
        ColumnTypeOptions {
            primary_key: true,
            auto_increment: true,
            ..Default::default()
        }
    }

    pub(crate) fn foreign_key(key: String) -> Self {
        ColumnTypeOptions {
            primary_key: false,
            foreign_key: key,
            unique: false,
            not_null: true,
            auto_increment: false,
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

impl Display for ColumnTypeOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.foreign_key.is_empty() {
            return write!(f, "{}", self.foreign_key);
        }
        Err(std::fmt::Error)
    }
}

impl ToSqlite for ColumnTypeOptions {
    fn on_create(&self, _query: &crate::QueryBuilder) -> Result<String, crate::Error> {
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
        if self.auto_increment {
            sql.push("AUTOINCREMENT");
        }
        Ok(sql.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query() -> crate::QueryBuilder {
        crate::QueryBuilder::default()
    }

    #[test]
    fn test_column_type_boolean() {
        let column_type = ColumnType::Boolean(ColumnTypeOptions::default());
        let query = query();
        assert_eq!(column_type.on_create(&query).unwrap(), "INTEGER");
    }

    #[test]
    fn test_column_type_to_sql() {
        let query = query();
        let column_type = ColumnType::Text(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(&query).unwrap(), "TEXT");

        let column_type = ColumnType::Integer(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(&query).unwrap(), "INTEGER");
    }

    #[test]
    fn test_column_type_options_to_sql() {
        let query = query();
        let column_type_options = ColumnTypeOptions::default();
        assert_eq!(column_type_options.on_create(&query).unwrap(), "");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            ..Default::default()
        };
        assert_eq!(
            column_type_options.on_create(&query).unwrap(),
            "PRIMARY KEY"
        );

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            auto_increment: true,
            ..Default::default()
        };
        assert_eq!(
            column_type_options.on_create(&query).unwrap(),
            "PRIMARY KEY AUTOINCREMENT"
        );
    }
}
