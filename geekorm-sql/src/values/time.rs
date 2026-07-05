//! # Time Values
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{ToValue, TryFromValue, Value};

impl From<SystemTime> for Value {
    fn from(value: SystemTime) -> Self {
        Value::Datetime(
            value
                .duration_since(UNIX_EPOCH)
                .expect("std::time issue")
                .as_secs(),
        )
    }
}

impl From<&SystemTime> for Value {
    fn from(value: &SystemTime) -> Self {
        Value::from(value.clone())
    }
}

impl ToValue for std::time::SystemTime {
    fn to_value(&self) -> Value {
        Value::from(self)
    }
}

impl TryFromValue for SystemTime {
    type Error = crate::Error;

    fn try_from_value(value: Value) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match value {
            Value::Datetime(data) => Ok(UNIX_EPOCH + Duration::from_secs(data)),
            // HACK: Not sure if this would work
            Value::Integer(data) => Ok(UNIX_EPOCH + Duration::from_secs(data as u64)),
            _ => todo!(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{ToValue, TryFromValue, Value};

    #[test]
    fn test_std_time() {
        let now = std::time::SystemTime::now();
        let value = now.to_value();
        assert!(matches!(value, Value::Datetime(_)));

        let og_time = <SystemTime as TryFromValue>::try_from_value(value)
            .expect("Failed to parse std::time::SystemTime");

        assert_eq!(
            og_time.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            now.duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
    }
}
