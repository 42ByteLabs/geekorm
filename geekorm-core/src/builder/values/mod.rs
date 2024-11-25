use std::{fmt::Display, str};

use serde::{Deserialize, Serialize, Serializer};

#[cfg(feature = "chrono")]
pub(crate) mod valchrono;
#[cfg(feature = "semver")]
pub(crate) mod valsemver;
#[cfg(feature = "uuid")]
pub(crate) mod valuuid;

use crate::{
    builder::keys::{foreign::ForeignKeyInteger, primary::PrimaryKeyInteger},
    PrimaryKey, TableBuilder, TablePrimaryKey,
};

/// List of Values
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Values {
    /// List of values
    pub(crate) values: Vec<Value>,
    /// List of columns in the order they were added
    pub(crate) order: Vec<String>,
}

impl Values {
    /// Create a new instance of Values
    pub fn new() -> Self {
        Values {
            values: Vec::new(),
            order: Vec::new(),
        }
    }

    /// Push a value to the list of values
    pub fn push(&mut self, column: String, value: impl Into<Value>) {
        self.order.push(column.clone());
        self.values.push(value.into());
    }

    /// Get a value by index from the list of values
    pub fn get(&self, column: &String) -> Option<&Value> {
        match self.order.iter().enumerate().find(|(_, o)| *o == column) {
            Some((i, _)) => self.values.get(i),
            None => None,
        }
    }

    /// Length / Count of the values stored
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.order
            .into_iter()
            .enumerate()
            .map(move |(index, _)| self.values[index].clone())
            .collect::<Vec<Value>>()
            .into_iter()
    }
}

/// A value for a column
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    /// A text (String) value
    Text(String),
    /// An integer (i32) value
    Integer(i32),
    /// A boolean (i32) value (0 or 1)
    /// This is because SQLite does not have a boolean type
    Boolean(i32),
    /// Identifier Key type (Primary / Forigen Key) which is a UUID
    Identifier(String),
    /// A binary blob value (vector of bytes)
    Blob(Vec<u8>),
    /// JSON blob
    Json(Vec<u8>),
    /// A NULL value
    Null,
}

impl Default for Value {
    fn default() -> Self {
        Value::Text(String::new())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Text(value) => write!(f, "{}", value),
            Value::Integer(value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Identifier(value) => write!(f, "{}", value),
            Value::Blob(value) | Value::Json(value) => {
                write!(f, "{}", str::from_utf8(value).unwrap_or(""))
            }
            Value::Null => write!(f, "NULL"),
        }
    }
}

impl From<PrimaryKey<String>> for Value {
    fn from(value: PrimaryKey<String>) -> Self {
        Value::Identifier(value.into())
    }
}

impl From<&PrimaryKey<String>> for Value {
    fn from(value: &PrimaryKey<String>) -> Self {
        Value::Identifier(value.clone().into())
    }
}

impl From<PrimaryKeyInteger> for Value {
    fn from(value: PrimaryKeyInteger) -> Self {
        Value::Integer(value.into())
    }
}

impl From<&PrimaryKeyInteger> for Value {
    fn from(value: &PrimaryKeyInteger) -> Self {
        Value::Integer((*value).into())
    }
}

// Where converting a ForeignKeyInteger to a Value,
// we only care about the integer value
impl<T> From<ForeignKeyInteger<T>> for Value
where
    T: TableBuilder + TablePrimaryKey,
{
    fn from(value: ForeignKeyInteger<T>) -> Self {
        Value::Integer(value.key)
    }
}

impl<T> From<&ForeignKeyInteger<T>> for Value
where
    T: TableBuilder + TablePrimaryKey,
{
    fn from(value: &ForeignKeyInteger<T>) -> Self {
        Value::Integer(value.key)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(if value { 1 } else { 0 })
    }
}

impl From<&bool> for Value {
    fn from(value: &bool) -> Self {
        Value::Boolean(if *value { 1 } else { 0 })
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::Null,
        }
    }
}

impl<T> From<&Option<T>> for Value
where
    T: Into<Value> + Clone,
{
    fn from(value: &Option<T>) -> Self {
        match value {
            Some(value) => value.clone().into(),
            None => Value::Null,
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value)
    }
}

impl From<&i32> for Value {
    fn from(value: &i32) -> Self {
        Value::Integer(*value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value as i32)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Integer(value as i32)
    }
}

impl From<Vec<String>> for Value {
    fn from(value: Vec<String>) -> Self {
        Value::Blob(serde_json::to_vec(&value).unwrap())
    }
}

impl From<&Vec<String>> for Value {
    fn from(value: &Vec<String>) -> Self {
        Value::Blob(serde_json::to_vec(value).unwrap())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Blob(value)
    }
}

impl From<&Vec<u8>> for Value {
    fn from(value: &Vec<u8>) -> Self {
        Value::Blob(value.clone())
    }
}

/// Serialize a Value
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Text(value) => serializer.serialize_str(value),
            Value::Integer(value) => serializer.serialize_i32(*value),
            Value::Boolean(value) => serializer.serialize_i32(*value),
            Value::Identifier(value) => serializer.serialize_str(value),
            // TODO(geekmasher): This might not be the correct way to serialize a blob
            Value::Blob(value) => serializer.serialize_bytes(value),
            // JSON
            Value::Json(value) => serde_json::from_slice::<serde_json::Value>(value)
                .map_err(serde::ser::Error::custom)?
                .serialize(serializer),
            // NULL
            Value::Null => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Text(value.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Text(v))
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(value as i32))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(value as i32))
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Boolean(if value { 1 } else { 0 }))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // TODO: is this the correct way to handle blobs?
                if value.starts_with(b"{") || value.starts_with(b"[") {
                    Ok(Value::Json(value.to_vec()))
                } else {
                    Ok(Value::Blob(value.to_vec()))
                }
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_map<A>(self, _accessor: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                Err(serde::de::Error::custom("Expects a struct"))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Values;

    #[test]
    fn test_values() {
        let mut values = Values::new();
        values.push("id".to_string(), 1);
        values.push("name".to_string(), "Bob");

        assert_eq!(values.len(), 2);
    }
}
