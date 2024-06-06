//! Implementations of `From` trait for chrono types.
//!
//! ```rust
//! use geekorm::prelude::*;
//! use chrono::{DateTime, Utc};
//!
//! #[derive(GeekTable, Default, serde::Serialize, serde::Deserialize)]
//! struct User {
//!     #[geekorm(primary_key)]
//!     id: PrimaryKeyInteger,
//!     #[geekorm(new = "Utc::now()")]
//!     created_at: DateTime<Utc>,
//! }
//!
//! let user = User::new();
//!
//! # let json = serde_json::to_string(&user).unwrap();
//!
//! ```
//!
use super::Value;
use chrono::{DateTime, TimeZone};

impl<Tz> From<DateTime<Tz>> for Value
where
    Tz: TimeZone,
{
    fn from(value: DateTime<Tz>) -> Self {
        Value::Text(value.to_rfc3339())
    }
}

impl<Tz> From<&DateTime<Tz>> for Value
where
    Tz: TimeZone,
{
    fn from(value: &DateTime<Tz>) -> Self {
        Value::Text(value.to_rfc3339())
    }
}
