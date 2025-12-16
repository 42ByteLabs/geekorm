//! # Create Query Builder

use crate::query::table::TableExpr;
use crate::{Error, Query, QueryType, SqlQuery, ToSql};

impl QueryType {
    pub(crate) fn sql_create(&self, query: &Query) -> SqlQuery {
        let mut full_query = SqlQuery::new();
        if let Some(table) = query.find_table_default() {
            full_query.push_str("CREATE TABLE IF NOT EXISTS ");
            full_query.push_str(&table.name);

            full_query.push_str(" (");

            table.columns.to_sql_stream(&mut full_query, query).unwrap();

            for foreign_key in table.columns.get_foreign_keys() {
                let (ctable, ccolumn) = foreign_key.get_foreign_key().unwrap();

                full_query.push_str(", FOREIGN KEY (");
                full_query.push_str(&foreign_key.name);

                full_query.push_str(") REFERENCES ");
                full_query.push_str(ctable);
                full_query.push('(');
                full_query.push_str(ccolumn);
                full_query.push(')');
            }

            full_query.push_str(");");
        }

        full_query
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::query::{
        Query,
        columns::{Column, ColumnOptions, Columns},
        columntypes::ColumnType,
    };
    use crate::{QueryType, Table};

    fn table() -> Table {
        Table {
            name: "Profile",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from((
                    "email".to_string(),
                    ColumnType::Text,
                    ColumnOptions::unique(),
                )),
            ])
            .into(),
        }
    }

    fn table_images() -> Table {
        Table {
            name: "Image",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("url".to_string(), ColumnType::Text)),
            ])
            .into(),
        }
    }

    #[test]
    fn test_create_query() {
        let table = table();
        let query = Query::create().table(table).build().unwrap();

        let sql = QueryType::Create.sql_create(&query);

        assert_eq!(
            sql.to_string(),
            "CREATE TABLE IF NOT EXISTS Profile (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT UNIQUE);"
        );
    }

    #[test]
    fn test_create_with_foreign_key() {
        let table = Table {
            name: "Profile",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from((
                    "email".to_string(),
                    ColumnType::Text,
                    ColumnOptions::unique(),
                )),
                Column::new_foreign_key("image_id".to_string(), "Image.id".to_string()),
            ])
            .into(),
        };

        let query = Query::create().table(table).build().unwrap();

        let sql = QueryType::Create.sql_create(&query);

        assert_eq!(
            sql.to_string(),
            "CREATE TABLE IF NOT EXISTS Profile (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT UNIQUE, image_id INTEGER, FOREIGN KEY (image_id) REFERENCES Image(id));"
        );
    }
}
