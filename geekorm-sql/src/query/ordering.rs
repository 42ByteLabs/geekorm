//! # Ordering
use crate::{SqlQuery, ToSql};

/// Order Clause
#[derive(Debug, Clone, Default)]
pub struct OrderClause {
    /// Columns name and order
    pub(crate) columns: Vec<(String, QueryOrder)>,
}

impl ToSql for OrderClause {
    fn sql(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut stream = String::new();
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
        stream
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
    use crate::{Column, ColumnOptions, ColumnType, Columns, Table, ToSql, query::Query};

    #[test]
    fn test_order_clause_to_sql() {
        let mut order_clause = OrderClause::default();
        order_clause.push("name".to_string(), QueryOrder::Asc);
        order_clause.push("email".to_string(), QueryOrder::Desc);

        let sql = order_clause.sql();

        assert_eq!(sql, "ORDER BY name ASC, email DESC");
    }
}
