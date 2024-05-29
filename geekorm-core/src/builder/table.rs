use crate::{Columns, Query, QueryBuilder, ToSqlite, Values};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The Table struct for defining a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tables {
    /// Tables in the database
    pub tables: Vec<Table>,
}

impl Tables {
    /// Create all tables in the database
    #[cfg(feature = "libsql")]
    pub async fn create_all(&self, conn: &libsql::Connection) -> Result<(), crate::Error> {
        for table in &self.tables {
            let query = table.create()?;
            conn.execute(query.to_str(), ())
                .await
                .map_err(|e| crate::Error::LibSQLError(format!("Error creating table: {}", e)))?;
        }
        Ok(())
    }
}

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
        self.columns.is_valid_column(column)
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

    /// Get all foreign keys in the table
    pub fn get_foreign_keys(&self) -> Vec<&crate::Column> {
        self.columns.get_foreign_keys()
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

    /// Create Query
    pub fn create(&self) -> Result<Query, crate::Error> {
        QueryBuilder::create().table(self.clone()).build()
    }
}

impl ToSqlite for Table {
    fn on_create(&self, query: &QueryBuilder) -> Result<String, crate::Error> {
        let mut queries = Vec::new();

        if query.pivot_tables.is_empty() {
            queries.push(format!(
                "CREATE TABLE IF NOT EXISTS {} {};",
                self.name,
                self.columns.on_create(query)?
            ));
        } else {
            todo!("Pivot tables are not yet supported");
        }

        Ok(queries.join("\n"))
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

        for cname in query.values.order.iter() {
            let column = query.table.columns.get(cname.as_str()).unwrap();

            // Get the column (might be an alias)
            let mut column_name = column.name.clone();
            if !column.alias.is_empty() {
                column_name = column.alias.to_string();
            }

            let value = query.values.get(cname).unwrap();

            // Skip auto increment columns
            if column.column_type.is_auto_increment() {
                continue;
            }

            columns.push(column_name.clone());

            // Add to Values
            match value {
                crate::Value::Identifier(_) | crate::Value::Text(_) => {
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

        for cname in query.values.order.iter() {
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

            let value = query.values.get(cname).unwrap();

            // Add to Values
            match value {
                crate::Value::Identifier(_) | crate::Value::Text(_) | crate::Value::Blob(_) => {
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
}
