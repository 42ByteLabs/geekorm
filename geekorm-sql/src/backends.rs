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
