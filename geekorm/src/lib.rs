pub mod builder;
pub mod queries;

pub use crate::builder::columns::{Column, Columns};
pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use crate::builder::table::Table;
pub use crate::queries::{QueryBuilder, QueryType};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("QueryBuilderError: {0}")]
    QueryBuilderError(String),

    #[error("Unknown Error / Generic Error occurred")]
    Unknown,
}

pub trait TableBuilder {
    fn table() -> Table;
    fn table_name() -> String;

    fn select() -> QueryBuilder {
        QueryBuilder::select()
    }
    fn create() -> QueryBuilder;
}

pub trait ToSqlite {
    fn on_create(&self) -> String;

    fn on_select(&self, query: &QueryBuilder) -> Result<String, Error> {
        Err(Error::QueryBuilderError(format!(
            "on_select not implemented for table: {}",
            query.table
        )))
    }
}

#[doc(hidden)]
pub mod prelude {
    pub use crate::builder::columns::{Column, Columns};
    pub use crate::builder::columntypes::{ColumnType, ColumnTypeOptions};
    pub use crate::builder::table::Table;

    pub use crate::queries::{QueryBuilder, QueryType};

    pub use crate::TableBuilder;
    pub use crate::ToSqlite;
}
