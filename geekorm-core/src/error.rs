//! Error Module for GeekORM

/// Error type for the crate
#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    /// Database Connection Error
    #[error("Connection Error: {0}")]
    ConnectionError(String),
    /// Database Schema Error
    #[error("Schema Error: {0}")]
    SchemaError(String),
    /// Database Migration Error
    #[cfg(feature = "migrations")]
    #[error("{0}")]
    MigrationError(#[from] MigrationError),

    /// Query Builder Error
    #[error("QueryBuilderError: {0} ({1})")]
    QueryBuilderError(String, String),

    /// Column Not Found
    #[error("ColumnNotFound: Table({0}) {1}")]
    ColumnNotFound(String, String),

    /// Column Skipped
    #[error("Column Skipped")]
    ColumnSkipped,

    /// No Rows was found in the database for the query
    #[error("No Rows Found - Query: '{query}'")]
    NoRowsFound {
        /// Query
        query: String,
    },

    /// Pagination Error
    #[cfg(feature = "pagination")]
    #[error("Pagination Error: {0}")]
    PaginationError(String),

    /// Not Implemented
    #[error("Not Implemented")]
    NotImplemented,

    /// Error Hashing Password
    #[error("Error Hashing Password: {0}")]
    HashingError(String),

    /// Serde Error
    #[error("Serde Error: {0}")]
    SerdeError(String),

    /// Unknown Variant
    #[error("Unknown Variant {0}")]
    UnknownVariant(String),

    /// Unknown / Generic Error
    #[error("Unknown Error / Generic Error occurred")]
    Unknown,

    /// TOTP Error
    #[cfg(feature = "two-factor-auth")]
    #[error("TOTP Error: {0}")]
    TotpError(String),
    /// SystemTime Error
    #[error("SystemTime Error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    /// IO Error
    #[error("IO Error: {0}")]
    IOError(String),

    /// LibSQL Error
    #[cfg(feature = "libsql")]
    #[error(
        "LibSQL Error: {error}\n -> {query}\nPlease report this error to the GeekORM developers"
    )]
    LibSQLError {
        /// Error message
        error: String,
        /// Query
        query: String,
    },

    /// RuSQLite Error
    #[cfg(feature = "rusqlite")]
    #[error("RuSQLite Error occurred: {0}")]
    RuSQLiteError(String),

    /// Query Syntax Error
    #[error(
        "Query Syntax Error: {error}\n -> {query}\nPlease report this error to the GeekORM developers"
    )]
    QuerySyntaxError {
        /// Error message
        error: String,
        /// Query
        query: String,
    },
}

/// GeekORM Migration Error
#[cfg(feature = "migrations")]
#[derive(Debug, thiserror::Error, Clone)]
pub enum MigrationError {
    /// Missing Table (table name)
    #[error("Missing Table `{0}`")]
    MissingTable(String),
    /// Missing Column (table name, column name)
    #[error("Missing Column `{table}.{column}`")]
    MissingColumn {
        /// Table name
        table: String,
        /// Column name
        column: String,
    },
    /// Column Type Mismatch (table name, column name, feature)
    #[error("Column Type Mismatch `{table}.{column}`: {feature}")]
    ColumnTypeMismatch {
        /// Table name
        table: String,
        /// Column name
        column: String,
        /// Feature
        feature: String,
    },

    /// New Table (table name)
    #[error("New Table `{table}`")]
    NewTable {
        /// Table name
        table: String,
    },
    /// New Column (table name, column name)
    #[error("New Column `{table}.{column}`")]
    NewColumn {
        /// Table name
        table: String,
        /// Column name
        column: String,
    },

    /// Upgrade Error (reason)
    #[error("Upgrade Error: {0}")]
    UpgradeError(String),
    /// Missing Migration (migration name)
    #[error("Missing Migration: {0}")]
    MissingMigration(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e.to_string())
    }
}

#[cfg(feature = "rusqlite")]
impl From<::rusqlite::Error> for Error {
    fn from(e: ::rusqlite::Error) -> Self {
        Self::RuSQLiteError(e.to_string())
    }
}
