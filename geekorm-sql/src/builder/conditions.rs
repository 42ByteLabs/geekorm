//! # Query Conditions

use super::QueryBuilder;
use crate::values::values::ValueBindingMode;
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
    fn sql(&self) -> String {
        self.to_sql(&QueryBuilder::default()).unwrap()
    }

    fn to_sql(&self, query: &QueryBuilder) -> Result<String, Error> {
        let mut stream = String::new();
        if !self.is_empty() {
            // Add the where clause to the SQL string
            stream.push_str("WHERE ");

            for (column, qcondition, wcondition) in &self.conditions {
                stream.push_str(column);
                stream.push(' ');
                stream.push_str(&qcondition.sql());
                stream.push(' ');

                match query.values.binding_mode {
                    ValueBindingMode::Placeholder => {
                        stream.push('?');
                    }
                    ValueBindingMode::Named => {
                        stream.push_str(&format!(":{}", column));
                    }
                    ValueBindingMode::Numeric => {
                        let index = query
                            .values
                            .get_index(column.as_str())
                            .expect("Failed to fetch value index");
                        stream.push_str(&format!("?{}", index));
                    }
                }

                if let Some(next_condition) = wcondition {
                    stream.push_str(&format!(" {} ", next_condition.sql()));
                }
            }
        }

        Ok(stream)
    }

    fn to_sql_stream(&self, stream: &mut String, query: &QueryBuilder) -> Result<(), Error> {
        if !query.where_clause.is_empty() {
            stream.push(' ');
            stream.push_str(&self.to_sql(query)?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToSql;
    use crate::builder::QueryBuilder;
    use crate::builder::tests::*;

    #[test]
    fn test_where_clause_eq() {
        let mut where_clause = WhereClause::default();
        where_clause.push("id".to_string(), QueryCondition::Eq);

        let query = where_clause.sql();

        assert_eq!(query, "WHERE id = ?");
    }

    #[test]
    fn test_where_clause_and() {
        let mut where_clause = WhereClause::default();
        where_clause.push("id".to_string(), QueryCondition::Eq);
        where_clause.push_condition(WhereCondition::And).unwrap();
        where_clause.push("name".to_string(), QueryCondition::Like);

        let query = where_clause.sql();

        assert_eq!(query, "WHERE id = ? AND name LIKE ?");
    }

    #[test]
    fn test_where_clause_or() {
        let mut where_clause = WhereClause::default();
        where_clause.push("id".to_string(), QueryCondition::Eq);
        where_clause.push_condition(WhereCondition::Or).unwrap();
        where_clause.push("name".to_string(), QueryCondition::Like);

        let query = where_clause.sql();

        assert_eq!(query, "WHERE id = ? OR name LIKE ?");
    }

    #[test]
    fn test_where_named_query() {
        let table = table_users();
        let query = QueryBuilder::select()
            .table(&table)
            .set_value_mode(ValueBindingMode::Named)
            .where_eq("id", 1)
            .or()
            .where_like("username", "geek")
            .build()
            .unwrap();

        assert_eq!(query.values.len(), 2);
        assert_eq!(
            query.as_sql(),
            "SELECT id, username, email, roles, profile FROM Users WHERE id = :id OR username LIKE :username;"
        );
    }

    #[test]
    fn test_where_numeric_query() {
        let table = table_users();
        let query = QueryBuilder::select()
            .table(&table)
            .set_value_mode(ValueBindingMode::Numeric)
            .where_like("username", "geek")
            .where_eq("id", 1)
            .build()
            .unwrap();

        assert_eq!(query.values.len(), 2);
        assert_eq!(
            query.as_sql(),
            "SELECT id, username, email, roles, profile FROM Users WHERE username LIKE ?1 AND id = ?2;"
        );
    }
}
