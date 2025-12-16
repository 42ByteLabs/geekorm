//! # Insert Query Builder

use crate::query::table::TableExpr;
use crate::{Query, QueryType, SqlQuery, ToSql, Value, Values};

impl QueryType {
    pub(crate) fn sql_insert(&self, query: &Query) -> SqlQuery {
        let mut full_query = SqlQuery::new();

        if let Some(table) = query.find_table_default() {
            full_query.push_str("INSERT INTO ");
            full_query.push_str(&table.name);

            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<String> = Vec::new();
            let mut parameters = Values::new();

            for (cname, value) in query.values.values() {
                let column = table.find_column(cname).unwrap();
                // Get the column (might be an alias)
                let column_name = column.name();

                // Skip auto increment columns
                if column.column_options.auto_increment {
                    continue;
                }

                columns.push(column_name.clone());

                // Add to Values
                match value {
                    Value::Identifier(_) | Value::Text(_) | Value::Json(_) => {
                        // Security: String values should never be directly inserted into the query
                        // This is to prevent SQL injection attacks
                        values.push(String::from("?"));
                        parameters.push(column_name, value.clone());
                    }
                    Value::Blob(value) => {
                        // Security: Blods should never be directly inserted into the query
                        values.push(String::from("?"));
                        parameters.push(column_name, value.clone());
                    }
                    Value::Integer(value) => values.push(value.to_string()),
                    Value::Boolean(value) => values.push(value.to_string()),
                    Value::Null => values.push("NULL".to_string()),
                }
            }

            full_query.push_str(" (");
            full_query.push_str(&columns.join(", "));

            full_query.push_str(") VALUES (");
            full_query.push_str(&values.join(", "));
            full_query.push(')');

            full_query.push(';');
        } else {
            full_query.push_str("-- No table specified for INSERT");
        }

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
    use crate::{QueryType, query::Query, query::table::Table};

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
    fn test_insert_query() {
        let table = table();
        let query = crate::Query::insert()
            .table(table)
            .add_value("id", 1)
            .add_value("name", "John Doe")
            .add_value("email", "john.doe@example.com")
            .build()
            .unwrap();
        let output = query.to_sql().unwrap();

        assert_eq!(
            output.to_string(),
            "INSERT INTO Test (name, email) VALUES (?, ?);"
        );
    }
}
