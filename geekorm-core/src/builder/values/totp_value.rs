//! Implementations for converting `TOTP` to `Value`.
//!
//! ```rust
//! use geekorm::prelude::*;
//! use totp_rs::TOTP;
//!
//! #[derive(GeekTable, Default, serde::Serialize, serde::Deserialize)]
//! struct User {
//!     #[geekorm(primary_key)]
//!     id: PrimaryKeyInteger,
//!     #[geekorm(new = "TOTP::default()")]
//!     totp: TOTP,
//! }
//!
//! let user = User::new();
//!
//! # let json = serde_json::to_string(&user).unwrap();
//! ```
//!
use super::Value;
use totp_rs::TOTP;

impl From<TOTP> for Value {
    fn from(value: TOTP) -> Self {
        serde_json::to_string(&value)
            .map(|s| Value::Text(s))
            .unwrap_or(Value::Null)
    }
}

impl From<&TOTP> for Value {
    fn from(value: &TOTP) -> Self {
        serde_json::to_string(&value)
            .map(|s| Value::Text(s))
            .unwrap_or(Value::Null)
    }
}

impl From<Value> for TOTP {
    fn from(value: Value) -> Self {
        serde_json::from_str(&value.to_string()).unwrap_or(TOTP::default())
    }
}
