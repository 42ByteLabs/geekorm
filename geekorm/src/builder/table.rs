use std::fmt::Display;

use crate::{Columns, QueryBuilder, ToSqlite};

#[derive(Debug, Clone, Default)]
pub struct Table {
    pub name: String,
    pub columns: Columns,
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
        // Resolve the rest of the query, and append if necessary
        let columns = self.columns.on_select(qb);
        if let Ok(columns) = columns {
            if columns.is_empty() {
                return Ok(format!("SELECT * FROM {}", self.name));
            }
            return Ok(format!("SELECT * FROM {} {}", self.name, columns));
        }
        columns
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
