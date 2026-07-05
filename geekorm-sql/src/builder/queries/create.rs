//! # Create Query Builder

use crate::builder::table::TableExpr;
use crate::{Error, QueryBuilder, QueryType, ToSql};

impl QueryType {
    pub(crate) fn sql_create(&self, query: &QueryBuilder) -> String {
        let mut full_query = String::new();

        if let Some(table) = query.find_table_default() {
            full_query.push_str(&format!("CREATE TABLE IF NOT EXISTS {} (", table.name));

            // Columns with types
            table.columns.to_sql_stream(&mut full_query, query).unwrap();

            let fkeys = table.columns.get_foreign_keys();
            if !fkeys.is_empty() {
                full_query.push_str(") ");

                for foreign_key in fkeys {
                    let (ctable, ccolumn) = foreign_key
                        .get_foreign_key()
                        .expect("Failed to get FK in create query");

                    full_query.push_str(&format!(
                        "FOREIGN KEY ({}) REFERENCES {} ({})",
                        foreign_key.name, ctable, ccolumn
                    ));
                }
            }
            full_query.push(')');
        }
        full_query.push(';');

        full_query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QueryType, Table,
        builder::{
            QueryBuilder,
            columns::{Column, ColumnOptions, Columns},
            columntypes::ColumnType,
            tests::{table_images, table_users},
        },
    };

    #[test]
    fn sqlite_create_query() {
        let table = table_images();
        let query = QueryBuilder::create()
            .table(&table)
            .build()
            .expect("Failed to create query");

        assert_eq!(
            query.query.as_str(),
            "CREATE TABLE IF NOT EXISTS Images (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT, url TEXT);"
        );
    }

    #[test]
    fn sqlite_create_joins() {
        let table = table_users();

        let query = QueryBuilder::create()
            .table(&table)
            .build()
            .expect("Failed to create query");

        assert_eq!(
            query.query.as_str(),
            "CREATE TABLE IF NOT EXISTS Users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT, email TEXT NOT NULL UNIQUE, roles INTEGER, profile INTEGER) FOREIGN KEY (roles) REFERENCES Roles (id)FOREIGN KEY (profile) REFERENCES Images (id));"
        );
    }
}
