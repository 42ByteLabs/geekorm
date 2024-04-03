/// This module contains the implementation for the `GeekConnection` trait for the `libsql` crate.
use std::collections::HashMap;

use libsql::{de, params::IntoValue};
use log::{debug, error};
use serde::{de::DeserializeOwned, Serialize};

use crate::{backends::GeekConnection, builder::models::QueryType, TableBuilder, Value, Values};

impl<T> GeekConnection for T
where
    T: TableBuilder + Serialize + DeserializeOwned,
{
    type Connection = libsql::Connection;
    type Row = T;
    type Rows = Vec<T>;
    type Error = libsql::Error;

    async fn create_table(connection: &Self::Connection) -> Result<(), Self::Error> {
        let query = T::create().build().unwrap();
        connection.execute(query.to_str(), ()).await?;
        Ok(())
    }

    async fn row_count(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<i64, Self::Error> {
        let mut statement = connection.prepare(query.to_str()).await?;
        let mut rows = statement.query(()).await?;

        let row = rows.next().await?.unwrap();
        Ok(row.get(0).unwrap())
    }

    async fn query(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Self::Rows, Self::Error> {
        // TODO(geekmasher): Use different patterns for different query types

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                error!("Error preparing query: `{}`", query.to_str());
                return Err(e);
            }
        };
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = convert_values(&query).unwrap();

        debug!("Query :: {:?}", query.to_str());
        debug!("Parameters :: {:?}", parameters.clone());

        // Execute the query
        let mut rows = match statement.query(parameters).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("Error executing query: `{}`", query.to_str());
                return Err(e);
            }
        };
        let mut results = Vec::new();

        while let Some(row) = rows.next().await? {
            results.push(de::from_row::<T>(&row).unwrap());
        }

        Ok(results)
    }

    async fn query_first(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Self::Row, Self::Error> {
        if query.query_type == QueryType::Update {
            error!("Query type is an `update`, use execute() instead as it does not return a row");
        }

        let rows = Self::query(connection, query).await?;
        match rows.into_iter().next() {
            Some(row) => Ok(row),
            None => Err(libsql::Error::NullValue),
        }
    }

    async fn execute(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), Self::Error> {
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = convert_values(&query).unwrap();
        connection.execute(query.to_str(), parameters).await?;
        Ok(())
    }

    async fn query_raw(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<HashMap<String, Value>>, Self::Error> {
        let params = convert_values(&query).unwrap();

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                error!("Error preparing query: `{}`", query.to_str());
                return Err(e);
            }
        };

        debug!("Query :: {:?}", query.to_str());
        debug!("Parameters :: {:?}", params);

        let mut rows = statement.query(params).await?;
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
    let values: Values = match query.query_type {
        QueryType::Insert | QueryType::Update => query.parameters.clone(),
        _ => query.values.clone(),
    };

    // for (column_name, value) in query.values.values {
    for column_name in &values.order {
        let value = values.get(&column_name.to_string()).unwrap();
        let column = query.table.columns.get(column_name.as_str()).unwrap();

        // Skip auto increment columns if the query is an insert
        if query.query_type == QueryType::Insert && column.column_type.is_auto_increment() {
            continue;
        } else if query.query_type == QueryType::Update && column.column_type.is_primary_key() {
            continue;
        }

        parameters.push(value.clone().into_value().unwrap());
    }
    Ok(parameters)
}

impl IntoValue for Value {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        Ok(match self {
            Value::Text(value) => libsql::Value::Text(value),
            Value::Integer(value) => libsql::Value::Integer(value as i64),
            Value::Boolean(value) => libsql::Value::Text(value.to_string()),
            Value::Identifier(value) => libsql::Value::Text(value),
            Value::Blob(value) => libsql::Value::Blob(value),
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
            libsql::Value::Blob(value) => Value::Blob(value),
            libsql::Value::Real(_) => {
                todo!("Real values are not supported yet")
            }
        }
    }
}
