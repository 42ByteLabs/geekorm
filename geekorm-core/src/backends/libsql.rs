/// This module contains the implementation for the `GeekConnection` trait for the `libsql` crate.
use std::collections::HashMap;

use libsql::{de, params::IntoValue};
use log::{debug, error};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    builder::models::QueryType, GeekConnection, GeekConnector, QueryBuilderTrait, TableBuilder,
    Value, Values,
};

impl GeekConnection for libsql::Connection {
    type Connection = libsql::Connection;

    fn connect() -> Self::Connection {
        libsql::Connection::connect()
    }
}

impl<T> GeekConnector for T
where
    T: TableBuilder + QueryBuilderTrait + Serialize + DeserializeOwned,
{
    type Connection = libsql::Connection;
    type Row = T;
    type Rows = Vec<T>;

    async fn create_table(connection: &Self::Connection) -> Result<(), crate::Error> {
        let query = T::create().build()?;
        debug!("Create Table Query :: {:?}", query.to_str());
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
        debug!("Row Count Query :: {:?}", query.to_str());
        let mut statement = connection
            .prepare(query.to_str())
            .await
            .map_err(|e| crate::Error::LibSQLError(format!("Error preparing query: `{}`", e)))?;
        let mut rows = statement
            .query(())
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?;

        let row = rows
            .next()
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?
            .unwrap();
        // Get the first row
        Ok(row.get(0).unwrap())
    }

    async fn query(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Self::Rows, crate::Error> {
        // TODO(geekmasher): Use different patterns for different query types

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                error!("Error preparing query: `{}`", query.to_str());
                error!("Parameters :: {:?}", query.parameters);
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = match convert_values(&query) {
            Ok(parameters) => parameters,
            Err(e) => {
                error!("Error converting values: `{}`", e);
                error!("Parameters :: {:?}", query.parameters);
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        debug!("Query :: {:?}", query.to_str());
        debug!("Parameters :: {:?}", parameters.clone());

        // Execute the query
        let mut rows = match statement.query(parameters).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("Error executing query: `{}`", query.to_str());
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };
        let mut results = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| crate::Error::LibSQLError(e.to_string()))?
        {
            results.push(de::from_row::<T>(&row).unwrap());
        }

        Ok(results)
    }

    async fn query_first(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Self::Row, crate::Error> {
        if query.query_type == QueryType::Update {
            error!("Query type is an `update`, use execute() instead as it does not return a row");
            return Err(crate::Error::LibSQLError(
                "Query type is an `update`".to_string(),
            ));
        }

        let rows = Self::query(connection, query).await?;
        match rows.into_iter().next() {
            Some(row) => Ok(row),
            None => Err(crate::Error::NoRowsFound),
        }
    }

    async fn execute(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error> {
        // Convert the values to libsql::Value
        let parameters: Vec<libsql::Value> = convert_values(&query).map_err(|e| {
            error!("Error converting values: `{}`", e);
            crate::Error::LibSQLError(e.to_string())
        })?;
        connection
            .execute(query.to_str(), parameters)
            .await
            .map_err(|e| {
                error!("Error executing query: `{}`", e);
                crate::Error::LibSQLError(e.to_string())
            })?;
        Ok(())
    }

    async fn query_raw(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<HashMap<String, Value>>, crate::Error> {
        let params = convert_values(&query).map_err(|e| {
            error!("Error converting values: `{}`", e);
            crate::Error::LibSQLError(e.to_string())
        })?;

        let mut statement = match connection.prepare(query.to_str()).await {
            Ok(statement) => statement,
            Err(e) => {
                error!("Error preparing query: `{}`", query.to_str());
                return Err(crate::Error::LibSQLError(e.to_string()));
            }
        };

        debug!("Query :: {:?}", query.to_str());
        debug!("Parameters :: {:?}", params);

        let mut rows = statement.query(params).await.map_err(|e| {
            error!("Error executing query: `{}`", query.to_str());
            crate::Error::LibSQLError(e.to_string())
        })?;
        let mut results: Vec<HashMap<String, Value>> = Vec::new();

        while let Some(row) = rows.next().await.map_err(|e| {
            error!("Error fetching row: `{}`", e);
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
