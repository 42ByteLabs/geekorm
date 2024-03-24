use core::fmt;
use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Serialize, Serializer};
use uuid::Uuid;

use crate::ToSqlite;

/// Primary Key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryKey {
    /// The
    pub value: String,
}

impl PrimaryKey {
    /// Create a new primary key
    pub fn new() -> Self {
        PrimaryKey::default()
    }
}

impl Display for PrimaryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Default for PrimaryKey {
    fn default() -> Self {
        PrimaryKey {
            value: Uuid::new_v4().to_string(),
        }
    }
}

impl ToSqlite for PrimaryKey {
    fn on_create(&self) -> String {
        String::from("PRIMARY KEY")
    }
}

impl From<String> for PrimaryKey {
    fn from(value: String) -> Self {
        PrimaryKey { value }
    }
}

impl From<&str> for PrimaryKey {
    fn from(value: &str) -> Self {
        PrimaryKey {
            value: String::from(value),
        }
    }
}

impl Serialize for PrimaryKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}
impl<'de> Deserialize<'de> for PrimaryKey {
    fn deserialize<D>(deserializer: D) -> Result<PrimaryKey, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrimaryKeyVisitor;

        impl<'de> Visitor<'de> for PrimaryKeyVisitor {
            type Value = PrimaryKey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a primary key")
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimaryKey, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKey::from(value))
            }
            fn visit_string<E>(self, value: String) -> Result<PrimaryKey, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKey::from(value))
            }
        }

        deserializer.deserialize_str(PrimaryKeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_key_serde() {
        let pk = PrimaryKey::new();
        let pk_json = serde_json::to_string(&pk).unwrap();
        assert_eq!(pk_json, format!("\"{}\"", pk.value));

        let pk_deserialized: PrimaryKey = serde_json::from_str(pk_json.as_str()).unwrap();
        assert_eq!(pk, pk_deserialized);
    }
}
