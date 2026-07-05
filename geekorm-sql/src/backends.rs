//! # Query Backends

/// Query backend enum
#[derive(Debug, Clone, Default)]
pub enum QueryBackend {
    /// SQLite backend
    Sqlite {
        /// SQLite Options
        options: SqliteBackendOptions,
    },
    /// PostgreSQL backend
    Postgres,

    /// Unknown backend
    #[default]
    Unknown,
}

/// Backend options to help with query building
#[derive(Debug, Clone, Default)]
pub struct SqliteBackendOptions {
    /// Is transactions enabled
    pub transactions: bool,
}

#[cfg(feature = "rusqlite")]
impl rusqlite::ToSql for crate::Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            crate::Value::Identifier(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*value as i64),
            )),
            crate::Value::Text(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(value.clone()),
            )),
            crate::Value::Integer(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*value),
            )),
            crate::Value::Blob(value) | crate::Value::Json(value) => Ok(
                rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Blob(value.clone())),
            ),
            crate::Value::Datetime(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*value as i64),
            )),
            crate::Value::Boolean(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*value as i64),
            )),
            crate::Value::Null => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Null,
            )),
        }
    }
}
