//! # URL Values

use url::Url;

use crate::{ToValue, TryFromValue, Value};

impl ToValue for Url {
    fn to_value(&self) -> crate::prelude::Value {
        Value::from(self)
    }
}

impl TryFromValue for Url {
    type Error = crate::Error;

    fn try_from_value(value: Value) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Url::parse(&value.to_string())?)
    }
}

impl From<Url> for Value {
    fn from(value: Url) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<&Url> for Value {
    fn from(value: &Url) -> Self {
        Value::from(value.clone())
    }
}
