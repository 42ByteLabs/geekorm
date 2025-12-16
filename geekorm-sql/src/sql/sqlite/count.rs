//! # Count Query Builder

use crate::query::table::TableExpr;
use crate::{Query, QueryType, SqlQuery, ToSql};

impl QueryType {
    pub(crate) fn sql_count(&self, query: &Query) -> SqlQuery {
        let mut full_query = SqlQuery::new();
        full_query.push_str("SELECT COUNT(1)");

        if let Some(table) = query.find_table_default() {
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

    use super::*;
    use crate::{QueryType, Table, ToSql};
    use crate::{
        backends::QueryBackend,
        query::{
            Query,
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
            ])
            .into(),
        }
    }

    #[test]
    fn test_count_table() {
        let table = table();
        let query = Query::count()
            .backend(QueryBackend::Sqlite)
            .table(table)
            .build()
            .expect("Failed to build count query");
        let output = query.to_sql();

        assert_eq!(output.unwrap().to_string(), "SELECT COUNT(1) FROM Test;");
    }

    #[test]
    fn test_count_where_eq() {
        let table = table();
        let query = Query::count()
            .backend(QueryBackend::Sqlite)
            .table(table)
            .where_eq("id", 1)
            .build()
            .expect("Failed to build count query with WHERE clause");
        let output = query.to_sql().unwrap();

        assert_eq!(
            output.to_string(),
            "SELECT COUNT(1) FROM Test WHERE id = ?;"
        );
    }
}
