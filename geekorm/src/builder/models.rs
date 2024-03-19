use crate::ToSqlite;

#[derive(Debug, Clone, Default)]
pub enum QueryType {
    Create,
    #[default]
    Select,
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone)]
pub enum QueryOrder {
    Asc,
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

#[derive(Debug, Clone, Default)]
pub enum QueryCondition {
    #[default]
    Eq,
    Ne,
    Like,
    Gt,
    Lt,
    Gte,
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

#[derive(Debug, Clone, Default)]
pub enum WhereCondition {
    #[default]
    And,
    Or,
}

impl WhereCondition {
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
