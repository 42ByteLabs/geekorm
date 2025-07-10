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

// #[cfg(feature = "geekorm")]
// impl From<&Backend> for QueryBackend {
//     fn from(backend: &Backend) -> Self {
//         match backend {
//             #[cfg(feature = "libsql")]
//             Backend::Libsql { .. } => QueryBackend::Sqlite,
//             #[cfg(feature = "rusqlite")]
//             Backend::Rusqlite { .. } => QueryBackend::Sqlite,
//             #[cfg(feature = "postgres")]
//             Backend::Postgres { .. } => QueryBackend::Postgres,
//             _ => QueryBackend::Unknown,
//         }
//     }
// }
//
// #[cfg(feature = "geekorm")]
// impl From<&Connection<'_>> for QueryBackend {
//     fn from(connection: &Connection<'_>) -> Self {
//         match connection.backend() {
//             #[cfg(feature = "libsql")]
//             Backend::Libsql { .. } => QueryBackend::Sqlite,
//             #[cfg(feature = "rusqlite")]
//             Backend::Rusqlite { .. } => QueryBackend::Sqlite,
//             #[cfg(feature = "postgres")]
//             Backend::Postgres { .. } => QueryBackend::Postgres,
//             _ => QueryBackend::Unknown,
//         }
//     }
// }
