//! # RuSQLite Backend
//!
//! **Example:**
//!
//! ```rust
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
//!     let connection = rusqlite::Connection::open_in_memory()?;
//!
//!     Users::create_table(&connection).await?;
//!
//!     let user = Users::new("geekmasher");
//!     user.save(&connection).await?;
//!
//!     Ok(())
//! }
//! # }
//! ```

use log::debug;
use rusqlite::ToSql;
use serde_rusqlite::*;

use super::GeekConnection;

impl GeekConnection for rusqlite::Connection {
    type Connection = rusqlite::Connection;
    type Row = rusqlite::Row<'static>;
    type Rows = rusqlite::Rows<'static>;
    type Statement = rusqlite::Statement<'static>;

    async fn create_table<T>(connection: &Self::Connection) -> std::result::Result<(), crate::Error>
    where
        T: crate::TableBuilder
            + crate::QueryBuilderTrait
            + Sized
            + serde::Serialize
            + serde::de::DeserializeOwned,
    {
        let query = T::query_create().build()?;
        debug!("Create Table Query :: {:?}", query.to_str());
        connection
            .execute(query.to_str(), ())
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;
        Ok(())
    }

    async fn query<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> std::result::Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut statement = connection
            .prepare(query.to_str())
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;

        let params = rusqlite::params_from_iter(query.parameters.into_iter());

        let mut results = Vec::new();

        let mut res = from_rows::<T>(
            statement
                .query(params)
                .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?,
        );
        while let Some(Ok(row)) = res.next() {
            results.push(row);
        }

        Ok(results)
    }

    async fn query_first<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> std::result::Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        debug!("Query First :: {:?}", query.to_str());
        let mut statement = connection
            .prepare(query.to_str())
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;

        let params = rusqlite::params_from_iter(query.values.into_iter());
        debug!("Query First Params :: {:?}", params);

        let mut res = from_rows::<T>(
            statement
                .query(params)
                .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?,
        );

        match res.next() {
            Some(Ok(row)) => Ok(row),
            _ => Err(crate::Error::RuSQLiteError("No rows found".to_string())),
        }
    }

    async fn execute<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> std::result::Result<(), crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        debug!("Execute :: {:?}", query.to_str());
        let mut statement = connection
            .prepare(query.to_str())
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;

        let params = rusqlite::params_from_iter(query.parameters.into_iter());
        debug!("Execute Params :: {:?}", params);

        statement
            .execute(params)
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;
        Ok(())
    }

    async fn row_count(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> std::result::Result<i64, crate::Error> {
        let mut statement = connection
            .prepare(query.to_str())
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;
        let params = rusqlite::params_from_iter(query.parameters.into_iter());
        let mut res = statement
            .query(params)
            .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?;

        match res.next() {
            Ok(Some(row)) => Ok(row
                .get(0)
                .map_err(|e| crate::Error::RuSQLiteError(e.to_string()))?),
            _ => Err(crate::Error::RuSQLiteError("No rows found".to_string())),
        }
    }
}

impl ToSql for crate::Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            crate::Value::Identifier(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(value.clone()),
            )),
            crate::Value::Text(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(value.clone()),
            )),
            crate::Value::Integer(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*value as i64),
            )),
            crate::Value::Blob(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Blob(value.clone()),
            )),
            crate::Value::Boolean(value) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*value as i64),
            )),
            crate::Value::Null => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Null,
            )),
        }
    }
}