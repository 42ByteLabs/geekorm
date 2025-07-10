//! # Ordering
use crate::ToSql;

/// Order Clause
#[derive(Debug, Clone, Default)]
pub struct OrderClause {
    /// Columns name and order
    pub(crate) columns: Vec<(String, QueryOrder)>,
}

impl ToSql for OrderClause {
    fn to_sql(&self, query: &super::QueryBuilder) -> Result<String, crate::Error> {
        let mut stream = String::new();
        self.to_sql_stream(&mut stream, query)?;
        Ok(stream)
    }

    fn to_sql_stream(
        &self,
        stream: &mut String,
        _query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        if self.is_empty() {
            return Ok(());
        }
        // If the last char is not a space, add a space
        if !stream.is_empty() && !stream.ends_with(' ') {
            stream.push(' ');
        }

        stream.push_str("ORDER BY ");

        for (index, (col, order)) in self.columns.iter().enumerate() {
            stream.push_str(col);
            stream.push(' ');
            stream.push_str(&order.sql());

            if index != self.columns.len() - 1 {
                stream.push_str(", ");
            }
        }
        Ok(())
    }
}

impl OrderClause {
    /// Is empty?
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Push a new column to the order clause
    pub fn push(&mut self, column: String, order: QueryOrder) {
        self.columns.push((column, order));
    }
}

/// Query Order (ASC / DESC)
#[derive(Debug, Clone)]
pub enum QueryOrder {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl ToSql for QueryOrder {
    fn sql(&self) -> String {
        match self {
            QueryOrder::Asc => String::from("ASC"),
            QueryOrder::Desc => String::from("DESC"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Column, ColumnOptions, ColumnType, Columns, Table, ToSql, builder::QueryBuilder};

    fn table() -> Table {
        Table {
            name: "Test",
            columns: Columns::new(vec![
                Column::from((
                    "id".to_string(),
                    ColumnType::Integer,
                    ColumnOptions::primary_key(),
                )),
                Column::from(("name".to_string(), ColumnType::Text)),
                Column::from(("email".to_string(), ColumnType::Text)),
            ])
            .into(),
        }
    }

    #[test]
    fn test_order_clause_to_sql() {
        let table = table();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let mut order_clause = OrderClause::default();
        order_clause.push("name".to_string(), QueryOrder::Asc);
        order_clause.push("email".to_string(), QueryOrder::Desc);

        let sql = order_clause.to_sql(&query).unwrap();

        assert_eq!(sql, "ORDER BY name ASC, email DESC");
    }
}
