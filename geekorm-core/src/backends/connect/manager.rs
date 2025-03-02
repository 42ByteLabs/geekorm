use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

use url::Url;

use super::{Backend, Connection};

/// Connection
#[derive(Default)]
pub struct ConnectionManager {
    backend: Mutex<VecDeque<Backend>>,
    notifier: tokio::sync::Notify,
}

impl<'a> ConnectionManager {
    /// Connect to a database
    pub async fn connect(connection: impl Into<String>) -> Result<Self, crate::Error> {
        let connection = connection.into();
        let connect = connection.as_str();

        if connect == ":memory:" {
            Self::in_memory().await
        } else if connect.starts_with("./") || connect.starts_with("/") {
            Self::path(PathBuf::from(connect)).await
        } else {
            let Ok(url) = Url::parse(&connection) else {
                return Err(crate::Error::ConnectionError(
                    "Unknown connection string".to_string(),
                ));
            };
            Self::url(url).await
        }
    }

    pub async fn in_memory() -> Result<Self, crate::Error> {
        if cfg!(feature = "libsql") {
            let db = ::libsql::Builder::new_local(":memory:").build().await?;
            let conn = db.connect().unwrap();

            let manager = Self::default();
            manager.insert_backend(Backend::Libsql { conn });
            Ok(manager)
        } else {
            Err(crate::Error::ConnectionError(
                "Unknown connection string".to_string(),
            ))
        }
    }

    pub async fn path(path: impl Into<PathBuf>) -> Result<Self, crate::Error> {
        let path = path.into();
        if cfg!(feature = "libsql") {
            let db = ::libsql::Builder::new_local_replica(path).build().await?;
            let conn = db.connect().unwrap();

            let manager = Self::default();
            manager.insert_backend(Backend::Libsql { conn });
            Ok(manager)
        } else {
            Err(crate::Error::ConnectionError(
                "Unknown connection string".to_string(),
            ))
        }
    }

    pub async fn url(url: Url) -> Result<Self, crate::Error> {
        match url.scheme() {
            #[cfg(feature = "libsql")]
            "libsql" | "sqlite" => {
                let path = url.path();
                let db = ::libsql::Builder::new_local_replica(path).build().await?;
                let conn = db.connect().unwrap();

                let manager = Self::default();
                manager.insert_backend(Backend::Libsql { conn });
                Ok(manager)
            }
            _ => Err(crate::Error::ConnectionError(
                "Unknown connection string".to_string(),
            )),
        }
    }

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

    pub fn insert_backend(&self, backend: Backend) {
        let mut conns = self.backend.lock().unwrap();
        conns.push_back(backend);

        self.notifier.notify_one();
    }
}

#[cfg(feature = "libsql")]
impl From<libsql::Connection> for ConnectionManager {
    fn from(value: libsql::Connection) -> Self {
        Self {
            backend: Mutex::new(VecDeque::from(vec![Backend::Libsql { conn: value }])),
            ..Default::default()
        }
    }
}

#[cfg(feature = "rusqlite")]
impl From<rusqlite::Connection> for ConnectionManager {
    fn from(value: rusqlite::Connection) -> Self {
        Self {
            backend: Mutex::new(VecDeque::from(vec![Backend::Rusqlite { conn: value }])),
            ..Default::default()
        }
    }
}
