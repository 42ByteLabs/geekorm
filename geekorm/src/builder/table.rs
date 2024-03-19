use std::fmt::Display;

use crate::{Columns, QueryBuilder, ToSqlite};

#[derive(Debug, Clone, Default)]
pub struct Table {
    pub name: String,
    pub columns: Columns,
}

impl Table {
    pub fn is_valid_column(&self, column: &str) -> bool {
        self.columns.is_valid_column(column)
    }
}

impl ToSqlite for Table {
    fn on_create(&self) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            self.name,
            self.columns.on_create()
        )
    }

    fn on_select(&self, qb: &QueryBuilder) -> Result<String, crate::Error> {
        let mut full_query = String::from("SELECT *");

        // Resolve the rest of the query, and append if necessary
        let columns = self.columns.on_select(qb);

        if let Ok(ref columns) = columns {
            // If the query is a count query, return the count query
            if qb.count {
                // TODO(geekmasher): Add support for single column count
                // for now, we will just return the count of all columns which is not ideal
                // and expensive
                full_query = String::from("SELECT COUNT(*)");
            }
            // FROM {table}
            full_query.push_str(" FROM ");
            full_query.push_str(&self.name);

            // WHERE {where_clause} ORDER BY {order_by}
            if !columns.is_empty() {
                full_query.push(' ');
                full_query.push_str(columns);
            }
            // LIMIT {limit} OFFSET {offset}
            if let Some(limit) = qb.limit {
                // TODO(geekmasher): Check offset
                full_query.push_str(" LIMIT ");
                full_query.push_str(&limit.to_string());
                if let Some(offset) = qb.offset {
                    full_query.push_str(" OFFSET ");
                    full_query.push_str(&offset.to_string());
                }
            }

            // End
            full_query = full_query.trim().to_string();
            full_query.push(';');
        }
        Ok(full_query)
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Table('{}')", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_table_to_sql() {
        use crate::{Column, ColumnType, ColumnTypeOptions, Table};

        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column::new(
                    "id".to_string(),
                    ColumnType::Integer(ColumnTypeOptions::primary_key()),
                ),
                Column::new(
                    "name".to_string(),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
            ]
            .into(),
        };

        assert_eq!(
            table.on_create(),
            "CREATE TABLE IF NOT EXISTS Test (id INTEGER NOT NULL PRIMARY KEY, name TEXT)"
        );
    }
}
