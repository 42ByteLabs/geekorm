//! # Connection
//!
//! ## Example
//!
//! ```rust
//! # #[cfg(feature = "libsql")] {
//!
//! use geekorm_core::backends::connect::Pool;
//!
//! #[tokio::main]
//! async fn main() {
//!     let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
//!     let conn = db.connect().unwrap();
//!     let pool = Pool::new(conn);
//!     // let connect = Connection::connect(":memory:").await.unwrap();
//!
//! }
//!
//! # }
//! ```
//!

#![allow(missing_docs)]

use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;

pub mod backend;
pub mod manager;

use manager::ConnectionManager;

pub struct Connection<'a> {
    pool: &'a ConnectionManager,
    query_count: AtomicUsize,
    backend: Backend,
}

#[derive(Default, Clone)]
pub enum Backend {
    #[cfg(feature = "libsql")]
    Libsql { conn: libsql::Connection },
    #[cfg(feature = "rusqlite")]
    Rusqlite { conn: rusqlite::Connection },
    #[default]
    Unknown,
}

impl Connection<'_> {
    pub fn count(&self) -> usize {
        self.query_count.load(std::sync::atomic::Ordering::Relaxed)
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
            Backend::Unknown => write!(f, "Backend::Unknown"),
        }
    }
}

// /// Database Driver
// pub trait Driver
// where
//     Self: Sized + Sync + Send + 'static,
// {
//     /// Connect to the database
//     #[allow(async_fn_in_trait, unused_variables)]
//     async fn connect(connection: impl Into<String>) -> Result<Self, crate::Error>;
// }
//
// /// Generic connection manager
// pub struct ConnectionManager<D>
// where
//     D: Driver,
// {
//     connection: Mutex<VecDeque<D>>,
// }

// pub struct Connection<'a, T>
// where
//     T: GeekConnection<Connection = T> + 'a,
// {
//     pool: &'a ConnectionManager,
//     backend: super::Backend,
// }
//
// impl<T> ConnectionManager
// where
//     T: GeekConnection<Connection = T> + 'static,
// {
//     pub fn new() -> Self {
//         Self {
//             connection: Mutex::new(VecDeque::new()),
//         }
//     }
//
//     pub fn get(&self) -> Connection<'_, T> {
//         let mut conns = self.connection.lock().unwrap();
//         let backend = conns.pop_front().unwrap();
//         Connection::<T> {
//             pool: self,
//             backend,
//         }
//     }
//
//     pub fn insert_backend(&self, backend: super::Backend) {
//         let mut conns = self.connection.lock().unwrap();
//         conns.push_back(backend);
//     }
// }

// impl bb8::ManageConnection for ConnectionManager
// where
//     Self: Sized + Sync + Send + 'static,
//     Self::Connection: GeekConnection<Connection = Self>,
// {
//     type Connection = Self;
//     type Error = libsql::Error;
//
//     async fn connect(&self) -> Result<Self::Connection, Self::Error> {
//         let conn = match self.backend {
//             #[cfg(feature = "libsql")]
//             Backend::Libsql { ref conn } => {
//                 let conn = conn.clone();
//                 Backend::Libsql { conn }
//             }
//             #[cfg(feature = "rusqlite")]
//             Backend::Rusqlite { ref conn } => {
//                 let conn = conn.clone();
//                 Backend::Rusqlite { conn }
//             }
//         };
//         Ok(conn)
//     }
//
//     async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
//         match self.backend {
//             #[cfg(feature = "libsql")]
//             Backend::Libsql { ref conn } => {
//                 conn.execute("SELECT 1", ()).await?;
//             }
//             #[cfg(feature = "rusqlite")]
//             Backend::Rusqlite { ref conn } => {
//                 conn.execute("SELECT 1", ()).await?;
//             }
//         }
//         Ok(())
//     }
//
//     fn has_broken(&self, conn: &mut Self::Connection) -> bool {
//         false
//     }
// }

// #[async_trait]
// pub trait Driver: Sync + Send {
//     async fn connect(
//         &self,
//         connection: impl Into<String>,
//     ) -> Result<Box<dyn GeekConnection>, crate::Error>;
// }
//
// #[cfg(feature = "libsql")]
// impl Driver for libsql::Connection {
//     async fn connect(
//         &self,
//         connection: impl Into<String>,
//     ) -> Result<Box<dyn GeekConnection<Connection = libsql::Connection>>, crate::Error> {
//         let connection_string = connection.into();
//
//         match connection_string.as_str() {
//             ":memory:" => {
//                 let db = libsql::Builder::new_local(":memory:").build().await?;
//                 let conn = db.connect().unwrap();
//
//                 Ok(Self::new(conn))
//             }
//             _ => Err(crate::Error::ConnectionError(
//                 "Unknown connection string".to_string(),
//             )),
//         }
//     }
// }
//
// /// GeekORM Connection
// // pub struct Connection<'a, T>
// // where
// //     T: GeekConnection<Connection = T> + 'a,
// // {
// //     pool: &'a Pool<T>,
// //     conn: Option<Backend>,
// // }
// // pub trait Connection<T>: Sync + Send
// // where
// //     T: GeekConnection<Connection = T>,
// // {
// //     fn query(&self, query: impl Into<String>) -> Result<(), crate::Error>;
// // }
//
// /// A pool of connections
// pub struct Pool<T> {
//     connections: Mutex<VecDeque<T>>,
// }
//
// /// GeekORM Backend Configuration
// enum Backend {
//     /// Use the libsql backend
//     #[cfg(feature = "libsql")]
//     Libsql {
//         /// The libsql database
//         database: libsql::Connection,
//     },
//     /// Use the rusqlite backend
//     #[cfg(feature = "rusqlite")]
//     Rusqlite {
//         /// The rusqlite connection
//         conn: rusqlite::Connection,
//     },
// }
//
// impl<'a, T> Pool<T>
// where
//     T: GeekConnection<Connection = T>,
//     Backend: From<T>,
// {
//     /// Create a new pool
//     pub fn new(connection: T) -> Self {
//         let pool = Self {
//             connections: Mutex::new(VecDeque::new()),
//         };
//         pool.insert_backend(Backend::from(connection));
//         pool
//     }
//
//     pub fn insert_backend(&self, backend: Backend) {
//         let mut conns = self.connections.lock().unwrap();
//         conns.push_back(backend);
//     }
//
//     /// Get a connection from the pool
//     pub async fn acquire(&self) -> dyn Connection<'a, T> {
//         let mut conns = self.connections.lock().unwrap();
//         let backend = conns.pop_front().unwrap();
//     }
//
//     // pub async fn connect(connection: impl Into<String>) -> Result<Pool<T>, crate::Error> {
//     //     let connection_string = connection.into();
//     //
//     //     match connection_string.as_str() {
//     //         #[cfg(feature = "libsql")]
//     //         ":memory:" => {
//     //             let db = libsql::Builder::new_local(":memory:").build().await?;
//     //             let conn = db.connect().unwrap();
//     //
//     //             Ok(Self::new(conn))
//     //         }
//     //         _ => Err(crate::Error::ConnectionError(
//     //             "Unknown connection string".to_string(),
//     //         )),
//     //     }
//     // }
// }
//
// impl<'a, T> Drop for Connection<'a, T>
// where
//     T: GeekConnection<Connection = T> + 'a,
// {
//     fn drop(&mut self) {
//         // On drop, we put the connection back into the pool
//         let conn = self.conn.take().unwrap();
//         self.pool.insert_backend(conn);
//     }
// }
//
// #[cfg(feature = "libsql")]
// impl From<libsql::Connection> for Backend {
//     fn from(database: libsql::Connection) -> Self {
//         Self::Libsql { database }
//     }
// }
//
// #[cfg(feature = "rusqlite")]
// impl From<rusqlite::Connection> for Backend {
//     fn from(conn: rusqlite::Connection) -> Self {
//         Self::Rusqlite { conn }
//     }
// }
// impl<'a, T> Connection<'a, T>
// where
//     T: GeekConnection<Connection = T> + 'a,
// {
//     /// Connect to a database
//     pub async fn connect(connection: impl Into<String>) -> Result<Pool<T>, crate::Error> {
//         let connection_string = connection.into();
//
//         let pool = Pool::new(connection_string);
//
//         match connection_string.as_str() {
//             #[cfg(feature = "libsql")]
//             ":memory:" => {
//                 let db = libsql::Builder::new_local(":memory:").build().await?;
//                 let conn = db.connect().unwrap();
//
//                 database.insert_connection(conn);
//             }
//             // #[cfg(feature = "libsql")]
//             // path if path.starts_with("libsql://") => {
//             //     let path = path.strip_prefix("libsql://").unwrap();
//             //     database.insert_connection(Self {
//             //         pool: &database,
//             //         backend: Backend::Libsql {
//             //             database: libsql::Builder::new_local_replica(path).build().await?,
//             //         },
//             //     });
//             // }
//             _ => {
//                 return Err(crate::Error::ConnectionError(
//                     "Unknown connection string".to_string(),
//                 ));
//             }
//         }
//
//         Ok(database)
//     }
// }
