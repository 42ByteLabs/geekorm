//! # Update Query Builder

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_update(&self, query: &QueryBuilder) -> String {
        let mut full_query = "UPDATE ".to_string();

        if let Some(table) = query.find_table_default() {
            let table_expr = TableExpr::new(&table.name);
            full_query.push_str(&table_expr.sql());

            // WHERE
            if !query.where_clause.is_empty() {
                full_query.push(' ');
                query
                    .where_clause
                    .to_sql_stream(&mut full_query, query)
                    .unwrap();
            }
        }

        full_query.push(';');
        full_query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QueryType, ToSql, builder::table::Table};
    use crate::{
        backends::QueryBackend,
        builder::{
            QueryBuilder,
            columns::{Column, ColumnOptions, Columns},
            columntypes::ColumnType,
        },
    };

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
                Column::from(("email".to_string(), ColumnType::Text)),
            ])
            .into(),
        }
    }

    #[test]
    fn test_update_query() {
        let table = table();
        let query = QueryBuilder::update()
            .backend(QueryBackend::Sqlite)
            .table(&table)
            .where_primary_key("1")
            .build()
            .unwrap();

        assert_eq!(query.query, "UPDATE Test WHERE id = ?;");
    }

    #[test]
    fn test_update_query_postgres() {
        let table = table();
        let query = QueryBuilder::update()
            .backend(QueryBackend::Postgres)
            .table(&table)
            .where_primary_key("1")
            .build()
            .unwrap();

        assert_eq!(query.query, "UPDATE Test WHERE id = $1;");
    }

    #[test]
    fn test_update_query_postgres_multiple_conditions() {
        let table = table();
        let query = QueryBuilder::update()
            .backend(QueryBackend::Postgres)
            .table(&table)
            .where_eq("id", 5)
            .and()
            .where_eq("name", "test")
            .build()
            .unwrap();

        assert_eq!(query.query, "UPDATE Test WHERE id = $1 AND name = $2;");
    }
}
