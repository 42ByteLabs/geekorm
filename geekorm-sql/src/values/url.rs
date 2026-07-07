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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use url::Url;

    #[test]
    fn test_url_values() {
        let url =
            Url::parse("https://42bytelabs.com:443/projects/geekorm").expect("Failed to parse URL");

        let value = Value::from(&url);
        assert!(matches!(value, Value::Text(_)));

        let og_url = <Url as TryFromValue>::try_from_value(value).expect("Failed to return URL");

        assert_eq!(url, og_url);
    }
}
