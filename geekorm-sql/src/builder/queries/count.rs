//! # Count Query Builder

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_count(&self, query: &QueryBuilder) -> String {
        let mut full_query = "SELECT COUNT(1)".to_string();

        if let Some(table) = query.table {
            let mut table = TableExpr::new(&table.name);
            if let Some(ref alias) = table.alias {
                table.alias(alias.clone());
            }
            table.to_sql_stream(&mut full_query, query).unwrap();

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
    fn test_count_sqlite() {
        let table = table();
        let query = QueryBuilder::count()
            .backend(crate::QueryBackend::Sqlite)
            .table(&table)
            .build()
            .unwrap();

        assert_eq!(query.query, "SELECT COUNT(1) FROM Test;");
    }

    #[test]
    fn test_count_where() {
        let table = table();
        let query = QueryBuilder::count()
            .backend(crate::QueryBackend::Sqlite)
            .table(&table)
            .where_eq("id", 1)
            .build()
            .unwrap();

        assert_eq!(query.query, "SELECT COUNT(1) FROM Test WHERE id = ?;");
    }
}
