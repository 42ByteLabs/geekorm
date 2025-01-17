#[cfg(feature = "migrations")]
use crate::AlterQuery;
#[cfg(feature = "migrations")]
use quote::quote;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

#[cfg(feature = "migrations")]
impl quote::ToTokens for ColumnType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ColumnType::Identifier(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Identifier(#options)
                });
            }
            ColumnType::Text(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Text(#options)
                });
            }
            ColumnType::Integer(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Integer(#options)
                });
            }
            ColumnType::Boolean(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Boolean(#options)
                });
            }
            ColumnType::Blob(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Blob(#options)
                });
            }
            ColumnType::ForeignKey(options) => tokens.extend(quote! {
                geekorm::ColumnType::ForeignKey(#options)
            }),
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

    #[cfg(feature = "migrations")]
    fn on_alter(&self, _query: &AlterQuery) -> Result<String, crate::Error> {
        match self {
            ColumnType::Text(opts) => {
                if opts.not_null {
                    Ok("TEXT NOT NULL DEFAULT ''".to_string())
                } else {
                    Ok("TEXT".to_string())
                }
            }
            ColumnType::Integer(opts) | ColumnType::Boolean(opts) => {
                if opts.not_null {
                    Ok("INTEGER NOT NULL DEFAULT 0".to_string())
                } else {
                    Ok("INTEGER".to_string())
                }
            }
            ColumnType::Blob(opts) => {
                if opts.not_null {
                    Ok("BLOB NOT NULL DEFAULT ''".to_string())
                } else {
                    Ok("BLOB".to_string())
                }
            }
            _ => Ok("BEANS".to_string()),
        }
    }
}

impl ColumnType {
    /// Check if the column type is a primary key
    pub fn is_primary_key(&self) -> bool {
        matches!(self, ColumnType::Identifier(_))
    }

    /// Check if the column type is nullable
    pub fn is_not_null(&self) -> bool {
        match self {
            ColumnType::Identifier(_) => false,
            ColumnType::ForeignKey(_) => false,
            ColumnType::Text(opts) => opts.not_null,
            ColumnType::Integer(opts) => opts.not_null,
            ColumnType::Boolean(opts) => opts.not_null,
            ColumnType::Blob(opts) => opts.not_null,
        }
    }

    /// Check if the column type is unique
    pub fn is_unique(&self) -> bool {
        match self {
            ColumnType::Identifier(_) => true,
            ColumnType::ForeignKey(_) => false,
            ColumnType::Text(opts) => opts.unique,
            ColumnType::Integer(opts) => opts.unique,
            ColumnType::Boolean(opts) => opts.unique,
            ColumnType::Blob(opts) => opts.unique,
        }
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
    /// Get the foreign key table by name
    pub fn foreign_key_table_name(&self) -> Option<String> {
        match self {
            ColumnType::ForeignKey(opts) => {
                let (t, _) = opts.foreign_key.split_once('.').unwrap();
                Some(t.to_string())
            }
            _ => None,
        }
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

#[cfg(feature = "migrations")]
impl quote::ToTokens for ColumnTypeOptions {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let primary_key = &self.primary_key;
        let foreign_key = &self.foreign_key;
        let unique = &self.unique;
        let not_null = &self.not_null;
        let auto_increment = &self.auto_increment;

        tokens.extend(quote! {
            geekorm::ColumnTypeOptions {
                primary_key: #primary_key,
                unique: #unique,
                not_null: #not_null,
                foreign_key: String::from(#foreign_key),
                auto_increment: #auto_increment,
            }
        });
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
    #[cfg(feature = "migrations")]
    use crate::builder::alter::AlterMode;

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

    #[test]
    fn test_alter_table_to_sql() {
        let query = crate::AlterQuery::new(AlterMode::AddColumn, "Table", "colname");

        let column_type = ColumnType::Text(ColumnTypeOptions::default());
        assert_eq!(column_type.on_alter(&query).unwrap(), "TEXT");

        let column_type = ColumnType::Text(ColumnTypeOptions {
            not_null: true,
            ..Default::default()
        });
        assert_eq!(
            column_type.on_alter(&query).unwrap(),
            "TEXT NOT NULL DEFAULT ''"
        );

        let column_type = ColumnType::Integer(ColumnTypeOptions::default());
        assert_eq!(column_type.on_alter(&query).unwrap(), "INTEGER");
    }
}
