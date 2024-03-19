use crate::{ColumnType, ToSqlite};

#[derive(Debug, Clone, Default)]
pub struct Columns {
    pub columns: Vec<Column>,
}

impl Columns {
    pub fn new() -> Self {
        Columns {
            columns: Vec::new(),
        }
    }

    pub fn is_valid_column(&self, column: &str) -> bool {
        for col in &self.columns {
            if col.name == column {
                return true;
            }
        }
        false
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

impl ToSqlite for Columns {
    fn on_create(&self) -> String {
        let mut sql = Vec::new();
        for column in &self.columns {
            sql.push(column.on_create());
        }
        sql.join(", ")
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

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub column_type: ColumnType,
}

impl Column {
    pub fn new(name: String, column_type: ColumnType) -> Self {
        Column { name, column_type }
    }
}

impl ToSqlite for Column {
    fn on_create(&self) -> String {
        format!("{} {}", self.name, self.column_type.on_create())
    }
}

#[cfg(test)]
mod tests {
    use crate::ColumnTypeOptions;

    #[test]
    fn test_column_to_sql() {
        use super::*;
        let column = Column::new(
            String::from("name"),
            ColumnType::Text(ColumnTypeOptions::default()),
        );
        assert_eq!(column.on_create(), "name TEXT");

        let column = Column::new(
            String::from("age"),
            ColumnType::Integer(ColumnTypeOptions::default()),
        );
        assert_eq!(column.on_create(), "age INTEGER");
    }
}
