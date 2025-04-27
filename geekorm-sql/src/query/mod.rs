//! # Query Builder

use geekorm_core::Values;

/// Query Builder
pub struct Query {
    pub(crate) query: String,

    /// Values are data for inserting
    pub(crate) values: Values,

    /// Parameters are data for binding
    pub(crate) params: Values,
}

impl Query {
    /// Get the SQL query string
    pub fn query(&self) -> String {
        self.query.clone()
    }
}
