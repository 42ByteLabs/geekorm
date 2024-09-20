//! # Implementations of `From` trait for semver types.
//!
//! ```rust
//! # #[cfg(feature = "semver")] {
//! use geekorm::prelude::*;
//! use semver::{Version, VersionReq};
//!
//! #[derive(Table, Clone, serde::Serialize, serde::Deserialize)]
//! struct Project {
//!     #[geekorm(primary_key, auto_increment)]
//!     id: PrimaryKey<i32>,
//!     /// Name of the project
//!     #[geekorm(unique)]
//!     name: String,
//|     /// Version
//!     version: Version,
//!     /// Required Version
//!     requirement: VersionReq,
//! }
//!
//! // Create a new Project with a semver Version
//! let project = Project::new(
//!     "geekorm",
//!     Version::parse("1.0.0").unwrap(),
//!     VersionReq::parse(">=0.6, <1.2.3").unwrap(),
//! );
//!
//! # assert!(project.requirement.matches(&project.version));
//!
//! # }
//! ```

use super::Value;
use semver::{Version, VersionReq};

impl From<Version> for Value {
    fn from(value: Version) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<&Version> for Value {
    fn from(value: &Version) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<Value> for Version {
    fn from(value: Value) -> Self {
        // TODO: This unwrap isn't great...
        Version::parse(&value.to_string()).unwrap()
    }
}

impl From<VersionReq> for Value {
    fn from(value: VersionReq) -> Self {
        Value::Text(value.to_string())
    }
}
impl From<&VersionReq> for Value {
    fn from(value: &VersionReq) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<Value> for VersionReq {
    fn from(value: Value) -> Self {
        // TODO: This unwrap isn't great
        VersionReq::parse(&value.to_string()).unwrap()
    }
}
