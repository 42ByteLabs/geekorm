//! # libsql
//!
//! This module contains the implementation for the `GeekConnection` trait for the `libsql` crate.
//!
//! ## LibSQL with timeout and retry
//!
//! Wrapper for a `libsql::Connection` that implements the `GeekConnection` trait.
//!
//! ```no_run
//! # #[cfg(all(feature = "libsql", feature = "backends-tokio"))] {
//! use std::sync::Arc;
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
//!     let connection = Arc::new(tokio::sync::Mutex::new(database.connect().unwrap()));
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
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

use crate::{
    GeekConnection, QueryBuilderTrait, TableBuilder, Value, Values, builder::models::QueryType,
};

#[cfg(feature = "backends-tokio")]
mod mutex;

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
        connection.execute(query.to_str(), ()).await.map_err(|e| {
            crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            }
        })?;
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
        let mut statement = connection.prepare(query.to_str()).await.map_err(|e| {
            crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            }
        })?;

        let parameters: Vec<libsql::Value> = convert_values(&query)?;

        let mut rows =
            statement
                .query(parameters)
                .await
                .map_err(|e| crate::Error::LibSQLError {
                    error: e.to_string(),
                    query: query.to_string(),
                })?;

        let row = match rows.next().await.map_err(|e| crate::Error::LibSQLError {
            error: e.to_string(),
            query: query.to_string(),
        })? {
            Some(row) => row,
            None => {
                #[cfg(feature = "log")]
                {
                    error!("Error fetching row count");
                }
                return Err(crate::Error::LibSQLError {
                    error: "Error fetching row count".to_string(),
                    query: query.to_string(),
                });
            }
        };
        // Get the first row
        Ok(row.get(0).map_err(|e| crate::Error::LibSQLError {
            error: e.to_string(),
            query: query.to_string(),
        })?)
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

        let mut statement = connection.prepare(query.to_str()).await.map_err(|e| {
            crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            }
        })?;

        let parameters: Vec<libsql::Value> = convert_values(&query)?;

        #[cfg(feature = "log")]
        {
            debug!("Parameters :: {:?}", parameters.clone());
        }

        // Execute the query
        let mut rows =
            statement
                .query(parameters)
                .await
                .map_err(|e| crate::Error::LibSQLError {
                    error: e.to_string(),
                    query: query.to_string(),
                })?;

        let mut results = Vec::new();

        while let Some(row) = rows.next().await.map_err(|e| crate::Error::LibSQLError {
            error: e.to_string(),
            query: query.to_string(),
        })? {
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
            return Err(crate::Error::LibSQLError {
                error: "Query type is an `update`".to_string(),
                query: query.to_string(),
            });
        }

        let mut statement = connection.prepare(query.to_str()).await.map_err(|e| {
            crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            }
        })?;

        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = convert_values(&query)?;

        #[cfg(feature = "log")]
        {
            debug!("Query :: {:?}", query.to_str());
            debug!("Parameters :: {:?}", parameters.clone());
        }

        // Execute the query
        let mut rows =
            statement
                .query(parameters)
                .await
                .map_err(|e| crate::Error::LibSQLError {
                    error: e.to_string(),
                    query: query.to_string(),
                })?;

        let row: libsql::Row = match rows.next().await? {
            Some(row) => row,
            None => {
                #[cfg(feature = "log")]
                {
                    error!("No rows found for query: `{}`", query.to_str());
                }
                return Err(crate::Error::NoRowsFound {
                    query: query.to_string(),
                });
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

    async fn execute(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error> {
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = convert_values(&query)?;

        connection
            .execute(query.to_str(), parameters)
            .await
            .map_err(|e| crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            })?;
        Ok(())
    }

    async fn batch(connection: &Self::Connection, query: crate::Query) -> Result<(), crate::Error> {
        connection
            .execute_batch(query.to_str())
            .await
            .map_err(|e| crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            })?;
        Ok(())
    }

    async fn query_raw(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<HashMap<String, Value>>, crate::Error> {
        let params = convert_values(&query)?;

        let mut statement = connection.prepare(query.to_str()).await.map_err(|e| {
            crate::Error::QuerySyntaxError {
                error: e.to_string(),
                query: query.to_string(),
            }
        })?;

        #[cfg(feature = "log")]
        {
            debug!("Query :: {:?}", query.to_str());
            debug!("Parameters :: {:?}", params);
        }

        let mut rows = statement
            .query(params)
            .await
            .map_err(|e| crate::Error::LibSQLError {
                error: e.to_string(),
                query: query.to_string(),
            })?;

        let mut results: Vec<HashMap<String, Value>> = Vec::new();

        while let Some(row) = rows.next().await? {
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

fn convert_values(query: &crate::Query) -> Result<Vec<libsql::Value>, crate::Error> {
    let mut parameters: Vec<libsql::Value> = Vec::new();

    // TODO(geekmasher): This is awful, need to refactor this
    let values: &Values = match query.query_type {
        QueryType::Insert | QueryType::Update => &query.parameters,
        _ => &query.values,
    };

    for (column_name, value) in &values.values {
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
            log::trace!("LIBSQL - Column('{}', '{}')", column_name, value);
        }

        parameters.push(
            value
                .clone()
                .into_value()
                .map_err(|e| crate::Error::LibSQLError {
                    error: format!("Error converting value - {}", e),
                    query: query.to_string(),
                })?,
        );
    }
    Ok(parameters)
}

/// Convert LibSQL Error to GeekORM Error
impl From<libsql::Error> for crate::Error {
    fn from(value: libsql::Error) -> Self {
        #[cfg(feature = "log")]
        {
            log::error!("LibSQL Error: `{}`", value);
        }

        crate::Error::LibSQLError {
            error: value.to_string(),
            query: String::new(),
        }
    }
}

impl From<(libsql::Error, String)> for crate::Error {
    fn from(value: (libsql::Error, String)) -> Self {
        #[cfg(feature = "log")]
        {
            log::error!("LibSQL Error: `{}`", value.0);
            log::error!("LibSQL Error Query: `{}`", value.1);
        }

        crate::Error::LibSQLError {
            error: value.0.to_string(),
            query: value.1,
        }
    }
}

impl IntoValue for Value {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        Ok(match self {
            Value::Text(value) => libsql::Value::Text(value),
            Value::Integer(value) => libsql::Value::Integer(value),
            Value::Boolean(value) => libsql::Value::Text(value.to_string()),
            // TODO: Identifier could be a Integer?
            Value::Identifier(value) => libsql::Value::Integer(value as i64),
            Value::Blob(value) | Value::Json(value) => libsql::Value::Blob(value),
            Value::Null => libsql::Value::Null,
        })
    }
}

impl From<libsql::Value> for Value {
    fn from(value: libsql::Value) -> Self {
        match value {
            libsql::Value::Text(value) => Value::Text(value),
            libsql::Value::Integer(value) => Value::Integer(value),
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
