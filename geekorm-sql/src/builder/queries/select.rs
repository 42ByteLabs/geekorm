//! # Select Query Builder

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_select(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();

        // Resolve the rest of the query, and append if necessary
        if let Some(table) = query.find_table_default() {
            full_query.push_str("SELECT ");

            table.columns.to_sql_stream(&mut full_query, query).unwrap();

            // FROM {table}
            let mut table = TableExpr::new(table.name);
            if let Some(ref alias) = table.alias {
                table.alias(alias.clone());
            }
            table.to_sql_stream(&mut full_query, query).unwrap();

            // JOIN
            query.joins.to_sql_stream(&mut full_query, query).unwrap();

            // WHERE {where_clause}
            query
                .where_clause
                .to_sql_stream(&mut full_query, query)
                .unwrap();

            // ORDER BY {order_by}
            query
                .order_by
                .to_sql_stream(&mut full_query, query)
                .unwrap();

            // LIMIT {limit} OFFSET {offset}
            if let Some(page) = query.page.as_ref() {
                page.to_sql_stream(&mut full_query, query).unwrap();
            }

            // End
            full_query = full_query.trim().to_string();
            full_query.push(';');
        }
        full_query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QueryOrder, QueryType, ToSql, Value, Values,
        backends::{QueryBackend, SqliteBackendOptions},
        builder::{
            columns::{Column, ColumnOptions, Columns},
            columntypes::ColumnType,
            table::Table,
            tests::{table_images, table_users},
        },
    };

    #[test]
    fn sqlite_select_basic() {
        let table = table_images();
        let query = QueryBuilder::select()
            .table(&table)
            .build()
            .expect("Failed to build basic SQL query");

        assert_eq!(query.query, "SELECT id, title, url FROM Images;");
    }

    #[test]
    fn sqlite_select_where() {
        let table = table_images();
        let query = QueryBuilder::select()
            .table(&table)
            .where_eq("title", "test")
            .build()
            .unwrap();

        assert_eq!(query.values.len(), 1);
        assert_eq!(
            query.query,
            "SELECT id, title, url FROM Images WHERE title = ?;"
        );

        let mut values = Values::new();
        values.push("title".to_string(), Value::from("test"));
        assert_eq!(query.values, values);
    }

    #[test]
    fn sqlite_order_clause() {
        let table = table_images();
        let query = QueryBuilder::select()
            .backend(QueryBackend::Sqlite {
                options: SqliteBackendOptions::default(),
            })
            .table(&table)
            .order_by("title", QueryOrder::Asc)
            .order_by("url", QueryOrder::Desc)
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "SELECT id, title, url FROM Images ORDER BY title ASC, url DESC;"
        );
    }

    #[test]
    fn sqlite_inner_join() {
        let table = table_users();
        let image_table = table_images();

        let query = QueryBuilder::select()
            .table(&table)
            .join(&image_table)
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "SELECT Users.id, Users.username, Users.email, Users.roles, Users.profile, Images.id, Images.title, Images.url FROM Users JOIN Images ON Users.profile = Images.id;"
        );
    }
}
