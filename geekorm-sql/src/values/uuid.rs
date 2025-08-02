//! # UUID
//!
//! This is the UUID value type for the GeekORM SQL library.
use super::Value;
use uuid::Uuid;

impl From<Uuid> for Value {
    fn from(value: Uuid) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<&Uuid> for Value {
    fn from(value: &Uuid) -> Self {
        Value::Text(value.to_string())
    }
}
