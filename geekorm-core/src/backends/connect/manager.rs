//! # Connection Manager
//!
//! The Connection Manager (as the name suggests) manages the connection pool for all of the
//! connections that are used during an applications life cycle. There should only be one manager
//! but it can be passed around as a reference until its needed.
//!
//! ## Acquiring a connection
//!
//! The primary feature of the manager is to acquire a connection from the pool. This is done using
//! async to prevent thread lockups while waiting for a connection.
//!
//! ```rust
//! # #[cfg(feature = "rusqlite")] {
//! use geekorm_core::backends::connect::{ConnectionManager, Connection};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Connection Manager creates a pool of connections to use
//!     let manager = ConnectionManager::connect(":memory:").await.unwrap();
//!
//!     // Acquired a connection from the pool used to perform query executions
//!     let connection = manager.acquire().await;
//!     // ... use the connection
//! }
//! # }
//! ```
//!
//! Async/Await is used to prevent thread locking waiting for a connection. Some tasks might take
//! seconds to perform and we don't want hang a thread waiting for a connection to become avalible.
//!
//! ## Transactions
//!
//! For very heavy actions you might want to use a transactions with many queries. You can do this
//! by using the `transactions()` function to obtain a new `Connection` in "transaction mode".
//!
//! ```rust
//! # #[cfg(feature = "rusqlite")] {
//! use geekorm_core::backends::connect::{ConnectionManager, Connection};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Connection Manager creates a pool of connections to use
//!     let manager = ConnectionManager::connect(":memory:").await.unwrap();
//!
//!     let connection = manager.transation().await;
//!     for id in 0..10 {
//!         // Add queries to the transation
//!     }
//!     // Perform the 10 queries as a transaction
//!     connection.execute_transaction().await.expect("Failed to perform transaction");
//! }
//! # }
//! ```
//!
//! ## Dropping a connection
//!
//! Dropping a connection is as easy as letting it drop out of scope of the borrow-checker.
//!
//! ```rust
//! # #[cfg(feature = "rusqlite")] {
//! use geekorm_core::backends::connect::{ConnectionManager, Connection};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Connection Manager creates a pool of connections to use
//!     let manager = ConnectionManager::connect(":memory:").await.unwrap();
//!
//!     {
//!         // Acquired a connection from the pool used to perform query executions
//!         let connection = manager.acquire().await;
//!         // ... use the connection
//!
//!     } // First connection is dropped
//!     
//!     // Get a new connection from the pool
//!     {
//!         let connection_2 = manager.acquire().await;
//!         // ... use the connection for different tasks
//!
//!     } // Second connection dropped
//! }
//! # }
//! ```
//!
//! Example of this would be a route in a web application acquires a connection from the pool,
//! performs actions and then drops it at the end to free up that connection.
//!

#![allow(unused_imports, unused_variables)]
use geekorm_sql::query::BatchQueries;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use url::Url;

use super::{Backend, Connection};
use crate::backends::connect::ConnectionType;

/// Connection Manager
///
/// The connection manager is used to manage the connections to the database.
pub struct ConnectionManager {
    /// The backend connections which are thread safe and can be shared between
    /// different parts of the code.
    ///
    /// The pool is a deque of connections, where the front of the deque is the
    /// next connection to be acquired and the back of the deque is the next
    /// connection to be released.
    backend: Arc<Mutex<VecDeque<Backend>>>,
    /// The type of database that the connection is connected to
    /// (e.g. in-memory, file-based, etc.)
    dbtype: super::ConnectionType,

    /// Notifier is used to notify the pool that a connection has been released
    /// and is ready to be acquired.
    ///
    /// The Arc is to allow the pool to be cloned and passed around to different
    /// parts of the code without having to worry about lifetimes.
    notifier: Arc<tokio::sync::Notify>,
}

impl Clone for ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            backend: Arc::clone(&self.backend),
            dbtype: self.dbtype.clone(),
            notifier: Arc::clone(&self.notifier),
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            backend: Arc::new(Mutex::new(VecDeque::new())),
            dbtype: ConnectionType::InMemory,
            notifier: Arc::new(tokio::sync::Notify::new()),
        }
    }
}

impl ConnectionManager {
    /// Get the type of database
    pub fn get_database_type(&self) -> super::ConnectionType {
        self.dbtype.clone()
    }

    /// Connect to a database
    ///
    /// *Examples:*
    /// - `:memory:`, `file://:memory:` (in-memory SQLite database)
    /// - `sqlite://./test.db`(SQLite database at `./test.db`)
    /// - `libsql:///path/to/db` (libsql database at `/path/to/db`)
    /// - `postgres://user:pass@localhost:5432/dbname` (Postgres database)
    ///
    pub async fn connect(connection: impl Into<String>) -> Result<Self, crate::Error> {
        let connection = connection.into();
        let connect = connection.as_str();

        if connect == ":memory:" {
            Self::in_memory().await
        } else if connect.starts_with("file:")
            || connect.starts_with("./")
            || connect.starts_with("/")
        {
            Self::path(PathBuf::from(connect)).await
        } else {
            let Ok(url) = Url::parse(&connection) else {
                return Err(crate::Error::ConnectionError(
                    "Error parsing the URL/URI".to_string(),
                ));
            };
            Self::url(url).await
        }
    }

    /// Connect to an in-memory database
    ///
    /// This is only supported for sqlite based databases.
    pub async fn in_memory() -> Result<Self, crate::Error> {
        let manager = Self {
            dbtype: ConnectionType::InMemory,
            ..Default::default()
        };

        #[cfg(feature = "libsql")]
        {
            let db = ::libsql::Builder::new_local(":memory:").build().await?;
            let conn = db.connect().unwrap();

            manager.insert_backend(Backend::Libsql { conn });
        }
        #[cfg(feature = "rusqlite")]
        {
            let conn = ::rusqlite::Connection::open_in_memory()?;

            manager.insert_backend(Backend::Rusqlite {
                conn: std::sync::Arc::new(conn),
            });
        }
        Ok(manager)
    }

    /// Connect to a database at a given path
    pub async fn path(path: impl Into<PathBuf>) -> Result<Self, crate::Error> {
        let path: PathBuf = path.into();
        #[cfg(feature = "log")]
        log::debug!("Connection to database path: {}", path.display());

        let Some(filename) = path.file_name() else {
            return Err(crate::Error::ConnectionError(
                "Database path requires to have a file name".to_string(),
            ));
        };

        if filename == ":memory:" || filename == "file::memory:" {
            return Self::in_memory().await;
        }

        // Create the parent directory if it doesn't exist (recursively)
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                #[cfg(feature = "log")]
                log::debug!("Creating parent directory: {}", parent.display());
                tokio::fs::create_dir_all(parent).await?;
            }
        };

        let manager = Self {
            dbtype: ConnectionType::Path { file: path.clone() },
            ..Default::default()
        };

        #[cfg(feature = "rusqlite")]
        {
            crate::backends::rusqlite::connect(&manager, &path)
                .await
                .expect("Failed to create `rusqlite` connection");
        }
        #[cfg(feature = "libsql")]
        {
            crate::backends::libsql::connect(&manager, &path)
                .await
                .expect("Failed to create `libsql` connection");
        }
        Ok(manager)
    }

    /// Connect to a database using a URL
    ///
    #[allow(unreachable_patterns)]
    pub async fn url(url: Url) -> Result<Self, crate::Error> {
        match url.scheme() {
            "file" => {
                let path = url.path();
                Self::path(path).await
            }
            #[cfg(feature = "libsql")]
            "libsql" | "sqlite" => {
                if let Some(host) = url.host_str() {
                    if host == "memory" || host == ":memory:" {
                        return Self::in_memory().await;
                    }
                    Err(crate::Error::ConnectionError(
                        "Remote connection handling is not yet supported".to_string(),
                    ))
                } else {
                    Self::path(url.path()).await
                }
            }
            #[cfg(feature = "rusqlite")]
            "rusqlite" | "sqlite" => {
                if let Some(host) = url.host_str() {
                    if host == "memory" || host == ":memory:" {
                        return Self::in_memory().await;
                    }
                    Err(crate::Error::ConnectionError(
                        "Remote connection handling is not yet supported".to_string(),
                    ))
                } else {
                    Self::path(url.path()).await
                }
            }
            _ => Err(crate::Error::ConnectionError(format!(
                "Unknown database URL scheme: {}",
                url.scheme()
            ))),
        }
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> Connection<'_> {
        self.notifier.notified().await;
        let mut conns = self.backend.lock().unwrap();
        let conn = conns.pop_front().unwrap();

        Connection {
            pool: self,
            query_count: AtomicUsize::new(0),
            backend: conn,
        }
    }

    /// Aquire a connector from the pool in Transation mode
    pub async fn transation(&self) -> Connection<'_> {
        // Do NOT acquire a backend connection and lock
        Connection {
            pool: self,
            query_count: AtomicUsize::new(0),
            backend: Backend::Transactions {
                queries: BatchQueries::new(),
            },
        }
    }

    /// Insert a connection back into the pool
    pub fn insert_backend(&self, backend: Backend) {
        let mut conns = self.backend.lock().unwrap();
        conns.push_back(backend);

        self.notifier.notify_one();
    }
}

#[cfg(feature = "libsql")]
impl From<libsql::Connection> for ConnectionManager {
    fn from(value: libsql::Connection) -> Self {
        let backend = Backend::Libsql { conn: value };
        let cm = Self::default();
        cm.insert_backend(backend);

        cm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::connect::ConnectionType;

    #[tokio::test]
    async fn test_connect_memory() {
        let cm = ConnectionManager::in_memory()
            .await
            .expect("Failed to connect to in-memory database");
        assert_eq!(cm.get_database_type(), ConnectionType::InMemory);

        let cm = ConnectionManager::connect(":memory:")
            .await
            .expect("Failed to connect to in-memory database");
        assert_eq!(cm.get_database_type(), ConnectionType::InMemory);

        let cm = ConnectionManager::connect("file::memory:")
            .await
            .expect("Failed to connect to in-memory database");
        assert_eq!(cm.get_database_type(), ConnectionType::InMemory);
    }

    #[tokio::test]
    async fn test_connect_path() {
        let path = PathBuf::from("./test.db");
        let cm = ConnectionManager::path(path.clone())
            .await
            .expect("Failed to connect to database");
        assert_eq!(cm.get_database_type(), ConnectionType::Path { file: path });

        let path = PathBuf::from("/tmp/test.db");
        let cm = ConnectionManager::path(path.clone())
            .await
            .expect("Failed to connect to database");
        assert_eq!(cm.get_database_type(), ConnectionType::Path { file: path });
    }

    #[cfg(feature = "rusqlite")]
    #[tokio::test]
    async fn test_connect_url() {
        let url = Url::parse("sqlite:///tmp/test.db").unwrap();
        let cm = ConnectionManager::url(url)
            .await
            .expect("Failed to connect to database");
        assert_eq!(
            cm.get_database_type(),
            ConnectionType::Path {
                file: PathBuf::from("/tmp/test.db")
            }
        );
    }
}
