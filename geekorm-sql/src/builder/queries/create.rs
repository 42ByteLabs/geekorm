//! # Create Query Builder

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};
use geekorm_core::Error;

impl QueryType {
    pub(crate) fn sql_create(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();
        if let Some(table) = query.table {
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
    use geekorm_core::{Column, ColumnType, ColumnTypeOptions, Table};

    use super::*;
    use crate::QueryType;
    use crate::builder::QueryBuilder;

    fn table() -> Table {
        Table {
            name: "Test".to_string(),
            database: None,
            columns: vec![
                Column::new(
                    "id".to_string(),
                    ColumnType::Identifier(ColumnTypeOptions {
                        primary_key: true,
                        foreign_key: String::new(),
                        unique: true,
                        not_null: true,
                        auto_increment: true,
                    }),
                ),
                Column::new(
                    "name".to_string(),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
                Column::new(
                    "email".to_string(),
                    ColumnType::Text(ColumnTypeOptions {
                        unique: true,
                        ..Default::default()
                    }),
                ),
            ]
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
}
