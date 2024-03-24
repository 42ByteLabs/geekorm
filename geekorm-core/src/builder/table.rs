use crate::{Columns, QueryBuilder, ToSqlite};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The Table struct for creating tables
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Table {
    /// Name of the table
    pub name: String,
    /// Columns in the table
    pub columns: Columns,
}

impl Table {
    /// Function to check if a column name is valid
    pub fn is_valid_column(&self, column: &str) -> bool {
        self.columns.is_valid_column(column)
    }

    /// Function to get the name of the primary key of the table
    pub fn get_primary_key(&self) -> Option<String> {
        self.columns.get_primary_key().map(|col| col.name.clone())
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
        let mut full_query = String::new();

        // Resolve the rest of the query, and append if necessary
        let columns = self.columns.on_select(qb);

        if let Ok(ref columns) = columns {
            if !qb.columns.is_empty() {
                let mut join_columns: Vec<String> = Vec::new();
                for column in &qb.columns {
                    // TODO(geekmasher):
                    if column.contains('.') {
                        join_columns.push(String::from(column));
                    } else {
                        todo!("Add support for column lookup");
                    }
                }
                full_query = format!("SELECT {}", join_columns.join(", "));
            }
            // If the query is a count query, return the count query
            else if qb.count {
                // TODO(geekmasher): Add support for single column count
                // for now, we will just return the count of all columns which is not ideal
                // and expensive
                full_query = String::from("SELECT COUNT(*)");
            } else {
                // Defaults to SELECT all
                full_query = String::from("SELECT *");
            }

            // FROM {table}
            full_query.push_str(" FROM ");
            full_query.push_str(&self.name);

            // JOIN
            if !qb.joins.is_empty() {
                full_query.push(' ');
                full_query.push_str(qb.joins.on_select(qb)?.as_str());
            }

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
