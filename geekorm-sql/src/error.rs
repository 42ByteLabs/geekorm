//! # GeekORM SQL Error Module

/// This is the Error for the GeekORM SQL module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to build the SQL query
    #[error("Failed to build SQL query: {error} ({location})")]
    QueryBuilderError {
        /// The error message
        error: String,
        /// Location
        location: String,
    },

    /// Failed to load SQL file
    #[error("SQL File not Found: {path}")]
    SqlFileNotFound {
        /// Path to the SQL file
        path: String,
    },

    /// IO error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}
