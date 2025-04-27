//! # Delete Query Builder
//!
//! Delete queries are only supported for tables with a primary key.

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_delete(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();

        if let Some(table) = query.table {
            full_query.push_str("DELETE ");

            let table_expr = TableExpr::new(&table.name);
            table_expr.to_sql_stream(&mut full_query, query).unwrap();

            // WHERE
            query
                .where_clause
                .to_sql_stream(&mut full_query, query)
                .unwrap();
        }

        full_query.push(';');
        full_query
    }
}

#[cfg(test)]
mod tests {
    use geekorm_core::{Column, ColumnType, ColumnTypeOptions, Table, Values};

    use super::*;
    use crate::{QueryType, ToSql};

    fn table() -> Table {
        Table {
            name: "Test".to_string(),
            database: None,
            columns: vec![
                Column::new(
                    "id".to_string(),
                    ColumnType::Integer(ColumnTypeOptions {
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
            ]
            .into(),
        }
    }

    #[test]
    fn test_sql_delete() {
        let table = table();
        let query = QueryBuilder::delete()
            .table(&table)
            .where_primary_key(1)
            .build()
            .unwrap();

        assert_eq!(query.query, "DELETE FROM Test WHERE id = ?;");
        assert_eq!(query.values.len(), 1);
    }
}
