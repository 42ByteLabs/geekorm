//! # v2 - geekorm v0.13+
use geekorm::Table;

/// This struct represents a database and is based on the `internal`
/// module of the `geekorm_derive` crate.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct DatabaseV2 {
    #[serde(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// The name of the database
    #[serde(skip)]
    pub(crate) name: String,
    /// The tables in the database
    pub(crate) tables: Vec<Table>,
}
