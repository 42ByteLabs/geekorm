//! # Transaction Queries

use crate::{Query, QueryBuilder, QueryType, ToSql, Values, query::BatchQueries};

/// Transaction Query
#[derive(Debug, Clone)]
pub struct TransactionQuery {
    /// Batch of queries
    pub(crate) queries: BatchQueries,

    /// Rollback
    rollback: bool,
}

impl TransactionQuery {
    /// Create a new transaction query
    pub fn new() -> TransactionQuery {
        TransactionQuery::default()
    }

    /// Set queries, replaces all queries
    pub fn queries(&mut self, queries: impl Into<BatchQueries>) -> &mut Self {
        self.queries = queries.into();
        self
    }

    /// Add Query
    pub fn query(&mut self, query: impl Into<Query>) -> &mut Self {
        let query = query.into();
        self.queries.push(query);
        self
    }

    /// Enable / Disable rollback transaction
    pub fn rollback(&mut self, rollback: bool) -> &mut Self {
        self.rollback = rollback;
        self
    }

    /// Build transaction
    pub fn build(&self) -> Result<Query, crate::Error> {
        let mut values = Values::new();
        // TODO(geekmasher): This clone is not great :/
        for query in self.queries.queries() {
            values.values.extend(query.values.values.clone());
        }

        Ok(Query {
            query: self.sql(),
            query_type: QueryType::Transaction,
            values,
            ..Default::default()
        })
    }
}

impl Default for TransactionQuery {
    fn default() -> Self {
        Self {
            queries: BatchQueries::new(),
            rollback: true,
        }
    }
}
impl QueryType {
    pub(crate) fn sql_transaction(&self, query: &QueryBuilder) -> String {
        if let Some(transaction) = &query.transaction {
            transaction.sql()
        } else {
            String::new()
        }
    }
}

impl ToSql for TransactionQuery {
    fn sql(&self) -> String {
        let mut stream = String::from("BEGIN TRANSACTION;\n\n");

        for query in self.queries.queries() {
            stream.push_str(&format!("{}\n", query.query));
        }

        stream.push_str("\nCOMMIT;");

        stream
    }
}

impl From<&BatchQueries> for BatchQueries {
    fn from(value: &BatchQueries) -> Self {
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QueryBuilder, Table, builder::tests::*};

    fn batches(table: &Table) -> BatchQueries {
        let mut queries = BatchQueries::new();
        queries.push(
            Query::update()
                .table(table)
                .where_eq("id", 1)
                .add_value("username", "geekmasher")
                .build()
                .unwrap(),
        );
        queries
    }

    #[test]
    fn sqlite_batch_to_query() {
        let users = table_users();
        let queries = batches(&users);

        let query = QueryBuilder::transaction()
            .queries(queries)
            .build()
            .expect("Failed to create transaction query");
        let sql = query.sql();

        assert_eq!(query.values.len(), 2);
        assert_eq!(
            sql,
            "BEGIN TRANSACTION;\n\nUPDATE Users SET username = ? WHERE id = ?;\n\nCOMMIT;"
        );
    }
}
