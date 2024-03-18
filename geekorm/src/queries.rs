use std::fmt::Display;

use crate::{Table, ToSqlite};

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

impl Display for QueryOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryOrder::Asc => write!(f, "ASC"),
            QueryOrder::Desc => write!(f, "DESC"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    pub(crate) table: Table,
    pub(crate) query_type: QueryType,
    pub(crate) order_by: Vec<(String, QueryOrder)>,
}

impl QueryBuilder {
    pub fn select() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Select,
            ..Default::default()
        }
    }

    pub fn create() -> QueryBuilder {
        QueryBuilder {
            query_type: QueryType::Create,
            ..Default::default()
        }
    }

    pub fn table(mut self, table: Table) -> Self {
        self.table = table.clone();
        self
    }

    pub fn order_by(mut self, column: &str, order: QueryOrder) -> Self {
        self.order_by.push((column.to_string(), order));
        self
    }

    pub fn build(&self) -> Result<String, crate::Error> {
        match self.query_type {
            QueryType::Create => Ok(self.table.on_create()),
            QueryType::Select => self.table.on_select(self),
            _ => todo!("Implement other query types"),
        }
    }
}
