//! Implementations of `From` trait for chrono types.
//!
//! ```rust
//! use geekorm::prelude::*;
//! use chrono::{DateTime, Utc};
//!
//! #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
//! struct User {
//!     id: PrimaryKeyInteger,
//!     created_at: DateTime<Utc>,
//! }
//! ```
use super::Value;
use chrono::{DateTime, TimeZone, Utc};

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

impl<Tz> TryFrom<Value> for DateTime<Tz>
where
    Tz: TimeZone<Offset = Utc>,
{
    type Error = crate::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tz = Tz::from_offset(&Utc); // Get the timezone offset
        match value {
            Value::Text(text) => match DateTime::parse_from_rfc3339(&text) {
                Ok(dt) => Ok(dt.with_timezone(&tz)),
                Err(_) => Err(Self::Error::custom("Invalid DateTime format")),
            },
            Value::Integer(timestamp) => {
                let dt = DateTime::from_timestamp_nanos(timestamp);
                Ok(dt.with_timezone(&tz)) // Convert to UTC
            }
            _ => Err(Self::Error::custom("Value is not a DateTime")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Values;

    #[test]
    fn test_datetime_conversion() {
        let dt = Utc::now();
        let value: Value = dt.into();
        let converted_dt: DateTime<Utc> = value.try_into().unwrap();
        assert_eq!(dt, converted_dt);
    }

    #[test]
    fn test_datetime_from_string() {
        let dt_str = "2023-10-01T12:00:00Z";
        let value = Value::Text(dt_str.to_string());
        let dt: DateTime<Utc> = value.try_into().unwrap();
        assert_eq!(dt.to_rfc3339(), dt_str);
    }
}
