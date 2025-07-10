//! # Create Query Builder

use crate::builder::table::TableExpr;
use crate::{Error, QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_create(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();
        if let Some(table) = query.find_table_default() {
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

    use super::*;
    use crate::builder::{
        QueryBuilder,
        columns::{Column, ColumnOptions, Columns},
        columntypes::ColumnType,
    };
    use crate::{QueryType, Table};

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
