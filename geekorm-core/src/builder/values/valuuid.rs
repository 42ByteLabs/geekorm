//! UUID value implementation for the Value enum.
use super::Value;
use crate::builder::keys::primary::PrimaryKeyUuid;

impl From<PrimaryKeyUuid> for Value {
    fn from(value: PrimaryKeyUuid) -> Self {
        Value::Text(value.value.to_string())
    }
}

impl From<&PrimaryKeyUuid> for Value {
    fn from(value: &PrimaryKeyUuid) -> Self {
        Value::Text(value.value.to_string())
    }
}
