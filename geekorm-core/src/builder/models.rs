use crate::ToSqlite;

/// Query Type (CREATE, SELECT, etc.)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum QueryType {
    /// Create a new table
    Create,
    /// Select data from a table
    #[default]
    Select,
    /// Insert data into a table
    Insert,
    /// Update data in a table
    Update,
    /// Delete data from a table
    Delete,
}

/// Query Order (ASC / DESC)
#[derive(Debug, Clone)]
pub enum QueryOrder {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl ToSqlite for QueryOrder {
    fn to_sqlite(&self) -> String {
        match self {
            QueryOrder::Asc => String::from("ASC"),
            QueryOrder::Desc => String::from("DESC"),
        }
    }
}

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

impl ToSqlite for QueryCondition {
    fn to_sqlite(&self) -> String {
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
        vec![
            WhereCondition::And.to_sqlite(),
            WhereCondition::Or.to_sqlite(),
        ]
    }
}

impl ToSqlite for WhereCondition {
    fn to_sqlite(&self) -> String {
        match self {
            WhereCondition::And => String::from("AND"),
            WhereCondition::Or => String::from("OR"),
        }
    }
}
