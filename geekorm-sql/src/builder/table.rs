//! # Table Expression

use crate::ToSql;

/// Table expression for SQL queries.
#[derive(Debug, Clone, Default)]
pub struct TableExpr {
    /// Table name
    pub name: String,
    /// Alias for the table
    pub alias: Option<String>,
}

impl ToSql for TableExpr {
    /// Generate the SQL for the table expression
    fn sql(&self) -> String {
        if let Some(ref alias) = self.alias {
            format!("{} AS {}", self.name, alias)
        } else {
            self.name.clone()
        }
    }

    fn to_sql_stream(
        &self,
        stream: &mut String,
        _query: &super::QueryBuilder,
    ) -> Result<(), geekorm_core::Error> {
        if !stream.is_empty() && !stream.ends_with(' ') {
            stream.push(' ');
        }

        stream.push_str("FROM ");
        stream.push_str(&self.sql());
        Ok(())
    }
}

impl TableExpr {
    /// Create a new table expression
    pub fn new(name: &str) -> Self {
        TableExpr {
            name: name.to_string(),
            alias: None,
        }
    }

    /// Set the alias for the table expression
    pub fn alias(&mut self, alias: String) {
        self.alias = Some(alias);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryBuilder;

    #[test]
    fn test_table() {
        let table = TableExpr::new("users");
        let mut sql = String::new();
        table
            .to_sql_stream(&mut sql, &QueryBuilder::default())
            .unwrap();
        assert_eq!(sql, "FROM users");
    }

    #[test]
    fn test_table_expr_alias() {
        let mut table = TableExpr::new("users");
        table.alias("u".to_string());
        let mut sql = String::new();
        table
            .to_sql_stream(&mut sql, &QueryBuilder::default())
            .unwrap();
        assert_eq!(sql, "FROM users AS u");
    }
}
