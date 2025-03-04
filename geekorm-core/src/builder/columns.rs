#[cfg(feature = "migrations")]
use super::alter::{AlterMode, AlterQuery};
use crate::{ColumnType, ToSqlite};
use serde::{Deserialize, Serialize};

/// A list of columns in a table
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Columns {
    /// List of columns
    pub columns: Vec<Column>,
}

impl Columns {
    /// Create a new instance of Columns
    pub fn new() -> Self {
        Columns {
            columns: Vec::new(),
        }
    }

    /// Validate if a column exists
    pub fn is_valid_column(&self, column: &str) -> bool {
        for col in &self.columns {
            if col.name == column {
                return true;
            }
        }
        false
    }

    /// Get the Primary Key column of a table
    pub fn get_primary_key(&self) -> Option<Column> {
        self.columns
            .iter()
            .find(|col| col.column_type.is_primary_key())
            .cloned()
    }

    /// Get the Foreign Keys columns of a table
    pub fn get_foreign_keys(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|col| matches!(col.column_type, ColumnType::ForeignKey(_)))
            .collect()
    }

    /// Get a column by name
    pub fn get(&self, column: &str) -> Option<&Column> {
        self.columns
            .iter()
            .find(|col| col.name == column || col.alias == column)
    }

    /// Get the length of the columns
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Check if the columns is empty
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

impl Iterator for Columns {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        self.columns.pop()
    }
}

impl From<Vec<Column>> for Columns {
    fn from(columns: Vec<Column>) -> Self {
        Columns { columns }
    }
}

#[cfg(feature = "migrations")]
impl quote::ToTokens for Columns {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let columns = &self.columns;
        tokens.extend(quote::quote! {
            geekorm::Columns {
                columns: Vec::from([
                    #(#columns),*
                ])
            }
        });
    }
}

impl ToSqlite for Columns {
    fn on_create(&self, query: &crate::QueryBuilder) -> Result<String, crate::Error> {
        let mut sql = Vec::new();
        for column in &self.columns {
            match column.on_create(query) {
                Ok(col) => sql.push(col),
                Err(crate::Error::ColumnSkipped) => {
                    // Skip the column
                    continue;
                }
                Err(e) => return Err(e),
            };
        }

        for foreign_key in self.get_foreign_keys() {
            let (ctable, ccolumn) = match &foreign_key.column_type {
                ColumnType::ForeignKey(opts) => {
                    let (ctable, ccolumn) = opts
                        .foreign_key
                        .split_once('.')
                        .expect("Invalid foreign key");
                    (ctable, ccolumn)
                }
                _ => unreachable!(),
            };

            sql.push(format!(
                "FOREIGN KEY ({parent}) REFERENCES {child}({child_column})",
                parent = foreign_key.name,
                child = ctable,
                child_column = ccolumn
            ));
        }

        Ok(format!("({})", sql.join(", ")))
    }

    fn on_select(&self, query: &crate::QueryBuilder) -> Result<String, crate::Error> {
        let mut full_query = String::new();

        // Support for WHERE
        if !query.where_clause.is_empty() {
            full_query.push_str("WHERE ");
            for column in &query.where_clause {
                full_query.push_str(column);
                full_query.push(' ');
            }
        }
        // Support for ORDER BY
        let mut order_by = Vec::new();
        if !query.order_by.is_empty() {
            for (column, order) in &query.order_by {
                // TODO(geekmasher): Validate that the column exists in the table
                order_by.push(format!("{} {}", column, order.to_sqlite()));
            }

            full_query += format!("ORDER BY {}", order_by.join(", ")).as_str();
        }
        Ok(full_query)
    }
}

/// A column in a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Name of the column
    pub name: String,
    /// Type of the column (e.g. TEXT, INTEGER, etc)
    pub column_type: ColumnType,

    /// Alias for the column
    pub alias: String,
    /// Metadata for the column
    pub skip: bool,
}

impl Column {
    /// Create a new instance of Column
    pub fn new(name: String, column_type: ColumnType) -> Self {
        Column {
            name,
            column_type,
            alias: String::new(),
            skip: false,
        }
    }

    /// Check if the column is a primary key
    pub fn is_primary_key(&self) -> bool {
        self.column_type.is_primary_key()
    }

    /// Check if the column is nullable
    pub fn is_not_null(&self) -> bool {
        self.column_type.is_not_null()
    }
    /// If the column unique
    pub fn is_unique(&self) -> bool {
        self.column_type.is_unique()
    }
}

impl Default for Column {
    fn default() -> Self {
        Column {
            name: String::new(),
            column_type: ColumnType::Text(Default::default()),
            alias: String::new(),
            skip: false,
        }
    }
}

#[cfg(feature = "migrations")]
impl quote::ToTokens for Column {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let coltype = &self.column_type;
        let alias = &self.alias;
        let skip = &self.skip;

        tokens.extend(quote::quote! {
            geekorm::Column {
                name: String::from(#name),
                column_type: #coltype,
                alias: String::from(#alias),
                skip: #skip,
            }
        });
    }
}

impl ToSqlite for Column {
    fn on_create(&self, query: &crate::QueryBuilder) -> Result<String, crate::Error> {
        if self.skip {
            return Err(crate::Error::ColumnSkipped);
        }

        let name = if !&self.alias.is_empty() {
            self.alias.clone()
        } else {
            self.name.clone()
        };
        Ok(format!("{} {}", name, self.column_type.on_create(query)?))
    }

    #[cfg(feature = "migrations")]
    fn on_alter(&self, query: &AlterQuery) -> Result<String, crate::Error> {
        Ok(match query.mode {
            AlterMode::AddTable => {
                format!("ALTER TABLE {} ADD COLUMN {};", query.table, query.column)
            }
            AlterMode::RenameTable => {
                format!("ALTER TABLE {} RENAME TO {};", query.table, query.column)
            }
            AlterMode::DropTable => {
                format!("ALTER TABLE {} DROP COLUMN {};", query.table, query.column)
            }
            AlterMode::AddColumn => {
                format!(
                    "ALTER TABLE {} ADD COLUMN {} {};",
                    query.table,
                    query.column,
                    self.column_type.on_alter(query)?
                )
            }
            AlterMode::RenameColumn => {
                format!(
                    "ALTER TABLE {} RENAME COLUMN {} TO {};",
                    query.table,
                    query.column,
                    query.rename.as_ref().unwrap_or(&query.column)
                )
            }
            AlterMode::DropColumn => {
                format!("ALTER TABLE {} DROP COLUMN {};", query.table, query.column)
            }
            AlterMode::Skip => {
                if query.column.is_empty() {
                    format!("-- Skipping {} this migration", query.table)
                } else {
                    format!(
                        "-- Skipping {}.{} this migration",
                        query.table, query.column
                    )
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColumnTypeOptions;

    fn create_table() -> crate::Table {
        crate::Table {
            name: String::from("users"),
            database: None,
            columns: Columns::from(vec![
                Column::new(
                    String::from("user_id"),
                    ColumnType::Integer(ColumnTypeOptions::default()),
                ),
                Column::new(
                    String::from("name"),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
                Column::new(
                    String::from("image_id"),
                    ColumnType::ForeignKey(ColumnTypeOptions {
                        foreign_key: String::from("images.id"),
                        ..Default::default()
                    }),
                ),
            ]),
        }
    }

    #[test]
    fn test_column_to_sql() {
        use super::*;
        let query = crate::QueryBuilder::default();
        let column = Column::new(
            String::from("name"),
            ColumnType::Text(ColumnTypeOptions::default()),
        );
        assert_eq!(column.on_create(&query).unwrap(), "name TEXT");

        let column = Column::new(
            String::from("age"),
            ColumnType::Integer(ColumnTypeOptions::default()),
        );
        assert_eq!(column.on_create(&query).unwrap(), "age INTEGER");

        // Test renaming the column
        let column = Column {
            name: String::from("id"),
            column_type: ColumnType::Integer(ColumnTypeOptions::default()),
            alias: String::from("user_id"),
            ..Default::default()
        };
        assert_eq!(column.on_create(&query).unwrap(), "user_id INTEGER");
    }

    #[test]
    fn test_foreign_key_to_sql() {
        let query = crate::QueryBuilder::new().table(create_table());

        let columns = query.table.columns.on_create(&query).unwrap();

        assert_eq!(
            columns,
            "(user_id INTEGER, name TEXT, image_id INTEGER, FOREIGN KEY (image_id) REFERENCES images(id))"
        );
    }

    #[test]
    fn test_alter_to_sql() {
        let query = crate::AlterQuery::new(AlterMode::AddColumn, "Table", "colname");

        let column = Column::new(
            String::from("name"),
            ColumnType::Text(ColumnTypeOptions::default()),
        );
        assert_eq!(
            column.on_alter(&query).unwrap(),
            "ALTER TABLE Table ADD COLUMN colname TEXT;"
        );
        let column = Column::new(
            String::from("name"),
            ColumnType::Text(ColumnTypeOptions {
                not_null: true,
                ..Default::default()
            }),
        );
        assert_eq!(
            column.on_alter(&query).unwrap(),
            "ALTER TABLE Table ADD COLUMN colname TEXT NOT NULL DEFAULT '';"
        );
    }
}
