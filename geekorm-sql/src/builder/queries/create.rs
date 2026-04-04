//! # Create Query Builder

use crate::builder::table::TableExpr;
use crate::{Error, QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_create(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();
        if let Some(table) = query.find_table_default() {
            full_query.push_str("CREATE TABLE IF NOT EXISTS ");
            full_query.push_str(&table.name);

            full_query.push_str(" (");

            table.columns.to_sql_stream(&mut full_query, query).unwrap();
            full_query.push(')');
        }
        full_query.push(';');

        full_query
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::builder::{
        QueryBuilder,
        columns::{Column, ColumnOptions, Columns},
        columntypes::ColumnType,
    };
    use crate::{QueryBackend, QueryType, Table};

    fn table() -> Table {
        Table {
            name: "Test",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from((
                    "email".to_string(),
                    ColumnType::Text,
                    ColumnOptions {
                        unique: true,
                        ..Default::default()
                    },
                )),
            ])
            .into(),
        }
    }

    #[test]
    fn test_create_query() {
        let table = table();
        let mut query = QueryBuilder::create();
        query.table(&table);

        let sql = query.query_type.sql_create(&query);

        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS Test (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT UNIQUE);"
        );
    }

    #[test]
    fn test_create_query_postgres() {
        let table = table();
        let mut query = QueryBuilder::create();
        query.backend(QueryBackend::Postgres);
        query.table(&table);

        let sql = query.query_type.sql_create(&query);

        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS Test (id SERIAL PRIMARY KEY, name TEXT, email TEXT UNIQUE);"
        );
    }

    #[test]
    fn test_create_query_postgres_with_boolean() {
        let table = Table {
            name: "Users",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("username".to_string(), ColumnType::Text)),
                Column::from(("active".to_string(), ColumnType::Boolean)),
            ])
            .into(),
        };

        let mut query = QueryBuilder::create();
        query.backend(QueryBackend::Postgres);
        query.table(&table);

        let sql = query.query_type.sql_create(&query);

        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS Users (id SERIAL PRIMARY KEY, username TEXT, active BOOLEAN);"
        );
    }

    #[test]
    fn test_create_query_postgres_with_blob() {
        let table = Table {
            name: "Files",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("filename".to_string(), ColumnType::Text)),
                Column::from(("data".to_string(), ColumnType::Blob)),
            ])
            .into(),
        };

        let mut query = QueryBuilder::create();
        query.backend(QueryBackend::Postgres);
        query.table(&table);

        let sql = query.query_type.sql_create(&query);

        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS Files (id SERIAL PRIMARY KEY, filename TEXT, data BYTEA);"
        );
    }

    #[test]
    fn test_create_query_postgres_with_bigint() {
        let table = Table {
            name: "BigData",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::BigInt,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from(("count".to_string(), ColumnType::BigInt)),
            ])
            .into(),
        };

        let mut query = QueryBuilder::create();
        query.backend(QueryBackend::Postgres);
        query.table(&table);

        let sql = query.query_type.sql_create(&query);

        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS BigData (id BIGSERIAL PRIMARY KEY, name TEXT, count BIGINT);"
        );
    }
}
