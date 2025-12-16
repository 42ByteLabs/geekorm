#![doc = include_str!("README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/42ByteLabs/geekorm/main/assets/geekorm.png"
)]
#![deny(missing_docs)]

// Builder Modules
pub use geekorm_core::Error;
pub use geekorm_core::builder::columns::{Column, Columns};
pub use geekorm_core::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use geekorm_core::builder::database::Database;
pub use geekorm_core::builder::table::Table;
// Keys Modules
pub use geekorm_core::builder::keys::foreign::{ForeignKey, ForeignKeyInteger};
#[cfg(feature = "uuid")]
pub use geekorm_core::builder::keys::primary::PrimaryKeyUuid;
pub use geekorm_core::builder::keys::primary::{PrimaryKey, PrimaryKeyInteger, PrimaryKeyString};

// Query Builder Modules
pub use geekorm_core::builder::models::{QueryCondition, QueryOrder, QueryType};
pub use geekorm_core::builder::values::{Value, Values};
#[cfg(feature = "migrations")]
pub use geekorm_core::builder::alter::{AlterMode, AlterQuery};

// Connection
#[cfg(feature = "connect")]
pub use geekorm_core::backends::connect::{Backend, Connection, manager::ConnectionManager};

/// Utils
pub mod utils {
    #[cfg(feature = "two-factor-auth")]
    pub use geekorm_core::TwoFactorAuth;
    pub use geekorm_core::utils::*;
}

#[cfg(feature = "migrations")]
pub use geekorm_core::migrations::{Migration, MigrationState};

// Derive Crate
pub use geekorm_derive::Data;
pub use geekorm_derive::Table;

// Depricated
pub use geekorm_derive::GeekTable;
pub use geekorm_derive::GeekValue;

// Traits
pub use geekorm_core::QueryBuilderTrait;
pub use geekorm_core::TableBuilder;
pub use geekorm_core::{GeekConnection, GeekConnector};

/// Re-export the `lazy_static` crate
#[cfg(feature = "migrations")]
#[doc(hidden)]
pub use lazy_static::lazy_static;

/// GeekORM Version
pub const GEEKORM_VERSION: &str = env!("CARGO_PKG_VERSION");
/// GeekORM Banner
pub const GEEKORM_BANNER: &str = r#"   ______          __   ____  ____  __  ___
  / ____/__  ___  / /__/ __ \/ __ \/  |/  /
 / / __/ _ \/ _ \/ //_/ / / / /_/ / /|_/ /
/ /_/ /  __/  __/ ,< / /_/ / _, _/ /  / /
\____/\___/\___/_/|_|\____/_/ |_/_/  /_/"#;

#[doc(hidden)]
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
    pub use crate::Table;

    // These are depricated
    pub use crate::GeekTable;
    pub use crate::GeekValue;

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
    #[cfg(feature = "migrations")]
    pub use geekorm_core::builder::alter::{AlterMode, AlterQuery};
    #[cfg(feature = "pagination")]
    pub use geekorm_core::queries::pages::Page;
    #[cfg(feature = "pagination")]
    pub use geekorm_core::queries::pagination::Pagination;

    // Keys Modules
    pub use geekorm_core::builder::keys::foreign::{ForeignKey, ForeignKeyInteger};
    #[cfg(feature = "uuid")]
    pub use geekorm_core::builder::keys::primary::PrimaryKeyUuid;
    pub use geekorm_core::builder::keys::primary::{
        PrimaryKey, PrimaryKeyInteger, PrimaryKeyString,
    };

    // Migrations Module
    #[cfg(feature = "migrations")]
    pub use geekorm_core::migrations::{Migration, MigrationState};

    // Helper Modules
    #[cfg(feature = "two-factor-auth")]
    pub use geekorm_core::TwoFactorAuth;

    pub use geekorm_core::builder::values::{Value, Values};
    // Query Builder Modules
    pub use geekorm_core::builder::models::{QueryCondition, QueryOrder, QueryType};
}
