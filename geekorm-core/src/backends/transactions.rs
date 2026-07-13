//! # Transaction Connector

use geekorm_sql::{Query, query::BatchQueries};
use std::sync::{Arc, Mutex};

use crate::GeekConnection;

/// SQLite Transaction Connector
#[derive(Debug, Clone, Default)]
pub struct TransactionConnector {
    pub(crate) queries: Arc<Mutex<BatchQueries>>,
}

impl TransactionConnector {
    /// New instance of the TransactionConnector
    pub fn new() -> Self {
        Self::default()
    }
    /// Push a query to the transaction connector
    pub fn push(&self, query: Query) {
        self.queries.lock().unwrap().push(query);
    }
}

impl GeekConnection for TransactionConnector {
    type Connection = TransactionConnector;

    async fn create_table<T>(connection: &Self::Connection) -> std::result::Result<(), crate::Error>
    where
        T: crate::TableBuilder
            + crate::QueryBuilderTrait
            + Sized
            + serde::Serialize
            + serde::de::DeserializeOwned,
    {
        println!("Beans create");
        Ok(())
    }

    async fn execute(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error> {
        println!("Beans exec");
        Ok(())
    }

    async fn query<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        println!("Beans query");
        Ok(Vec::new())
    }

    async fn query_first<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        println!("Beans first");
        Err(crate::Error::Unknown)
    }

    async fn batch(connection: &Self::Connection, query: crate::Query) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn row_count(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<i64, crate::Error> {
        Ok(0)
    }

    fn is_transaction(connection: &Self::Connection) -> bool {
        println!("TRANSACTION CONNECTOR");
        true
    }
}
