//! GeekORM is a simple and lightweight ORM for Rust. It is designed to be simple and easy to use.
//!
//! This project is still in the early stages of development, and as such, it may not be suitable for all use cases.
//!
//! ### Features
//!
//! - Simple and lightweight
//! - Uses derive macros for easy table creation
//! - Query builder pattern for building SQL queries
//!     - Only supports SQLite at the moment
//!
//! ### Example
//!
//! Here is a simple example of how to use GeekORM:
//!
//! ```rust
//! use geekorm::prelude::*;
//! use geekorm::{GeekTable, QueryOrder};
//!
//! #[derive(Debug, Clone, GeekTable)]
//! struct User {
//!    username: String,
//!    email: String,
//!    age: i32,
//!    postcode: Option<String>,
//! }
//!
//! // Use the `create` method to build a CREATE TABLE query
//! let create_table = User::create().build().expect("Failed to build query");
//! println!("Create Table Query: {}", create_table);
//!
//! // Use the `select` method to build a SELECT query along with different conditions
//! // and ordering
//! let select_user = User::select()
//!     .where_eq("username", "geekmasher")
//!     .and()
//!     .where_gt("age", 20)
//!     .order_by("age", QueryOrder::Asc)
//!     .limit(10)
//!     .build()
//!     .expect("Failed to build query");
//! println!("Select User Query: {}", select_user);
//!
//! // Print the values that will be used in the query
//! // This is useful for passing values to a database driver or connection in the correct order
//! println!("Select User Values: {:?}", select_user.values);
//! ```
//!
//! ### Unsupported Features
//!
//! If you are building a complex application, GeekORM may not be the best choice for you.
//! GeekORM is designed to be simple and lightweight, and as such, it does not support some of the more advanced features that other ORMs may offer.
//!
//! Here is a list of some of the features that GeekORM does not support (but may support in the future):
//!
//! - Automatic Migrations
//! - Relationships (e.g. One-to-Many, Many-to-Many)
//! - Transactions
//! - Connection Pooling
//!
//! If there are any features you would like to see in GeekORM, please open an issue on the GitHub repository.

#![deny(missing_docs)]

// Builder Modules
pub use geekorm_core::builder::columns::{Column, Columns};
pub use geekorm_core::builder::columntypes::{ColumnType, ColumnTypeOptions};
pub use geekorm_core::builder::keys::{ForeignKey, PrimaryKey};
pub use geekorm_core::builder::models::{QueryCondition, QueryOrder, QueryType};
pub use geekorm_core::builder::table::Table;
/// Values
pub use geekorm_core::builder::values::{Value, Values};

// Query Modules
pub use geekorm_core::queries::QueryBuilder;

// Derive Crate
/// GeekTable Derive Macro
pub use geekorm_derive::GeekTable;

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

    pub use crate::GeekTable;
    // Traits
    pub use geekorm_core::TableBuilder;
    pub use geekorm_core::ToSqlite;
}
