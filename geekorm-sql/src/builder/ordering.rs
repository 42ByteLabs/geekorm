//! # Ordering
use std::collections::HashMap;

use crate::{ToSql, Value, Values};

/// Order Clause
#[derive(Debug, Clone, Default)]
pub struct OrderClause {
    /// Columns name and order
    pub(crate) columns: Vec<(String, QueryOrder)>,
}

impl ToSql for OrderClause {
    fn to_sql_stream(
        &self,
        stream: &mut String,
        query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        if !self.is_empty() {
            // If the last char is not a space, add a space
            if !stream.is_empty() && !stream.ends_with(' ') {
                stream.push(' ');
            }

            stream.push_str("ORDER BY ");

            for (index, (col, order)) in self.columns.iter().enumerate() {
                stream.push_str(col);
                stream.push(' ');

                order.to_sql_stream(stream, query).unwrap();

                if index != self.columns.len() - 1 {
                    stream.push_str(", ");
                }
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

/// Case Expression
///
/// This is used for things like Enums
#[derive(Debug, Clone)]
pub struct CaseExpression {
    /// Column of the case check
    column: String,
    /// Values of the colums
    cases: Values,
}

impl CaseExpression {
    /// New Case Expression
    pub fn new(column: String, cases: Values) -> Self {
        Self { column, cases }
    }
}

/// Query Order (ASC / DESC)
#[derive(Debug, Clone)]
pub enum QueryOrder {
    /// Ascending
    Asc,
    /// Descending
    Desc,
    /// Nulls first
    NullsFirst,
    /// Nulls last
    NullsLast,
    /// Case
    Case(CaseExpression),
}

impl ToSql for QueryOrder {
    fn sql(&self) -> String {
        match self {
            QueryOrder::Asc => String::from("ASC"),
            QueryOrder::Desc => String::from("DESC"),
            QueryOrder::NullsFirst => String::from("NULLS FIRST"),
            QueryOrder::NullsLast => String::from("NULLS LAST"),
            QueryOrder::Case(_) => panic!("should never get here"),
        }
    }

    fn to_sql_stream(
        &self,
        stream: &mut String,
        query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        match self {
            QueryOrder::Case(case) => case.to_sql_stream(stream, query).unwrap(),
            _ => {
                stream.push_str(&self.sql());
            }
        }
        Ok(())
    }
}

impl ToSql for CaseExpression {
    fn to_sql_stream(
        &self,
        stream: &mut String,
        _query: &super::QueryBuilder,
    ) -> Result<(), crate::Error> {
        let mut cases = Vec::new();

        stream.push_str(&format!("CASE {} ", self.column));

        for (key, case) in &self.cases.values {
            cases.push(format!("WHEN '{}' THEN {}", key, case))
        }

        let sql = cases.join(" ");
        stream.push_str(&sql);
        stream.push_str(" END");

        Ok(())
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
                Column::from(("role".to_string(), ColumnType::Text)),
                Column::from(("email".to_string(), ColumnType::Text)),
            ])
            .into(),
        }
    }

    fn cases() -> Values {
        let mut cases = Values::new();
        cases.push("Admin".to_string(), 1);
        cases.push("Mod".to_string(), 2);
        cases.push("User".to_string(), 3);
        cases
    }

    #[test]
    fn test_order_clause() {
        let table = table();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let mut order_clause = OrderClause::default();
        order_clause.push("name".to_string(), QueryOrder::Asc);
        order_clause.push("email".to_string(), QueryOrder::Desc);

        let sql = order_clause.to_sql(&query).unwrap();

        assert_eq!(sql, "ORDER BY name ASC, email DESC");
    }

    #[test]
    fn test_order_clause_nulls() {
        let table = table();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let mut order_clause = OrderClause::default();
        order_clause.push("name".to_string(), QueryOrder::NullsFirst);

        let sql = order_clause.to_sql(&query).unwrap();
        assert_eq!(sql, "ORDER BY name NULLS FIRST");
    }

    #[test]
    fn test_case_expr() {
        let table = table();
        let mut query = QueryBuilder::select();
        query.table(&table);

        let case = CaseExpression::new("role".to_string(), cases());
        let mut sql = String::new();
        case.to_sql_stream(&mut sql, &query).unwrap();

        assert_eq!(
            sql,
            "CASE role WHEN 'Admin' THEN 1 WHEN 'Mod' THEN 2 WHEN 'User' THEN 3 END"
        );
    }

    #[test]
    fn test_order_clause_case() {
        let table = table();
        let case = CaseExpression::new("role".to_string(), cases());

        let query = QueryBuilder::select()
            .table(&table)
            .order_by("role", QueryOrder::Case(case))
            .build()
            .unwrap();

        assert_eq!(
            query.query,
            "SELECT id, name, role, email FROM Test ORDER BY role CASE role WHEN 'Admin' THEN 1 WHEN 'Mod' THEN 2 WHEN 'User' THEN 3 END;"
        );
    }
}
