//! # Delete Query Builder
//!
//! Delete queries are only supported for tables with a primary key.

use crate::query::table::TableExpr;
use crate::{Query, QueryType, SqlQuery, ToSql};

impl QueryType {
    pub(crate) fn sql_delete(&self, query: &Query) -> SqlQuery {
        let mut full_query = SqlQuery::new();

        if let Some(table) = query.find_table_default() {
            full_query.push_str("DELETE ");

            let table_expr = TableExpr::new(table.name);
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
    use super::*;
    use crate::query::{
        columns::{Column, ColumnOptions, Columns},
        columntypes::ColumnType,
    };
    use crate::{QueryType, Table, ToSql};

    fn table() -> Table {
        Table {
            name: "Test",
            columns: Columns::new(vec![
                Column::primary_key("id"),
                Column::from(("name".to_string(), ColumnType::Text)),
            ])
            .into(),
        }
    }

    #[test]
    fn test_sql_delete() {
        let table = table();
        let query = Query::delete()
            .table(table)
            .where_primary_key(1)
            .build()
            .unwrap();
        let output = query.to_sql().unwrap();

        assert_eq!(output.to_string(), "DELETE FROM Test WHERE id = ?;");
        assert_eq!(query.values.len(), 1);
    }
}
