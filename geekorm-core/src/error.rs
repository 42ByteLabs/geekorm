//! Error Module for GeekORM

/// Error type for the crate
#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
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
    #[error("No Rows Found in the database for the query")]
    NoRowsFound,

    /// Not Implemented
    #[error("Not Implemented")]
    NotImplemented,

    /// Error Hashing Password
    #[error("Error Hashing Password: {0}")]
    HashingError(String),

    /// Serde Error
    #[error("Serde Error: {0}")]
    SerdeError(String),

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

    /// LibSQL Error
    #[cfg(feature = "libsql")]
    #[error("LibSQL Error occurred: {0}")]
    LibSQLError(String),

    /// RuSQLite Error
    #[cfg(feature = "rusqlite")]
    #[error("RuSQLite Error occurred: {0}")]
    RuSQLiteError(String),
}
