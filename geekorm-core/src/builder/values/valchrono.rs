//! Implementations of `From` trait for chrono types.
//!
//! ```rust
//! use geekorm::prelude::*;
//! use chrono::{DateTime, Utc};
//!
//! #[derive(GeekTable, Default)]
//! struct User {
//!     id: PrimaryKeyInteger,
//!     created_at: DateTime<Utc>,
//! }
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
