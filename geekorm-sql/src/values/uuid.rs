//! # UUID
//!
//! This is the UUID value type for the GeekORM SQL library.

use super::Value;
use crate::{ToValue, TryFromValue};
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

impl ToValue for Uuid {
    fn to_value(&self) -> Value {
        Value::Text(self.to_string())
    }
}

impl TryFromValue for Uuid {
    type Error = crate::Error;

    fn try_from_value(value: Value) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match value {
            Value::Text(value) => Ok(Uuid::parse_str(value.as_str())?),
            _ => Err(crate::Error::TryFromValueError {
                field: "uuid".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ToValue, TryFromValue, Value};
    use uuid::Uuid;

    #[test]
    fn test_uuid() {
        let uuid = Uuid::new_v4();
        let value = Value::from(uuid);
        // TODO(geekmasher): Is this really what we want to do?
        let og_value = <Uuid as TryFromValue>::try_from_value(value).expect("Failed to parse UUID");

        assert_eq!(uuid, og_value);
    }
}
