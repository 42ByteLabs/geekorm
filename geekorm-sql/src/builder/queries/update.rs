//! # Update Query Builder

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_update(&self, query: &QueryBuilder) -> String {
        let mut full_query = "UPDATE ".to_string();

        if let Some(table) = query.find_table_default() {
            let mut table_expr = TableExpr::new(table.name);

            table.columns.to_sql_stream(&mut full_query, query).unwrap();

            if let Some(ref alias) = table_expr.alias {
                table_expr.alias(alias.clone());
            }
            table_expr.to_sql_stream(&mut full_query, query).unwrap();

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
            .where_eq("id", 1)
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "UPDATE id, name, email FROM Test WHERE id = ?;"
        );
    }
}
