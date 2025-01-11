use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

#[cfg(not(feature = "backends-tokio"))]
use std::sync::Mutex;
#[cfg(feature = "backends-tokio")]
use tokio::sync::Mutex;

use crate::{GeekConnection, QueryBuilderTrait, TableBuilder};

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

impl<C> GeekConnection for Arc<Mutex<C>>
where
    Self: Sync + Send + 'static,
    C: GeekConnection<Connection = libsql::Connection>,
{
    type Connection = Arc<Mutex<libsql::Connection>>;

    async fn create_table<T>(connection: &Self::Connection) -> Result<(), crate::Error>
    where
        T: TableBuilder + QueryBuilderTrait + Sized + Serialize + DeserializeOwned,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < TIMEOUT {
            match connection.try_lock() {
                Ok(conn) => return C::create_table::<T>(&conn).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
        Err(crate::Error::LibSQLError(
            "Error getting write lock on connection".to_string(),
        ))
    }

    async fn row_count(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<i64, crate::Error> {
        let start = std::time::Instant::now();
        while start.elapsed() < TIMEOUT {
            match connection.try_lock() {
                Ok(conn) => return C::row_count(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
        Err(crate::Error::LibSQLError(
            "Error getting write lock on connection in row_count".to_string(),
        ))
    }

    async fn query<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < TIMEOUT {
            match connection.try_lock() {
                Ok(conn) => return C::query::<T>(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        Err(crate::Error::LibSQLError(
            "Error getting write lock on connection".to_string(),
        ))
    }

    async fn query_first<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < TIMEOUT {
            match connection.try_lock() {
                Ok(conn) => return C::query_first::<T>(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
        Err(crate::Error::LibSQLError(
            "Error getting write lock on connection".to_string(),
        ))
    }

    async fn execute<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < TIMEOUT {
            match connection.try_lock() {
                Ok(conn) => return C::execute::<T>(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
        Err(crate::Error::LibSQLError(
            "Error getting write lock on connection in execute".to_string(),
        ))
    }
}
