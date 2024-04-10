use std::collections::HashMap;

use crate::{Query, QueryBuilderTrait, TableBuilder, Value};

/// This module contains the LibSQL backend
#[cfg(feature = "libsql")]
pub mod libsql;

/// This trait is used to define the connection to the database.
///
/// The main focus of this trait is to provide a way to connect to the database for any Table that
/// implements it.
pub trait GeekConnector
where
    Self: TableBuilder + QueryBuilderTrait + Sized + serde::Serialize + serde::de::DeserializeOwned,
{
    /// The connection type
    type Connection;
    /// The row
    type Row;
    /// The rows to return type
    type Rows;

    /// Create a table in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn create_table(connection: &Self::Connection) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Run a SELECT Count query on the database and return the number of rows
    #[allow(async_fn_in_trait, unused_variables)]
    async fn row_count(connection: &Self::Connection, query: Query) -> Result<i64, crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Execute a query on the database and do not return any rows
    #[allow(async_fn_in_trait, unused_variables)]
    async fn execute(connection: &Self::Connection, query: Query) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Query the database with an active Connection and Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query(
        connection: &Self::Connection,
        query: Query,
    ) -> Result<Self::Rows, crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Query the database with an active Connection and Query and return the first row.
    ///
    /// Note: Make sure the query is limited to 1 row to avoid retrieving multiple rows
    /// and only using the first one.
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query_first(
        connection: &Self::Connection,
        query: Query,
    ) -> Result<Self::Row, crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Query the database with an active Connection and Query and return a list of GeekORM Values.
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query_raw(
        connection: &Self::Connection,
        query: Query,
    ) -> Result<Vec<HashMap<String, Value>>, crate::Error> {
        Err(crate::Error::NotImplemented)
    }
}
