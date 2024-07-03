#![doc = include_str!("README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/42ByteLabs/geekorm/main/assets/geekorm.png"
)]
#![deny(missing_docs)]

// Builder Modules
pub use geekorm_core::builder::columns::{Column, Columns};
pub use geekorm_core::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use geekorm_core::builder::table::Table;
pub use geekorm_core::Error;
// Keys Modules
pub use geekorm_core::builder::keys::foreign::{ForeignKey, ForeignKeyInteger};
#[cfg(feature = "uuid")]
pub use geekorm_core::builder::keys::primary::PrimaryKeyUuid;
pub use geekorm_core::builder::keys::primary::{PrimaryKey, PrimaryKeyInteger, PrimaryKeyString};

// Query Builder Modules
pub use geekorm_core::builder::models::{QueryCondition, QueryOrder, QueryType};
pub use geekorm_core::builder::values::{Value, Values};

// Query Modules
pub use geekorm_core::queries::Query;
pub use geekorm_core::queries::QueryBuilder;

/// Utils
pub mod utils {
    pub use geekorm_core::utils::*;
}

// Derive Crate
pub use geekorm_derive::Data;
pub use geekorm_derive::Table;

/// GeekORM Version
pub const GEEKORM_VERSION: &str = env!("CARGO_PKG_VERSION");
/// GeekORM Banner
pub const GEEKORM_BANNER: &str = r#"   ______          __   ____  ____  __  ___
  / ____/__  ___  / /__/ __ \/ __ \/  |/  /
 / / __/ _ \/ _ \/ //_/ / / / /_/ / /|_/ /
/ /_/ /  __/  __/ ,< / /_/ / _, _/ /  / /
\____/\___/\___/_/|_|\____/_/ |_/_/  /_/"#;

pub mod prelude {
    //! GeekORM prelude
    //!
    //! The prelude module re-exports the most commonly used traits and types from the GeekORM crate.
    //!
    //! The prelude is useful for importing commonly used items in a single line.
    //!
    //! # Example
    //!
    //! ```rust
    //! use geekorm::prelude::*;
    //! ```

    pub use crate::Data;
    /// Table
    pub use crate::Table;

    // Traits

    /// Query Builder Trait
    pub use geekorm_core::QueryBuilderTrait;
    /// Table Builder Trait
    pub use geekorm_core::TableBuilder;
    /// Table Primary Key Trait
    pub use geekorm_core::TablePrimaryKey;
    /// SQLite Trait
    pub use geekorm_core::ToSqlite;
    // Backends Module
    pub use geekorm_core::{GeekConnection, GeekConnector};

    // Builder Modules
    pub use geekorm_core::builder::columns::{Column, Columns};
    pub use geekorm_core::builder::columntypes::{ColumnType, ColumnTypeOptions};
    pub use geekorm_core::builder::table::Table as BuilderTable;

    // Keys Modules
    pub use geekorm_core::builder::keys::foreign::{ForeignKey, ForeignKeyInteger};
    #[cfg(feature = "uuid")]
    pub use geekorm_core::builder::keys::primary::PrimaryKeyUuid;
    pub use geekorm_core::builder::keys::primary::{
        PrimaryKey, PrimaryKeyInteger, PrimaryKeyString,
    };

    pub use geekorm_core::builder::values::{Value, Values};
    // Query Builder Modules
    pub use geekorm_core::builder::models::{QueryCondition, QueryOrder, QueryType};
    // Query Modules
    pub use geekorm_core::queries::Query;
    pub use geekorm_core::queries::QueryBuilder;
}
