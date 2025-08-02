//! # Query Backends

/// Query backend enum
#[derive(Debug, Clone, Default)]
pub enum QueryBackend {
    /// SQLite backend
    Sqlite,
    /// PostgreSQL backend
    Postgres,

    /// Unknown backend
    #[default]
    Unknown,
}
