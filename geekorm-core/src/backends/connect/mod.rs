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
