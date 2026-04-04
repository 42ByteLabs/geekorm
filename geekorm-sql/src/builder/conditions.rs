//! # Query Conditions

use super::QueryBuilder;
use crate::{Error, ToSql};

/// Query Condition (EQ, NE, etc.)
#[derive(Debug, Clone, Default)]
pub enum QueryCondition {
    /// Equal
    #[default]
    Eq,
    /// Not Equal
    Ne,
    /// Like
    Like,
    /// Greater Than
    Gt,
    /// Less Than
    Lt,
    /// Greater Than or Equal to
    Gte,
    /// Less Than or Equal to
    Lte,
}

impl ToSql for QueryCondition {
    fn sql(&self) -> String {
        match self {
            QueryCondition::Eq => String::from("="),
            QueryCondition::Ne => String::from("!="),
            QueryCondition::Like => String::from("LIKE"),
            QueryCondition::Gt => String::from(">"),
            QueryCondition::Lt => String::from("<"),
            QueryCondition::Gte => String::from(">="),
            QueryCondition::Lte => String::from("<="),
        }
    }
}

/// Where Condition (AND, OR)
#[derive(Debug, Clone, Default)]
pub enum WhereCondition {
    /// And condition
    #[default]
    And,
    /// Or condition
    Or,
}

impl WhereCondition {
    /// Get all where conditions as a vector of strings
    pub fn all() -> Vec<String> {
        vec![WhereCondition::And.sql(), WhereCondition::Or.sql()]
    }
}

impl ToSql for WhereCondition {
    fn sql(&self) -> String {
        match self {
            WhereCondition::And => String::from("AND"),
            WhereCondition::Or => String::from("OR"),
        }
    }
}

/// Query Where clause
#[derive(Debug, Clone, Default)]
pub struct WhereClause {
    conditions: Vec<(String, QueryCondition, Option<WhereCondition>)>,
}

impl WhereClause {
    /// If the where clause is empty
    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    /// Push a new condition to the where clause
    pub fn push(&mut self, column: String, condition: QueryCondition) {
        self.conditions.push((column, condition, None));
    }

    /// Push a new condition to the where clause with a condition
    ///
    /// This is used to chain conditions together
    pub fn push_condition(&mut self, condition: WhereCondition) -> Result<(), Error> {
        if self.is_empty() {
            return Err(Error::QueryBuilderError {
                error: String::from("Cannot push condition to empty where clause"),
                location: String::from("push_condition"),
            });
        }
        // Get the last condition
        if let Some(last) = self.conditions.last_mut() {
            last.2 = Some(condition);
        } else {
            return Err(Error::QueryBuilderError {
                error: String::from("Cannot push condition to empty where clause"),
                location: String::from("push_condition"),
            });
        }
        Ok(())
    }
}

impl ToSql for WhereClause {
    fn to_sql_stream(&self, stream: &mut String, query: &QueryBuilder) -> Result<(), Error> {
        if self.is_empty() {
            return Ok(());
        }
        // If the last char is not a space, add a space
        if !stream.is_empty() && !stream.ends_with(' ') {
            stream.push(' ');
        }

        // Add the where clause to the SQL string
        stream.push_str("WHERE ");

        let mut param_index = 1;
        for (column, qcondition, wcondition) in &self.conditions {
            stream.push_str(&column);
            stream.push(' ');
            stream.push_str(&qcondition.sql());

            // Use different placeholder syntax based on backend
            match query.backend {
                crate::QueryBackend::Postgres => {
                    stream.push_str(&format!(" ${}", param_index));
                    param_index += 1;
                }
                _ => {
                    stream.push_str(" ?");
                }
            }

            if let Some(next_condition) = wcondition {
                stream.push_str(&format!(" {} ", next_condition.sql()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryBackend;
    use crate::ToSql;
    use crate::builder::QueryBuilder;

    #[test]
    fn test_where_clause() {
        let mut where_clause = WhereClause::default();
        where_clause.push("id".to_string(), QueryCondition::Eq);
        where_clause.push_condition(WhereCondition::And).unwrap();
        where_clause.push("name".to_string(), QueryCondition::Like);

        let mut query = String::new();
        where_clause
            .to_sql_stream(&mut query, &QueryBuilder::default())
            .unwrap();

        assert_eq!(query, "WHERE id = ? AND name LIKE ?");
    }

    #[test]
    fn test_where_clause_postgres() {
        let mut where_clause = WhereClause::default();
        where_clause.push("id".to_string(), QueryCondition::Eq);
        where_clause.push_condition(WhereCondition::And).unwrap();
        where_clause.push("name".to_string(), QueryCondition::Like);

        let mut query_builder = QueryBuilder::default();
        query_builder.backend(QueryBackend::Postgres);

        let mut query = String::new();
        where_clause
            .to_sql_stream(&mut query, &query_builder)
            .unwrap();

        assert_eq!(query, "WHERE id = $1 AND name LIKE $2");
    }

    #[test]
    fn test_where_clause_postgres_multiple() {
        let mut where_clause = WhereClause::default();
        where_clause.push("age".to_string(), QueryCondition::Gt);
        where_clause.push_condition(WhereCondition::Or).unwrap();
        where_clause.push("status".to_string(), QueryCondition::Eq);
        where_clause.push_condition(WhereCondition::And).unwrap();
        where_clause.push("active".to_string(), QueryCondition::Eq);

        let mut query_builder = QueryBuilder::default();
        query_builder.backend(QueryBackend::Postgres);

        let mut query = String::new();
        where_clause
            .to_sql_stream(&mut query, &query_builder)
            .unwrap();

        assert_eq!(query, "WHERE age > $1 OR status = $2 AND active = $3");
    }
}
