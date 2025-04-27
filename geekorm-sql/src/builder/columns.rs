//! # Columns
use geekorm_core::{Column, ColumnType, Columns, Error};

use super::QueryType;
use crate::ToSql;

impl ToSql for Column {
    fn to_sql_stream(&self, stream: &mut String, query: &super::QueryBuilder) -> Result<(), Error> {
        match query.query_type {
            QueryType::Create => {
                stream.push_str(&self.name);
                stream.push(' ');
                self.column_type.to_sql_stream(stream, query)?;
            }
            _ => {
                if self.skip {
                    return Ok(());
                }
                let name = if !&self.alias.is_empty() {
                    self.alias.clone()
                } else {
                    self.name.clone()
                };

                if query.joins.is_empty() {
                    stream.push_str(&name);
                } else {
                    stream.push_str(&query.table.unwrap().get_fullname(name.as_str()).unwrap());
                }
            }
        }
        Ok(())
    }
}

impl ToSql for Columns {
    fn to_sql_stream(&self, stream: &mut String, query: &super::QueryBuilder) -> Result<(), Error> {
        let mut sql = Vec::new();

        for col in &self.columns {
            sql.push(col.to_sql(query)?);
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
                "FOREIGN KEY ({}) REFERENCES {}({})",
                foreign_key.name, ctable, ccolumn
            ));
        }

        stream.push_str(&sql.join(", "));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Query;
    use crate::builder::QueryBuilder;
    use geekorm_core::{Column, ColumnType, ColumnTypeOptions, Table};

    fn table() -> Table {
        Table {
            name: "Test".to_string(),
            database: None,
            columns: vec![
                Column::new(
                    "id".to_string(),
                    ColumnType::Identifier(ColumnTypeOptions {
                        primary_key: true,
                        foreign_key: String::new(),
                        unique: true,
                        not_null: true,
                        auto_increment: true,
                    }),
                ),
                Column::new(
                    "name".to_string(),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
                Column::new(
                    "email".to_string(),
                    ColumnType::Text(ColumnTypeOptions::default()),
                ),
                Column::new(
                    "image_id".to_string(),
                    ColumnType::ForeignKey(ColumnTypeOptions {
                        foreign_key: "images.id".to_string(),
                        ..Default::default()
                    }),
                ),
            ]
            .into(),
        }
    }

    #[test]
    fn test_single_column_to_sql() {
        let table = table();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let column = Column::new(
            "id".to_string(),
            ColumnType::Identifier(ColumnTypeOptions::default()),
        );

        let column_sql = column.to_sql(&query).unwrap();

        assert_eq!(column_sql.as_str(), "id");
    }

    #[test]
    fn test_columns_to_sql() {
        let table = table();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let columns = Columns::to_sql(&table.columns, &query).unwrap();

        assert_eq!(
            columns.as_str(),
            "id, name, email, image_id, FOREIGN KEY (image_id) REFERENCES images(id)"
        );
    }
}
