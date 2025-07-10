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

            // // JOIN
            // if !qb.joins.is_empty() {
            //     full_query.push(' ');
            //     full_query.push_str(qb.joins.on_select(qb)?.as_str());
            // }

            // WHERE {where_clause} ORDER BY {order_by}
            if !query.where_clause.is_empty() {
                query
                    .where_clause
                    .to_sql_stream(&mut full_query, query)
                    .unwrap();
            }

            if !query.order_by.is_empty() {
                query
                    .order_by
                    .to_sql_stream(&mut full_query, query)
                    .unwrap();
            }

            // LIMIT {limit} OFFSET {offset}
            if let Some(limit) = query.limit {
                // TODO(geekmasher): Check offset
                full_query.push_str(" LIMIT ");
                full_query.push_str(&limit.to_string());
                if let Some(offset) = query.offset {
                    full_query.push_str(" OFFSET ");
                    full_query.push_str(&offset.to_string());
                }
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
        backends::QueryBackend,
        builder::{
            columns::{Column, ColumnOptions, Columns},
            columntypes::ColumnType,
            table::Table,
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
                Column::from(("image_id".to_string(), ColumnType::ForeignKey)),
            ])
            .into(),
        }
    }

    #[test]
    fn test_select_sqlite() {
        let table = table();
        let query = QueryBuilder::select()
            .backend(QueryBackend::Sqlite)
            .table(&table)
            .build()
            .unwrap();

        assert_eq!(query.query, "SELECT id, name, email FROM Test;");
    }

    #[test]
    fn test_select_where() {
        let table = table();
        let query = QueryBuilder::select()
            .backend(QueryBackend::Sqlite)
            .table(&table)
            .where_eq("name", "test")
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "SELECT id, name, email FROM Test WHERE name = ?;"
        );
        assert_eq!(query.values.len(), 1);

        let mut values = Values::new();
        values.push("name".to_string(), Value::from("test"));
        assert_eq!(query.values, values);
    }

    #[test]
    fn test_order_clause() {
        let table = table();
        let query = QueryBuilder::select()
            .backend(QueryBackend::Sqlite)
            .table(&table)
            .order_by("name", QueryOrder::Asc)
            .order_by("email", QueryOrder::Desc)
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "SELECT id, name, email FROM Test ORDER BY name ASC, email DESC;"
        );
    }
}
