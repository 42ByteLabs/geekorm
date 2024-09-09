//! # Backend Module for GeekORM
//!
//! **Example:**
//!
//! ```no_run
//! # #[cfg(feature = "rusqlite")] {
//! # use anyhow::Result;
//! use geekorm::prelude::*;
//!
//! #[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
//! pub struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     pub id: PrimaryKeyInteger,
//!     #[geekorm(unique)]
//!     pub username: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let connection = rusqlite::Connection::open_in_memory()
//!
//!     Users::create_table(&connection).await?;
//!
//!     let users = vec!["geekmasher", "bob", "alice", "eve", "mallory", "trent"];
//!     for user in users {
//!         let user = Users::new(user);
//!         user.save(&connection).await?;
//!     }
//!     
//!     // Fetch or create a user
//!     let mut geek = Users::new("geekmasher");
//!     geek.fetch_or_create(&connection).await?;
//!     
//!     // Fetch a user by their username (exact match)
//!     let geekmasher = Users::fetch_by_username(&connection, "geekmasher").await?;
//!
//!     // Search for a user (partial match)
//!     let search = Users::search(&connection, "geek").await?;
//!     # assert_eq!(search.len(), 1);
//!
//!
//!     Ok(())
//! }
//! # }

use std::collections::HashMap;

use crate::{Query, QueryBuilder, QueryBuilderTrait, TableBuilder, TablePrimaryKey, Value};

#[cfg(feature = "libsql")]
pub mod libsql;
#[cfg(feature = "rusqlite")]
pub mod rusqlite;

/// GeekConnection is the trait used for models to interact with the database.
///
/// This trait is used to define the methods that are used to interact with the database.
pub trait GeekConnector
where
    Self: Sized + TableBuilder + QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Query the database with an active Connection and Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query<'a, T>(
        connection: impl Into<&'a T>,
        query: Query,
    ) -> Result<Vec<Self>, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
    {
        Ok(T::query::<Self>(connection.into(), query).await?)
    }

    /// Query the first row from the database with an active Connection and Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query_first<'a, T>(
        connection: impl Into<&'a T>,
        query: Query,
    ) -> Result<Self, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
    {
        Ok(T::query_first::<Self>(connection.into(), query).await?)
    }

    /// Execute a query on the database and do not return any rows
    #[allow(async_fn_in_trait, unused_variables)]
    async fn execute<'a, T>(connection: impl Into<&'a T>, query: Query) -> Result<(), crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
    {
        Ok(T::execute::<Self>(connection.into(), query).await?)
    }

    /// Create a table in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn create_table<'a, T>(connection: impl Into<&'a T>) -> Result<(), crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
        Self: serde::Serialize,
    {
        Ok(T::create_table::<Self>(connection.into()).await?)
    }

    /// Count the number of rows based on a Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn row_count<'a, T>(
        connection: impl Into<&'a T>,
        query: Query,
    ) -> Result<i64, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
    {
        Ok(T::row_count(connection.into(), query).await?)
    }

    /// Update the current object in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn update<'a, T>(&self, connection: impl Into<&'a T>) -> Result<(), crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
    {
        Self::execute(connection, Self::query_update(self)).await
    }

    /// Save the current object to the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn save<'a, T>(&mut self, connection: impl Into<&'a T>) -> Result<(), crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a;

    /// Delete the current object from the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn delete<'a, T>(&self, connection: impl Into<&'a T>) -> Result<(), crate::Error>
    where
        T: GeekConnection + 'a,
    {
        Err(crate::Error::NotImplemented)
    }

    /// Fetches all of the foreign key values for the current object
    #[allow(async_fn_in_trait, unused_variables)]
    async fn fetch<'a, T>(&mut self, connection: impl Into<&'a T>) -> Result<(), crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a;

    /// Fetch all rows from the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn fetch_all<'a, T>(connection: impl Into<&'a T>) -> Result<Vec<Self>, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
    {
        Ok(T::query::<Self>(
            connection.into(),
            QueryBuilder::select().table(Self::table()).build()?,
        )
        .await?)
    }

    /// Fetch or create a row in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn fetch_or_create<'a, T>(
        &mut self,
        connection: impl Into<&'a T>,
    ) -> Result<(), crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a;

    /// Search for a row in the database based on specific criteria
    #[cfg(feature = "search")]
    #[allow(async_fn_in_trait, unused_variables)]
    async fn search<'a, T>(
        connection: impl Into<&'a T>,
        search: impl Into<String>,
    ) -> Result<Vec<Self>, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a;

    /// Fetch the first row from the database (based on the primary key)
    #[allow(async_fn_in_trait, unused_variables)]
    async fn first<'a, T>(connection: impl Into<&'a T>) -> Result<Self, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
        Self: TablePrimaryKey,
    {
        Ok(T::query_first::<Self>(
            connection.into(),
            Self::query_select()
                .table(Self::table())
                .order_by(
                    &Self::primary_key(),
                    crate::builder::models::QueryOrder::Asc,
                )
                .limit(1)
                .build()?,
        )
        .await?)
    }

    /// Fetch last row from the database (based on the primary key)
    #[allow(async_fn_in_trait, unused_variables)]
    async fn last<'a, T>(connection: impl Into<&'a T>) -> Result<Self, crate::Error>
    where
        T: GeekConnection<Connection = T> + 'a,
        Self: TablePrimaryKey,
    {
        Ok(T::query_first::<Self>(
            connection.into(),
            Self::query_select()
                .table(Self::table())
                .order_by(
                    &Self::primary_key(),
                    crate::builder::models::QueryOrder::Desc,
                )
                .limit(1)
                .build()?,
        )
        .await?)
    }
}

/// GeekConnection is the trait that all backends must implement to be able
/// to interact with the database.
pub trait GeekConnection {
    /// Single item
    type Row;
    /// Multiple items
    type Rows;
    /// Native Connection
    type Connection;
    /// Native Statement (if any)
    type Statement;

    /// Create a table in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn create_table<T>(connection: &Self::Connection) -> Result<(), crate::Error>
    where
        T: TableBuilder
            + QueryBuilderTrait
            + Sized
            + serde::Serialize
            + serde::de::DeserializeOwned,
    {
        Err(crate::Error::NotImplemented)
    }

    /// Run a SELECT Count query on the database and return the number of rows
    #[allow(async_fn_in_trait, unused_variables)]
    async fn row_count(connection: &Self::Connection, query: Query) -> Result<i64, crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Execute a query on the database and do not return any rows
    #[allow(async_fn_in_trait, unused_variables)]
    async fn execute<T>(connection: &Self::Connection, query: Query) -> Result<(), crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        Err(crate::Error::NotImplemented)
    }

    /// Query the database with an active Connection and Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query<T>(connection: &Self::Connection, query: Query) -> Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        Err(crate::Error::NotImplemented)
    }

    /// Query the database with an active Connection and Query and return the first row.
    ///
    /// Note: Make sure the query is limited to 1 row to avoid retrieving multiple rows
    /// and only using the first one.
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query_first<T>(connection: &Self::Connection, query: Query) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
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
