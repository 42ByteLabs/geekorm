use crate::{Columns, QueryBuilder, ToSqlite, Values};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The Table struct for defining a table
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
        if let Some((table, column)) = column.split_once('.') {
            if table != self.name {
                return false;
            }
            self.columns.is_valid_column(column)
        } else {
            self.columns.is_valid_column(column)
        }
    }

    /// Get the name of the primary key column
    pub fn get_primary_key(&self) -> String {
        self.columns
            .columns
            .iter()
            .find(|col| col.column_type.is_primary_key())
            .map(|col| col.name.clone())
            .unwrap_or_else(|| String::from("id"))
    }

    /// Get the foreign key by table name
    pub fn get_foreign_key(&self, table_name: String) -> &crate::Column {
        for column in self.columns.get_foreign_keys() {
            if column.column_type.is_foreign_key_table(&table_name) {
                return column;
            }
        }
        panic!("No foreign key found for column: {}", table_name);
    }

    /// Get the full name of a column (table.column)
    pub fn get_fullname(&self, column: &str) -> Result<String, crate::Error> {
        let column = self.columns.get(column).ok_or_else(|| {
            crate::Error::ColumnNotFound(self.name.to_string(), column.to_string())
        })?;
        let name = if column.alias.is_empty() {
            column.name.clone()
        } else {
            column.alias.clone()
        };
        Ok(format!("{}.{}", self.name, name))
    }
}

impl ToSqlite for Table {
    fn on_create(&self, query: &QueryBuilder) -> Result<String, crate::Error> {
        Ok(format!(
            "CREATE TABLE IF NOT EXISTS {} {};",
            self.name,
            self.columns.on_create(query)?
        ))
    }

    fn on_select(&self, qb: &QueryBuilder) -> Result<String, crate::Error> {
        let mut full_query = String::new();

        // Resolve the rest of the query, and append if necessary
        let columns = self.columns.on_select(qb);

        if let Ok(ref columns) = columns {
            if qb.count {
                // If the query is a count query, return the count query
                full_query = String::from("SELECT COUNT(1)");
            } else {
                // Select selective columns
                let mut select_columns: Vec<String> = Vec::new();

                let scolumns: Vec<String> = if !qb.columns.is_empty() {
                    qb.columns.clone()
                } else {
                    self.columns
                        .columns
                        .iter()
                        .filter(|col| !col.skip)
                        .map(|col| col.name.clone())
                        .collect()
                };

                for column in scolumns {
                    // TODO(geekmasher): Validate that the column exists in the table
                    if qb.joins.is_empty() {
                        // If the query does not join multiple tables, we can use the column name directly
                        select_columns.push(column);
                    } else {
                        // We have to use the full column name
                        if column.contains('.') {
                            // Table.column
                            select_columns.push(column);
                        } else {
                            // Lookup the column in the table
                            let fullname = qb.table.get_fullname(&column)?;
                            select_columns.push(fullname);
                        }
                    }
                }
                full_query = format!("SELECT {}", select_columns.join(", "));
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

    fn on_insert(&self, query: &QueryBuilder) -> Result<(String, Values), crate::Error> {
        let mut full_query = format!("INSERT INTO {} ", self.name);

        let mut columns: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();
        let mut parameters = Values::new();

        for (cname, value) in query.values.values.iter() {
            let column = query.table.columns.get(cname.as_str()).unwrap();

            // Get the column (might be an alias)
            let mut column_name = column.name.clone();
            if !column.alias.is_empty() {
                column_name = column.alias.to_string();
            }

            // Skip auto increment columns
            if column.column_type.is_auto_increment() {
                continue;
            }

            columns.push(column_name.clone());

            // Add to Values
            match value {
                crate::Value::Identifier(_) | crate::Value::Text(_) | crate::Value::Json(_) => {
                    // Security: String values should never be directly inserted into the query
                    // This is to prevent SQL injection attacks
                    values.push(String::from("?"));
                    parameters.push(column_name, value.clone());
                }
                crate::Value::Blob(value) => {
                    // Security: Blods should never be directly inserted into the query
                    values.push(String::from("?"));
                    parameters.push(column_name, value.clone());
                }
                crate::Value::Integer(value) => values.push(value.to_string()),
                crate::Value::Boolean(value) => values.push(value.to_string()),
                crate::Value::Null => values.push("NULL".to_string()),
            }
        }

        // Generate the column names
        full_query.push('(');
        full_query.push_str(&columns.join(", "));
        full_query.push(')');

        // Generate values
        full_query.push_str(" VALUES (");
        full_query.push_str(&values.join(", "));
        full_query.push(')');
        full_query.push(';');

        Ok((full_query, parameters))
    }

    fn on_update(&self, query: &QueryBuilder) -> Result<(String, Values), crate::Error> {
        let mut full_query = format!("UPDATE {} SET ", self.name);

        let mut columns: Vec<String> = Vec::new();
        let mut parameters = Values::new();

        for (cname, value) in query.values.values.iter() {
            let column = query.table.columns.get(cname.as_str()).unwrap();

            // Skip if primary key
            if column.column_type.is_primary_key() || cname == "id" {
                continue;
            }
            // Get the column (might be an alias)
            let mut column_name = column.name.clone();
            if !column.alias.is_empty() {
                column_name = column.alias.to_string();
            }

            // Add to Values
            match value {
                crate::Value::Identifier(_)
                | crate::Value::Text(_)
                | crate::Value::Blob(_)
                | crate::Value::Json(_) => {
                    // Security: String values should never be directly inserted into the query
                    // This is to prevent SQL injection attacks
                    columns.push(format!("{} = ?", column_name));
                    parameters.push(column_name, value.clone());
                }
                crate::Value::Integer(value) => {
                    columns.push(format!("{} = {}", column_name, value))
                }
                crate::Value::Boolean(value) => {
                    columns.push(format!("{} = {}", column_name, value))
                }
                crate::Value::Null => columns.push(format!("{} = NULL", column_name)),
            }
        }

        // Generate the column names
        full_query.push_str(&columns.join(", "));

        // WHERE
        // TODO(geekmasher): We only support updating by primary key
        let primary_key_name = query.table.get_primary_key();
        let primary_key = query.values.get(&primary_key_name).unwrap();
        let where_clause = format!(" WHERE {} = {}", primary_key_name, primary_key);
        full_query.push_str(&where_clause);
        full_query.push(';');

        Ok((full_query, parameters))
    }

    /// Function to delete a row from the table
    ///
    /// Only supports deleting by primary key
    fn on_delete(&self, query: &QueryBuilder) -> Result<(String, Values), crate::Error> {
        let mut full_query = format!("DELETE FROM {}", self.name);
        let mut parameters = Values::new();

        // Delete by primary key
        let primary_key_name = self.get_primary_key();
        let primary_key = query.values.get(&primary_key_name).unwrap();

        parameters.push(primary_key_name.to_string(), primary_key.clone());

        full_query.push_str(&format!(" WHERE {} = ?;", primary_key_name));

        Ok((full_query, parameters))
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

    fn table() -> Table {
        use crate::{Column, ColumnType, ColumnTypeOptions};

        Table {
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
        }
    }

    #[test]
    fn test_table_to_sql() {
        let table = table();

        let query = crate::QueryBuilder::select().table(table.clone());
        // Basic CREATE and SELECT
        assert_eq!(
            table.on_create(&query).unwrap(),
            "CREATE TABLE IF NOT EXISTS Test (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT);"
        );
        assert_eq!(
            table.on_select(&query).unwrap(),
            "SELECT id, name FROM Test;"
        );

        let query = crate::QueryBuilder::select()
            .table(table.clone())
            .where_eq("name", "this");
        assert_eq!(
            table.on_select(&query).unwrap(),
            "SELECT id, name FROM Test WHERE name = ?;"
        );
    }

    #[test]
    fn test_count() {
        let table = table();

        let query = crate::QueryBuilder::select().table(table.clone()).count();
        assert_eq!(
            table.on_select(&query).unwrap(),
            "SELECT COUNT(1) FROM Test;"
        );

        let query = crate::QueryBuilder::select()
            .table(table.clone())
            .count()
            .where_eq("name", "this");
        assert_eq!(
            table.on_select(&query).unwrap(),
            "SELECT COUNT(1) FROM Test WHERE name = ?;"
        );

        let query = crate::QueryBuilder::select()
            .table(table.clone())
            .count()
            .where_ne("name", "this");
        assert_eq!(
            table.on_select(&query).unwrap(),
            "SELECT COUNT(1) FROM Test WHERE name != ?;"
        );
    }

    #[test]
    fn test_row_delete() {
        let table = table();

        let query = crate::QueryBuilder::delete()
            .table(table.clone())
            .where_eq("id", 1);
        let (delete_query, _) = table.on_delete(&query).unwrap();

        assert_eq!(delete_query, "DELETE FROM Test WHERE id = ?;");
    }

    #[test]
    fn test_is_valid_column() {
        let table = table();

        assert!(table.is_valid_column("id"));
        assert!(table.is_valid_column("name"));
        assert!(!table.is_valid_column("name2"));
        // Test with table name
        assert!(table.is_valid_column("Test.name"));
        assert!(!table.is_valid_column("Tests.name"));
    }
}
