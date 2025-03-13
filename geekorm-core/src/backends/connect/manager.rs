//! # Connection Manager
#![allow(unused_imports, unused_variables)]
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

use url::Url;

use crate::backends::connect::ConnectionType;

use super::{Backend, Connection};

/// Connection
#[derive(Default)]
pub struct ConnectionManager {
    backend: Mutex<VecDeque<Backend>>,
    /// The type of database
    dbtype: super::ConnectionType,

    notifier: tokio::sync::Notify,
}

impl Clone for ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            backend: Mutex::new(self.backend.lock().unwrap().clone()),
            ..Default::default()
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
            backend: Mutex::new(VecDeque::new()),
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
            backend: Mutex::new(VecDeque::new()),
            dbtype: ConnectionType::Path { file: path.clone() },
            ..Default::default()
        };

        #[cfg(feature = "libsql")]
        {
            let db = ::libsql::Builder::new_local(path).build().await?;
            let conn = db.connect().unwrap();

            manager.insert_backend(Backend::Libsql { conn });
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
        let cm = Self {
            backend: Mutex::new(VecDeque::new()),
            ..Default::default()
        };
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
