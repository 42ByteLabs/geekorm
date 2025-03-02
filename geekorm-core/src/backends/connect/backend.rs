use crate::GeekConnection;

use super::{Backend, Connection};

impl<'a> GeekConnection for Connection<'a> {
    type Connection = Self;

    async fn create_table<T>(connection: &Self::Connection) -> Result<(), crate::Error>
    where
        T: crate::TableBuilder
            + crate::QueryBuilderTrait
            + Sized
            + serde::Serialize
            + serde::de::DeserializeOwned,
    {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::create_table::<T>(&conn).await
            }
            _ => unimplemented!(),
        }
    }

    async fn batch(connection: &Self::Connection, query: crate::Query) -> Result<(), crate::Error> {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::batch(&conn, query).await
            }
            _ => unimplemented!(),
        }
    }

    async fn query<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::query(&conn, query).await
            }
            _ => unimplemented!(),
        }
    }

    async fn execute(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error> {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::execute(conn, query).await
            }
            _ => unimplemented!(),
        }
    }

    async fn row_count(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<i64, crate::Error> {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::row_count(&conn, query).await
            }
            _ => unimplemented!(),
        }
    }

    async fn query_raw(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<std::collections::HashMap<String, crate::Value>>, crate::Error> {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::query_raw(&conn, query).await
            }
            _ => unimplemented!(),
        }
    }

    async fn query_first<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        connection
            .query_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match &connection.backend {
            #[cfg(feature = "libsql")]
            Backend::Libsql { conn, .. } => {
                <libsql::Connection as GeekConnection>::query_first(&conn, query).await
            }
            _ => unimplemented!(),
        }
    }
}
