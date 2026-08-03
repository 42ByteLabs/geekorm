//! # Insert Query Builder

use crate::backends::SqliteBackendOptions;
use crate::builder::table::TableExpr;
use crate::{Query, QueryBuilder, QueryType, ToSql, Value, Values};

impl QueryType {
    pub(crate) fn sql_insert(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::from("INSERT ");

        if let Some(table) = query.find_table_default() {
            // Update or rollback
            full_query.push_str(&match query.backend {
                crate::QueryBackend::Sqlite {
                    options: SqliteBackendOptions { transactions: true },
                } => format!("OR ROLLBACK INTO {}", table.name),
                _ => format!("INTO {}", table.name),
            });

            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<String> = Vec::new();

            for nvalue in query.values.values() {
                let column = table.find_column(nvalue.name()).unwrap();
                // Get the column (might be an alias)
                let column_name = column.name();

                // Skip auto increment columns
                if column.column_options.auto_increment {
                    continue;
                }

                columns.push(column_name.clone());

                let value_param = nvalue.to_sql(query).unwrap();
                values.push(value_param);
            }

            full_query.push_str(" (");
            full_query.push_str(&columns.join(", "));

            full_query.push_str(") VALUES (");
            full_query.push_str(&values.join(", "));
            full_query.push(')');

            full_query.push(';');
        } else {
            return String::from("No table specified");
        }

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
    use crate::{QueryType, builder::QueryBuilder, builder::table::Table};

    fn table() -> Table {
        Table::new(
            "Test",
            Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from(("email".to_string(), ColumnType::Text)),
            ]),
        )
    }

    #[test]
    fn sqlite_insert_query() {
        let table = table();
        let query = crate::QueryBuilder::insert()
            .table(&table)
            .add_value("id", 1)
            .add_value("name", "John Doe")
            .add_value("email", "john.doe@example.com")
            .build()
            .unwrap();

        assert_eq!(query.values.len(), 2);
        assert_eq!(
            query.query,
            "INSERT INTO Test (name, email) VALUES (:name, :email);"
        );
    }

    #[test]
    fn sqlite_insert_rollback() {
        let table = table();
        let query = crate::QueryBuilder::insert()
            .backend(crate::QueryBackend::Sqlite {
                options: SqliteBackendOptions { transactions: true },
            })
            .table(&table)
            .add_value("id", 1)
            .add_value("name", "John Doe")
            .add_value("email", "john.doe@example.com")
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "INSERT OR ROLLBACK INTO Test (name, email) VALUES (?, ?);"
        );
    }
}
