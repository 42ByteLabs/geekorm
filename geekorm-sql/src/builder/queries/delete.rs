//! # Delete Query Builder
//!
//! Delete queries are only supported for tables with a primary key.

use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_delete(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();

        if let Some(table) = query.find_table_default() {
            full_query.push_str("DELETE ");

            let table_expr = TableExpr::new(table.name);
            table_expr.to_sql_stream(&mut full_query, query).unwrap();

            // WHERE
            if !query.where_clause.is_empty() {
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
    use crate::builder::{
        columns::{Column, ColumnOptions, Columns},
        columntypes::ColumnType,
    };
    use crate::{QueryType, Table, ToSql};

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
            ])
            .into(),
        }
    }

    #[test]
    fn test_sql_delete() {
        let table = table();
        let query = QueryBuilder::delete()
            .table(&table)
            .where_eq("id", 1)
            .build()
            .unwrap();

        assert_eq!(query.query, "DELETE FROM Test WHERE id = ?;");
        assert_eq!(query.values.len(), 1);
    }
}
