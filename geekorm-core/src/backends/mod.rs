use crate::Query;

/// This module contains the LibSQL backend
#[cfg(feature = "libsql")]
pub mod libsql;

/// This trait is used to define the connection to the database.
pub trait GeekConnection {
    /// The connection type
    type Connection;
    /// The rows to return type
    type Rows;
    /// The error type
    type Error;

    /// Query the database with an active Connection and Query
    #[allow(async_fn_in_trait)]
    async fn query(connection: &Self::Connection, query: Query) -> Result<Self::Rows, Self::Error>;
}
