//! # Connection
//!
//! ## Example
//!
//! ```rust
//! # #[cfg(feature = "libsql")] {
//!
//! use geekorm_core::backends::connect::{ConnectionManager, Connection};
//!
//! #[tokio::main]
//! async fn main() {
//!     let manager = ConnectionManager::connect(":memory:").await.unwrap();
//!
//!     let connection = manager.acquire().await;
//!
//!     // ... do stuff with the connection
//! }
//!
//! # }
//! ```
//!

use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;

pub mod backend;
pub mod manager;

pub use manager::ConnectionManager;
use url::Url;

/// A connection to a database backend.
///
/// This is a wrapper around the actual connection to the database.
pub struct Connection<'a> {
    pool: &'a ConnectionManager,
    query_count: AtomicUsize,
    backend: Backend,
}

/// Backend is an enum that represents the different types of backends that
/// can be used to connect to a database.
#[derive(Default, Clone)]
pub enum Backend {
    /// A libsql connection
    #[cfg(feature = "libsql")]
    Libsql {
        /// The inner connection
        conn: libsql::Connection,
    },
    /// Unknown backend
    #[default]
    Unknown,
}

/// Connection Type
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ConnectionType {
    /// In-memory database
    #[default]
    InMemory,
    /// File-based database
    Path {
        /// Database file path
        file: PathBuf,
    },
    /// Remote database
    Remote {
        /// Database URL
        url: Url,
    },
}

impl Connection<'_> {
    /// Count the number of queries that have been executed since the
    /// connection was created.
    ///
    /// This is useful for testing to ensure that the expected number
    /// of queries have been executed against the database.
    pub fn count(&self) -> usize {
        self.query_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Display for Connection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { .. } => {
                write!(f, "Backend::Libsql({})", self.pool.get_database_type())
            }
            Backend::Unknown => write!(f, "Backend::Unknown"),
        }
    }
}

impl Drop for Connection<'_> {
    fn drop(&mut self) {
        // On drop, we put the connection back into the pool
        // TODO: Can we remove this clone?
        self.pool.insert_backend(self.backend.clone());
    }
}

impl Debug for Connection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { .. } => write!(f, "Backend::Libsql"),
            Backend::Unknown => write!(f, "Backend::Unknown"),
        }
    }
}

impl Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::InMemory => write!(f, "InMemory"),
            ConnectionType::Path { .. } => write!(f, "Path"),
            ConnectionType::Remote { .. } => write!(f, "Remote"),
        }
    }
}
