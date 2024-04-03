use std::{collections::HashMap, fmt::Display, str};

use serde::{Serialize, Serializer};

use crate::{
    builder::keys::{foreign::ForeignKeyInteger, primary::PrimaryKeyInteger},
    PrimaryKey, TableBuilder, TablePrimaryKey,
};

/// List of Values
#[derive(Debug, Clone, Default)]
pub struct Values {
    /// List of values
    pub(crate) values: HashMap<String, Value>,
    /// List of columns in the order they were added
    pub(crate) order: Vec<String>,
}

impl Values {
    /// Create a new instance of Values
    pub fn new() -> Self {
        Values {
            values: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Push a value to the list of values
    pub fn push(&mut self, column: String, value: impl Into<Value>) {
        self.order.push(column.clone());
        self.values.insert(column, value.into());
    }

    /// Get a value by index from the list of values
    pub fn get(&self, column: &String) -> Option<&Value> {
        self.values.get(column)
    }
}

impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.order
            .into_iter()
            .map(move |column| self.values[&column].clone())
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
            Value::Blob(value) => write!(f, "{}", str::from_utf8(&value).unwrap_or("")),
            Value::Null => write!(f, "NULL"),
        }
    }
}

impl From<PrimaryKey<String>> for Value {
    fn from(value: PrimaryKey<String>) -> Self {
        Value::Identifier(value.into())
    }
}

impl From<PrimaryKeyInteger> for Value {
    fn from(value: PrimaryKeyInteger) -> Self {
        Value::Integer(value.into())
    }
}

impl From<&PrimaryKeyInteger> for Value {
    fn from(value: &PrimaryKeyInteger) -> Self {
        Value::Integer(value.clone().into())
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

impl From<&Option<String>> for Value {
    fn from(value: &Option<String>) -> Self {
        match value {
            Some(value) => Value::Text(value.to_string()),
            None => Value::Null,
        }
    }
}

impl From<&Option<i32>> for Value {
    fn from(value: &Option<i32>) -> Self {
        match value {
            Some(value) => Value::Integer(value.clone()),
            None => Value::Null,
        }
    }
}

impl From<&Option<bool>> for Value {
    fn from(value: &Option<bool>) -> Self {
        match value {
            Some(value) => Value::Boolean(if *value { 1 } else { 0 }),
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
        Value::Integer(value.clone())
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

            Value::Null => serializer.serialize_none(),
        }
    }
}
