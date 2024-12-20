//! # libsql
//!
//! This module contains the implementation for the `GeekConnection` trait for the `libsql` crate.
//!
//! ## LibSQL with timeout and retry
//!
//! Wrapper for a `libsql::Connection` that implements the `GeekConnection` trait.
//!
//! ```no_run
//! # #[cfg(feature = "libsql")] {
//! use std::sync::{Arc, RwLock};
//! use geekorm::prelude::*;
//!
//! #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
//! struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     id: PrimaryKey<i32>,
//!     #[geekorm(unique)]
//!     username: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let database = libsql::Builder::new_local(":memory:").build().await?;
//!     let connection = Arc::new(RwLock::new(database.connect().unwrap()));
//!
//!     Users::create_table(&connection).await?;
//!
//!     let users = vec!["geekmasher", "bob", "alice", "eve", "mallory", "trent"];
//!     for user in users {
//!         let mut new_user = Users::new(user);
//!         new_user.save(&connection).await?;
//!     }
//!     Ok(())
//! }
//! # }
//! ```
use libsql::{de, params::IntoValue};
#[cfg(feature = "log")]
use log::{debug, error};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    builder::models::QueryType, GeekConnection, QueryBuilderTrait, TableBuilder, Value, Values,
};

impl GeekConnection for libsql::Connection {
    type Connection = libsql::Connection;

    async fn create_table<T>(connection: &Self::Connection) -> Result<(), crate::Error>
    where
        T: TableBuilder + QueryBuilderTrait + Sized + Serialize + DeserializeOwned,
    {
        let query = T::query_create().build()?;
        #[cfg(feature = "log")]
        {
            debug!("Create Table Query :: {:?}", query.to_str());
        }
        connection
            .execute(query.to_str(), ())
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?;
        Ok(())
    }

    async fn row_count(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<i64, crate::Error> {
        #[cfg(feature = "log")]
        {
            debug!("Row Count Query :: {:?}", query.to_str());
        }
        let mut statement = connection
            .prepare(query.to_str())
            .await
            .map_err(|e| crate::Error::LibSQLError(format!("Error preparing query: `{}`", e)))?;

        let parameters: Vec<libsql::Value> = match convert_values(&query) {
            Ok(parameters) => parameters,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    debug!("Error converting values: `{}`", e);
                    debug!("Parameters :: {:?}", query.parameters);
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        let mut rows = match statement.query(parameters).await {
            Ok(rows) => rows,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    debug!("Error executing query: `{}`", query.to_str());
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        let row = match rows
            .next()
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?
        {
            Some(row) => row,
            None => {
                #[cfg(feature = "log")]
                {
                    error!("Error fetching row count");
                }
                return Err(crate::Error::LibSQLError(
                    "Error fetching row count".to_string(),
                ));
            }
        };
        // Get the first row
        Ok(row
            .get(0)
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?)
    }

    async fn query<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        #[cfg(feature = "log")]
        {
            debug!("Query :: {:?}", query.to_str());
        }

        // TODO(geekmasher): Use different patterns for different query types

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    debug!("Error preparing query: `{}`", query.to_str());
                    debug!("Parameters :: {:?}", query.parameters);
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = match convert_values(&query) {
            Ok(parameters) => parameters,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    debug!("Error converting values: `{}`", e);
                    debug!("Parameters :: {:?}", query.parameters);
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        #[cfg(feature = "log")]
        {
            debug!("Parameters :: {:?}", parameters.clone());
        }

        // Execute the query
        let mut rows = match statement.query(parameters).await {
            Ok(rows) => rows,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    debug!("Error executing query: `{}`", query.to_str());
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };
        let mut results = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?
        {
            results.push(de::from_row::<T>(&row).map_err(|e| {
                #[cfg(feature = "log")]
                {
                    error!("Error deserializing row: `{}`", e);
                }
                crate::Error::SerdeError(e.to_string())
            })?);
        }

        Ok(results)
    }

    async fn query_first<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        // TODO: Should we always make sure the query limit is set to 1?
        if query.query_type == QueryType::Update {
            #[cfg(feature = "log")]
            {
                error!(
                    "Query type is an `update`, use execute() instead as it does not return a row"
                );
            }
            return Err(crate::Error::LibSQLError(
                "Query type is an `update`".to_string(),
            ));
        }

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    debug!("Error preparing query: `{}`", query.to_str());
                    debug!("Parameters :: {:?}", query.parameters);
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = match convert_values(&query) {
            Ok(parameters) => parameters,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    error!("Error converting values: `{}`", e);
                    error!("Parameters :: {:?}", query.parameters);
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        #[cfg(feature = "log")]
        {
            debug!("Query :: {:?}", query.to_str());
            debug!("Parameters :: {:?}", parameters.clone());
        }

        // Execute the query
        let mut rows = match statement.query(parameters).await {
            Ok(rows) => rows,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    error!("Error executing query: `{}`", query.to_str());
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        let row: libsql::Row = match rows
            .next()
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?
        {
            Some(row) => row,
            None => {
                #[cfg(feature = "log")]
                {
                    error!("No rows found for query: `{}`", query.to_str());
                }
                return Err(crate::Error::NoRowsFound);
            }
        };

        Ok(de::from_row::<T>(&row).map_err(|e| {
            #[cfg(feature = "log")]
            {
                error!("Error deserializing row: `{}`", e);
            }
            crate::Error::SerdeError(e.to_string())
        })?)
    }

    async fn execute<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = convert_values(&query).map_err(|e| {
            #[cfg(feature = "log")]
            {
                error!("Error converting values: `{}`", e);
            }
            crate::Error::LibSQLError(e.to_string())
        })?;
        connection
            .execute(query.to_str(), parameters)
            .await
            .map_err(|e| {
                #[cfg(feature = "log")]
                {
                    error!("Error executing query: `{}`", e);
                }
                crate::Error::LibSQLError(e.to_string())
            })?;
        Ok(())
    }

    async fn query_raw(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<HashMap<String, Value>>, crate::Error> {
        let params = convert_values(&query).map_err(|e| {
            #[cfg(feature = "log")]
            {
                error!("Error converting values: `{}`", e);
            }
            crate::Error::LibSQLError(e.to_string())
        })?;

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                #[cfg(feature = "log")]
                {
                    error!("Error preparing query: `{}`", query.to_str());
                }
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        #[cfg(feature = "log")]
        {
            debug!("Query :: {:?}", query.to_str());
            debug!("Parameters :: {:?}", params);
        }

        let mut rows = statement.query(params).await.map_err(|e| {
            #[cfg(feature = "log")]
            {
                error!("Error executing query: `{}`", query.to_str());
            }
            crate::Error::LibSQLError(e.to_string())
        })?;
        let mut results: Vec<HashMap<String, Value>> = Vec::new();

        while let Some(row) = rows.next().await.map_err(|e| {
            #[cfg(feature = "log")]
            {
                error!("Error fetching row: `{}`", e);
            }
            crate::Error::LibSQLError(e.to_string())
        })? {
            let mut values: HashMap<String, Value> = HashMap::new();

            for (index, column_name) in query.columns.iter().enumerate() {
                let value = row.get_value(index as i32).unwrap();
                values.insert(column_name.to_string(), value.into());
            }
            results.push(values);
        }

        Ok(results)
    }
}

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

impl<C> GeekConnection for Arc<RwLock<C>>
where
    Self: std::marker::Sync + std::marker::Send,
    C: GeekConnection<Connection = libsql::Connection>,
{
    type Connection = Arc<RwLock<libsql::Connection>>;

    async fn create_table<T>(connection: &Self::Connection) -> Result<(), crate::Error>
    where
        T: TableBuilder + QueryBuilderTrait + Sized + Serialize + DeserializeOwned,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < TIMEOUT {
            match connection.try_write() {
                Ok(conn) => return C::create_table::<T>(&conn).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
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
            match connection.try_write() {
                Ok(conn) => return C::row_count(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
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
            match connection.try_write() {
                Ok(conn) => return C::query::<T>(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
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
            match connection.try_write() {
                Ok(conn) => return C::query_first::<T>(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
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
            match connection.try_write() {
                Ok(conn) => return C::execute::<T>(&conn, query).await,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        Err(crate::Error::LibSQLError(
            "Error getting write lock on connection in execute".to_string(),
        ))
    }
}

fn convert_values(query: &crate::Query) -> Result<Vec<libsql::Value>, crate::Error> {
    let mut parameters: Vec<libsql::Value> = Vec::new();

    // TODO(geekmasher): This is awful, need to refactor this
    let values: Values = match query.query_type {
        QueryType::Insert | QueryType::Update => query.parameters.clone(),
        _ => query.values.clone(),
    };

    for column_name in &values.order {
        let value = values
            .get(&column_name.to_string())
            .ok_or(crate::Error::LibSQLError(format!(
                "Error getting value for column - {}",
                column_name
            )))?;

        // Check if the column exists in the table
        // The column_name could be in another table not part of the query (joins)
        if let Some(column) = query.table.columns.get(column_name.as_str()) {
            // Skip auto increment columns if the query is an insert
            if query.query_type == QueryType::Insert && column.column_type.is_auto_increment() {
                continue;
            } else if query.query_type == QueryType::Update && column.column_type.is_primary_key() {
                continue;
            }
        }

        #[cfg(feature = "log")]
        {
            log::debug!("LIBSQL - Column('{}', '{}')", column_name, value);
        }

        parameters.push(
            value.clone().into_value().map_err(|e| {
                crate::Error::LibSQLError(format!("Error converting value - {}", e))
            })?,
        );
    }
    Ok(parameters)
}

impl IntoValue for Value {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        Ok(match self {
            Value::Text(value) => libsql::Value::Text(value),
            Value::Integer(value) => libsql::Value::Integer(value as i64),
            Value::Boolean(value) => libsql::Value::Text(value.to_string()),
            // TODO: Identifier could be a Integer?
            Value::Identifier(value) => libsql::Value::Text(value),
            Value::Blob(value) | Value::Json(value) => libsql::Value::Blob(value),
            Value::Null => libsql::Value::Null,
        })
    }
}

impl From<libsql::Value> for Value {
    fn from(value: libsql::Value) -> Self {
        match value {
            libsql::Value::Text(value) => Value::Text(value),
            libsql::Value::Integer(value) => Value::Integer(value as i32),
            libsql::Value::Null => Value::Null,
            libsql::Value::Blob(value) => {
                // TODO: Is this the best way of doing this?
                if let Some(start) = value.get(0) {
                    if *start == b'{' || *start == b'[' {
                        return Value::Json(value);
                    }
                }
                Value::Blob(value)
            }
            libsql::Value::Real(_) => {
                todo!("Real values are not supported yet")
            }
        }
    }
}
