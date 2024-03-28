use libsql::{de, params::IntoValue};
use serde::{de::DeserializeOwned, Serialize};

use crate::{backends::GeekConnection, TableBuilder, Value};

impl<T> GeekConnection for T
where
    T: TableBuilder + Serialize + DeserializeOwned,
{
    type Connection = libsql::Connection;
    type Rows = Vec<T>;
    type Error = libsql::Error;

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

        let mut statement = connection.prepare(query.to_str()).await?;
        // Convert the values to libsql::Value
        let mut parameters: Vec<libsql::Value> = Vec::new();

        // for (column_name, value) in query.values.values {
        for column_name in &query.values.order {
            let value = query.values.get(&column_name.to_string()).unwrap();
            let column = query.table.columns.get(column_name.as_str()).unwrap();

            // Skip auto increment columns if the query is an insert
            if query.query_type == crate::builder::models::QueryType::Insert
                && column.column_type.is_auto_increment()
            {
                continue;
            }

            parameters.push(value.clone().into_value()?);
        }

        // Execute the query
        let mut rows = statement.query(parameters).await?;
        let mut results = Vec::new();

        while let Some(row) = rows.next().await? {
            results.push(de::from_row::<T>(&row).unwrap());
        }

        Ok(results)
    }
}

impl IntoValue for Value {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        Ok(match self {
            Value::Text(value) => libsql::Value::Text(value),
            Value::Integer(value) => libsql::Value::Integer(value as i64),
            Value::Boolean(value) => libsql::Value::Text(value.to_string()),
            Value::Identifier(value) => libsql::Value::Text(value),
            Value::Null => libsql::Value::Null,
        })
    }
}
