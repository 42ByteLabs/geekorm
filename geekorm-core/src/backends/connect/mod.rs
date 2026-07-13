//! # Connection
//!
//! GeekORM has a connection type that is used for performing the execution of queries on a
//! database.
//!
//! The `connect()` function is is a high-level construct to allow a user to connector to different
//! types of databases like in-memory, via a path or URL.
//!
//! **In-Memory Database:**
//!
//! To create a in-memory SQLite database, you only need to do the following:
//!
//! ```rust
//! # #[cfg(feature = "rusqlite")] {
//! use geekorm_core::backends::connect::{ConnectionManager, Connection};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Connection Manager creates a pool of connections to use
//!     let manager_connect = ConnectionManager::connect(":memory:").await.unwrap();
//!     // Or call the helper function
//!     let manager_in_memory = ConnectionManager::in_memory().await.unwrap();
//!
//!     // Acquired a connection from the pool used to perform query executions
//!     let connection_connect = manager_connect.acquire().await;
//!
//!     // ... do stuff with the connection
//! }
//! # }
//! ```
//!

use geekorm_sql::Query;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use url::Url;

use crate::backends::GeekConnection;
use geekorm_sql::query::BatchQueries;

pub mod backend;
pub mod manager;

pub use manager::ConnectionManager;

/// A connection to a database backend.
///
/// This is a wrapper around the actual connection to the database.
pub struct Connection<'a> {
    /// Connection Pool
    pool: &'a ConnectionManager,
    /// Number of queries run
    query_count: AtomicUsize,
    /// Backend to execute queries
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
        conn: ::libsql::Connection,
    },
    /// A rusqlite connection
    #[cfg(feature = "rusqlite")]
    Rusqlite {
        /// The inner connection
        conn: std::sync::Arc<::rusqlite::Connection>,
    },

    /// Transactions
    Transactions {
        /// Queries for the batch
        queries: BatchQueries,
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

    /// Checks the connection if it is in transaction mode
    pub fn is_transation_mode(&self) -> bool {
        matches!(self.backend, Backend::Transactions { .. })
    }

    /// Execute transaction
    pub async fn execute_transaction(&self) -> Result<(), crate::Error> {
        match &self.backend {
            Backend::Transactions { queries } => {
                let conn = self.pool.acquire().await;

                let query = Query::transaction().queries(queries).build()?;

                Connection::batch(&conn, query.into()).await
            }
            _ => Err(crate::Error::TransactionError(
                "Backend is not in transaction mode".to_string(),
            )),
        }
    }
}

impl Display for Connection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { .. } => {
                write!(f, "Backend::Libsql({})", self.pool.get_database_type())
            }
            #[cfg(feature = "rusqlite")]
            Backend::Rusqlite { .. } => {
                write!(f, "Backend::Rusqlite({})", self.pool.get_database_type())
            }
            Backend::Transactions { .. } => write!(f, "Backend::Transactions"),
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
            #[cfg(feature = "rusqlite")]
            Backend::Rusqlite { .. } => write!(f, "Backend::Rusqlite"),
            Backend::Transactions { .. } => write!(f, "Backend::Transactions"),
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
