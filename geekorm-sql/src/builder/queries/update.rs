//! # Update Query Builder
//!
//!

use crate::backends::SqliteBackendOptions;
use crate::builder::table::TableExpr;
use crate::{QueryBuilder, QueryType, ToSql, Value, Values};

impl QueryType {
    pub(crate) fn sql_update(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();

        let mut columns: Vec<String> = Vec::new();
        let mut parameters = Values::new();

        if let Some(table) = query.find_table_default() {
            // Update or rollback
            full_query = match query.backend {
                crate::QueryBackend::Sqlite {
                    options: SqliteBackendOptions { transactions: true },
                } => format!("UPDATE OR ROLLBACK {} SET ", table.name),
                _ => format!("UPDATE {} SET ", table.name),
            };

            let values = query.values.values();
            assert_ne!(values.len(), 0);

            for (cname, value) in values {
                // Security: The column must be a column is knows
                let column = table.find_column(cname).unwrap();
                // Get the column (might be an alias)
                let column_name = column.name();

                // Skip PK or auto increment columns
                if column.column_options.primary_key {
                    continue;
                }

                // Add to Values
                match value {
                    Value::Identifier(_) | Value::Text(_) | Value::Blob(_) | Value::Json(_) => {
                        // Security: String values should never be directly inserted into the query
                        // This is to prevent SQL injection attacks
                        columns.push(format!("{} = ?", column_name));
                        parameters.push(column_name, value.clone());
                    }
                    Value::Integer(value) => {
                        columns.push(format!("{} = {}", column_name, value));
                    }
                    Value::Datetime(value) => {
                        columns.push(format!("{} = {}", column_name, value));
                    }
                    Value::Boolean(value) => columns.push(format!("{} = {}", column_name, value)),
                    Value::Null => columns.push(format!("{} = NULL", column_name)),
                }
            }

            // Generate the column names
            full_query.push_str(&columns.join(", "));

            // WHERE
            // TODO(geekmasher): We only support updating by primary key
            if let Some(primary_key) = table.get_primary_key() {
                let primary_key_name = primary_key.name.clone();
                let primary_key = query.values.get(&primary_key_name).unwrap();
                let where_clause = format!(" WHERE {} = {}", primary_key_name, primary_key);
                full_query.push_str(&where_clause);
            }
        }

        full_query.push(';');
        full_query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::SqliteBackendOptions;
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
    fn sqlite_update_query() {
        let table = table();
        let query = QueryBuilder::update()
            .table(&table)
            .add_value("id", "1")
            .add_value("name", "bob")
            .add_value("email", "bob@example.com")
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "UPDATE Test SET name = ?, email = ? WHERE id = 1;"
        );
    }

    #[test]
    fn sqlite_update_rollback() {
        let table = table();
        let query = QueryBuilder::update()
            .backend(QueryBackend::Sqlite {
                options: SqliteBackendOptions { transactions: true },
            })
            .table(&table)
            .add_value("id", "1")
            .add_value("name", "bob")
            .add_value("email", "bob@example.com")
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "UPDATE OR ROLLBACK Test SET name = ?, email = ? WHERE id = 1;"
        );
    }
}
