//! # Backend Module for GeekORM
//!
//! **Example:**
//!
//! Here is an example of how to use GeekORM with a mock connection.
//!
//! ```no_run
//! # #[cfg(feature = "backends")] {
//! # use anyhow::Result;
//! use geekorm::prelude::*;
//!
//! # #[derive(Debug, Clone)]
//! # struct Connection;
//! # impl GeekConnection for Connection {
//! #     type Connection = Self;
//! # }
//!
//! #[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
//! pub struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     pub id: PrimaryKey<i32>,
//!     #[geekorm(unique)]
//!     pub username: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a new connection (this is a mock connection)
//!     let connection = Connection {};
//!
//!     Users::create_table(&connection).await?;
//!
//!     let users = vec!["geekmasher", "bob", "alice", "eve", "mallory", "trent"];
//!     for user in users {
//!         let mut new_user = Users::new(user);
//!         new_user.save(&connection).await?;
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
//!     // Fetch first and last user
//!     let first_user = Users::first(&connection).await?;
//!     # assert_eq!(first_user.username, "geekmasher");
//!     let last_user = Users::last(&connection).await?;
//!     # assert_eq!(last_user.username, "trent");
//!
//!
//!     Ok(())
//! }
//! # }
//! ```

use std::collections::HashMap;

use crate::{Query, QueryBuilder, QueryBuilderTrait, TableBuilder, TablePrimaryKey, Value};

#[cfg(feature = "libsql")]
pub mod libsql;
#[cfg(feature = "rusqlite")]
pub mod rusqlite;

/// GeekConnection is the trait used for models to interact with the database.
///
/// This trait is used to define the methods that are used to interact with the database.
pub trait GeekConnector<'a, C>
where
    C: GeekConnection<Connection = C> + 'a,
    Self: Sized + TableBuilder + QueryBuilderTrait + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Query the database with an active Connection and Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query(connection: &'a C, query: Query) -> Result<Vec<Self>, crate::Error> {
        C::query::<Self>(connection, query).await
    }

    /// Query the first row from the database with an active Connection and Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn query_first(connection: &'a C, query: Query) -> Result<Self, crate::Error> {
        C::query_first::<Self>(connection, query).await
    }

    /// Execute a query on the database and do not return any rows
    #[allow(async_fn_in_trait, unused_variables)]
    async fn execute(connection: &'a C, query: Query) -> Result<(), crate::Error> {
        C::execute::<Self>(connection, query).await
    }

    /// Create a table in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn create_table(connection: &'a C) -> Result<(), crate::Error> {
        C::create_table::<Self>(connection).await
    }

    /// Count the number of rows based on a Query
    #[allow(async_fn_in_trait, unused_variables)]
    async fn row_count(connection: &'a C, query: Query) -> Result<i64, crate::Error> {
        C::row_count(connection, query).await
    }

    /// Count the total number of rows in the table
    #[allow(async_fn_in_trait, unused_variables)]
    async fn total(connection: &'a C) -> Result<i64, crate::Error> {
        C::row_count(
            connection,
            Self::query_count().table(Self::table()).build()?,
        )
        .await
    }

    /// Update the current object in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn update(&mut self, connection: &'a C) -> Result<(), crate::Error> {
        C::execute::<Self>(connection, Self::query_update(self)).await
    }

    /// Save the current object to the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn save(&mut self, connection: &'a C) -> Result<(), crate::Error>;

    /// Delete the current object from the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn delete(&self, connection: &'a C) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented)
    }

    /// Fetches all of the foreign key values for the current object
    #[allow(async_fn_in_trait, unused_variables)]
    async fn fetch(&mut self, connection: &'a C) -> Result<(), crate::Error>;

    /// Fetch all rows from the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn fetch_all(connection: &'a C) -> Result<Vec<Self>, crate::Error> {
        C::query::<Self>(
            connection,
            QueryBuilder::select().table(Self::table()).build()?,
        )
        .await
    }

    /// Fetch or create a row in the database
    #[allow(async_fn_in_trait, unused_variables)]
    async fn fetch_or_create(&mut self, connection: &'a C) -> Result<(), crate::Error>;

    /// Search for a row in the database based on specific criteria
    #[cfg(feature = "search")]
    #[allow(async_fn_in_trait, unused_variables)]
    async fn search(
        connection: &'a C,
        search: impl Into<String>,
    ) -> Result<Vec<Self>, crate::Error>;

    /// Fetch the first row from the database (based on the primary key)
    #[allow(async_fn_in_trait, unused_variables)]
    async fn first(connection: &'a C) -> Result<Self, crate::Error>
    where
        Self: TablePrimaryKey,
    {
        C::query_first::<Self>(
            connection,
            Self::query_select()
                .table(Self::table())
                .order_by(
                    &Self::primary_key(),
                    crate::builder::models::QueryOrder::Asc,
                )
                .limit(1)
                .build()?,
        )
        .await
    }

    /// Fetch last row from the database (based on the primary key)
    #[allow(async_fn_in_trait, unused_variables)]
    async fn last(connection: &'a C) -> Result<Self, crate::Error>
    where
        Self: TablePrimaryKey,
    {
        C::query_first::<Self>(
            connection,
            Self::query_select()
                .table(Self::table())
                .order_by(
                    &Self::primary_key(),
                    crate::builder::models::QueryOrder::Desc,
                )
                .limit(1)
                .build()?,
        )
        .await
    }
}

/// GeekConnection is the trait that all backends must implement to be able
/// to interact with the database.
pub trait GeekConnection {
    /// Native Connection
    type Connection;

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
